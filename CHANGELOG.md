# Changelog

Tous les changements notables de BSE sont documentés ici.

Le format suit [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), et le projet utilise un système de **paliers numérotés** (`v001`, `v002`…) décrit dans [VERSIONING.md](./VERSIONING.md).

---

## [Unreleased]

À venir dans `v009` :
- `bse-crdt::YrsBackend` — code prêt par agent

À venir dans `v011` :
- `bse-storage::SqliteStorage` — code prêt par agent

---

## [v008] — 2026-06-05

### 🌐 Server foundation — axum + WebSocket

Premier serveur HTTP fonctionnel. Routes basiques (`/health`, `/api/info`),
WebSocket echo (`/ws/rooms/:room_id`), graceful shutdown sur Ctrl-C.
Pas encore de logique de room collaborative (CRDT en v009).

### 🎉 Added (par agent en parallèle)

**`bse-server`**
- Modules dédiés (tous < 300 lignes) :
  - `main.rs` : tokio entry, listen + graceful shutdown
  - `lib.rs` : facade pour les tests d'intégration
  - `config.rs` : `ServerConfig::from_env()` (var `BSE_BIND_ADDR`)
  - `routes.rs` : assembly `axum::Router` + TraceLayer + CORS
  - `tracing_setup.rs` : init `tracing` (mirror bse-app)
  - `handlers/health.rs` : `GET /health → {"status":"ok"}`
  - `handlers/info.rs` : `GET /api/info → {name, version, protocol}`
  - `ws/upgrade.rs` : `GET /ws/rooms/:room_id` echo (text+binary)
- 4 tests unitaires + 3 tests d'intégration via `tower::ServiceExt::oneshot`.

### 🛠 Dépendances ajoutées (à `bse-server` uniquement)
- `axum = "0.7"` avec feature `ws`
- `tokio = "1"` features `full`
- `tower-http` (trace + cors)
- `serde`, `serde_json` (workspace)

### ✅ Smoke test manuel
```bash
cargo run -p bse-server &
curl http://localhost:8080/health
# {"status":"ok"}
curl http://localhost:8080/api/info
# {"name":"BSE Server","version":"0.0.2","protocol":"bse.v1"}
```

---

## [v007] — 2026-06-05

### 🌳 Quadtree spatial index + viewport culling

Le moteur ne dessine plus que les éléments visibles dans le viewport.
Sur une scène de 10 000 éléments avec 100 visibles, on passe d'O(N) à
O(log N + K). Le compteur `Elements : N (M visible)` dans la status bar
reflète l'efficacité du culling en temps réel.

### 🎉 Added

**Nouveau crate `bse-spatial`** (porté par agent en parallèle)
- `Quadtree<V>` générique avec API claire : `new` / `insert` / `remove` /
  `query` / `len` / `is_empty` / `clear` / `bounds`.
- Stratégie "straddler" : un bbox qui chevauche plusieurs quadrants reste
  dans le parent (pas de duplication, query dédup naturellement).
- 3 modules ≤ 253 lignes : `lib.rs` (48), `tree.rs` (119), `node.rs` (253).
- **8 tests unitaires** + **3 tests property-based** (proptest) +
  **1 doctest** : tous verts.

**`bse-app`**
- `BseApp.spatial: Quadtree<ElementId>` (bounds `±1_000_000`, 16 items/leaf,
  depth 10).
- `rebuild_spatial()` appelée à chaque frame : reconstruit l'index depuis
  la scène (rebuild incrémental en v007.1).
- `canvas::show()` reçoit `&Quadtree` et retourne le nombre d'éléments
  rendus (utilisé par la status bar).
- `canvas/draw.rs::elements()` : utilise `spatial.query(world_viewport)`
  pour filtrer avant tri Z et rendu.

**`bse-ui`**
- `StatusInfo.visible_count` ajouté.
- Affichage : `Elements : N (M visible)` dans la status bar.

### 🛠 Qualité
- `cargo clippy --workspace --all-targets -- -D warnings` ✅ pedantic
- `cargo fmt --all -- --check` ✅
- `cargo test --workspace` : **85 tests verts**
- Tous les fichiers < 300 lignes

### 📌 Voir le culling en action
```bash
cargo run --release -p bse-app
# Crée 20 rectangles. Zoom et pan : observe "20 (X visible)" varier.
# Au zoom extrême, X tombe à 0 quand on est hors viewport.
```

---

## [v006] — 2026-06-05

### 🖋 Outil Pen — perfect-freehand porté en Rust

Premier outil de dessin libre. L'utilisateur sélectionne **Pen** dans
la toolbar, click + drag dans le canvas, et un tracé pressure-sensible
apparaît, mimant naturellement l'encre. Algorithme `perfect-freehand`
de Steve Ruiz porté en Rust (par un agent en parallèle).

### 🎉 Added

**Nouveau crate `bse-pen`** (porté par agent en parallèle)
- Port complet du module `perfect-freehand` (~795 lignes en 5 modules
  sous le plafond 300 lignes : `options.rs`, `outline.rs`, `stroke_points.rs`,
  `vec.rs`, `lib.rs`).
- Public API minimale :
  - `InputPoint { x, y, pressure }`
  - `StrokeOptions { size, thinning, smoothing, streamline, start_taper,
    end_taper, simulate_pressure }`
  - `get_stroke(points, options) -> Vec<bse_types::Vec2>` (outline polygon)
- **17 tests verts** (16 unit + 1 doctest)
- Aucune dépendance externe hors `bse-types`
- Conventions Rust 2024 + `clippy pedantic` + `cargo fmt`

**`bse-canvas`**
- Nouvelle variante `ToolState::DrawingStroke { points: Vec<InputPoint> }`.
- `bse-pen` ajouté comme dépendance.

**`bse-app`**
- `canvas/input.rs::handle_pen_drag` : gère le cycle drag pour le Pen
  (accumule des points avec pression `0.5` par défaut, commit en élément
  `Pen` à `drag_stopped`).
- `canvas/draw.rs` :
  - `commit_stroke(points)` : produit l'`Element` `Pen` final.
  - `paint_stroke_outline()` : appelle `bse_pen::get_stroke` puis rend
    le polygone via `egui::Shape::convex_polygon` (couleur du `PenStyle`).
  - Le rendu fonctionne aussi bien pour la prévisualisation live que pour
    les strokes commités.

### 🔧 Changed
- `bse-pen` réintégré aux workspace members.

### 🛠 Qualité
- `cargo clippy --workspace --all-targets -- -D warnings` ✅ pedantic
- `cargo fmt --all -- --check` ✅
- `cargo test --workspace` : **73 tests** verts (incluant les agents v008/v009/v011 dont les crates sont déjà compilés mais pas encore taggués)
- Tous les fichiers < 300 lignes

### 📌 Lancer
```bash
cargo run --release -p bse-app
# Cliquer "Pen" dans la toolbar
# Click + drag : un tracé naturel apparaît (perfect-freehand)
```

---

## [v005] — 2026-06-05

### 🟨 Premier outil interactif : Rectangle

L'utilisateur peut maintenant **dessiner** sur le canvas. Sélectionner
l'outil Rectangle (ou Ellipse, Line), drag dans le canvas, et un élément
persistent dans la scène. Brand yellow `#FFD02F` pour le fill, ink `#1C1C1E`
pour le stroke — direct depuis le design system Miro.

### 🎉 Added

**`bse-canvas`**
- `ToolState` enum : `Idle` ou `DrawingShape { anchor_world, current_world }`.
- `CanvasState::set_tool` : switching d'outil réinitialise `ToolState`.
- Réexport de `ToolState` depuis `lib.rs`.

**`bse-app`**
- `Scene` ajoutée à `BseApp` (stockage en mémoire des éléments).
- Nouveau module `canvas/draw.rs` :
  - `elements()` : itère les éléments z-sortés et les peint via `Painter`.
  - `tool_preview()` : peint la forme en cours de drag (preview bleu transparent).
  - `commit_shape()` : convertit un drag fini en `Element` (Rectangle / Ellipse / Line).
  - Helpers `paint_rectangle` / `paint_ellipse` / `paint_line` / `world_to_screen`.
- `canvas/input.rs` : `handle_drawing()` gère le cycle drag_started → dragged → drag_stopped pour les outils de shape.

**`bse-ui`**
- `toolbar` prend maintenant `&mut CanvasState` (au lieu de `&mut ToolKind`) pour invoquer `set_tool` et reset `ToolState`.
- `StatusInfo::element_count` ajouté, affiché dans la status bar.

### 🔧 Changed

- `canvas::show(ui, canvas, scene)` : passe désormais aussi la Scene.
- `bse-app/Cargo.toml` : `bse-crdt` retiré (sera réajouté en v009 avec yrs).
- `Cargo.toml` workspace : `bse-pen` et `bse-spatial` déclarés en `[workspace.dependencies]` mais en `exclude` des members (ajoutés en v006 / v007 quand intégrés).

### 🛠 Quality
- `cargo check -p bse-app` ✅
- `cargo clippy --all-targets -- -D warnings` (sur tous les crates non-agent) ✅ pedantic
- `cargo test` : **32 tests verts**
- Aucun fichier > 300 lignes

### 📌 Lancer
```bash
cargo run --release -p bse-app
# Sélectionner "Rect" dans la toolbar
# Click + drag dans le canvas → un rectangle jaune apparaît
# Switcher d'outil avec Select, Ellipse, Line, etc.
```

---

## [v004] — 2026-06-05

### 🎥 Canvas pan / zoom + grille adaptative

La caméra est désormais pleinement interactive. La fenêtre montre une
grille qui s'adapte au zoom, on peut naviguer dans l'espace infini
avec souris + clavier, et l'origine reste un repère visuel.

### 🎉 Added
- **Restructuration** : `canvas_panel.rs` → module `canvas/` avec sous-modules
  - `panel` : entrée principale (orchestre input + grid + origin marker)
  - `input` : gestion des inputs (pan, zoom)
  - `grid` : grille adaptative (lignes minor + major, snap nice numbers)
- **Pan** :
  - Drag du bouton central de la souris
  - Espace maintenu + drag du bouton primaire (pattern Figma)
- **Zoom** :
  - Molette ancrée sur la position du curseur (factor `1 + scroll * 0.005`)
  - Clampé entre `MIN_ZOOM = 0.05` et `MAX_ZOOM = 50.0`
- **Grille adaptative** :
  - Espacement choisi pour rester ~14 px sur écran (snap à 1, 2, 5 × 10^n)
  - Lignes mineures `#EEF0F3` + lignes majeures (toutes les 5) `#E0E2E8`
- **Origin marker** : croix discrète qui se déplace avec la caméra,
  cachée si hors viewport

### 🔧 Changed
- `crates/bse-app/src/canvas/` remplace `canvas_panel.rs`
- L'origin marker est désormais en coords monde (suit la caméra)

### 🛠 Tests
- 2 nouveaux tests unitaires sur `pick_spacing` (snap nice numbers + monotonie)
- **Total : 32 tests verts**

### 📏 Tailles de fichiers (toujours < 300 lignes)
- `canvas/mod.rs` : 17 lignes
- `canvas/panel.rs` : 48 lignes
- `canvas/input.rs` : 62 lignes
- `canvas/grid.rs` : 155 lignes (le plus long, mais structuré + tests)

### 📌 Lancer en local
```bash
cargo run --release -p bse-app
# Espace + drag souris : pan
# Molette : zoom centré sur le curseur
```

---

## [v003] — 2026-06-05

### 🪟 Fenêtre desktop fonctionnelle

Premier palier avec une vraie fenêtre BSE qui s'ouvre. La fenêtre affiche
une toolbar de sélection d'outils, un canvas central avec background
Miro (surface `#F7F8FA`) et croix d'origine, et une status bar
contenant zoom, FPS, outil actif et état de connexion.

### 🎉 Added
- **`eframe` 0.30** intégré (features : `wgpu`, `default_fonts`, `persistence`)
- **`BseApp`** struct implémentant `eframe::App` avec orchestration
  toolbar / canvas / status bar
- **`bse-ui::toolbar`** : sélection d'outil parmi 6 (Select/Pen/Rectangle/
  Ellipse/Line/Text) avec highlight Miro yellow `#FFD02F` sur l'actif
- **`bse-ui::status_bar`** : version + milestone + outil + zoom + FPS +
  connection indicator (palette Miro)
- **`bse-app::canvas_panel`** : zone centrale avec background, croix
  d'origine, label de phase
- **FPS counter** lissé (EMA α=0.1) dans la status bar
- **Splitting du binaire** : `lib.rs` + `main.rs` (testabilité, `bse_app::run()`)

### 🔧 Changed
- `bse-app` est maintenant à la fois `[[bin]] name = "bse"` ET `[lib]`
- `bse-ui` dépend de `eframe` (avant : aucune UI lib)

### 🛠 Infrastructure
- Toujours `cargo clippy -- -D warnings` propre (pedantic)
- Toujours 30 tests verts
- Release build : 58 s sur Windows 11 / Rust 1.94

### 📌 Lancer en local
```bash
cargo run --release -p bse-app
```

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
