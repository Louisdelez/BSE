# 03.05 — Modèle de données

> Les structures de données fondamentales : Project, Scene, Element, Camera.

## Hiérarchie

```
User
  └── Project (1..N)
        ├── Members (1..N : User × Role)
        ├── Scene (1)
        │     └── Element (0..N)
        │           ├── Pen
        │           ├── Rectangle
        │           ├── Ellipse
        │           ├── Text
        │           ├── Image
        │           ├── Postit
        │           ├── MindmapNode
        │           └── Connector
        ├── Assets (0..N : image, fichier)
        └── Comments (0..N : ancré sur Element)
```

## Project

```rust
pub struct Project {
    pub id: ProjectId,             // UUID v7 (sorted by time)
    pub name: String,
    pub owner_id: UserId,
    pub created_at: u64,           // unix ms
    pub updated_at: u64,
    pub settings: ProjectSettings,
    pub members: Vec<ProjectMember>,
}

pub struct ProjectSettings {
    pub background_color: Color,
    pub grid_visible: bool,
    pub grid_size: f32,
    pub default_template: Option<TemplateId>,
    pub e2e_encryption: bool,      // si activée, room key required
    pub allow_anonymous: bool,
    pub max_members: u32,
}

pub enum Role {
    Owner,
    Facilitator,
    Editor,
    Viewer,
}

pub struct ProjectMember {
    pub user_id: UserId,
    pub role: Role,
    pub added_at: u64,
}
```

## Scene

La scène est l'**état CRDT** d'un projet. Elle contient les éléments.

```rust
pub struct Scene {
    // Représentation interne CRDT
    doc: yrs::Doc,                 // ou loro::LoroDoc
    elements_map: yrs::Map,        // root map des éléments
    layers: yrs::Array,            // ordre des layers
}

impl Scene {
    pub fn add_element(&mut self, el: Element) -> ElementId;
    pub fn update_element(&mut self, id: ElementId, patch: ElementPatch);
    pub fn delete_element(&mut self, id: ElementId);
    pub fn get(&self, id: ElementId) -> Option<&Element>;
    pub fn iter(&self) -> impl Iterator<Item = &Element>;
    pub fn query_viewport(&self, bbox: Rect) -> Vec<&Element>;  // via spatial index
}
```

## Element

```rust
pub struct Element {
    pub id: ElementId,             // UUID v7
    pub kind: ElementKind,
    pub transform: Transform,
    pub style: Style,
    pub z: i32,                    // ordre de superposition (layer)
    pub created_at: u64,
    pub created_by: PeerId,
    pub locked: bool,
    pub tags: Vec<String>,
}

pub enum ElementKind {
    Pen {
        points: Vec<StrokePoint>,
    },
    Rectangle {
        width: f32,
        height: f32,
        corner_radius: f32,
    },
    Ellipse {
        width: f32,
        height: f32,
    },
    Line {
        x2: f32,
        y2: f32,
    },
    Polygon {
        points: Vec<Vec2>,
    },
    Text {
        content: String,
        width: Option<f32>,
        font: FontStyle,
    },
    Image {
        asset_id: AssetId,
        width: f32,
        height: f32,
        crop: Option<Rect>,
    },
    Postit {
        text: String,
        color: PostitColor,
        author_label: Option<String>,
    },
    MindmapNode {
        text: String,
        parent: Option<ElementId>,
        children: Vec<ElementId>,
    },
    Connector {
        from: ConnectorEnd,
        to: ConnectorEnd,
        path: ConnectorPath,
    },
    Group {
        children: Vec<ElementId>,
    },
}
```

## Sub-types importants

### Transform
```rust
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,      // radians
    pub scale_x: f32,
    pub scale_y: f32,
}
```

### Style
```rust
pub struct Style {
    pub stroke: Option<Color>,
    pub stroke_width: f32,
    pub stroke_dash: Option<DashPattern>,
    pub fill: Option<Color>,
    pub opacity: f32,        // 0..1
    pub blend_mode: BlendMode,
}
```

### StrokePoint (pour dessin libre)
```rust
pub struct StrokePoint {
    pub x: f32,
    pub y: f32,
    pub pressure: f32,       // 0..1
    pub tilt_x: f32,
    pub tilt_y: f32,
    pub t_ms: u32,           // timestamp delta depuis début stroke
}
```

### PenStyle
```rust
pub struct PenStyle {
    pub color: Color,
    pub size: f32,            // taille de base
    pub smoothing: f32,       // 0..1
    pub thinning: f32,        // -1..1 (effet pression)
    pub streamline: f32,      // 0..1
    pub start_taper: f32,
    pub end_taper: f32,
}
```

### Connector
```rust
pub struct ConnectorEnd {
    // soit ancré à un élément, soit en coords libres
    pub anchor: ConnectorAnchor,
    pub arrow: ArrowStyle,
}

pub enum ConnectorAnchor {
    Element { id: ElementId, side: Side, offset: f32 },
    Free { x: f32, y: f32 },
}

pub enum ConnectorPath {
    Straight,
    Curved,
    Orthogonal,
}
```

### Color
```rust
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
```

## Camera (état local du client, non persisté)

```rust
pub struct Camera {
    pub x: f32,         // centre du viewport en coords monde
    pub y: f32,
    pub zoom: f32,      // 1.0 = échelle 1:1
    pub viewport_width: f32,    // taille pixels écran
    pub viewport_height: f32,
}

impl Camera {
    pub fn world_to_screen(&self, world: Vec2) -> Vec2;
    pub fn screen_to_world(&self, screen: Vec2) -> Vec2;
    pub fn viewport_world_rect(&self) -> Rect;
}
```

## Awareness (éphémère)

```rust
pub struct PeerAwareness {
    pub peer_id: PeerId,
    pub user: UserInfo,
    pub color: Color,           // assignée par le serveur
    pub cursor: Option<Vec2>,   // en coords monde
    pub camera_focus: Option<Rect>,  // pour "follow user"
    pub selection: Vec<ElementId>,
    pub active_tool: ToolKind,
    pub is_typing: bool,
}

pub struct UserInfo {
    pub user_id: UserId,
    pub display_name: String,
    pub avatar_url: Option<String>,
}
```

## Asset

```rust
pub struct Asset {
    pub id: AssetId,            // sha256 du contenu
    pub mime_type: String,
    pub size: u64,
    pub uploaded_by: UserId,
    pub uploaded_at: u64,
    pub width: Option<u32>,     // si image
    pub height: Option<u32>,
}
```

Stocké en S3 sous `/projects/{pid}/assets/{sha256}`.

## Comment

```rust
pub struct Comment {
    pub id: CommentId,
    pub anchor: CommentAnchor,
    pub author_id: UserId,
    pub content: String,        // markdown
    pub created_at: u64,
    pub thread_parent: Option<CommentId>,
    pub resolved: bool,
}

pub enum CommentAnchor {
    Element { id: ElementId },
    Point { x: f32, y: f32 },
    Region { rect: Rect },
}
```

## Représentation dans le CRDT (yrs)

Avec yrs (Y-CRDT), on mappe :

```
yrs::Doc (root)
├── "elements" : yrs::Map<ElementId, yrs::Map<field, value>>
│        ├── "el_xxx"
│        │    ├── "kind" : yrs::Map { type: "Rectangle", width: 100, ... }
│        │    ├── "transform" : yrs::Map
│        │    ├── "style" : yrs::Map
│        │    └── "z" : i32
│        └── "el_yyy"
│             └── ...
├── "comments" : yrs::Array<yrs::Map>
├── "settings" : yrs::Map
└── "metadata" : yrs::Map
```

Chaque champ d'un élément est un **registre LWW** ou un type CRDT spécifique. Les modifications sont incrementales.

### Cas particulier du texte

Le contenu d'un `Text` ou `Postit` utilise un **yrs::Text** dédié (CRDT de texte). Cela permet l'édition collaborative caractère par caractère sans conflit.

### Cas particulier du dessin

Les `StrokePoint` d'un `Pen` sont stockés en **yrs::Array<StrokePoint>**. Pendant qu'un peer dessine, il append des points ; chaque append est une op CRDT.

**Optimisation** : pour réduire le volume, on bufférise localement et on envoie les points par batchs de 16-32.

## Versioning des éléments

Chaque élément a un `updated_at` mis à jour à chaque op CRDT. Cela permet :
- Détecter ce qui a changé pour le rendu (cache invalidation)
- Afficher « modifié il y a 2 min »
- Filtrer dans la recherche

## Index spatial (côté client)

Maintenu **en parallèle** du CRDT, dérivé de lui (pas dans le CRDT) :

```rust
pub struct SpatialIndex {
    quadtree: Quadtree<ElementId>,
}

impl SpatialIndex {
    pub fn rebuild_from_scene(&mut self, scene: &Scene);
    pub fn update_element(&mut self, id: ElementId, old_bbox: Rect, new_bbox: Rect);
    pub fn query(&self, viewport: Rect) -> Vec<ElementId>;
}
```

Détail dans [../06-CANVAS-INFINI/03-spatial-indexing.md](../06-CANVAS-INFINI/03-spatial-indexing.md).

## Persistence : sérialisation des éléments

- En transit (WS) : CRDT op binaire (yrs Update encode)
- Au repos (Postgres WAL) : même format binaire
- Snapshot S3 : CRDT state binaire encodé
- Export utilisateur : JSON lisible (champ par champ)

## Migrations futures

Pour l'évolution du modèle, on prévoit :
- Champ `model_version` au niveau Project
- Migrations lazy à l'ouverture d'un vieux projet
- Tests de round-trip sur tous les modèles passés
