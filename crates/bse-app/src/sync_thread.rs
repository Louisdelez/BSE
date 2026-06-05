//! Worker thread that bridges the async [`bse_sync::SyncClient`]
//! with the synchronous egui event loop.
//!
//! The egui thread cannot block on async I/O, so all WebSocket traffic
//! lives on a dedicated thread running a current-thread tokio runtime.
//! [`SyncHandle`] is the egui-side entry point : it owns one channel
//! for commands (egui → worker) and one for events (worker → egui).
//!
//! v010.3 wires CRDT ops over the same WebSocket :
//! - cursor throttling at 30 Hz (already done by `bse-sync::LocalCursor`),
//! - remote peer cursors decoded from awareness messages,
//! - `SyncCmd::Op` ships a freshly-encoded CRDT snapshot to the room,
//! - inbound `ServerMessage::Op` / `Snapshot` are surfaced as
//!   `SyncEvent::RemoteOp` for the egui side to apply.
//!
//! Reconnect logic is still deferred ; the worker loop drops the
//! connection on any stream error and waits for a fresh `Connect`.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use bse_protocol::{AwarenessPayload, OpPayload, ServerMessage};
use bse_sync::{ClientConfig, ConnectionState, LocalCursor, SyncClient};
use bse_types::{Color, PeerId, Vec2};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tracing::{info, warn};

/// Egui → worker commands.
///
/// `Disconnect` is part of the public surface but is not yet wired
/// from the UI ; a future settings dialog will use it.
#[derive(Debug)]
#[allow(dead_code)]
pub enum SyncCmd {
    /// Try to establish a connection. The previous one (if any) is closed.
    Connect(ClientConfig),
    /// Push the current local cursor position. The worker throttles to 30 Hz.
    Cursor(Vec2),
    /// Broadcast a fresh CRDT snapshot/update to the room.
    Op(Vec<u8>),
    /// Politely close the connection.
    Disconnect,
}

/// Worker → egui events.
#[derive(Clone, Debug)]
pub enum SyncEvent {
    /// State of the WebSocket connection.
    State(ConnectionState),
    /// A remote peer has moved its cursor.
    PeerCursor { peer_id: PeerId, position: Vec2 },
    /// A new peer has joined the room.
    PeerJoin {
        peer_id: PeerId,
        display_name: String,
        color: Color,
    },
    /// A peer has left the room.
    PeerLeave(PeerId),
    /// A remote CRDT op (or snapshot) ready to be applied locally.
    RemoteOp(Vec<u8>),
}

/// Egui-side handle to the worker thread.
pub struct SyncHandle {
    cmd_tx: UnboundedSender<SyncCmd>,
    event_rx: UnboundedReceiver<SyncEvent>,
    _thread: std::thread::JoinHandle<()>,
}

impl SyncHandle {
    /// Spawn the worker thread and return a handle to it.
    #[must_use]
    pub fn spawn() -> Self {
        let (cmd_tx, cmd_rx) = unbounded_channel::<SyncCmd>();
        let (event_tx, event_rx) = unbounded_channel::<SyncEvent>();
        let thread = std::thread::Builder::new()
            .name("bse-sync".to_string())
            .spawn(move || run_worker(cmd_rx, event_tx))
            .expect("spawn bse-sync worker thread");
        Self {
            cmd_tx,
            event_rx,
            _thread: thread,
        }
    }

    /// Queue a command. Silently dropped if the worker has exited.
    pub fn send(&self, cmd: SyncCmd) {
        let _ = self.cmd_tx.send(cmd);
    }

    /// Drain all events emitted by the worker since the last call.
    pub fn drain_events(&mut self) -> Vec<SyncEvent> {
        let mut out = Vec::new();
        while let Ok(event) = self.event_rx.try_recv() {
            out.push(event);
        }
        out
    }
}

fn run_worker(cmd_rx: UnboundedReceiver<SyncCmd>, event_tx: UnboundedSender<SyncEvent>) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("build tokio current-thread runtime");
    rt.block_on(worker_loop(cmd_rx, event_tx));
}

async fn worker_loop(mut cmd_rx: UnboundedReceiver<SyncCmd>, event_tx: UnboundedSender<SyncEvent>) {
    let mut client: Option<SyncClient> = None;
    let mut local_cursor = LocalCursor::new();
    let mut peer_names: HashMap<PeerId, Arc<str>> = HashMap::new();
    let mut interval = tokio::time::interval(Duration::from_millis(33));
    let mut last_cursor_emit = Instant::now();
    let mut pending_cursor: Option<Vec2> = None;

    let _ = event_tx.send(SyncEvent::State(ConnectionState::Offline));

    loop {
        interval.tick().await;

        // Drain pending commands.
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                SyncCmd::Connect(config) => {
                    let _ = event_tx.send(SyncEvent::State(ConnectionState::Connecting));
                    match SyncClient::connect(config).await {
                        Ok(c) => {
                            info!(target: "bse::sync", "connected");
                            client = Some(c);
                            let _ = event_tx.send(SyncEvent::State(ConnectionState::Connected));
                        }
                        Err(err) => {
                            warn!(target: "bse::sync", error = %err, "connect failed");
                            client = None;
                            let _ = event_tx.send(SyncEvent::State(ConnectionState::Offline));
                        }
                    }
                }
                SyncCmd::Cursor(pos) => {
                    pending_cursor = Some(pos);
                }
                SyncCmd::Op(bytes) => {
                    if let Some(c) = client.as_mut() {
                        let payload = OpPayload { seq: 0, bytes };
                        if let Err(err) = c.send_op(payload).await {
                            warn!(target: "bse::sync", error = %err, "send_op failed");
                        }
                    }
                }
                SyncCmd::Disconnect => {
                    if let Some(c) = client.take() {
                        let _ = c.close().await;
                    }
                    let _ = event_tx.send(SyncEvent::State(ConnectionState::Offline));
                }
            }
        }

        // Emit the throttled cursor if we have a connection.
        if let (Some(pos), Some(c)) = (pending_cursor, client.as_mut())
            && last_cursor_emit.elapsed() >= Duration::from_millis(33)
            && let Some(bytes) = local_cursor.maybe_emit(pos)
        {
            let payload = AwarenessPayload {
                peer_id: PeerId::new(),
                bytes,
            };
            if let Err(err) = c.send_awareness(payload).await {
                warn!(target: "bse::sync", error = %err, "send_awareness failed");
            }
            last_cursor_emit = Instant::now();
            pending_cursor = None;
        }

        // Pull pending server messages.
        if let Some(c) = client.as_mut() {
            loop {
                match c.next_message().await {
                    Ok(Some(msg)) => translate_message(msg, &event_tx, &mut peer_names),
                    Ok(None) => break,
                    Err(err) => {
                        warn!(target: "bse::sync", error = %err, "stream error, dropping connection");
                        client = None;
                        let _ = event_tx.send(SyncEvent::State(ConnectionState::Offline));
                        break;
                    }
                }
            }
        }
    }
}

fn translate_message(
    msg: ServerMessage,
    event_tx: &UnboundedSender<SyncEvent>,
    peer_names: &mut HashMap<PeerId, Arc<str>>,
) {
    match msg {
        ServerMessage::PeerJoin(p) => {
            peer_names.insert(p.peer_id, Arc::<str>::from(p.display_name.as_str()));
            let _ = event_tx.send(SyncEvent::PeerJoin {
                peer_id: p.peer_id,
                display_name: p.display_name,
                color: Color::rgb(p.color.0, p.color.1, p.color.2),
            });
        }
        ServerMessage::PeerLeave(p) => {
            peer_names.remove(&p.peer_id);
            let _ = event_tx.send(SyncEvent::PeerLeave(p.peer_id));
        }
        ServerMessage::Awareness(a) => {
            if let Ok(pos) = bse_sync::decode_cursor(&a.bytes) {
                let _ = event_tx.send(SyncEvent::PeerCursor {
                    peer_id: a.peer_id,
                    position: pos,
                });
            }
        }
        ServerMessage::Op(op) => {
            let _ = event_tx.send(SyncEvent::RemoteOp(op.bytes));
        }
        ServerMessage::Snapshot(s) => {
            // v010.3 : the server broadcasts our own client-emitted
            // snapshots as `Op`, but a real server may also push a
            // `Snapshot` on join. Yrs accepts both formats indifferently
            // through `apply_remote_update`.
            let _ = event_tx.send(SyncEvent::RemoteOp(s.bytes));
        }
        ServerMessage::Welcome(_) | ServerMessage::Error(_) | ServerMessage::Pong(_) => {}
    }
}
