# 06.05 — Pipeline de rendu

> Le détail du rendu GPU frame par frame.

## Vue d'ensemble

```
   ┌──────────────────────────────────────────────────────┐
   │                  Frame N                              │
   └──────────────────────────────────────────────────────┘
   
   1. Input gathering           (winit events)
   2. State update              (apply pending events to camera, scene)
   3. Awareness merge           (remote cursors, selections)
   4. Visible query             (quadtree)
   5. Build batches             (sort by type, prepare GPU buffers)
   6. Submit to GPU :
      a. RenderPass : background
      b. RenderPass : shapes (instanced SDF)
      c. RenderPass : strokes
      d. RenderPass : text
      e. RenderPass : images
      f. RenderPass : remote cursors
      g. RenderPass : selection overlays
      h. RenderPass : egui (UI chrome)
   7. Present                   (swap)
   8. Network emit              (cursor throttle, etc.)
```

## Initialisation wgpu

```rust
struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    
    pipelines: Pipelines,
    bind_group_layouts: BindGroupLayouts,
    samplers: Samplers,
    
    // Buffers réutilisables
    transient_vertex_buffer: wgpu::Buffer,  // ring buffer
    transient_index_buffer: wgpu::Buffer,
}
```

## Pipeline 1 — Background

### Programme
- Vertex shader : full-screen quad
- Fragment shader : génère la grille en fonction des UV monde

### WGSL
```wgsl
@fragment
fn fs_main(@location(0) world_pos: vec2<f32>) -> @location(0) vec4<f32> {
    let grid_size: f32 = 50.0;
    let grid_factor = abs(fract(world_pos / grid_size - 0.5) - 0.5) / fwidth(world_pos / grid_size);
    let line = min(grid_factor.x, grid_factor.y);
    let color = mix(GRID_COLOR, BG_COLOR, 1.0 - smoothstep(0.0, 1.0, line));
    return color;
}
```

Avantages : grille parfaitement nette à toutes les échelles, aucun moiré.

## Pipeline 2 — Shapes (rect, ellipse)

### Instance buffer
```rust
#[repr(C)]
#[derive(Clone, Copy)]
struct ShapeInstance {
    // Transform (mat3 packed)
    matrix: [f32; 6],     // ax, ay, bx, by, tx, ty
    // Style
    fill: [u8; 4],
    stroke: [u8; 4],
    stroke_width: f32,
    // Kind discriminator
    kind: u32,            // 0 = rect, 1 = ellipse, 2 = rounded-rect
    corner_radius: f32,
    half_size: [f32; 2],
}
```

### Vertex shader
- 1 quad par instance (4 vertices)
- Position calculée par instance matrix
- UV propagés au fragment (en world space relatif au centre)

### Fragment shader (SDF)
```wgsl
fn sdf_rect(p: vec2<f32>, half_size: vec2<f32>, radius: f32) -> f32 {
    let d = abs(p) - half_size + vec2<f32>(radius);
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0) - radius;
}

fn sdf_ellipse(p: vec2<f32>, half_size: vec2<f32>) -> f32 {
    // approx ellipse SDF
    let r = p / half_size;
    return (length(r) - 1.0) * min(half_size.x, half_size.y);
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    let d = match in.kind {
        0u => sdf_rect(in.uv_centered, in.half_size, in.corner_radius),
        1u => sdf_ellipse(in.uv_centered, in.half_size),
        _ => 1.0,
    };
    
    let alpha = 1.0 - smoothstep(-1.0, 1.0, d);  // anti-aliasing
    let fill = in.fill * alpha;
    
    // Stroke
    let stroke_alpha = 1.0 - smoothstep(in.stroke_width - 1.0, in.stroke_width + 1.0, abs(d));
    let final_color = mix(fill, in.stroke, stroke_alpha);
    
    return final_color;
}
```

Avantage SDF : anti-aliasing **automatique et parfait**, indépendant de la résolution.

## Pipeline 3 — Strokes (dessin libre)

### Approche
- Pour chaque stroke : générer un polygon par `perfect-freehand`
- Trianguler ce polygon (lyon_tessellation)
- Upload des vertices au GPU
- Render simple (color fill)

### Optimisation pour les strokes "live" (en cours de dessin)
- Le stroke change à chaque frame (nouveau point appended)
- On recalcule la fin du polygone uniquement (incremental)
- Cache le buffer triangulé en RAM

### Code
```rust
fn render_stroke(stroke: &Pen, target: &mut RenderTarget) {
    // Génère le polygon depuis les points
    let outline = perfect_freehand::get_stroke(&stroke.points, &stroke.options);
    
    // Trianguler
    let mesh = tessellate_polygon(&outline);
    
    // Upload + draw
    target.draw_triangles(&mesh, stroke.style.color);
}
```

## Pipeline 4 — Texte

Choix : crate `glyphon` (atlas raster).

```rust
struct TextSystem {
    font_system: cosmic_text::FontSystem,
    cache: glyphon::Cache,
    atlas: glyphon::TextAtlas,
    renderer: glyphon::TextRenderer,
}

fn render_text(text: &Text, world_pos: Vec2, ...) {
    let buffer = cosmic_text::Buffer::new(...);
    buffer.set_text(&text.content, ...);
    
    self.renderer.prepare(
        &self.device,
        &self.queue,
        &self.atlas,
        viewport_resolution,
        [glyphon::TextArea { buffer, top: ..., left: ..., ... }],
        cache,
    )?;
    
    self.renderer.render(&self.atlas, render_pass)?;
}
```

## Pipeline 5 — Images

```rust
struct ImageBatch {
    instances: Vec<ImageInstance>,
}

struct ImageInstance {
    matrix: [f32; 6],
    texture_layer: u32,  // index dans un texture array
    uv_min: [f32; 2],
    uv_max: [f32; 2],
    tint: [u8; 4],
}
```

- Texture array pour batcher plusieurs images en 1 draw call
- Si trop d'images : plusieurs batches
- Mipmaps activées pour le zoom out

## Pipeline 6 — Remote cursors

Très simple : N quads + texte. Pas de batching nécessaire (volume faible).

```rust
fn render_remote_cursors(cursors: &[AwarenessState], render_pass: &mut RenderPass) {
    for cur in cursors {
        // 1. Quad pour le triangle curseur
        draw_cursor_triangle(cur.pos, cur.color);
        // 2. Texte du nom
        draw_name_label(cur.user.name, cur.pos + offset);
    }
}
```

## Pipeline 7 — Selection overlays

Bordures autour des éléments sélectionnés (par l'utilisateur ou par les peers).

```rust
fn render_selection(selected: &[ElementId], color: Color) {
    for id in selected {
        let bbox = scene.element_bbox(*id);
        draw_dashed_rectangle(bbox, color, dash_pattern);
    }
}
```

Dash pattern via shader (modulo de la distance UV).

## Pipeline 8 — egui overlay

eframe gère ça. egui rend ses panels par-dessus la zone canvas.

## Synchronisation GPU-CPU

### Frame budget interactif
- CPU prépare le frame N+1 pendant que GPU rend le frame N
- 1 frame de latence acceptable
- Double-buffering activé par défaut

### Vsync
- Activé par défaut → 60 ou 144 FPS selon écran
- Désactivable en setting pour tests perf

## Sub-pixel rendering

Tout en SDF → naturellement sub-pixel.

Pour le texte : cosmic-text + glyphon gèrent ça avec leur cache de glyphes.

## Multi-sampling

- MSAA optionnel (2x ou 4x) en setting
- Coût GPU non négligeable mais améliore la qualité au zoom intermédiaire
- Default : pas de MSAA (SDF suffit)

## Color management

- BSE travaille en **sRGB** (canvas de base)
- Conversion linéaire en interne pour les blends corrects
- wgpu surface en `Bgra8UnormSrgb`

## Cibles de performance

| Métrique | Cible |
|---|---|
| Frame time p50 | <8 ms |
| Frame time p99 | <16 ms |
| Draw calls par frame | <50 |
| GPU memory (1000 éléments) | <50 MB |

## Optimisations futures

- **Indirect rendering** (GPU-driven culling)
- **Compute shaders** pour la tessellation des strokes
- **Vello** intégré pour les paths complexes (mindmap connecteurs courbés)

## Liens

- Performance détaillée → [06-performance.md](./06-performance.md)
- Stack rendu → [../04-STACK-TECHNIQUE/03-rendu-canvas.md](../04-STACK-TECHNIQUE/03-rendu-canvas.md)
