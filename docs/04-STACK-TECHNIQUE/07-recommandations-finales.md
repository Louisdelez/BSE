# 04.07 — Stack BSE recommandée

> Récapitulatif synthétique de toutes les décisions techniques.

## Le tableau récapitulatif

| Couche | Choix |
|---|---|
| Langage | **Rust** (client + serveur) |
| Build | **Cargo** (workspace multi-crates) |
| OS cibles | Windows 10+, macOS 12+, Linux (X11+Wayland), WASM (v2) |

### Client

| Composant | Crate | Version cible |
|---|---|---|
| App framework | `eframe` | latest |
| UI | `egui` | latest |
| Fenêtre / OS events | `winit` (via eframe) | latest |
| Rendu GPU | `wgpu` | latest stable |
| Rendu vectoriel complexe | `vello` | v0.5+ |
| Texte | `glyphon` | latest |
| Image décodage | `image` | latest |
| WebSocket client | `tokio-tungstenite` | latest |
| Async runtime | `tokio` | latest |
| CRDT | `yrs` ou `loro` | latest (cf [05-COLLABORATION/03](../05-COLLABORATION-TEMPS-REEL/03-choix-bse.md)) |
| Sérialisation | `serde` + `rmp-serde` | latest |
| DB locale | `sqlx` (sqlite) | latest |
| Math 2D | `glam` | latest |
| Triangulation | `lyon_tessellation` | latest |
| Dessin pression | `perfect-freehand` (port Rust) ou implem maison | latest |
| Logs | `tracing` | latest |
| Configuration | `serde` + TOML | - |
| Path système | `directories` | latest |
| HTTP client | `reqwest` | latest |
| OAuth | `oauth2` | latest |
| Crypto | `ring` ou `rustcrypto/aes-gcm` | latest |
| Tests | `cargo test` + `proptest` | - |

### Serveur

| Composant | Crate |
|---|---|
| HTTP framework | `axum` |
| Async runtime | `tokio` |
| WebSocket | `axum::extract::ws` (basé sur tokio-tungstenite) |
| DB client | `sqlx` (postgres) |
| CRDT serveur | `yrs` (même version que client !) |
| S3 client | `aws-sdk-s3` |
| Sérialisation | `serde` + `rmp-serde` |
| Auth JWT | `jsonwebtoken` |
| OAuth | `oauth2` |
| Password hashing | `argon2` |
| Logs structurés | `tracing` + `tracing-subscriber` |
| Métriques | `metrics` + `metrics-exporter-prometheus` |
| Traces | `tracing-opentelemetry` |
| Migrations | `sqlx::migrate!` |
| Rate limiting | `tower::limit` + maison |
| TLS | `rustls` |
| Config | `figment` ou `config` |
| CORS | `tower-http::cors` |

## Crates "partagés" (workspace)

Dans `crates/bse-protocol/` et `crates/bse-types/` :
- Types des messages WS
- Modèles de données partagés
- Constantes / version du protocole

```toml
# workspace Cargo.toml
[workspace]
members = [
  "crates/bse-app",
  "crates/bse-ui",
  "crates/bse-canvas",
  "crates/bse-model",
  "crates/bse-crdt",
  "crates/bse-render",
  "crates/bse-sync",
  "crates/bse-storage",
  "crates/bse-protocol",
  "crates/bse-server",
]

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
rmp-serde = "1"
tracing = "0.1"
anyhow = "1"
thiserror = "1"
uuid = { version = "1", features = ["v4", "v7", "serde"] }
```

## Tooling

### Dev local
- **rustup** stable
- **cargo-watch** (recompile auto)
- **cargo-nextest** (test runner rapide)
- **cargo-deny** (audit licences + vuln)
- **cargo-machete** (deps inutilisées)
- **mold** ou **sccache** (linker rapide)

### CI
- **GitHub Actions** (free pour OSS)
- Matrix : Linux x86_64, macOS arm64, Windows x86_64
- Jobs : `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo nextest run`, `cargo deny check`
- Cache : `actions/cache` + `Swatinem/rust-cache`

### Container
- **Multi-stage Dockerfile** pour le serveur
- Base : `rust:1.84-slim` → build → `gcr.io/distroless/cc-debian12` runtime
- Image finale : ~30 MB

### Versionning
- **SemVer** pour les releases publiques
- **CalVer** sur les images Docker (`YYYY.MM.DD-sha`)

### Packaging client

| Plateforme | Outil | Format |
|---|---|---|
| Windows | `cargo-wix` ou `tauri-bundler` | MSI / EXE |
| macOS | `cargo-bundle` + `codesign` | .app + DMG |
| Linux | `cargo-deb`, `cargo-rpm`, AppImage | .deb, .rpm, .AppImage, Flatpak |

### Auto-update
- `cargo-dist` orchestre les builds
- Update via `self_update` crate côté client (opt-in)

## Coûts d'infrastructure estimés (mode SaaS BSE Cloud)

Pour 1000 utilisateurs actifs, 100 projets actifs simultanés :

| Service | Type | Coût mensuel |
|---|---|---|
| Serveur BSE | VPS 8 vCPU / 16 GB / 200 GB | ~80 € |
| Postgres managé | HA, 100 GB | ~100 € |
| S3 (R2) | 500 GB stockage + 1 TB egress | ~25 € |
| Reverse proxy / TLS | Caddy + LE | gratuit |
| Monitoring | Grafana Cloud free + UptimeRobot | ~0-30 € |
| Logs | Loki self-host ou Better Stack | ~20 € |
| **Total** | | **~230 €/mois** |

Soit ~0.23 €/user actif. Très soutenable.

## Pour le contributeur

### Setup en 5 minutes
```bash
# Prérequis : rustup, git, docker compose
git clone https://github.com/bse-app/bse
cd bse
cargo build --release
# Lance le serveur en local
docker compose -f deploy/dev.yml up -d
# Lance le client
cargo run -p bse-app
```

### Couverture
- Workflow VS Code / Rust Analyzer
- Hot reload partiel via `cargo-watch -x run`

## Non-décisions (à trancher en cours de route)

| Sujet | Status | Date butoir |
|---|---|---|
| yrs vs Loro | À benchmarker au MVP | Fin M2 |
| vello vs custom SDF only | À évaluer en v0.5 | Début M5 |
| OIDC providers à intégrer en priorité | À sonder | Avant v0.1 |
| Cross-compilation cible iOS/Android | Hors v1 | v2 |

## Liens vers détails

| Sujet | Doc |
|---|---|
| Pourquoi Rust | [01-rust-pourquoi.md](./01-rust-pourquoi.md) |
| GUI framework | [02-gui-framework.md](./02-gui-framework.md) |
| Rendu | [03-rendu-canvas.md](./03-rendu-canvas.md) |
| Networking | [04-networking.md](./04-networking.md) |
| DB | [05-base-donnees.md](./05-base-donnees.md) |
| Assets | [06-stockage-assets.md](./06-stockage-assets.md) |
| CRDT | [../05-COLLABORATION-TEMPS-REEL/03-choix-bse.md](../05-COLLABORATION-TEMPS-REEL/03-choix-bse.md) |
| Liste exhaustive crates | [../12-REFERENCES/01-crates-rust.md](../12-REFERENCES/01-crates-rust.md) |
