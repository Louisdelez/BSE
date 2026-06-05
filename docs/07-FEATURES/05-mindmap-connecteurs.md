# 07.05 — Mindmap et connecteurs

> La feature qui différencie BSE des canvas génériques.

## Mindmap

### Vision

> Un nœud central → des branches → des sous-branches. Édition rapide au clavier. Auto-layout option.

### Modèle

```rust
pub struct MindmapNode {
    pub text: String,                  // dans yrs::Text
    pub parent: Option<ElementId>,
    pub children: Vec<ElementId>,
    pub style: NodeStyle,
}

pub struct NodeStyle {
    pub fill: Color,
    pub border: Color,
    pub text_color: Color,
    pub shape: NodeShape,
    pub size: NodeSize,
}

pub enum NodeShape {
    RoundedRectangle,
    Capsule,
    Hexagon,
    Cloud,
}

pub enum NodeSize {
    Small,
    Medium,
    Large,
    Auto,  // dimensionné selon texte
}
```

### Édition au clavier

Quand un nœud est sélectionné :
- **Tab** : crée un enfant
- **Enter** : crée un frère
- **Backspace** sur empty : supprime le nœud
- **Arrows** : navigation parent/enfant/frère
- **Espace + arrows** : déplace le nœud

### Auto-layout

Quand on ajoute / supprime un nœud, le mindmap se réorganise automatiquement (option).

Algorithmes possibles :
- **Tree layout horizontal** : nœud central, branches à gauche et droite
- **Radial layout** : nœud central, branches en cercle
- **Tree layout vertical** : top-down ou bottom-up

```rust
fn auto_layout_mindmap(root: ElementId, scene: &mut Scene, algorithm: LayoutAlgorithm) {
    let positions = compute_layout(root, scene, algorithm);
    for (id, pos) in positions {
        scene.set_transform(id, pos);
    }
}
```

Auto-layout désactivable : l'utilisateur peut placer manuellement.

### Connecteurs entre nœuds

Chaque relation parent-enfant a un connecteur visuel :
- Style : courbe Bézier (par défaut), droit, orthogonal
- Couleur héritée du nœud parent
- Largeur configurable

### Mouvement (movable tree)

Drag d'un nœud :
- Si drop sur un autre nœud → reparenting
- Si drop hors → repositionnement libre

**Problème CRDT** : reparenting concurrent → cycle potentiel.

→ Avec yrs : détection de cycle côté client (cf [../05-COLLABORATION-TEMPS-REEL/06-conflits-cas-limites.md](../05-COLLABORATION-TEMPS-REEL/06-conflits-cas-limites.md)).
→ Avec Loro : Movable Tree CRDT natif (recommandé).

## Connecteurs (smart connectors)

### Vision

Une flèche qui relie 2 éléments et **suit** leurs mouvements automatiquement.

### Modèle

```rust
pub struct Connector {
    pub from: ConnectorEnd,
    pub to: ConnectorEnd,
    pub path: ConnectorPath,
    pub style: ConnectorStyle,
}

pub enum ConnectorEnd {
    Element {
        id: ElementId,
        anchor: AnchorPoint,    // ou Auto (le plus proche du destinataire)
    },
    Free {
        x: f32,
        y: f32,
    },
}

pub enum AnchorPoint {
    Auto,
    Top,
    Bottom,
    Left,
    Right,
    TopLeft, TopRight, BottomLeft, BottomRight,
    Center,
}

pub enum ConnectorPath {
    Straight,    // ligne droite
    Curved,      // Bézier
    Orthogonal,  // ─┐ angles 90°
}

pub struct ConnectorStyle {
    pub stroke: Color,
    pub stroke_width: f32,
    pub dash: DashPattern,
    pub start_arrow: ArrowHead,
    pub end_arrow: ArrowHead,
    pub label: Option<String>,
}
```

### Création

Outil **flèche** activé (`A`) :
1. Hover un élément → highlight cercle bleu au point d'ancrage proposé
2. Click → fixe l'ancre de départ
3. Drag → la flèche suit
4. Hover un autre élément → highlight ancre d'arrivée
5. Click → finalise

### Re-routing automatique

Quand on déplace un élément ancré :
- La flèche **suit** (calcul de path à chaque frame)
- Avec `AnchorPoint::Auto` : le système choisit l'ancre la plus naturelle

### Pathfinding pour orthogonal

Pour un connecteur orthogonal, on calcule un chemin avec angles 90° :
- Algorithme A* sur une grille fictive
- Évite (si possible) de passer à travers d'autres éléments
- Mid-points configurables manuellement

Implémentation : peut être déléguée à `vello` (paths SVG-like) si trop complexe en custom.

### Labels sur connecteurs

Un connecteur peut avoir un label texte au milieu :
- Édition inline
- Suit le path
- Background du label = couleur de fond projet (pour masquer la ligne)

## Connecteur libre (flèche libre)

Une flèche **non ancrée** est juste une ligne avec embouts. Pas de smart routing.

## Conflits CRDT

### Suppression d'un élément ancré
Si Alice supprime le rectangle, Bob's flèche qui pointait dessus reste avec une extrémité libre (`ConnectorEnd::Free` au dernier point connu).

Implémentation :
```rust
fn on_element_deleted(scene: &mut Scene, deleted: ElementId) {
    for connector in scene.connectors_referencing(deleted) {
        // Convertir l'ancre en Free au dernier point connu
        let last_point = compute_anchor_position(...);
        connector.detach_end(deleted, ConnectorEnd::Free { x, y });
    }
}
```

### Cycle dans mindmap
Cf cas 9 dans [../05-COLLABORATION-TEMPS-REEL/06-conflits-cas-limites.md](../05-COLLABORATION-TEMPS-REEL/06-conflits-cas-limites.md).

## UX

### Templates de mindmap
Le menu « Nouveau » propose :
- Mindmap vierge (1 nœud central)
- Mindmap radial pré-rempli (8 branches)
- Templates pré-built (cf [07-templates.md](./07-templates.md))

### Outil mindmap dédié
Active un mode où :
- Click sur vide → crée un nœud
- Click sur nœud → édite le texte
- Drag d'un nœud sur un autre → reparente
- Outil flèche désactivé (les connecteurs sont automatiques)

### Outil flèche dédié
Active un mode où :
- Click + drag entre 2 éléments → crée un connecteur
- Click sur connecteur existant → édite

## Performance

### Re-routing en temps réel
Quand un élément ancré bouge à 60 FPS, le re-routing doit suivre :
- Calcul de path en <1 ms par connecteur
- Cache : si l'élément ne bouge pas, pas de recalcul
- Pour orthogonal : grille pré-calculée

### Mindmap massif
Pour 500+ nœuds :
- Spatial index intégré (l'index spatial standard fonctionne)
- LOD : nœuds très éloignés simplifiés en dot

## Tests

- Créer / éditer / déplacer nœuds
- Reparenting drag & drop
- Connecteurs ancrés suivent le déplacement
- Suppression élément ancré → connecteur passe en Free
- Cycle detection (si yrs)
- Auto-layout fonctionnel

## Liens

- Modèle → [../03-ARCHITECTURE/05-modele-donnees.md](../03-ARCHITECTURE/05-modele-donnees.md)
- Conflits CRDT → [../05-COLLABORATION-TEMPS-REEL/06-conflits-cas-limites.md](../05-COLLABORATION-TEMPS-REEL/06-conflits-cas-limites.md)
- Choix Loro pour movable tree → [../05-COLLABORATION-TEMPS-REEL/03-choix-bse.md](../05-COLLABORATION-TEMPS-REEL/03-choix-bse.md)
- Templates de mindmap → [07-templates.md](./07-templates.md)
