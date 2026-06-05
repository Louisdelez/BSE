# 06.03 — Indexation spatiale

> Comment trouver vite « quels éléments sont dans le viewport ».

## Le problème

Une scène peut contenir 10 000 éléments. À chaque frame, on doit :
- Identifier ceux **visibles** (dans le viewport)
- Identifier ceux **sous le curseur** (hit test)
- Identifier ceux **dans une rectangle de sélection**

Approche naïve : itérer sur tous les éléments → O(N) à chaque frame. À 10K éléments, ça commence à coûter (~5-10 ms juste pour itérer).

Solution : un **index spatial** qui répond ces requêtes en O(log N + K) où K = nombre de résultats.

## Choix : Quadtree

### Pourquoi Quadtree ?

- ✅ **Simple à implémenter**
- ✅ **Très efficace en 2D**
- ✅ **Update incrémentaux faciles** (insert/remove)
- ✅ **Excellent pour les requêtes par rectangle**
- ✅ **Adapté aux distributions clustered** (typiques dans un canvas)

### Alternatives considérées

| Structure | Pour BSE ? | Note |
|---|---|---|
| **Quadtree** | ✅ #1 | Simple et efficace |
| **R-tree** | ⭐⭐⭐⭐ | Plus complexe, meilleur pour overlapping bboxes |
| **K-d tree** | ⭐⭐ | Pas optimal pour rectangles |
| **Grid** | ⭐⭐⭐ | Trop rigide pour zoom infini |
| **BVH** | ⭐⭐⭐ | Plus orienté rendu 3D |

**R-tree** est légèrement supérieur en théorie pour notre cas (rectangles), mais Quadtree gagne en simplicité d'implémentation et de debug.

### Crates Rust
- **`quadtree-rs`** : simple
- **`kdtree`** : pas adapté
- **`rstar`** : R-tree mature et performant — alternative valide

Choix initial : implémenter un Quadtree minimaliste maison (1-2 jours), puis évaluer `rstar` si besoins évoluent.

## Structure du Quadtree

```rust
pub struct Quadtree {
    root: QuadNode,
    bounds: Rect,
    max_items: usize,    // ~16
    max_depth: u32,      // ~8
}

pub enum QuadNode {
    Leaf(Vec<(ElementId, Rect)>),
    Branch {
        bounds: Rect,
        children: Box<[QuadNode; 4]>,  // NW, NE, SW, SE
    },
}
```

### Insertion

```rust
fn insert(&mut self, id: ElementId, bbox: Rect) {
    self.root.insert(id, bbox, 0, self.max_items, self.max_depth);
}

fn insert_node(&mut self, id: ElementId, bbox: Rect, depth: u32, max_items: usize, max_depth: u32) {
    match self {
        QuadNode::Leaf(items) => {
            items.push((id, bbox));
            if items.len() > max_items && depth < max_depth {
                self.split();
            }
        }
        QuadNode::Branch { children, .. } => {
            for child in children {
                if child.bounds().intersects(&bbox) {
                    child.insert(id, bbox, depth + 1, max_items, max_depth);
                }
            }
        }
    }
}
```

### Query

```rust
fn query(&self, viewport: Rect) -> Vec<ElementId> {
    let mut results = Vec::new();
    self.root.query(viewport, &mut results);
    results
}

fn query_node(&self, viewport: Rect, results: &mut Vec<ElementId>) {
    match self {
        QuadNode::Leaf(items) => {
            for (id, bbox) in items {
                if viewport.intersects(bbox) {
                    results.push(*id);
                }
            }
        }
        QuadNode::Branch { children, .. } => {
            for child in children {
                if child.bounds().intersects(&viewport) {
                    child.query(viewport, results);
                }
            }
        }
    }
}
```

### Suppression

Plus délicat car un élément peut être dans plusieurs feuilles si son bbox traverse les frontières.

Approche : on garde une **map externe** `ElementId → Vec<LeafPtr>` pour retrouver vite. Suppression = retirer de chaque feuille concernée.

### Update (move d'un élément)

```rust
fn update(&mut self, id: ElementId, old_bbox: Rect, new_bbox: Rect) {
    if old_bbox == new_bbox { return; }
    self.remove(id, old_bbox);
    self.insert(id, new_bbox);
}
```

Optimisation : si le nouveau bbox tombe dans la même feuille, juste mettre à jour le bbox stocké.

## Bounding box des éléments

Pour chaque élément, on a besoin de sa **bbox**. Calcul :

```rust
fn element_bbox(elem: &Element) -> Rect {
    match &elem.kind {
        ElementKind::Rectangle { width, height, .. } => {
            Rect::from_center_size(elem.transform.position(), Vec2::new(*width, *height))
        }
        ElementKind::Pen { points, .. } => {
            let mut min = Vec2::splat(f32::INFINITY);
            let mut max = Vec2::splat(f32::NEG_INFINITY);
            for p in points {
                min = min.min_elementwise(Vec2::new(p.x, p.y));
                max = max.max_elementwise(Vec2::new(p.x, p.y));
            }
            // Ajouter la largeur du trait
            Rect::from_min_max(min, max).expand(elem.style.stroke_width)
        }
        // ... autres types
    }
}
```

Pour la rotation : prendre l'AABB du shape rotated.

## Gestion du resize de la scène

Le quadtree a des bounds fixes au départ. Si un élément est ajouté en dehors :
- Option A : agrandir le quadtree (re-build)
- Option B : extend bounds en montant en parent

**BSE** : démarrer avec bounds très larges (`±1M`), rebuild si dépassement (rare).

## Reconstruction périodique

Au fil des modifications, l'arbre peut devenir déséquilibré (e.g., beaucoup d'éléments concentrés dans une zone après déplacement).

Solution : rebuild complet périodique :
- Trigger : >20% des éléments ont bougé depuis le dernier rebuild
- Coût : O(N log N)
- Fait en background sur un autre thread

## Précision

À très haut zoom, on pourrait dépasser la précision de la grille du quadtree. **Pas un problème pratique** : les bbox des éléments restent en f32, et le quadtree subdivise jusqu'à `max_depth` (≈ 8 niveaux = précision suffisante).

## Performances visées

- Build d'un quadtree de 10K éléments : <100 ms
- Insert un élément : <50 µs
- Query viewport (1000 éléments visibles parmi 10K) : <500 µs
- Update (move) : <50 µs

## Hit testing

Au-delà du viewport culling, on utilise le quadtree pour le hit test :

```rust
fn hit_test(&self, point: WorldPos) -> Option<ElementId> {
    let nearby = self.query(Rect::from_center_size(point, Vec2::splat(1.0)));
    
    // Test plus précis sur les candidates
    for id in nearby.iter().rev() {  // ordre top-to-bottom (front-to-back)
        let elem = self.scene.get(*id)?;
        if elem.contains_point(point) {
            return Some(*id);
        }
    }
    None
}
```

`contains_point` est spécifique au type (ex: ellipse test paramétrique, polygon ray-casting, stroke proximity).

## Box selection

Pour le drag select avec une boîte :

```rust
fn box_select(&self, box_world: Rect) -> Vec<ElementId> {
    let candidates = self.query(box_world);
    candidates.into_iter()
        .filter(|id| {
            let elem = self.scene.get(*id);
            elem.map_or(false, |e| box_world.contains_rect(&e.bbox()))  // strict containment
            // or: box_world.intersects(&e.bbox())   // partial
        })
        .collect()
}
```

Configurable : sélection « strict containment » (Figma style) vs « intersection » (PowerPoint style).

## Tests

- Insert + query : couverture exhaustive
- Update : verifier non-duplication
- Stress test : 100K elements aléatoires, mesurer temps de queries
- Property tests : `query(R)` retourne **exactement** les éléments dont la bbox intersecte R

## Liens

- Culling et LOD → [04-culling-lod.md](./04-culling-lod.md)
- Rendu → [05-pipeline-rendu.md](./05-pipeline-rendu.md)
- Modèle de données → [../03-ARCHITECTURE/05-modele-donnees.md](../03-ARCHITECTURE/05-modele-donnees.md)
