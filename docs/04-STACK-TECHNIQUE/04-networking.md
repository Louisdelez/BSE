# 04.04 — Networking

> WebSocket + axum + tokio pour la v1. QUIC/iroh en v1.x pour le P2P.

## TL;DR

> **WebSocket via `axum` + `tokio-tungstenite`** pour le client-serveur classique.  
> **`iroh` (QUIC + NAT traversal)** envisagé pour un mode P2P en v1.x.

## Couche transport : WebSocket

### Pourquoi WebSocket ?

1. **Bidirectionnel full-duplex** : le serveur peut pousser des events sans polling
2. **Standard mature** sur TCP, traverse tous les firewalls / proxies
3. **TLS natif** (`wss://`)
4. **Excellent support Rust** : `tokio-tungstenite`, `axum::extract::ws`
5. **Tooling de débogage** universel

### Pourquoi pas HTTP polling / SSE ?
- Polling : latence élevée, charge serveur excessive
- Server-Sent Events : unidirectionnel, mal adapté

### Pourquoi pas QUIC dès le départ ?
- TLS sur UDP, multiplexage natif, 0-RTT — séduisant
- Mais moins traversant en entreprise (UDP bloqué chez certains FW)
- Écosystème WS reste roi en 2026
- **On y reviendra en v1.x via `iroh`** pour les déploiements P2P

## Stack côté serveur

### Axum (HTTP + WS routing)

```rust
use axum::{
    routing::{get, post},
    Router,
    extract::ws::WebSocketUpgrade,
};

async fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/auth/login", post(auth::login))
        .route("/api/projects", get(projects::list).post(projects::create))
        .route("/api/projects/:id", get(projects::get))
        .route("/ws/rooms/:id", get(ws_upgrade))
        .layer(middleware::auth())
        .layer(middleware::cors())
        .layer(middleware::trace())
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    Path(room_id): Path<RoomId>,
    AuthUser(user): AuthUser,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| {
        room_manager::handle_peer(socket, room_id, user)
    })
}
```

### tokio runtime

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let config = config::load()?;
    let db = db::connect(&config.database).await?;
    let storage = storage::s3_client(&config.storage).await?;
    
    let app = router(config, db, storage);
    
    let listener = tokio::net::TcpListener::bind(&config.server.bind).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

### Room actor (1 task par room)

```rust
struct Room {
    project_id: ProjectId,
    crdt_doc: yrs::Doc,
    peers: HashMap<PeerId, PeerHandle>,
    ...
}

impl Room {
    async fn run(mut self, mut peer_join_rx: Receiver<PeerHandle>) {
        let mut checkpoint_interval = tokio::time::interval(Duration::from_secs(300));
        let mut idle_timer = tokio::time::interval(Duration::from_secs(60));
        
        loop {
            tokio::select! {
                Some(peer) = peer_join_rx.recv() => self.add_peer(peer).await,
                Some((peer_id, msg)) = self.next_peer_message() => 
                    self.handle_message(peer_id, msg).await,
                _ = checkpoint_interval.tick() => self.checkpoint().await,
                _ = idle_timer.tick() => {
                    if self.peers.is_empty() {
                        self.persist_and_shutdown().await;
                        break;
                    }
                }
            }
        }
    }
}
```

## Stack côté client

### Connexion WS

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};

async fn connect_to_room(
    server_url: &str,
    room_id: &RoomId,
    token: &str,
) -> Result<WsConnection> {
    let url = format!("{}/ws/rooms/{}?token={}", server_url, room_id, token);
    let (ws_stream, _response) = connect_async(&url).await?;
    Ok(WsConnection { ws_stream, room_id: *room_id, ... })
}
```

### Reconnexion automatique

Exponential backoff sur reconnect :
- 1ère tentative immédiate
- Puis 2s, 5s, 10s, 30s, 60s, max 60s
- Reset du backoff après 5min connecté
- État UI clair : `Connected`, `Reconnecting`, `Offline`

### Throttle de l'awareness

Le curseur s'envoie à max 30 Hz :
```rust
let mut last_cursor_send = Instant::now();
const CURSOR_PERIOD: Duration = Duration::from_millis(33);  // ~30 Hz

if last_cursor_send.elapsed() >= CURSOR_PERIOD {
    sync.send_cursor(cursor_pos).await;
    last_cursor_send = Instant::now();
}
```

## Heartbeat / keepalive

```rust
// Côté client
let mut heartbeat = tokio::time::interval(Duration::from_secs(15));
let mut last_pong = Instant::now();

loop {
    tokio::select! {
        _ = heartbeat.tick() => {
            sync.send(ClientMessage::Ping(now_ms())).await;
            if last_pong.elapsed() > Duration::from_secs(30) {
                // Connexion morte → reconnect
                break;
            }
        }
        Some(msg) = ws_recv.next() => {
            match msg {
                ServerMessage::Pong(_) => last_pong = Instant::now(),
                _ => handle_msg(msg),
            }
        }
    }
}
```

## Compression

### TLS compresse déjà
TLS 1.3 supporte la compression. Pas besoin d'une couche supplémentaire.

### Compression au niveau message (optionnelle)
Pour les messages volumineux (snapshots), compression `zstd` ou `lz4` :
```rust
let compressed = zstd::stream::encode_all(payload, 3)?;
ws.send(Message::Binary(compressed)).await?;
```

Mais à éviter pour les ops fréquentes (overhead > gain).

## Cas P2P / iroh (v1.x+)

Pour des déploiements où on veut **pas de serveur central** (équipe de 3-5 amis, étudiants…), `iroh` offre :

- **Connexions P2P directes** entre peers, sans serveur intermédiaire
- **NAT traversal automatique** (UDP hole punching)
- **Fallback relais** quand le hole punching échoue
- **Identité par PeerId** (pas d'IP)
- **QUIC sous-jacent** : crypto + multiplexage

### Architecture P2P potentielle

```
Peer A ◄─── direct P2P (QUIC) ──► Peer B
   ▲                                ▲
   │       ┌──────────────┐         │
   └──────►│ iroh relay   │◄────────┘
           └──────────────┘
           (en cas d'échec direct)
```

### Sync sans serveur
- Les peers s'échangent les ops CRDT directement
- L'élu *leader* de la room héberge le snapshot (rotatif)
- Persistance dépend du peer : disque local

C'est une **option de déploiement supplémentaire**, pas un remplacement du client-serveur. Cible v1.x.

## Sécurité réseau

### TLS obligatoire en production
- `wss://` (WebSocket over TLS)
- Cert via Let's Encrypt (côté reverse proxy) ou cert manuel
- Rustls comme implem TLS (préféré OpenSSL en Rust)

### Origins / CSRF
- Whitelist d'origines en `CORS`
- Token vérifié dans le query param ou subprotocol pour le WS

### Rate limiting
- Au niveau HTTP : `tower::limit::RateLimitLayer`
- Au niveau room : max ops/sec/peer
- Au niveau global : max nouvelles connexions/IP/sec

### DDoS
- Reverse proxy (Caddy, nginx, Traefik) avec rate limit
- Cloudflare ou équivalent en SaaS Cloud BSE

## Observabilité réseau

Métriques exposées :
- `bse_ws_connections_active` (gauge)
- `bse_ws_connections_total` (counter)
- `bse_ws_messages_total{direction}` (counter)
- `bse_ws_message_bytes{direction}` (counter, histogram)
- `bse_ws_latency_seconds` (histogram)
- `bse_ws_disconnects_total{reason}` (counter)

## Estimations de charge

### Pour 1 room avec 10 peers actifs (dessin)

- Cursor : 10 peers × 30 Hz × 24 octets ≈ 7 KB/s par peer reçus
- Ops dessin : 10 peers × 5 Hz × 200 octets ≈ 10 KB/s
- Total : ~20 KB/s par peer
- Soit ~160 kbps. Tout à fait gérable.

### Pour le serveur

Avec 100 rooms × 10 peers = 1000 connexions actives :
- Mémoire : ~50 MB pour les connexions + ~50 MB par room CRDT = ~5 GB
- CPU : modeste (Rust async très efficace)
- Bande passante : ~20 MB/s entrant, ~200 MB/s sortant (broadcast)

Une VPS 4 vCPU / 8 GB peut gérer ~50 rooms confortablement.

## Décisions clés

| Décision | Choix | Raison |
|---|---|---|
| Transport client-serveur | WebSocket / wss | Mature, traversant, écosystème |
| Lib WS serveur | axum + tokio-tungstenite | Standard Rust |
| Lib WS client | tokio-tungstenite | idem |
| Sérialisation | MessagePack (rmp-serde) | Compact, rapide |
| TLS implem | rustls | Pas d'OpenSSL |
| Compression | zstd opt sur snapshots | Pas sur ops fréquentes |
| P2P (futur) | iroh (QUIC) | NAT traversal, prod-proven |

## Sources

- *Rust WebSocket Guide: tokio-tungstenite, axum & JoinSet* — websocket.org
- *Building Real-Time Apps with Rust WebSockets: Tokio + Axum in 2026* — rustify.rs
- *iroh: IP addresses break, dial keys instead* — github.com/n0-computer/iroh
