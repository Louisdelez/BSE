# 03.02 — Architecture du client

> Architecture interne de l'application desktop BSE.

## Vue d'ensemble par couches

```
┌─────────────────────────────────────────────────────────────┐
│                       App Layer                              │
│   (main.rs, lifecycle, fenêtres, settings utilisateur)       │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                       UI Layer                               │
│   (egui : panels, toolbars, dialogues, menus contextuels)    │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                     Canvas Layer                             │
│   (Viewport, caméra, sélection, outils, manipulations)       │
└─────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────▼───────────────────────────────┐
│                     State Layer                              │
│   (Scène, éléments, CRDT engine, undo/redo)                  │
└─────────────────────────────────────────────────────────────┘
                              │
┌────────────────┬────────────┴───────────────┬───────────────┐
│  Render Layer  │   Sync Layer               │  Storage Layer│
│  (wgpu/vello)  │   (tokio, ws, msg pack)    │  (sqlite, fs) │
└────────────────┴────────────────────────────┴───────────────┘
```

## Organisation Cargo

```
bse/
├── Cargo.toml                   (workspace)
├── crates/
│   ├── bse-app/                 (binaire desktop, main.rs)
│   ├── bse-ui/                  (composants egui, panels)
│   ├── bse-canvas/              (viewport, outils, manipulation)
│   ├── bse-model/               (types : Element, Scene, Project)
│   ├── bse-crdt/                (wrapper sur yrs/Loro)
│   ├── bse-render/              (renderer wgpu/vello)
│   ├── bse-sync/                (client réseau, tokio, ws)
│   ├── bse-storage/             (sqlite, filesystem)
│   ├── bse-protocol/            (types partagés client/serveur)
│   └── bse-server/              (binaire serveur)
└── shared/
    └── bse-types/               (DTOs communs)
```

## Le cycle de vie d'un frame

À 60 FPS, BSE doit boucler en **≤16,6 ms**. Cycle type :

```
   ┌─►  1. Lire l'input (clavier, souris, stylet)     ~0.2 ms
   │
   │    2. Mettre à jour la caméra / sélection         ~0.1 ms
   │
   │    3. Traiter les events réseau entrants          ~0.5 ms
   │       - Appliquer deltas CRDT à la scène
   │       - Mettre à jour awareness (curseurs distants)
   │
   │    4. Calculer la liste des éléments visibles     ~0.3 ms
   │       (via spatial index + viewport)
   │
   │    5. UI egui (toolbar, panels, menus)            ~1-2 ms
   │
   │    6. Rendu wgpu/vello                            ~3-8 ms
   │       - Background
   │       - Éléments visibles
   │       - Curseurs distants
   │       - Sélection
   │       - Overlays UI egui
   │
   │    7. Présenter le frame                          ~1-3 ms
   └───  8. Émettre les events sortants (sync)         ~0.2 ms
```

Total **typique** : 6-15 ms → 60-144 FPS soutenu.

## Modules détaillés

### `bse-app`
- Point d'entrée
- Initialise wgpu, fenêtre (winit)
- Démarre la boucle principale
- Charge la config utilisateur
- Coordonne les autres modules

### `bse-ui`
- Toolbar de sélection d'outils
- Panel de propriétés (élément sélectionné)
- Sidebar (liste de projets, présence)
- Dialogues (auth, settings, export)
- Tout l'UI **chrome** non-canvas

### `bse-canvas`
- Le widget central : la toile
- Gère la **caméra** (pan, zoom)
- Reçoit les inputs canvas (drag, draw, select)
- Implémente les **outils** : Select, Pen, Rectangle, Ellipse, Text, Image, Postit, Connector
- Coordonne la **sélection** et les **manipulations**

### `bse-model`
- Types fondamentaux : `Element`, `Scene`, `Project`, `Camera`, `Style`
- Type tag enum pour les éléments :
```rust
enum ElementKind {
  Pen { points: Vec<StrokePoint>, style: PenStyle },
  Rectangle { rect: Rect, style: ShapeStyle },
  Ellipse { rect: Rect, style: ShapeStyle },
  Text { content: String, font: FontStyle },
  Image { asset_id: AssetId, transform: Transform },
  Postit { text: String, color: Color },
  MindmapNode { ... },
  Connector { from: ElementId, to: ElementId, style: ConnectorStyle },
}
```

### `bse-crdt`
- Encapsule yrs ou Loro
- Expose une API stable côté `bse-model`
- Permet de switcher d'implémentation si nécessaire
- Gère les transactions CRDT (apply local op + propagation)

### `bse-render`
- Initialise wgpu (device, queue, swap chain)
- Pipelines de rendu : shapes, strokes, text, images
- Cache de textures (images chargées, glyphes texte)
- Stratégie de batching (regrouper draw calls)
- Utilise vello pour le rendu vectoriel complexe (en option)

### `bse-sync`
- Gère la connexion WebSocket avec le serveur
- Sérialise/désérialise les messages (MessagePack)
- Maintient l'état de connexion (connected, reconnecting, offline)
- Bufferise les ops sortantes en cas de déconnexion
- Resync au reconnect (via CRDT)

### `bse-storage`
- SQLite local pour : projets connus, snapshots offline, cache assets
- Filesystem pour : binaires lourds (images), exports

### `bse-protocol`
- Types des messages échangés entre client et serveur
- Versionning du protocole
- Code de sérialisation MessagePack

## Threads et tâches

```
Main thread (UI thread)
├── winit event loop
├── egui rendering
├── wgpu rendering
└── input handling

Tokio runtime (separate threadpool)
├── Sync task
│   ├── WebSocket connection
│   ├── Message serialize/deserialize
│   └── Outbound op queue
├── Storage task
│   ├── SQLite operations
│   └── File I/O
└── Asset loader task
    └── Decode images, GPU upload
```

Communication entre threads :
- **Crossbeam channels** (`crossbeam::channel`) ou **tokio mpsc**
- **Arc<RwLock<Scene>>** pour la scène partagée

## Gestion de l'état

### État local immutable visible au render
```rust
pub struct FrameState {
    pub scene: Arc<Scene>,           // snapshot du frame
    pub camera: Camera,
    pub selection: SelectionState,
    pub peers: HashMap<PeerId, PeerAwareness>,
}
```

### État mutable interne (background)
```rust
pub struct AppState {
    pub crdt_doc: Arc<RwLock<CrdtDoc>>,
    pub sync_client: SyncClient,
    pub storage: Storage,
    pub config: UserConfig,
}
```

Le rendu utilise **un snapshot immutable** par frame. Les modifications passent par les channels.

## Stratégie d'entrée (input handling)

### Hiérarchie de priorité
1. **egui hover** intercepte d'abord (panels, toolbar)
2. **Canvas tool actif** ensuite
3. **Default actions** (pan avec espace, zoom mol roulette)

### Outils
Chaque outil implémente un trait :
```rust
trait Tool {
    fn on_pointer_down(&mut self, ctx: &mut ToolCtx, pos: WorldPos);
    fn on_pointer_move(&mut self, ctx: &mut ToolCtx, pos: WorldPos);
    fn on_pointer_up(&mut self, ctx: &mut ToolCtx, pos: WorldPos);
    fn cursor(&self) -> Cursor;
}
```

## Persistance locale

### SQLite schéma (client)
```sql
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    server_url TEXT,
    last_opened INTEGER,
    snapshot BLOB,            -- CRDT state snapshot
    config TEXT               -- JSON config
);

CREATE TABLE assets_cache (
    id TEXT PRIMARY KEY,
    project_id TEXT,
    sha256 TEXT,
    local_path TEXT,
    size INTEGER,
    last_used INTEGER
);

CREATE TABLE pending_ops (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id TEXT,
    op_blob BLOB,             -- CRDT op pending sync
    created_at INTEGER
);
```

## Comportement offline

1. Connexion serveur tombe → client passe en mode `offline`
2. Toute édition continue à fonctionner localement
3. Les ops s'accumulent dans `pending_ops`
4. Indicateur UI : « 3 modifications en attente de synchro »
5. Reconnexion → flush du buffer dans l'ordre
6. CRDT garantit la convergence avec les ops distantes

## Liens

- Serveur → [03-serveur.md](./03-serveur.md)
- Protocole → [04-protocole-reseau.md](./04-protocole-reseau.md)
- GUI framework → [../04-STACK-TECHNIQUE/02-gui-framework.md](../04-STACK-TECHNIQUE/02-gui-framework.md)
- Rendu → [../04-STACK-TECHNIQUE/03-rendu-canvas.md](../04-STACK-TECHNIQUE/03-rendu-canvas.md)
