# 04.03 — Rendu du canvas

> Comment on dessine la toile : wgpu, vello, et pipelines de rendu.

## TL;DR

> **wgpu** comme couche d'abstraction GPU (Vulkan/Metal/DX12/WebGPU).  
> **vello** en option pour le rendu vectoriel complexe (path, courbes Bézier).  
> Pipelines maison pour les cas simples (rectangles, ellipses, strokes simples).

## Pourquoi GPU ?

Un canvas infini avec 1000-10000 éléments doit être redessiné chaque frame (à cause du pan/zoom continu). CPU = limites de bande passante. **GPU = parallélisme massif**, exactement adapté.

### Coût d'un rendu typique
- 1000 rectangles tracés en CPU : ~5-10 ms (= 100-200 FPS max)
- 1000 rectangles tracés en GPU (batché) : <1 ms (= 1000+ FPS)
- Avec 10K éléments la différence devient critique.

## wgpu : la fondation

[wgpu](https://wgpu.rs/) est l'implémentation de référence de WebGPU en Rust, qui tourne nativement sur :
- **Windows** : Vulkan ou DirectX 12
- **macOS / iOS** : Metal
- **Linux** : Vulkan
- **Browser** : WebGPU (ou WebGL2 en fallback)
- **Android** : Vulkan

C'est l'API GPU **portable** la plus mûre en Rust. Toutes les autres options (raw Vulkan, ash, gfx-rs old) sont soit moins portables soit plus bas niveau.

### Concepts wgpu utiles à BSE
- **Device** : interface GPU (1 par app)
- **Queue** : queue d'exécution des commandes
- **SwapChain / SurfaceTexture** : framebuffer présenté à l'écran
- **RenderPass** : passe de rendu avec ses attachments
- **Pipeline** : programme GPU avec ses shaders (vertex + fragment)
- **Buffer / Texture** : ressources mémoire GPU
- **BindGroup** : association de ressources à un pipeline

## Pipelines de rendu BSE

On aura plusieurs pipelines, chacun pour un type d'élément :

### Pipeline 1 — Background (grille, fond)
Très simple : un quad fullscreen, fragment shader génère la grille via les coords UV monde.

### Pipeline 2 — Shapes (rectangles, ellipses)
- Instanced rendering : 1 quad répété N fois
- Buffer d'instances contenant `(transform, style, kind)`
- Le fragment shader détermine la forme via SDF (signed distance field) :
  - Rectangle : `length(max(abs(p) - half_extent, 0))`
  - Ellipse : approximation SDF
  - Cercle : `length(p) - radius`

Avantages SDF : anti-aliasing **gratuit** (par dérivation), bord lisse au zoom.

### Pipeline 3 — Strokes (dessin libre, perfect-freehand)
Le tracé pression-sensible n'est pas un quad simple :
- Génération du polygone d'épaisseur variable depuis les points (via algorithm perfect-freehand)
- Triangulation (earcut, triangle, ou simple fan)
- Upload du polygone dans un buffer

Optimisation : pour les strokes **en cours** (interactif), on accumule les points et on régénère localement (sub-100 ms). Pour les strokes **figés** (autres peers), on stocke le polygone triangulé directement.

### Pipeline 4 — Texte
Deux approches :

**Approche A — Atlas de glyphes raster** :
- Tex atlas RGBA contenant les glyphes (256-512 caractères ASCII + cyrillique + …)
- Chaque char = 1 quad mappant sur l'atlas
- Crate **glyphon** ou **wgpu_glyph** font ça out of the box
- Limite : flou au zoom

**Approche B — MSDF (Multi-channel SDF)** :
- Glyphes stockés en SDF
- Texte net à toutes les échelles
- Plus complexe à implémenter

**Choix v1** : approche A avec glyphon. Approche B en v1.x si on a des demandes de zoom extrême.

### Pipeline 5 — Images
- Chaque image = 1 quad texturé
- Bind group avec la texture
- Mipmaps pour le filtering au zoom-out

### Pipeline 6 — Curseurs distants (peers)
- Pipeline ultra simple : N triangles + label texte
- Bouge à 30 Hz, doit être redessiné chaque frame de toutes manières

## Vello : pour le complexe

[Vello](https://github.com/linebender/vello) est un renderer 2D **GPU-compute** (shaders compute) capable de rendre des vecteurs complexes : paths, courbes Bézier, fills, strokes avancés. Inspiré de Skia/Cairo mais GPU-first.

### Quand l'utiliser pour BSE ?
- Tracés complexes (mindmap avec courbes Bézier)
- Connecteurs orthogonaux avec arrondis
- Style hand-drawn (équivalent Rough.js)
- Cas où une SDF custom serait trop lourde

### État en 2026
- **Alpha mature** : utilisable mais évolue
- Performance excellente sur GPUs récents
- Intégration wgpu native

**Stratégie BSE** : commencer **sans** vello (pipelines custom). Introduire vello en v0.5 ou v1.0 quand on a besoin de paths complexes.

## Stratégies d'optimisation

### Batching
Regrouper les éléments du même type dans des draw calls :
- 1000 rectangles → 1 draw call instancé
- Plutôt que 1000 draw calls

### Culling
Ne rendre que ce qui est dans le viewport. Cf [../06-CANVAS-INFINI/04-culling-lod.md](../06-CANVAS-INFINI/04-culling-lod.md).

### LOD (Level of Detail)
- Éléments très petits à l'écran (<5px) → version simplifiée (point) ou skip
- Texte illisible → ne pas faire de texte rendering
- Images très éloignées → mipmap basse

### Dirty rectangles ?
Tentation : ne re-rendre que les zones modifiées. **Mais** au pan/zoom continu, toute la scène change. Donc on rend tout à chaque frame, et on parie sur l'efficacité du GPU.

### Caching d'éléments figés
Les éléments qui ne bougent pas pourraient être **pré-rendus dans une texture** une fois, puis blittés. Optimisation v1.x si nécessaire.

## Gestion mémoire GPU

### Buffer de la scène
- 1 buffer `vertices` global, mis à jour quand des éléments changent
- 1 buffer `instances` par type
- Ré-upload **partiel** (offset+size) quand un élément est modifié

### Textures
- Atlas pour les glyphes texte
- Cache d'images chargées (LRU, max 256 MB GPU)
- Eviction des images non visibles depuis 1 min

## Frame budget détaillé

À 60 FPS = 16,6 ms par frame.

```
                                  Budget
─────────────────────────────────────────
Capture inputs                    0.2 ms
Update camera & state             0.3 ms
Spatial query (visible elements)  0.5 ms
egui UI logic                     1-2 ms
Préparation des buffers GPU       0.5 ms
RenderPass background              0.2 ms
RenderPass shapes                 0.5-1.5 ms
RenderPass strokes                0.5-2 ms
RenderPass text                   0.5-1 ms
RenderPass images                 0.3-1 ms
RenderPass remote cursors         0.1 ms
RenderPass egui overlay           0.5-1 ms
Compose + present                 1-3 ms
─────────────────────────────────────────
Total                             ~6-13 ms
```

Marge confortable pour 60 FPS. 144 FPS (6.9 ms budget) est tendu mais atteignable sur GPU récent.

## Choix logiciels

| Besoin | Crate |
|---|---|
| GPU API | `wgpu` |
| Texte | `glyphon` (sur wgpu) ou `cosmic-text` |
| SVG parsing | `usvg` (pour import) |
| Image decoding | `image` |
| Vector 2D complex | `vello` (v0.5+) |
| Triangulation | `lyon_tessellation` |
| Math | `glam` |
| Color | `palette` ou maison |

## Plan d'implémentation par jalon

| Jalon | Pipelines à implémenter |
|---|---|
| MVP | Background + Shape SDF (rect, ellipse) + Stroke triangulé + Curseurs distants |
| v0.1 | + Texte (glyphon) + Images |
| v0.5 | + Vello pour mindmap, connecteurs courbes |
| v1.0 | + Optimisations : caching, LOD, multi-threading des uploads |

## Sources

- [wgpu.rs](https://wgpu.rs/)
- [linebender/vello](https://github.com/linebender/vello)
- *Rust GPU Programming with wgpu: The 2026 Guide* — rustify.rs
