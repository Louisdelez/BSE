# Changelog

Tous les changements notables de BSE sont documentés ici.

Le format suit [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), et le projet utilise un système de **paliers numérotés** (`v001`, `v002`…) décrit dans [VERSIONING.md](./VERSIONING.md).

---

## [Unreleased]

À venir dans `v003` :
- Fenêtre desktop avec eframe + wgpu
- Toolbar egui minimale
- Background rendu via wgpu paint callback

---

## [v002] — 2026-06-05

### 🎉 Cargo workspace fonctionnel

Premier palier de code de BSE. Le workspace Rust est entièrement scaffoldé,
compile sans erreur ni warning, passe `cargo clippy -- -D warnings` et
expose 30 tests verts.

### 🛠 Infrastructure
- **Workspace Cargo** avec 11 crates, edition 2024, toolchain pinné à 1.94
- **Profils** : `dev`, `release` (LTO thin, codegen-units 1), `release-fast`
- **Configs** : `rust-toolchain.toml`, `rustfmt.toml`, `clippy.toml`, `.editorconfig`
- **CI GitHub Actions** matrix Windows / macOS / Linux : fmt, clippy, test, doc
- **Dependabot** weekly pour cargo + monthly pour github-actions
- **Templates** PR + issue (bug, feature)

### 🎉 Added — 11 crates
| Crate | Rôle | Lignes |
|---|---|---|
| `bse-types` | Geometry (Vec2, Rect, Transform), IDs typés, Color | ~600 |
| `bse-protocol` | Wire protocol client/server (Client/ServerMessage) | ~150 |
| `bse-model` | Domain : Element, Scene, Camera, Style | ~400 |
| `bse-crdt` | Trait `CrdtBackend` + placeholder `InMemoryBackend` | ~110 |
| `bse-render` | Trait `Renderer` + `NullRenderer` + `FrameStats` | ~75 |
| `bse-canvas` | `CanvasState`, `ToolKind` | ~50 |
| `bse-storage` | Trait `LocalStorage` | ~35 |
| `bse-sync` | `ConnectionState`, `SyncError` | ~45 |
| `bse-ui` | `AppInfo` | ~25 |
| `bse-app` | Binaire `bse` (smoke test v002) | ~45 |
| `bse-server` | Binaire `bse-server` (smoke test v002) | ~25 |

### ✅ Qualité
- `cargo check --workspace` ✓
- `cargo clippy --workspace --all-targets -- -D warnings` ✓ (pedantic enabled)
- `cargo fmt --all -- --check` ✓
- `cargo test --workspace` ✓ — **30 tests verts** (21 dans bse-types, 9 dans bse-model)
- `cargo doc --workspace --no-deps` ✓
- Aucun fichier > 300 lignes
- Documentation `///` sur tous les items publics
- Conventions Rust 2024 idiomatiques (`self` by value sur méthodes `to_*` Copy, etc.)

### 📌 Out of scope (intentionnel)
- `wgpu` / `eframe` arrivent en v003
- `yrs` arrive en v009
- `tokio` / `axum` arrivent en v008
- `sqlx` arrive en v011

### 📚 Documentation
- **Design system Miro adopté** via `npx getdesign@latest add miro`
  - Nouveau fichier `DESIGN.md` à la racine (724 lignes)
  - Nouveau fichier `docs/08-UX-UI/05-design-system.md`

---

## [v001] — 2026-06-05

### 🎉 Initial release — Documentation complète

Premier palier de BSE : la **documentation fondatrice complète** du projet, posant la vision, l'architecture, la stack technique, les features, la sécurité, le déploiement et la roadmap.

### 📚 Documentation

#### Structure
- **70 fichiers Markdown** organisés en **13 dossiers thématiques** dans `/docs`
- Système de navigation hiérarchisé via `README.md` racine + `docs/README.md`
- Convention de nommage cohérente : `NN-NOM/NN-titre.md`

#### Contenu

**00-VUE-ENSEMBLE** (4 fichiers)
- Introduction au projet
- Vision produit (3 paris, 5 valeurs, 5 principes directeurs)
- Glossaire (50+ termes)
- Roadmap macro 24 mois

**01-BRAINSTORMING-RECHERCHE** (6 fichiers)
- Théorie d'Osborn et perte de productivité (Diehl & Stroebe, Mullen)
- 17 techniques (brainstorming classique, brainwriting, 6-3-5, SCAMPER, six chapeaux…)
- Outils existants : Miro, Mural, FigJam, Excalidraw, tldraw, Whimsical
- Facilitation : structure de session 60-90 min, mode facilitateur
- Brainstorming distant et asynchrone
- IA comme partenaire d'idéation

**02-ETAT-DE-LART** (6 fichiers)
- Figma / FigJam : architecture multiplayer + réécriture Rust
- Miro / Mural : enterprise + facilitation
- Excalidraw : pseudo-P2P + E2E encryption
- tldraw : Cloudflare Durable Objects + sync engine
- Tableau comparatif complet
- Positionnement différenciateur BSE

**03-ARCHITECTURE** (6 fichiers)
- Vue d'ensemble client/serveur
- Architecture interne client (8 layers, Cargo workspace 10 crates)
- Architecture serveur (room actors, persistence WAL + checkpoint)
- Protocole réseau (WebSocket + MessagePack, format des messages)
- Modèle de données (Project / Scene / Element / Camera)
- 10 diagrammes ASCII

**04-STACK-TECHNIQUE** (7 fichiers)
- Pourquoi Rust (8 arguments)
- GUI framework : choix egui + wgpu + winit (vs iced, slint, gpui, dioxus)
- Rendu canvas : wgpu pipelines + vello + SDF rendering
- Networking : axum + tokio + tokio-tungstenite (+ iroh futur)
- Base de données : PostgreSQL + SQLite + S3
- Stockage assets : content-addressed SHA-256
- Stack recommandée finale

**05-COLLABORATION-TEMPS-REEL** (6 fichiers)
- Fondamentaux CRDT
- Comparaison yrs / Loro / Automerge / Diamond Types
- Choix BSE : yrs en v1.0, évaluation Loro en v0.5
- Présence et curseurs distants
- Undo / Redo en multi-user
- 18 cas limites et conflits + politique de résolution

**06-CANVAS-INFINI** (6 fichiers)
- Système de coordonnées (world / screen / camera)
- Pan / zoom (smooth, inertie, animations)
- Spatial indexing (Quadtree)
- Culling + LOD (5 niveaux)
- Pipeline de rendu (8 passes wgpu)
- Performance : cibles, budgets, profiling

**07-FEATURES** (8 fichiers)
- Dessin libre (perfect-freehand, pression stylet)
- Formes géométriques (rect, ellipse, ligne, flèche…)
- Texte et typographie (édition CRDT collaborative)
- Images et médias (formats, variants, mipmaps)
- Mindmap et connecteurs intelligents (smart routing)
- Post-its (création rapide, voting, anonyme)
- 15 templates de session
- Export / import (PNG, SVG, PDF, JSON, Excalidraw)

**08-UX-UI** (4 fichiers)
- 7 principes de design
- Toolbar et 11 outils
- Multi-curseurs et présence riche
- 80+ raccourcis clavier

**09-SECURITE** (4 fichiers)
- Authentification (OIDC, JWT, magic link, MFA)
- Permissions RBAC (5 rôles)
- Chiffrement (TLS, E2E optionnel comme Excalidraw)
- Threat model STRIDE (18 menaces analysées)

**10-DEPLOIEMENT** (4 fichiers)
- 4 architectures cibles (standalone, LAN, cloud, SaaS)
- Self-host Docker Compose + Caddy + K8s
- BSE Cloud (vision future)
- Distribution binaires Windows/macOS/Linux + auto-update

**11-ROADMAP-EXECUTION** (4 fichiers)
- MVP 3 mois découpé en 12 sprints
- Jalons v0.1 → v1.0 (12 mois)
- 18 risques identifiés avec mitigations
- Équipe : profils, recrutement, culture

**12-REFERENCES** (4 fichiers)
- 100+ crates Rust catalogués
- 30+ projets open-source à étudier
- 20+ papers académiques (CRDTs, brainstorming, design thinking)
- 80+ liens externes

### 🛠️ Infrastructure du repo

- Repo Git initialisé
- `.gitignore` Rust + standard
- `LICENSE` MIT (Copyright 2026 Louis Delez)
- `README.md` racine (pitch + navigation)
- `VERSIONING.md` (convention v0XX + Conventional Commits)
- `CHANGELOG.md` (ce fichier)

### 📊 Métriques

- **70 fichiers** Markdown
- **~10 000 lignes** de documentation
- **~440 KB** de contenu
- **13 dossiers** thématiques

### 🔗 Liens internes

Toute la doc est interconnectée via liens relatifs Markdown. Point d'entrée : [README.md](./README.md) → [docs/README.md](./docs/README.md).

---

## Format

Pour chaque palier `v0XX`, les sections suivantes sont utilisées si pertinent :

- `🎉 Added` — nouvelles features
- `🔧 Changed` — modifications de comportement
- `🚧 Deprecated` — fonctionnalités vouées à être retirées
- `🗑 Removed` — fonctionnalités retirées
- `🐛 Fixed` — corrections de bugs
- `🔒 Security` — corrections de sécurité
- `⚡ Performance` — améliorations de perf
- `📚 Documentation` — doc seule
- `🛠 Infrastructure` — build, CI, deps
