# 06.04 — Culling et Level-of-Detail

> Ne pas rendre ce qu'on ne voit pas. Simplifier ce qui est petit.

## Culling : ne pas rendre l'invisible

### Viewport culling

À chaque frame :
1. Calculer le viewport en coords monde (cf [02-camera-zoom-pan.md](./02-camera-zoom-pan.md))
2. Étendre légèrement (margin de 10% pour anticiper le pan)
3. Query le spatial index
4. Rendre uniquement les éléments retournés

```rust
fn visible_elements(&self) -> Vec<ElementId> {
    let viewport_world = self.camera.viewport_world_rect();
    let margin = viewport_world.size() * 0.1;
    let extended = viewport_world.expand_xy(margin);
    self.spatial_index.query(extended)
}
```

Sans culling, à 10K éléments dispersés : on rendrait tout chaque frame → bottleneck CPU.

### Combien on cull en moyenne ?

Pour une scène typique (10K éléments, viewport montre ~5%) :
- Visibles : ~500
- Cullés : 9500

→ 20× moins de travail GPU.

### Limites du culling

- **Éléments très étirés** (lignes très longues) ont une bbox immense → souvent visibles même quand pas dans le centre
- **Strokes longs** : pareil
- Le quadtree gère ça bien (un long bbox sera dans plusieurs cellules)

## Level of Detail (LOD)

Quand un élément est rendu mais **très petit à l'écran** (zoom-out extrême), on peut le simplifier.

### Critère
Taille en pixels = `world_size * camera.zoom`.

| Taille pixels | Action |
|---|---|
| > 30 px | Rendu complet (toutes propriétés visibles) |
| 5-30 px | Rendu simplifié (pas de texte, pas de stroke style fin) |
| 1-5 px | Rendu LOD 2 : juste un dot coloré |
| < 1 px | Skip (mais compte dans une heatmap optionnelle) |

### Implementation par type

#### Texte
- Au-delà de 8 px de hauteur ligne : rendu normal
- 4-8 px : rendu en placeholder rectangle gris
- <4 px : skip (illisible de toute façon)

#### Image
- Variant texture selectionnée selon taille écran (`thumb`/`medium`/`original`)
- Si <5 px : un quad coloré moyen

#### Stroke de pen
- Au-delà de 20 points visibles : rendu complet
- Sous 20 points visibles : un segment droit du début au end
- <3 px : un point

#### Rectangle / ellipse
- Au-delà de 5 px : rendu SDF complet (anti-alias)
- Sous : rendu pixel discret (rapide)

#### Mindmap nodes
- Au-delà de 30 px : texte + bord
- Sous : un cercle de la couleur du node

### Code
```rust
fn render_element(elem: &Element, camera: &Camera, ...) {
    let screen_size = elem.bbox().size() * camera.zoom;
    let max_dim = screen_size.x.max(screen_size.y);
    
    let lod = match max_dim {
        s if s < 1.0 => Lod::Skip,
        s if s < 5.0 => Lod::Dot,
        s if s < 30.0 => Lod::Simplified,
        _ => Lod::Full,
    };
    
    match lod {
        Lod::Skip => return,
        Lod::Dot => render_dot(elem, ...),
        Lod::Simplified => render_simplified(elem, ...),
        Lod::Full => render_full(elem, ...),
    }
}
```

## Frustum culling vs Occlusion culling

- **Frustum** (= viewport culling) : on l'a couvert ci-dessus.
- **Occlusion** (un élément en cache un autre, donc on skip l'arrière) : généralement pas worth pour un canvas 2D avec peu de superposition.

BSE ne fait pas d'occlusion culling en v1.

## Streaming de la scène (très grandes scènes)

Pour des scènes >100K éléments, on pourrait :
- Charger en mémoire seulement ce qui est dans le viewport étendu
- Stream les autres en background
- Pratiquement jamais nécessaire à notre échelle, mais à garder en tête pour v2+

## Mesures de performance

À atteindre :

| Scénario | Cible v1.0 |
|---|---|
| 100 éléments visibles, 10K total | 144 FPS |
| 1000 éléments visibles, 100K total | 60 FPS |
| Pan rapide pendant rendering | Smooth, pas de hitch |
| Zoom out extrême (tout visible) | 30 FPS minimum |

## Tests

- Benchmark synthétique : scène générée aléatoirement avec N éléments
- Mesure FPS pour N = 100, 1K, 10K, 100K
- Profiling avec `tracy` ou `puffin`

## Liens

- Spatial index → [03-spatial-indexing.md](./03-spatial-indexing.md)
- Pipeline rendu → [05-pipeline-rendu.md](./05-pipeline-rendu.md)
- Performance → [06-performance.md](./06-performance.md)
