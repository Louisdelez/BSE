# 04.05 — Base de données

> Postgres serveur + SQLite client + S3-compatible pour les binaires.

## TL;DR

> **Serveur** : PostgreSQL (metadata, WAL, sessions).  
> **Client** : SQLite (cache, snapshots offline, projets locaux).  
> **Binaires** : S3-compatible (MinIO en self-host, ou S3/R2/B2 en cloud).

## Postgres côté serveur

### Pourquoi Postgres ?

- ⭐ **Le couteau suisse** : SQL, JSONB, full-text search, partitioning, listen/notify
- ⭐ **ACID solide**, isolation transactionnelle bien définie
- ⭐ **Réplication mature** (streaming, logical)
- ⭐ **Écosystème Rust excellent** : `sqlx`, `sea-orm`, `diesel`
- ⭐ **Performance suffisante** à notre échelle (1000-10000 projets actifs)

### Pourquoi pas alternative ?

- **MySQL** : OK mais Postgres a plus de features (JSONB, listen/notify)
- **CockroachDB / YugabyteDB** : surdimensionné, complexe à opérer
- **TiKV / FoundationDB** : transactional KV, mais on perd SQL
- **SurrealDB** : intéressant mais trop jeune en 2026 pour BSE prod
- **SQLite** : suffit en mono-instance, mais limites en multi-write
- **Cassandra** : excellent à scale énorme, mais on n'est pas là

### Schéma BSE (cf. [../03-ARCHITECTURE/03-serveur.md](../03-ARCHITECTURE/03-serveur.md))

Tables principales : `users`, `projects`, `project_members`, `wal_entries`, `audit_logs`, `sessions`.

### Choix de client Rust

Comparaison rapide (2026) :

| Lib | Style | Async | Type-safe queries | Bon pour BSE ? |
|---|---|---|---|---|
| **sqlx** | Async, query-builder + macro | ✅ | ✅ (compile-time check) | ✅ **#1** |
| **sea-orm** | ORM | ✅ | ✅ | ⚠️ Plus lourd |
| **diesel** | ORM type-safe | sync (async wrapper) | ✅✅ | ⚠️ Async moins natif |
| **tokio-postgres** | Async raw | ✅ | ❌ | OK pour low-level |

**Choix BSE : `sqlx`**. Bon compromis ergonomie / performance / type-safety.

```rust
let user = sqlx::query_as!(
    User,
    "SELECT id, email, name FROM users WHERE id = $1",
    user_id
)
.fetch_one(&pool)
.await?;
```

### Pool de connexions
- `sqlx::PgPool` avec 10-20 connexions max
- Configurable selon charge
- Timeout par requête : 5 s par défaut

### Migrations
- `sqlx::migrate!` (intégré sqlx)
- Migrations en `migrations/` dans le repo, versionnées
- Run automatique au démarrage du serveur en dev ; opt-in en prod

### Backups
- `pg_dump` daily
- WAL archiving pour PITR (Point-in-Time Recovery)
- Réplica streaming en prod

## SQLite côté client

### Pourquoi SQLite ?

- ⭐ **Zero-config**, fichier unique
- ⭐ **Robuste** : utilisé par tous (Firefox, Chrome, Android, iOS, macOS)
- ⭐ **Excellent support Rust** : `rusqlite`, `sqlx` (avec feature sqlite)
- ⭐ **WAL mode** pour les écritures concurrentes

### Usage côté client BSE

```sql
-- Schéma client simplifié
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    server_url TEXT,                   -- NULL = projet local seulement
    last_opened INTEGER,
    last_sync INTEGER,
    snapshot_blob BLOB,                -- CRDT state binaire
    snapshot_version INTEGER
);

CREATE TABLE assets_cache (
    sha256 TEXT PRIMARY KEY,
    project_id TEXT,
    local_path TEXT NOT NULL,
    size INTEGER,
    last_used INTEGER
);

CREATE TABLE pending_ops (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT NOT NULL,
    op_blob BLOB NOT NULL,
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_pending_ops_project ON pending_ops(project_id, id);
```

### Lib Rust
- **`rusqlite`** : simple, sync, blocage tokio task
- **`sqlx` avec feature sqlite** : async natif
- **Choix BSE** : `sqlx` pour uniformité avec serveur

### Location du fichier
- Windows : `%APPDATA%\BSE\local.db`
- macOS : `~/Library/Application Support/BSE/local.db`
- Linux : `~/.local/share/bse/local.db`

Crate : `directories` pour les chemins standards.

## Binaires : S3-compatible

### Pourquoi S3-compatible ?

Les images, snapshots gros, exports peuvent peser plusieurs MB. Postgres n'est pas optimal pour les blobs (Bytea > 1MB devient lourd).

### Options

| Backend | Self-host | Production-ready | Use case BSE |
|---|---|---|---|
| **MinIO** | ✅ | ✅ | Self-host par défaut |
| **AWS S3** | ❌ | ✅✅ | SaaS Cloud BSE |
| **Cloudflare R2** | ❌ | ✅ | SaaS Cloud BSE (sans egress fees) |
| **Backblaze B2** | ❌ | ✅ | Budget option |
| **SeaweedFS** | ✅ | ✅ | Alternative MinIO |
| **Local filesystem** | ✅ | ⚠️ | Demo / single user |

### Lib Rust : `aws-sdk-s3`
SDK officiel AWS, compatible avec tout backend S3 (incluant MinIO).

```rust
let s3 = aws_sdk_s3::Client::new(&config);

// Upload
s3.put_object()
    .bucket("bse-assets")
    .key(format!("projects/{}/assets/{}", project_id, sha256))
    .body(ByteStream::from(bytes))
    .send()
    .await?;

// Download
let resp = s3.get_object()
    .bucket("bse-assets")
    .key(key)
    .send()
    .await?;
```

### Organisation des clés

```
bse-bucket/
├── projects/
│   ├── {project_id}/
│   │   ├── snapshots/
│   │   │   ├── 000001.crdt
│   │   │   ├── 000002.crdt
│   │   │   └── ...
│   │   └── assets/
│   │       ├── {sha256-1}      (sans extension, MIME en metadata)
│   │       ├── {sha256-2}
│   │       └── ...
└── exports/
    └── {project_id}/
        └── {date}.pdf
```

### Garbage collection
- Snapshots : garder le N derniers (configurable)
- Assets : si plus aucun élément ne référence un asset, GC après 30 jours
- Implémenté en cron job daily

## Stratégie de cache

### Cache client (in-memory)
- Liste des projets : reload à chaque démarrage
- Assets : cache en mémoire + disque (LRU 500 MB)
- Snapshots offline : à jour à chaque sync

### Cache serveur
- Sessions JWT : cache in-memory (DashMap)
- Métadonnées projets actifs : in-memory (room state)
- Pas de Redis en v1 — direct dans la mémoire des room actors

## Sauvegarde / DR

| Composant | Sauvegarde |
|---|---|
| Postgres | pg_dump daily + WAL archiving |
| S3 / MinIO | Versioning activé + lifecycle policy |
| Clients (SQLite) | À la charge de l'utilisateur (laissé local) |

Pour la cible auto-hébergeable, un `bse-backup` CLI sera fourni :
```bash
bse-backup create --dest ./backups/
bse-backup restore --src ./backups/2026-06-05.tar
```

## Migrations futures

Si BSE doit scaler à 100K+ projets actifs :

- Sharding par projet_id
- Read replicas Postgres
- S3 → multi-bucket par tenant
- Réflechir à Citus (sharding Postgres)

C'est très loin de la v1.

## Décisions

| Décision | Choix | Alt envisagée |
|---|---|---|
| DB serveur | PostgreSQL | MySQL, SurrealDB |
| Client DB Rust | sqlx | sea-orm |
| DB client | SQLite | sled, redb |
| Object storage | S3-compatible (MinIO default) | filesystem brut |
| Lib S3 | aws-sdk-s3 | rust-s3 |
| Backup strategy | pg_dump + WAL | logical replication |

## Sources

- *Rust ORMs in 2026: Diesel vs SQLx vs SeaORM vs Rusqlite* — Aarambh Dev Hub
- *Crud-bench: benchmarking embedded and networked DBs* — SurrealDB
