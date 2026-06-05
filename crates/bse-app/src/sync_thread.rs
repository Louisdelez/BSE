//! Worker thread that bridges the async [`bse_sync::SyncClient`]
//! with the synchronous egui event loop.
//!
//! The egui thread cannot block on async I/O, so all WebSocket traffic
//! lives on a dedicated thread running a current-thread tokio runtime.
//! [`SyncHandle`] is the egui-side entry point : it owns one channel
//! for commands (egui → worker) and one for events (worker → egui).
//!
//! v019 adds resilience :
//! - the last [`ClientConfig`] is remembered and replayed automatically
//!   when the stream dies (network blip, server restart, …) ;
//! - retries use exponential backoff (`INITIAL_BACKOFF` → `MAX_BACKOFF`)
//!   and surface `ConnectionState::Reconnecting` to the UI ;
//! - `SyncCmd::Op` payloads emitted while offline are queued and
//!   replayed once the next connection is established.

use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use bse_protocol::{AwarenessPayload, OpPayload, ServerMessage};
use bse_sync::{ClientConfig, ConnectionState, LocalCursor, SyncClient};
use bse_types::{Color, PeerId, Vec2};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tracing::{info, warn};

/// First backoff delay after a failed connect / lost connection.
const INITIAL_BACKOFF: Duration = Duration::from_secs(1);
/// Backoff cap : we never wait longer than this between attempts.
const MAX_BACKOFF: Duration = Duration::from_secs(30);
/// How many op payloads we keep buffered while offline before dropping
/// the oldest. Each op is the full CRDT snapshot, so duplicates are
/// fine ; only the last one matters.
const MAX_OP_QUEUE: usize = 64;

/// Egui → worker commands.
#[derive(Debug)]
#[allow(dead_code)]
pub enum SyncCmd {
    /// Try to establish a connection. The previous one (if any) is closed.
    Connect(ClientConfig),
    /// Push the current local cursor position. The worker throttles to 30 Hz.
    Cursor(Vec2),
    /// Broadcast a fresh CRDT snapshot/update to the room.
    Op(Vec<u8>),
    /// Politely close the connection and stop auto-reconnecting.
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

/// Internal worker-loop state.
///
/// Held in a single struct so the `match` arms below can mutate parts
/// of it without juggling a dozen local variables.
struct WorkerState {
    client: Option<SyncClient>,
    /// Last config seen on `SyncCmd::Connect`. Used to retry after a
    /// connection drop.
    last_config: Option<ClientConfig>,
    /// `true` when the user explicitly asked for disconnect ; suppresses
    /// auto-reconnect attempts.
    user_disconnected: bool,
    /// Current backoff delay. Doubles on each failed attempt, capped.
    backoff: Duration,
    /// `Some(t)` while waiting for the backoff window to elapse ;
    /// `None` when not in a reconnect cycle.
    next_attempt_at: Option<Instant>,
    /// Ops emitted while offline ; flushed on (re)connect.
    pending_ops: VecDeque<Vec<u8>>,
    local_cursor: LocalCursor,
    peer_names: HashMap<PeerId, Arc<str>>,
    last_cursor_emit: Instant,
    pending_cursor: Option<Vec2>,
}

impl WorkerState {
    fn new() -> Self {
        Self {
            client: None,
            last_config: None,
            user_disconnected: false,
            backoff: INITIAL_BACKOFF,
            next_attempt_at: None,
            pending_ops: VecDeque::new(),
            local_cursor: LocalCursor::new(),
            peer_names: HashMap::new(),
            last_cursor_emit: Instant::now(),
            pending_cursor: None,
        }
    }

    fn enqueue_op(&mut self, bytes: Vec<u8>) {
        if self.pending_ops.len() >= MAX_OP_QUEUE {
            self.pending_ops.pop_front();
        }
        self.pending_ops.push_back(bytes);
    }

    fn bump_backoff(&mut self) {
        self.backoff = (self.backoff * 2).min(MAX_BACKOFF);
    }

    fn reset_backoff(&mut self) {
        self.backoff = INITIAL_BACKOFF;
        self.next_attempt_at = None;
    }
}

async fn worker_loop(
    mut cmd_rx: UnboundedReceiver<SyncCmd>,
    event_tx: UnboundedSender<SyncEvent>,
) {
    let mut state = WorkerState::new();
    let mut interval = tokio::time::interval(Duration::from_millis(33));

    let _ = event_tx.send(SyncEvent::State(ConnectionState::Offline));

    loop {
        interval.tick().await;

        // 1. Drain pending commands.
        while let Ok(cmd) = cmd_rx.try_recv() {
            handle_cmd(cmd, &mut state, &event_tx).await;
        }

        // 2. Attempt reconnect if the backoff window has elapsed.
        if state.client.is_none()
            && !state.user_disconnected
            && let Some(at) = state.next_attempt_at
            && Instant::now() >= at
            && let Some(config) = state.last_config.clone()
        {
            attempt_connect(config, &mut state, &event_tx).await;
        }

        // 3. Emit the throttled cursor if we have a connection.
        if let (Some(pos), Some(c)) = (state.pending_cursor, state.client.as_mut())
            && state.last_cursor_emit.elapsed() >= Duration::from_millis(33)
            && let Some(bytes) = state.local_cursor.maybe_emit(pos)
        {
            let payload = AwarenessPayload {
                peer_id: PeerId::new(),
                bytes,
            };
            if let Err(err) = c.send_awareness(payload).await {
                warn!(target: "bse::sync", error = %err, "send_awareness failed");
            }
            state.last_cursor_emit = Instant::now();
            state.pending_cursor = None;
        }

        // 4. Pull pending server messages.
        if let Some(c) = state.client.as_mut() {
            let mut drop_client = false;
            loop {
                match c.next_message().await {
                    Ok(Some(msg)) => translate_message(msg, &event_tx, &mut state.peer_names),
                    Ok(None) => break,
                    Err(err) => {
                        warn!(target: "bse::sync", error = %err, "stream error, will reconnect");
                        drop_client = true;
                        break;
                    }
                }
            }
            if drop_client {
                state.client = None;
                if state.last_config.is_some() && !state.user_disconnected {
                    schedule_reconnect(&mut state, &event_tx);
                } else {
                    let _ = event_tx.send(SyncEvent::State(ConnectionState::Offline));
                }
            }
        }
    }
}

async fn handle_cmd(
    cmd: SyncCmd,
    state: &mut WorkerState,
    event_tx: &UnboundedSender<SyncEvent>,
) {
    match cmd {
        SyncCmd::Connect(config) => {
            state.last_config = Some(config.clone());
            state.user_disconnected = false;
            state.reset_backoff();
            attempt_connect(config, state, event_tx).await;
        }
        SyncCmd::Cursor(pos) => {
            state.pending_cursor = Some(pos);
        }
        SyncCmd::Op(bytes) => {
            if let Some(c) = state.client.as_mut() {
                let payload = OpPayload {
                    seq: 0,
                    bytes: bytes.clone(),
                };
                if let Err(err) = c.send_op(payload).await {
                    warn!(target: "bse::sync", error = %err, "send_op failed ; queueing for retry");
                    state.enqueue_op(bytes);
                    state.client = None;
                    schedule_reconnect(state, event_tx);
                }
            } else {
                state.enqueue_op(bytes);
            }
        }
        SyncCmd::Disconnect => {
            state.user_disconnected = true;
            state.last_config = None;
            state.next_attempt_at = None;
            if let Some(c) = state.client.take() {
                let _ = c.close().await;
            }
            let _ = event_tx.send(SyncEvent::State(ConnectionState::Offline));
        }
    }
}

async fn attempt_connect(
    config: ClientConfig,
    state: &mut WorkerState,
    event_tx: &UnboundedSender<SyncEvent>,
) {
    let _ = event_tx.send(SyncEvent::State(ConnectionState::Connecting));
    match SyncClient::connect(config).await {
        Ok(mut c) => {
            info!(target: "bse::sync", "connected");
            // Replay any ops queued while offline.
            let queue = std::mem::take(&mut state.pending_ops);
            for bytes in queue {
                let payload = OpPayload { seq: 0, bytes };
                if let Err(err) = c.send_op(payload).await {
                    warn!(target: "bse::sync", error = %err, "send_op (replay) failed");
                    break;
                }
            }
            state.client = Some(c);
            state.reset_backoff();
            let _ = event_tx.send(SyncEvent::State(ConnectionState::Connected));
        }
        Err(err) => {
            warn!(target: "bse::sync", error = %err, "connect failed");
            state.client = None;
            schedule_reconnect(state, event_tx);
        }
    }
}

fn schedule_reconnect(state: &mut WorkerState, event_tx: &UnboundedSender<SyncEvent>) {
    if state.user_disconnected || state.last_config.is_none() {
        let _ = event_tx.send(SyncEvent::State(ConnectionState::Offline));
        return;
    }
    let delay = state.backoff;
    state.next_attempt_at = Some(Instant::now() + delay);
    state.bump_backoff();
    info!(target: "bse::sync", delay_secs = delay.as_secs(), "scheduling reconnect");
    let _ = event_tx.send(SyncEvent::State(ConnectionState::Reconnecting));
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
            let _ = event_tx.send(SyncEvent::RemoteOp(s.bytes));
        }
        ServerMessage::Welcome(_) | ServerMessage::Error(_) | ServerMessage::Pong(_) => {}
    }
}
