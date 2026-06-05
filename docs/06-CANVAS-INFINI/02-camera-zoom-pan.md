# 06.02 — Caméra : zoom et pan

> Comment l'utilisateur navigue dans la toile infinie.

## Interactions

### Pan

Déplacement de la vue (la caméra) latéralement.

| Input | Action |
|---|---|
| Espace + drag souris | Pan |
| Trackpad 2-finger drag | Pan |
| Drag avec touch (mobile) | Pan |
| Middle mouse drag | Pan |
| Pad numerique : arrows ← → ↑ ↓ | Pan d'un demi-viewport |

Implementation :
```rust
fn on_pan_drag(delta_screen: Vec2, camera: &mut Camera) {
    camera.x -= delta_screen.x / camera.zoom;
    camera.y -= delta_screen.y / camera.zoom;
}
```

### Zoom

Échelle vers une portion de la toile.

| Input | Action |
|---|---|
| Molette souris | Zoom à la position du curseur |
| Trackpad pinch | Zoom à la position du curseur |
| Ctrl + molette | Zoom à la position du curseur |
| Ctrl + `+` / `-` | Zoom au centre |
| Ctrl + `0` | Reset zoom à 1.0 |
| Ctrl + `Shift + 1` | Fit to content (zoom auto) |

Implementation (zoom centré sur le curseur) :
```rust
fn on_zoom(scroll_amount: f32, cursor_screen: Vec2, camera: &mut Camera, viewport: Vec2) {
    let cursor_world_before = screen_to_world(camera, viewport, cursor_screen);
    
    let zoom_factor = (1.0 + scroll_amount * ZOOM_SENSITIVITY).clamp(0.1, 10.0);
    camera.zoom = (camera.zoom * zoom_factor).clamp(MIN_ZOOM, MAX_ZOOM);
    
    let cursor_world_after = screen_to_world(camera, viewport, cursor_screen);
    
    // Le point sous le curseur doit rester sous le curseur
    let drift = cursor_world_before - cursor_world_after;
    camera.x += drift.x;
    camera.y += drift.y;
}
```

### Inertie (smooth scrolling)

Pour une expérience fluide, on applique de l'inertie au pan/zoom :
- État `velocity_x, velocity_y, zoom_velocity`
- À chaque frame : intégration + dissipation (~0.85)
- Stop sous une threshold de bruit

```rust
fn update_inertia(camera: &mut Camera, vel: &mut Velocity, dt: f32) {
    camera.x += vel.x * dt;
    camera.y += vel.y * dt;
    
    vel.x *= 0.85_f32.powf(dt * 60.0);
    vel.y *= 0.85_f32.powf(dt * 60.0);
    
    if vel.magnitude() < 0.5 { *vel = Velocity::ZERO; }
}
```

### Animations vers une position

Pour les actions « aller à un élément », « fit to content », on anime :

```rust
struct CameraAnimation {
    from: Camera,
    to: Camera,
    start: Instant,
    duration: Duration,
    easing: EasingFn,
}

fn tick(&mut self, camera: &mut Camera) -> bool {
    let t = elapsed / duration;
    let eased = self.easing(t.clamp(0.0, 1.0));
    camera.x = lerp(from.x, to.x, eased);
    camera.y = lerp(from.y, to.y, eased);
    camera.zoom = lerp(from.zoom, to.zoom, eased);
    t < 1.0
}
```

Easing : `ease_in_out_cubic` par défaut.

## Bornes

### Bornes de zoom
```
MIN_ZOOM = 0.05   // 5% — voir une très grande zone
MAX_ZOOM = 50.0   // 5000% — détails très fins
```

### Bornes de pan
Aucune théoriquement (canvas infini). Mais on clamp à `±1_000_000` pour éviter les problèmes de précision f32.

```rust
camera.x = camera.x.clamp(-1_000_000.0, 1_000_000.0);
camera.y = camera.y.clamp(-1_000_000.0, 1_000_000.0);
```

## Fit to content / Zoom to selection

```rust
fn fit_to_rect(target: Rect, viewport: Vec2, padding: f32, animated: bool) -> Camera {
    let target_with_padding = target.expand(padding);
    let zoom_x = viewport.x / target_with_padding.width();
    let zoom_y = viewport.y / target_with_padding.height();
    let zoom = zoom_x.min(zoom_y).min(MAX_ZOOM);
    
    Camera {
        x: target.center().x,
        y: target.center().y,
        zoom,
    }
}
```

Raccourcis :
- `Ctrl+1` : zoom 100%
- `Ctrl+2` : fit selection
- `Ctrl+3` : fit content

## Smooth zoom continu

Pour une UX premium, le zoom à la molette est **interpolé** :
- Chaque tick molette ajoute à un `target_zoom`
- À chaque frame, `camera.zoom` se rapproche du `target_zoom` (lerp 0.3)
- Résultat : zoom fluide même avec une molette dégueulasse

## Pan/zoom pendant qu'on dessine

Combos utiles :
- **Espace** maintenu pendant qu'on dessine → pan temporaire
- **Alt + molette** pendant qu'on dessine → zoom temporaire
- Les modifications du tool actif sont mises en pause pendant ces actions

## Gestion du touch (mobile / tablette)

Pour iPad/Android (compagnon v1.x) :
- 1 doigt : draw / select
- 2 doigts : pan + zoom (pinch)
- 3 doigts : raccourci undo (?)

Sur Windows tablet mode (avec stylet) :
- Stylet : draw
- Doigt : pan + zoom
- Bouton du stylet : raccourcis

## Performance du pan/zoom

Le pan/zoom déclenche **un redraw complet** chaque frame :
- 60 FPS = 16.6 ms budget
- Si pan/zoom rapide + 10K éléments = stress test

Mitigation :
- Spatial index pour culling rapide (cf [03-spatial-indexing.md](./03-spatial-indexing.md))
- LOD au zoom-out (cf [04-culling-lod.md](./04-culling-lod.md))
- GPU batching agressif

## Limites volontaires

- **Pas de rotation de caméra** en v1.0 (rare en whiteboard, ajoute complexité)
- **Pas de 3D / perspective** (canvas 2D pur)

## Cas spéciaux

### Following peer
Mode où la caméra suit celle d'un autre peer (Bob est présentateur, Alice « follow »).

- Lecture de l'awareness camera de Bob
- Animation lerp vers cette caméra
- Désactivation au moindre input local d'Alice

### Multi-écran / multi-fenêtre
Si l'utilisateur a plusieurs fenêtres BSE ouvertes sur des projets différents : chacune a sa caméra. Pas de coordination.

## Tests UX

- Pan fluide même sur trackpad bas de gamme
- Zoom centré sur le curseur (pas qui drifte)
- Fit to content cadre proprement avec padding visible
- Pas de jitter sur les bords de zoom min/max

## Liens

- Coordonnées → [01-systeme-coordonnees.md](./01-systeme-coordonnees.md)
- Performance → [06-performance.md](./06-performance.md)
- Raccourcis clavier → [../08-UX-UI/04-raccourcis-clavier.md](../08-UX-UI/04-raccourcis-clavier.md)
