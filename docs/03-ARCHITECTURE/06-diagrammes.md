# 03.06 — Diagrammes architecture

> Collection des diagrammes ASCII utiles pour visualiser l'architecture.

## D1 — Vue système complète

```
┌──────────────────────────────────────────────────────────────────────────────┐
│  USERS                                                                       │
│                                                                              │
│   Alice (Win)     Bob (macOS)      Charlie (Linux)     Dany (Linux)          │
│      │                │                  │                  │                │
└──────┼────────────────┼──────────────────┼──────────────────┼────────────────┘
       │                │                  │                  │
       ▼                ▼                  ▼                  ▼
   ┌──────┐         ┌──────┐           ┌──────┐           ┌──────┐
   │ BSE  │         │ BSE  │           │ BSE  │           │ BSE  │
   │Client│         │Client│           │Client│           │Client│
   └──┬───┘         └──┬───┘           └──┬───┘           └──┬───┘
      │                │                  │                  │
      └────────────────┼──────────────────┼──────────────────┘
                       │   wss://...      │
                       ▼                  ▼
            ┌──────────────────────────────────────────┐
            │            Reverse Proxy (Caddy/Nginx)    │
            │            TLS termination                │
            └──────────────────┬────────────────────────┘
                               │
                               ▼
            ┌──────────────────────────────────────────┐
            │            BSE Server (Rust)              │
            │  ┌──────┐   ┌──────────┐  ┌──────────┐    │
            │  │REST  │   │WebSocket │  │RoomMgr   │    │
            │  └──────┘   └──────────┘  └──────────┘    │
            └──┬─────────────┬─────────────┬────────────┘
               │             │             │
               ▼             ▼             ▼
        ┌──────────┐ ┌───────────┐  ┌──────────────┐
        │Postgres  │ │ S3/MinIO  │  │  OIDC IdP    │
        │(meta+WAL)│ │(snapshots)│  │ (Google/etc) │
        └──────────┘ └───────────┘  └──────────────┘
```

## D2 — Vue interne client

```
┌────────────────────────────────────────────────────────────────┐
│                       BSE Client Application                    │
│                                                                 │
│  ┌────────────┐                                                 │
│  │  winit     │  (window, events)                               │
│  └─────┬──────┘                                                 │
│        │                                                        │
│        ▼                                                        │
│  ┌────────────┐                                                 │
│  │   egui     │  (top toolbar, side panels, dialogs)            │
│  └─────┬──────┘                                                 │
│        │                                                        │
│        ▼                                                        │
│  ┌────────────┐    ┌──────────────┐    ┌─────────────────┐      │
│  │  Canvas    │◄──►│ Spatial Idx  │◄──►│  Scene (CRDT)   │      │
│  │ widget     │    │ (Quadtree)   │    │   (yrs::Doc)    │      │
│  └─────┬──────┘    └──────────────┘    └────────┬────────┘      │
│        │                                        │               │
│        ▼                                        ▼               │
│  ┌────────────┐                          ┌──────────────┐       │
│  │  wgpu      │                          │  SyncClient  │       │
│  │  Renderer  │                          │  (tokio+WS)  │       │
│  └────────────┘                          └──────┬───────┘       │
│                                                 │               │
│  ┌────────────────────────────────────────────┐ │               │
│  │              SQLite local                  │◄┘               │
│  │  (projects, snapshots offline, asset cache)│                 │
│  └────────────────────────────────────────────┘                 │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

## D3 — Cycle de vie d'une opération d'édition

```
[Alice] Outil "rectangle" actif, drag souris

  1.   Canvas widget capture le drag
       └─► Crée un Element { kind: Rectangle, ... }
       
  2.   Scene::add_element(rect)
       └─► yrs::Doc transaction
       └─► CRDT op générée (Update binaire)
       
  3.   Spatial index : insert(rect.id, bbox)
  
  4.   Renderer marqué dirty → re-render au prochain frame
  
  5.   SyncClient envoie l'op sur WS
       └─► msgpack encode
       └─► tokio_tungstenite::send
       
  6.   Op arrive au Server
       └─► Room actor reçoit
       └─► yrs::Doc.apply_update(op)
       └─► Append au WAL Postgres
       └─► Broadcast aux autres peers
       
  7.   [Bob] reçoit l'op
       └─► Scene::apply_remote_op(op)
       └─► Spatial index update
       └─► Render dirty
       
  8.   [Bob] voit le rectangle apparaître ~50ms après [Alice]
```

## D4 — Reconnexion après offline

```
[Alice] dessine 5 traits offline (réseau coupé)

  Local
  ├── Stroke #1 → CRDT op #1 → pending_ops queue
  ├── Stroke #2 → CRDT op #2 → pending_ops queue
  ├── Stroke #3 → CRDT op #3 → pending_ops queue
  ├── Stroke #4 → CRDT op #4 → pending_ops queue
  └── Stroke #5 → CRDT op #5 → pending_ops queue

[Alice] reconnecte
  
  1. WS handshake avec last_seen_op = 117
  
  2. Server vérifie : « notre WAL est à 125 »
     └─► envoie les ops 118..125 manquantes
     
  3. Client applique ops 118..125 (Bob et Charlie ont édité pendant)
  
  4. CRDT merge sans conflit (par construction)
  
  5. Client flush ses pending_ops 1..5 vers le serveur
     └─► chacune avec un timestamp logique propre
     
  6. Server applique, broadcast aux autres peers
  
  7. Bob/Charlie voient les 5 traits d'Alice apparaître
```

## D5 — Topologie de zoom (caméra + monde infini)

```
        Monde infini (coords float)
        
        +∞ ◄────────────────────────────► +∞
                       y
                       │
                       │
        +∞             │            +∞
         ◄─────────────┼────────────►  x
                       │
                       │
                       ▼
                       │
                  ┌────────────┐
                  │            │
                  │  Viewport  │   ◄── caméra (x, y, zoom)
                  │ (visible)  │
                  │            │
                  └────────────┘
                       │
                       ▼
                  ┌──────────────────┐
                  │ Spatial query    │
                  │ → Quadtree node  │
                  │ → Elements visi- │
                  │   bles à dessiner│
                  └──────────────────┘
```

## D6 — Pipeline de rendu

```
                  ┌─────────────────┐
                  │ Liste éléments  │
                  │ visibles        │
                  └────────┬────────┘
                           │
                  ┌────────▼────────┐
                  │ Sort par layer  │
                  │ + type batching │
                  └────────┬────────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
        ┌─────────┐  ┌──────────┐  ┌──────────┐
        │Shape    │  │Stroke    │  │Image     │
        │batches  │  │batches   │  │batches   │
        └────┬────┘  └─────┬────┘  └─────┬────┘
             │             │             │
             └─────────────┼─────────────┘
                           ▼
                    ┌────────────┐
                    │ wgpu draw  │
                    │ calls      │
                    └─────┬──────┘
                          │
                          ▼
                    ┌────────────┐
                    │ Compose    │
                    │ UI egui    │
                    └─────┬──────┘
                          │
                          ▼
                    ┌────────────┐
                    │  Present   │
                    └────────────┘
```

## D7 — Architecture des rooms côté serveur

```
                   ┌─────────────────────────┐
                   │   Axum + tokio          │
                   │   (event loop unique)   │
                   └────────────┬────────────┘
                                │
                ┌───────────────┼───────────────┐
                │               │               │
                ▼               ▼               ▼
        ┌─────────────┐  ┌─────────────┐  ┌─────────────┐
        │  Room A     │  │  Room B     │  │  Room C     │
        │  (task)     │  │  (task)     │  │  (task)     │
        │             │  │             │  │             │
        │ select! {   │  │ select! {   │  │ select! {   │
        │  ws msgs    │  │  ws msgs    │  │  ws msgs    │
        │  peer join  │  │  peer join  │  │  peer join  │
        │  checkpoint │  │  checkpoint │  │  checkpoint │
        │ }           │  │ }           │  │ }           │
        └──────┬──────┘  └──────┬──────┘  └──────┬──────┘
               │                │                │
               └────────────────┼────────────────┘
                                ▼
                        ┌──────────────┐
                        │ Persistence  │
                        │ (PG pool +   │
                        │  S3 client)  │
                        └──────────────┘
```

## D8 — Stack de chiffrement E2E (mode optionnel)

```
┌──────────────────┐        ┌──────────────────┐
│  Alice client    │        │  Bob client      │
│                  │        │                  │
│  room_key (AES)  │        │  room_key (AES)  │
│  (shared in URL) │        │  (shared in URL) │
└────────┬─────────┘        └────────▲─────────┘
         │                           │
         │ encrypt(op, room_key)     │ decrypt(op, room_key)
         │                           │
         ▼                           │
  ┌──────────────┐  cipher  ┌──────────────┐
  │              │ ──────►  │   Server     │
  │              │          │              │
  │              │  cipher  │  (NE PEUT    │
  │              │ ◄──────  │   PAS LIRE)  │
  └──────────────┘          └──────────────┘
         │                           │
         │                           │
         │   relay only              │
         └───────────────────────────┘
```

## D9 — Modèle de menace simplifié

```
                ┌─────────────────────────────────┐
                │       Threats                   │
                └─────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        ▼                     ▼                     ▼
   ┌─────────┐         ┌─────────────┐       ┌─────────────┐
   │ External│         │ Insider     │       │ Hostile     │
   │ attacker│         │ (rogue user)│       │ Server      │
   └────┬────┘         └──────┬──────┘       └──────┬──────┘
        │                     │                     │
        │                     │                     │ (mitigated by E2E)
        ▼                     ▼                     ▼
   ┌─────────┐         ┌─────────────┐       ┌─────────────┐
   │ TLS     │         │ RBAC        │       │ Room key    │
   │ JWT auth│         │ Audit logs  │       │ never sent  │
   │ Rate    │         │ Rate limit  │       │ to server   │
   │ limit   │         │             │       │             │
   └─────────┘         └─────────────┘       └─────────────┘
```

## D10 — Topologie multi-instance (v2 scaling)

```
                ┌──────────────┐
                │   Clients    │
                └──────┬───────┘
                       │
                       ▼
                ┌──────────────┐
                │ Load Balancer│
                │  +Redis lkup │
                └──────┬───────┘
                       │ sticky session par room_id
              ┌────────┼────────┐
              ▼        ▼        ▼
        ┌────────┐ ┌────────┐ ┌────────┐
        │Server 1│ │Server 2│ │Server 3│
        │ rooms  │ │ rooms  │ │ rooms  │
        │ A,B,D  │ │ C,F    │ │ E,G,H  │
        └───┬────┘ └───┬────┘ └───┬────┘
            │          │          │
            └──────────┼──────────┘
                       ▼
            ┌────────────────────────┐
            │ Postgres HA (primary + │
            │ replica)               │
            └────────────────────────┘
            ┌────────────────────────┐
            │ S3 distributed         │
            └────────────────────────┘
```
