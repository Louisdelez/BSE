# 12.01 — Crates Rust utiles

> Liste exhaustive des dépendances pour BSE, classées par usage.

## Client app

### App framework
- `eframe` — boilerplate desktop autour d'egui
- `egui` — immediate-mode UI
- `egui-wgpu` — backend wgpu pour egui

### Fenêtre / OS
- `winit` — fenêtre cross-platform
- `directories` — chemins standards par OS
- `keyring` — secure credential storage (OS keychain)
- `arboard` — clipboard cross-platform
- `open` — ouvrir URL dans browser
- `rfd` — file dialogs natifs

### Rendu GPU
- `wgpu` — abstraction GPU portable
- `bytemuck` — cast safe pour buffers GPU
- `glam` — math vec2/vec3/mat4

### Rendu 2D haut niveau
- `vello` — renderer vectoriel GPU-compute (alpha → v0.5)
- `lyon` / `lyon_tessellation` — tessellation de paths
- `tiny-skia` — fallback CPU 2D rasterizer

### Texte
- `glyphon` — text rendering atlas-based (wgpu)
- `cosmic-text` — text shaping + layout
- `unicode-segmentation` — graphème clusters

### Images
- `image` — décodage / encodage standard
- `resvg` / `usvg` — SVG parsing et rendering
- `webp` — encodage WebP
- `imageproc` — image manipulation

### Math
- `glam` — game math vec/mat
- `euclid` — types géométriques (rect, point, transform)
- `palette` — color space

### Dessin libre
- `perfect-freehand-rs` (port maison) — stroke pression-sensible

## Networking

### HTTP / WS
- `axum` — framework HTTP
- `tokio` — async runtime
- `tokio-tungstenite` — WebSocket client/serveur
- `tower` — middleware tower
- `tower-http` — middlewares HTTP (CORS, trace, compression)
- `hyper` — couche bas niveau (utilisée par axum)
- `reqwest` — HTTP client (côté client BSE pour OIDC etc.)

### P2P (futur)
- `iroh` — P2P QUIC avec NAT traversal
- `iroh-base`, `iroh-blobs`, `iroh-docs`, `iroh-gossip`
- `libp2p` (alternative)

### Serialization
- `serde`, `serde_derive` — fundamentals
- `serde_json` — JSON
- `rmp-serde` — MessagePack
- `bincode` — binary encoding
- `serde_bytes` — bytes optimization

### TLS
- `rustls` — TLS implementation pure Rust
- `tokio-rustls` — tokio integration
- `rustls-pemfile` — parsing certs

## CRDT

### Principal
- `yrs` — Y-CRDT en Rust (Yjs port)
- `loro` — alternative moderne
- `automerge` — alternative versioning Git-like

### Auxiliaires
- `diamond-types` — CRDT texte ultra-rapide (référence perf)

## Stockage

### Bases de données
- `sqlx` — async SQL (Postgres + SQLite)
- `sea-orm` — ORM alternative
- `rusqlite` — SQLite sync

### S3
- `aws-sdk-s3` — SDK officiel AWS, compatible MinIO/R2/etc.
- `aws-config` — credentials et config

### Filesystem
- `std::fs` + `tokio::fs`
- `walkdir` — itération récursive
- `notify` — file system events (watch)

### Caches
- `lru` — LRU cache map
- `moka` — concurrent cache (async-friendly)
- `dashmap` — concurrent HashMap

## Auth & sécurité

### OAuth / OIDC
- `oauth2` — flow OAuth2 standard
- `openidconnect` — OIDC sur top d'oauth2

### JWT
- `jsonwebtoken` — encode/décode JWT (HS256, RS256, ES256)

### Hashing
- `argon2` — password hashing (recommandé)
- `bcrypt` — alternative classique
- `ring` — crypto primitives (alternative)

### Crypto symétrique
- `aes-gcm` — AES-GCM authenticated encryption
- `chacha20poly1305` — alternative ChaCha20-Poly1305

### Random
- `rand` — random standard
- `rand::rngs::OsRng` — crypto-secure

### Time
- `chrono` — date/time + timezone
- `time` — alternative moderne, no_std

## Logging & observabilité

- `tracing` — structured logging
- `tracing-subscriber` — backends (stdout, file, JSON)
- `tracing-appender` — file rotation
- `tracing-opentelemetry` — OTLP traces export
- `metrics` — metric facade
- `metrics-exporter-prometheus` — Prometheus
- `opentelemetry` — base OTLP

## Configuration

- `figment` — config polyvalent (TOML, env, defaults)
- `config` — alternative
- `dotenvy` — `.env` files

## CLI

- `clap` — argument parser standard
- `clap_derive` — derive support

## UUIDs et identifiants

- `uuid` — UUID v4, v7
- `ulid` — alternative sortable

## Tests

- `cargo-nextest` — runner rapide
- `proptest` — property-based testing
- `quickcheck` — alternative
- `criterion` — benchmarks rigoureux
- `mockall` — mocks
- `pretty_assertions` — assertions lisibles
- `insta` — snapshot testing

## Error handling

- `anyhow` — pour les applications
- `thiserror` — pour les bibliothèques (custom errors)
- `eyre` — alternative anyhow plus riche

## Async helpers

- `futures` — utilities futures
- `async-trait` — pour trait async (until stabilized)
- `pin-project` — pin projection
- `tokio-util` — bonus utilities (codecs, sync)
- `tokio-stream` — Stream sur tokio

## Concurrence

- `parking_lot` — Mutex/RwLock plus rapides
- `crossbeam` — channels lock-free
- `rayon` — parallélisme data

## Sérialisation custom

- `flatbuffers` — alternative MessagePack/Protobuf
- `prost` — Protobuf
- `borsh` — sérialisation déterministe

## Performance / profiling

- `dhat` — heap profiling
- `puffin` — profiler intégré UI
- `tracy-client` — Tracy profiler
- `cpuprofiler` — CPU sampling
- `mimalloc` ou `tikv-jemallocator` — allocateurs alternatifs

## Build / dev

- `cargo-watch` — recompile auto
- `cargo-deny` — audit licences + vulns
- `cargo-audit` — check vulnérabilités RustSec
- `cargo-machete` — deps inutilisées
- `cargo-dist` — release pipeline
- `cargo-bundle` / `cargo-deb` / `cargo-rpm` — packaging
- `cargo-wix` — Windows MSI
- `sccache` — compile cache

## Tracking

À mettre à jour régulièrement. Vérifier `cargo outdated` mensuel.

## Liens

- Stack recommandée → [../04-STACK-TECHNIQUE/07-recommandations-finales.md](../04-STACK-TECHNIQUE/07-recommandations-finales.md)
- Projets open-source → [02-projets-open-source.md](./02-projets-open-source.md)
