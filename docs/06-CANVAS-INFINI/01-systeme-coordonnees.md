# 06.01 — Système de coordonnées

> Comment BSE représente l'espace 2D et son lien avec ce qu'on voit à l'écran.

## Deux espaces, une transformation

BSE manipule deux espaces de coordonnées :

1. **World space** (espace monde) : la toile infinie. Coordonnées flottantes, illimitées en théorie. Tous les éléments sont stockés dans cet espace.
2. **Screen space** (espace écran) : les pixels physiques de la fenêtre. Origine en haut-gauche, x → droite, y → bas.

La **caméra** définit la transformation entre les deux.

```
        World space                       Screen space
    ┌──────────────────────┐         ┌──────────────────────┐
    │                      │         │                      │
    │     Element X        │  ────►  │   X (transformed)    │
    │       (10, 10)       │         │     (500, 300)       │
    │                      │         │                      │
    │       Camera         │         │                      │
    │       (0, 0)         │         │      Viewport        │
    │       zoom 1.0       │         │     1920x1080        │
    └──────────────────────┘         └──────────────────────┘
```

## Convention orientations

BSE utilise la convention **« y vers le bas »** (comme l'écran), à la différence des maths où y va vers le haut.

- World : `(0, 0)` est arbitraire, `x →` à droite, `y ↓` en bas
- Screen : `(0, 0)` en haut-gauche, idem `x →`, `y ↓`

Avantage : pas de flip y dans les calculs.

## Caméra

```rust
pub struct Camera {
    pub x: f32,        // position du centre du viewport en coords monde
    pub y: f32,
    pub zoom: f32,     // 1.0 = 1:1, 2.0 = zoom 2x, 0.5 = dézoom
}
```

## Transformations

### Screen → World
```rust
fn screen_to_world(camera: &Camera, viewport: Vec2, screen: Vec2) -> Vec2 {
    let offset_from_center = screen - viewport / 2.0;
    let world_offset = offset_from_center / camera.zoom;
    Vec2::new(camera.x, camera.y) + world_offset
}
```

### World → Screen
```rust
fn world_to_screen(camera: &Camera, viewport: Vec2, world: Vec2) -> Vec2 {
    let world_offset = world - Vec2::new(camera.x, camera.y);
    let screen_offset = world_offset * camera.zoom;
    viewport / 2.0 + screen_offset
}
```

### Viewport en coords monde
```rust
fn viewport_world_rect(camera: &Camera, viewport: Vec2) -> Rect {
    let half = viewport / 2.0 / camera.zoom;
    Rect {
        min: Vec2::new(camera.x - half.x, camera.y - half.y),
        max: Vec2::new(camera.x + half.x, camera.y + half.y),
    }
}
```

## Précision

### Coords float32 ou float64 ?

f32 vs f64 :
- **f32** : 7 chiffres significatifs. Précision dégradée pour des éléments très éloignés de l'origine (au-delà de 100K unités).
- **f64** : 15-17 chiffres significatifs. Précision excellente partout, mais 2× la taille mémoire.

**Choix BSE : f32 pour les coords monde**, en restreignant la zone praticable à `±1_000_000` unités. Au-delà, on ré-origine.

### Re-origination
Si l'utilisateur dérive très loin (>1M unités), on peut recentrer :
1. Calculer le centroïde des éléments
2. Soustraire à tous les éléments (op CRDT batch)
3. Mettre à jour la caméra
4. **Pas fait en v1** : on parie sur le fait que les utilisateurs ne s'éloignent pas autant.

## Limites pratiques

| Métrique | Limite v1.0 |
|---|---|
| Coord world min/max | ±1_000_000 unités |
| Zoom min | 0.01 (1%) |
| Zoom max | 50.0 (5000%) |
| Taille minimum d'un élément (world) | 1.0 unité |
| Taille maximum d'un élément (world) | 100_000 unités |

## Unités

Les coords monde sont **dimensionless** par défaut. Pour fixer une référence :
- **1 unité world ≈ 1 pixel à zoom 1.0**
- À zoom 2.0, un élément de 100 unités prend 200 pixels écran
- Pas de notion de DPI dans l'espace monde (gestion DPI au niveau présentation)

## DPI handling

- `winit` expose le `scale_factor` (1.0, 1.5, 2.0 selon écran)
- L'application multiplie le `viewport_size` reçu par le `scale_factor` pour le rendu
- Les conversions caméra utilisent toujours le viewport physique (en pixels)

## Grille (optionnelle)

Une grille peut être affichée au fond pour aider à se repérer :

```
        World space avec grille
    ──┼──┼──┼──┼──┼──┼──
      │  │  │  │  │  │
    ──┼──┼──┼──┼──┼──┼──
      │  │ █│  │  │  │     ← élément aligné
    ──┼──┼──┼──┼──┼──┼──
```

- Taille de grille configurable (10, 20, 50, 100 unités)
- Visible/invisible par toggle
- Snap to grid activable (les déplacements snappent)
- Au zoom très bas, on simplifie ou cache la grille pour éviter le moiré

## Sub-pixel & pression

- Les coords monde sont en f32 → précision sub-pixel naturelle
- À zoom 1.0, un déplacement de 0.5 unité = 0.5 pixel (anti-aliasing)
- Pour les strokes de stylo : les points sont en coords monde f32, ce qui permet de capturer la précision du stylet

## Snap

Modes de snap disponibles :
- **Grid snap** : à la grille
- **Element snap** : aligne sur les centres/bords d'autres éléments
- **Smart guides** : lignes pointillées éphémères qui apparaissent quand on s'aligne sur un autre élément

Activables séparément. Désactivés par défaut pour ne pas frustrer.

## Liens

- Caméra → [02-camera-zoom-pan.md](./02-camera-zoom-pan.md)
- Spatial index → [03-spatial-indexing.md](./03-spatial-indexing.md)
- Rendu → [05-pipeline-rendu.md](./05-pipeline-rendu.md)
