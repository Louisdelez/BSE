# 07.02 — Formes géométriques

> Rectangles, ellipses, lignes, polygones, flèches.

## Liste des formes (v1.0)

| Forme | Outil | Raccourci |
|---|---|---|
| Rectangle | R | R |
| Ellipse | O | O |
| Ligne | L | L |
| Flèche | A | A |
| Polygone régulier (n côtés) | P | (sous-menu) |
| Étoile | (sous-menu) | - |
| Triangle | (raccourci P=3) | - |

## Création

### Click and drag
- Mousedown → définit le coin haut-gauche
- Drag → étend
- Mouseup → finalise

### Modifiers
- **Shift maintenu** : forme contrainte (carré pour rect, cercle pour ellipse, ligne droite)
- **Alt maintenu** : forme centrée sur le point de départ
- **Shift + Alt** : combinaison

### Click sans drag
Crée une forme de taille par défaut (100×100 par exemple) au point de click.

## Propriétés communes

### Style
```rust
pub struct ShapeStyle {
    pub fill: Option<Color>,
    pub stroke: Option<Color>,
    pub stroke_width: f32,
    pub stroke_dash: Option<DashPattern>,
    pub corner_radius: f32,  // pour rect
    pub opacity: f32,
}

pub enum DashPattern {
    Solid,
    Dashed,    // 8-4
    Dotted,    // 2-2
    Custom(Vec<f32>),
}
```

### Transform
Position, rotation, scale via la struct `Transform` standard.

## Flèche (cas particulier)

Une flèche est une ligne avec :
- Embout à un ou deux bouts
- Styles d'embout : flèche, triangle, cercle, carré, aucun

```rust
pub struct Arrow {
    pub start_arrow: ArrowHead,
    pub end_arrow: ArrowHead,
}

pub enum ArrowHead {
    None,
    Arrow,       // ▶
    Triangle,    // ▶ rempli
    Diamond,     // ◆
    Circle,      // ●
}
```

### Flèche connectée (smart connector)
Une flèche peut être **ancrée à 2 éléments**. Quand on déplace un élément, la flèche suit. Cf [05-mindmap-connecteurs.md](./05-mindmap-connecteurs.md).

## Édition après création

### Resize
- 8 handles autour du bbox de l'élément sélectionné
- Drag → resize
- Shift drag → préserve l'aspect ratio
- Alt drag → resize symétrique

### Rotate
- Handle de rotation au-dessus de la bbox
- Drag circulaire
- Shift drag → snap à 15° increments

### Move
- Drag du centre
- Arrows clavier → move pixel par pixel
- Shift + arrows → move 10 px

## Properties panel

Quand un ou plusieurs éléments sont sélectionnés, un panel à droite montre :

```
┌─────────────────────────────┐
│ Sélection (3 éléments)       │
├─────────────────────────────┤
│ Position : x=100, y=200      │
│ Taille    : w=150, h=80      │
│ Rotation : 0°                │
├─────────────────────────────┤
│ Fill     : ▓ #FF6B6B         │
│ Stroke   : ─ #000000  2px    │
│ Dash     : ─── solid         │
│ Corner   : 8 px              │
│ Opacity  : ▓▓▓▓▓░░░ 80%      │
└─────────────────────────────┘
```

Multi-select : montre les valeurs communes, « mixed » pour les divergentes.

## Snap

### Smart guides
Quand on déplace un élément, des lignes pointillées apparaissent quand on s'aligne sur un autre élément :

```
┌───────┐
│   A   │
└───────┘
- - - - - - - - - - -
        ┌───────┐
        │   B   │      ← B en train d'être déplacé, ligne s'affiche
        └───────┘
```

### Grid snap
Optionnel. Les déplacements snappent à la grille (taille configurable).

### Element snap
Snappe sur les bords/centres d'autres éléments à proximité (< 10 px screen).

## Distribution / Alignement (multi-select)

Quand >2 éléments sélectionnés, actions dans le toolbar :
- Aligner à gauche / droite / haut / bas / centre H / centre V
- Distribuer horizontalement / verticalement

## Group / Ungroup

- Ctrl+G : grouper les éléments sélectionnés
- Ctrl+Shift+G : dégrouper
- Un groupe se comporte comme un élément seul pour le déplacement

```rust
pub enum ElementKind {
    Group { children: Vec<ElementId> },
    // ...
}
```

## Layer (z-order)

Actions sur l'ordre :
- Apporter au premier plan (Ctrl+Shift+])
- Avancer (Ctrl+])
- Reculer (Ctrl+[)
- Mettre à l'arrière-plan (Ctrl+Shift+[)

Implementation : `z: i32` sur chaque élément. Sort au render.

## Hit testing

Pour la sélection, on doit tester si un point est dans un élément :

```rust
fn contains_point(elem: &Element, world: Vec2) -> bool {
    let local = elem.transform.world_to_local(world);
    match &elem.kind {
        ElementKind::Rectangle { width, height, .. } => {
            local.x.abs() <= width / 2.0 && local.y.abs() <= height / 2.0
        }
        ElementKind::Ellipse { width, height } => {
            let r = Vec2::new(local.x / (width / 2.0), local.y / (height / 2.0));
            r.length() <= 1.0
        }
        ElementKind::Pen { points, style } => {
            // distance min à la polyline + half stroke_width
            min_distance_to_polyline(&points, local) <= style.stroke_width / 2.0
        }
        // ...
    }
}
```

## Cibles

- Création d'une forme : <33 ms d'input à first paint
- Resize/move fluide à 60 FPS sur 1000 éléments scène
- Hit test : <1 ms via spatial index pre-filter

## Tests

- Création/édition/suppression chaque type de forme
- Modifiers (shift, alt) testés
- Snap testé
- Group/ungroup avec nested groups
- Z-order après remove/add/reorder

## Liens

- Modèle → [../03-ARCHITECTURE/05-modele-donnees.md](../03-ARCHITECTURE/05-modele-donnees.md)
- Rendu → [../06-CANVAS-INFINI/05-pipeline-rendu.md](../06-CANVAS-INFINI/05-pipeline-rendu.md)
- Connecteurs intelligents → [05-mindmap-connecteurs.md](./05-mindmap-connecteurs.md)
