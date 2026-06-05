# 03.04 — Protocole réseau

> Le format précis des messages échangés entre client et serveur.

## Couche transport

- **WebSocket over TLS** (`wss://`) par défaut
- **MessagePack** comme format de sérialisation (compact, rapide)
- **Sub-protocol** `bse.v1`
- En option future : QUIC via iroh (cf. [../04-STACK-TECHNIQUE/04-networking.md](../04-STACK-TECHNIQUE/04-networking.md))

## Pourquoi MessagePack ?

- Compact (2-4× plus petit que JSON)
- Plus rapide à parser que JSON
- Garde la structure (objets, arrays)
- Excellent support Rust (`rmp-serde`)
- Pas de schéma à compiler (vs Protobuf, qui serait une option v2)

## Format général des messages

Tous les messages WS sont des **enveloppes typées** :

```rust
#[derive(Serialize, Deserialize)]
#[serde(tag = "t", content = "p")]
pub enum ClientMessage {
    Hello(HelloPayload),
    Op(OpPayload),
    Awareness(AwarenessPayload),
    Cursor(CursorPayload),
    RequestSnapshot,
    Ping(u64),
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "t", content = "p")]
pub enum ServerMessage {
    Welcome(WelcomePayload),
    Snapshot(SnapshotPayload),
    Op(OpPayload),
    Awareness(AwarenessPayload),
    PeerJoin(PeerJoinPayload),
    PeerLeave(PeerLeavePayload),
    Error(ErrorPayload),
    Pong(u64),
}
```

Le champ `t` (type) est court pour économiser des octets.

## Handshake d'ouverture

```
   Client                                       Serveur
     │                                            │
     │── WS UPGRADE wss://srv/ws/rooms/{id}?token=<JWT>
     │   subprotocols: bse.v1                    │
     │                                            │
     │── HelloPayload ──────────────────────────►│
     │   { peer_id, client_version, last_seen_op }│
     │                                            │
     │◄── WelcomePayload ────────────────────────│
     │   { server_version, peer_count, my_color }│
     │                                            │
     │◄── SnapshotPayload (si last_seen_op trop ancien)
     │   { crdt_state_v: <binary> }              │
     │                                            │
     │── (boucle normale ops + awareness) ──────►│
     │                                            │
```

### HelloPayload
```rust
struct HelloPayload {
    peer_id: String,         // UUID stable du peer
    client_version: String,  // semver
    last_seen_op: Option<u64>,  // pour resync incrémental après reco
    user_token: Option<String>,  // JWT si pas dans la query
}
```

### WelcomePayload
```rust
struct WelcomePayload {
    server_version: String,
    your_color: (u8, u8, u8),    // couleur assignée pour ce peer
    peers: Vec<PeerSummary>,     // peers déjà présents
    server_time_ms: u64,
}
```

## Op (opération CRDT)

Le **type de message le plus fréquent**. C'est une op CRDT (yrs Update ou Loro encoded op).

```rust
struct OpPayload {
    seq: u64,               // séquence locale du peer émetteur
    bytes: Bytes,           // encoded CRDT op (binaire)
}
```

- **Client → Serveur** : op produite localement
- **Serveur → autres clients** : op broadcastée
- Le **client émetteur ne reçoit pas** son propre op rebroadcast

## Awareness (état éphémère)

L'awareness est l'**état éphémère partagé** non persisté : curseur, sélection, focus.

```rust
struct AwarenessPayload {
    peer_id: String,
    state: AwarenessState,
}

struct AwarenessState {
    cursor: Option<CursorState>,
    selection: Vec<ElementId>,
    user: Option<UserInfo>,    // nom, avatar — envoyé au début
    is_typing: Option<bool>,
    last_active_ms: u64,
}

struct CursorState {
    x: f32,
    y: f32,
    tool: ToolKind,
}
```

Spécificités :
- Envoyé **throttlé** (max 30 Hz, sinon noie le réseau)
- **Pas persisté** côté serveur (in-memory uniquement)
- Sur disconnect, le peer est retiré de l'awareness

## Cursor (cas particulier d'awareness, haute fréquence)

Pour économiser, on a un message séparé pour le curseur uniquement :

```rust
struct CursorPayload {
    x: i32,  // coord monde, sous-pixel encodé en fixed-point
    y: i32,
}
```

- Très petit (16 octets payload)
- Envoyé throttlé à 30 Hz max
- Le serveur diffuse aux autres peers de la room

## Snapshot

Quand un peer rejoint, il reçoit l'état complet :

```rust
struct SnapshotPayload {
    crdt_state: Bytes,        // état CRDT complet encodé
    awareness: Vec<PeerAwareness>,
}
```

Si le peer avait un état antérieur (via `last_seen_op`), il reçoit seulement les ops manquantes.

## PeerJoin / PeerLeave

Notifications de présence :

```rust
struct PeerJoinPayload {
    peer_id: String,
    user: UserInfo,
    color: (u8, u8, u8),
    joined_at_ms: u64,
}

struct PeerLeavePayload {
    peer_id: String,
}
```

## Error

```rust
struct ErrorPayload {
    code: ErrorCode,
    message: String,
}

enum ErrorCode {
    Unauthorized,
    RateLimitExceeded,
    InvalidOp,
    RoomFull,
    InternalError,
}
```

Le serveur peut fermer la connexion après un Error fatal.

## Ping/Pong

```rust
ClientMessage::Ping(timestamp_ms)
ServerMessage::Pong(timestamp_ms)
```

- Ping toutes les **15 s** côté client
- Pong réponse immédiate du serveur
- Si pas de Pong en 30 s → reconnexion

## Anti-patterns de protocole évités

### ❌ JSON pour tout
Trop verbeux pour des envois fréquents (cursor 30 Hz × 10 peers = bcp de bytes).

### ❌ Op par caractère pour le texte
On utilise les ops CRDT natives (batching automatique).

### ❌ Pas d'awareness séparée
Mélanger awareness et ops fragilise tout. On garde le séparé.

### ❌ Compression niveau message
TLS compresse déjà. Pas besoin de double couche.

## Versionning du protocole

- Version dans le sub-protocol : `bse.v1`, `bse.v2`...
- Client annonce sa version dans `HelloPayload`
- Serveur peut rejeter avec `Error::Unauthorized` si incompatible
- Politique de support : N-1 (le serveur supporte la version courante et la précédente)

## Schémas annexes

### Élément (modèle)
```rust
#[derive(Serialize, Deserialize)]
struct Element {
    id: ElementId,
    kind: ElementKind,
    style: ElementStyle,
    transform: Transform,
    layer: i32,
    created_at: u64,
    updated_at: u64,
    created_by: PeerId,
}
```

### Transform (transformation 2D)
```rust
struct Transform {
    x: f32,
    y: f32,
    rotation: f32,  // radians
    scale_x: f32,
    scale_y: f32,
}
```

## Estimations de bande passante

Pour 10 peers actifs en train de dessiner :

| Message | Fréquence | Taille | BW/peer |
|---|---|---|---|
| Cursor | 30 Hz | ~20 octets | 0.6 KB/s |
| Op dessin | ~5 Hz | ~200 octets | 1 KB/s |
| Awareness | ~1 Hz | ~80 octets | 0.08 KB/s |
| **Total émis** | | | **~1.7 KB/s** |
| **Total reçu** (9 autres peers) | | | **~15 KB/s** |

Soit ~120 kbps en réception par peer dans un scénario chargé. Très raisonnable.

## Tests du protocole

- Tests unitaires de sérialisation/désérialisation
- Tests de compatibilité ascendante (v1 lit v1 + v2)
- Tests fuzzing sur l'input (anti-crash)
- Property-based tests sur les round-trips
