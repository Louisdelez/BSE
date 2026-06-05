# 03.03 — Architecture du serveur

> Le backend BSE : axum + tokio + Postgres + S3.

## Vue d'ensemble

```
                  ┌──────────────────────────────┐
                  │       BSE Server (Rust)      │
                  │                              │
   HTTPS / WSS ──►│  ┌────────────────────────┐  │
                  │  │   Axum router          │  │
                  │  │   (REST + WS routes)   │  │
                  │  └────────┬───────────────┘  │
                  │           │                  │
                  │  ┌────────▼───────────────┐  │
                  │  │   Auth middleware      │  │
                  │  │   (JWT verification)   │  │
                  │  └────────┬───────────────┘  │
                  │           │                  │
                  │  ┌────────┴──────────────┐   │
                  │  │                       │   │
                  │  ▼                       ▼   │
                  │ ┌──────┐         ┌─────────┐ │
                  │ │ REST │         │ Gateway │ │
                  │ │ Hdlrs│         │   WS    │ │
                  │ └──┬───┘         └────┬────┘ │
                  │    │                  │      │
                  │    └─────────┬────────┘      │
                  │              ▼               │
                  │      ┌───────────────┐       │
                  │      │ Room Manager  │       │
                  │      │ (Arc<Map>)    │       │
                  │      └──────┬────────┘       │
                  │             │                │
                  │   ┌─────────┼─────────┐      │
                  │   ▼         ▼         ▼      │
                  │ ┌────┐    ┌────┐    ┌────┐   │
                  │ │ R1 │    │ R2 │... │ Rn │   │
                  │ └─┬──┘    └─┬──┘    └─┬──┘   │
                  │   │         │         │      │
                  │   └─────────┼─────────┘      │
                  │             │                │
                  │      ┌──────▼────────┐       │
                  │      │ Persistence   │       │
                  │      │ Layer         │       │
                  │      └──────┬────────┘       │
                  └─────────────┼────────────────┘
                                │
                  ┌─────────────┴───────────────┐
                  ▼                             ▼
            ┌─────────┐                  ┌──────────────┐
            │Postgres │                  │   S3/MinIO   │
            │(meta+   │                  │  (snapshots, │
            │ users + │                  │   images)    │
            │ rooms + │                  │              │
            │ WAL)    │                  │              │
            └─────────┘                  └──────────────┘
```

## Composants principaux

### 1. Axum router
Le routeur HTTP. Gère :
- `/health` — healthcheck
- `/api/auth/...` — auth (login, refresh, OAuth callback)
- `/api/projects/...` — CRUD projets
- `/api/projects/{id}/assets/...` — upload d'assets
- `/api/me` — profil utilisateur
- `/ws/rooms/{id}` — upgrade WebSocket vers une room

### 2. Auth middleware
Vérifie le JWT sur les routes protégées :
- Extrait le bearer token de `Authorization:`
- Vérifie signature (clé publique ou secret partagé)
- Charge l'utilisateur en cache
- Injecte `User` dans les extensions de la request

### 3. Gateway WebSocket
Sur `/ws/rooms/{id}` :
1. Vérifie auth (JWT dans query param ou subprotocol)
2. Vérifie permission sur le projet
3. Upgrade HTTP → WS
4. Récupère ou crée la *Room actor*
5. Lui transmet le nouveau peer

### 4. Room Manager
- HashMap globale : `Arc<DashMap<ProjectId, RoomHandle>>`
- À l'arrivée du premier peer d'un projet : crée la Room actor
- Au départ du dernier peer : démarre un *grace period* (5 min) puis dispose la Room
- Gère les limites (max rooms, max peers/room)

### 5. Room actor (1 par projet actif)
C'est le **cœur du système**. Une tokio task dédiée par room :

```rust
pub struct Room {
    project_id: ProjectId,
    crdt_doc: yrs::Doc,
    peers: HashMap<PeerId, PeerConnection>,
    awareness: Awareness,
    persistence: PersistenceHandle,
}

impl Room {
    async fn run(mut self) {
        loop {
            select! {
                // Nouveau message d'un peer
                msg = self.next_peer_message() => self.handle_msg(msg).await,
                // Nouveau peer arrive
                peer = self.peer_join_rx.recv() => self.add_peer(peer),
                // Timer de checkpoint
                _ = checkpoint_interval.tick() => self.persist_checkpoint().await,
                // Shutdown
                _ = self.shutdown_signal => break,
            }
        }
    }
}
```

Responsabilités :
- Appliquer les ops CRDT entrantes au document partagé
- Broadcaster les ops à tous les autres peers
- Maintenir l'awareness (curseurs, sélections)
- Persister périodiquement (checkpoint)
- Journaliser les ops (WAL)

### 6. Persistence Layer

#### Postgres (méta-données)
```sql
-- Schéma simplifié
CREATE TABLE users (
    id UUID PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    name TEXT,
    avatar_url TEXT,
    auth_provider TEXT,         -- 'local', 'google', 'github'...
    created_at TIMESTAMPTZ NOT NULL,
    last_login TIMESTAMPTZ
);

CREATE TABLE projects (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    owner_id UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    snapshot_key TEXT,           -- clé S3 du snapshot le plus récent
    snapshot_version BIGINT,
    settings JSONB
);

CREATE TABLE project_members (
    project_id UUID REFERENCES projects(id),
    user_id UUID REFERENCES users(id),
    role TEXT NOT NULL,          -- 'owner', 'editor', 'viewer', 'facilitator'
    added_at TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (project_id, user_id)
);

CREATE TABLE wal_entries (
    id BIGSERIAL PRIMARY KEY,
    project_id UUID REFERENCES projects(id),
    op_blob BYTEA NOT NULL,      -- CRDT op
    peer_id TEXT,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_wal_project_time ON wal_entries(project_id, id);
```

#### S3 / MinIO (snapshots + assets)
```
/projects/{project_id}/snapshots/{version}.crdt   (CRDT state binaire)
/projects/{project_id}/assets/{sha256}            (image, fichier)
```

## Stratégie de persistance — WAL + checkpoint

Inspiré de Postgres et Figma :

1. **WAL (Write-Ahead Log)** : chaque op CRDT est *append* dans Postgres immédiatement.
2. **Checkpoint** : périodiquement (toutes les 5 min ou 1000 ops), on persiste un *snapshot complet* du CRDT en S3.
3. **Truncation** : les entrées WAL antérieures au checkpoint peuvent être tronquées (ou conservées pour historique).

Avantage : durabilité forte (chaque op est durable) + reconstruction rapide (snapshot + tail WAL).

```
État au temps T = Checkpoint(T') + Σ(WAL entries entre T' et T)
```

## Gestion de la connexion

### Cycle de vie d'un peer
```
1. Connexion WS
2. Handshake : send auth + project_id
3. Server vérifie permission
4. Server envoie initial state (current snapshot)
5. Client envoie ses pending ops accumulées offline
6. Boucle send/receive
7. Disconnect → cleanup awareness, garder ops persistées
```

### Limites par room
- Max peers/room : configurable, défaut **50** (cible 100)
- Max ops/sec/peer : rate-limited à 100 (anti-spam)
- Max payload size : 1 MB par message

## Multi-instance et scaling horizontal

### Single instance (v1)
- Toutes les rooms dans 1 processus
- Suffit pour ~100 rooms simultanées sur machine moderne

### Multi-instance (v2)
Pour scaler :
- **Sticky session par room** : un projet est routé toujours sur la même instance
- Routage via Redis lookup (`room_id → instance_id`)
- Migration de room : sauvegarder l'état, transférer

### Géo-distribution (v3+)
- Plusieurs régions
- Réplication async des projets
- Router le client vers la région la plus proche du projet propriétaire

## Sécurité serveur

- **TLS obligatoire** en production (rustls + axum-server)
- **Auth JWT** sur toutes les routes API
- **Rate limiting** par IP et par utilisateur
- **CORS** strict
- **Validation** des inputs (taille, format)
- **Audit logs** des actions sensibles (création/suppression projet, partage)

Détail dans [../09-SECURITE/](../09-SECURITE/).

## Configuration serveur

```toml
# bse-server.toml
[server]
bind = "0.0.0.0:8080"
tls_cert = "/etc/bse/cert.pem"
tls_key = "/etc/bse/key.pem"

[database]
postgres_url = "postgres://bse:..."
pool_size = 20

[storage]
s3_endpoint = "https://minio.local"
s3_bucket = "bse"
s3_access_key = "..."
s3_secret_key = "..."

[auth]
jwt_secret = "..."  # ou jwt_public_key_path pour RS256
oidc_issuer = "https://accounts.google.com"
oidc_client_id = "..."
oidc_client_secret = "..."

[limits]
max_rooms = 1000
max_peers_per_room = 50
max_ops_per_sec_per_peer = 100
```

## Observabilité

- **Logs structurés** (`tracing` + JSON output)
- **Métriques Prometheus** exposées sur `/metrics`
  - `bse_active_rooms` (gauge)
  - `bse_active_peers` (gauge)
  - `bse_ops_per_second` (counter)
  - `bse_ws_latency` (histogram)
- **Traces OTLP** (tracing-opentelemetry)

## Tests

- **Unitaires** : chaque module
- **Intégration** : faire tourner serveur + DB, scénarios CRUD
- **Load testing** : `k6` ou `vegeta`, simuler 100 peers / 10 rooms
- **Soak tests** : 24 h continues à charge modérée

## Liens

- Protocole réseau → [04-protocole-reseau.md](./04-protocole-reseau.md)
- Modèle de données → [05-modele-donnees.md](./05-modele-donnees.md)
- Auth → [../09-SECURITE/01-authentification.md](../09-SECURITE/01-authentification.md)
- Déploiement → [../10-DEPLOIEMENT/](../10-DEPLOIEMENT/)
