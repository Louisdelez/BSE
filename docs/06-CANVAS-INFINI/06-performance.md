# 06.06 — Performance

> Cibles, instrumentation, optimisation.

## Cibles de performance

| Métrique | Cible v1.0 |
|---|---|
| FPS (60Hz screen) | 60 soutenu, jamais sous 30 |
| FPS (144Hz screen) | 144 soutenu sur GPU récent |
| Démarrage cold | <500 ms |
| Démarrage warm (cache) | <200 ms |
| Memory au repos | <100 MB |
| Memory en session (1000 elem) | <250 MB |
| Memory en session (10K elem) | <600 MB |
| CPU au repos | <2% |
| Latence input pointer | <2 frames |
| Latence sync LAN p95 | <50 ms |
| Latence sync WAN p95 | <200 ms |
| Bandwidth typique 10 peers | <100 KB/s |

## Hardware cible

### Bas de gamme (doit fonctionner correctement)
- CPU : Intel i5 6e gen, 4 cores, 8 GB RAM
- GPU : intégré Intel HD 620 ou équivalent
- 100% des features doivent fonctionner, mais sur grosses scènes peut tomber à 30 FPS

### Mid-range (cible primaire)
- CPU : i5/Ryzen 5 récent, 8 cores, 16 GB RAM
- GPU : intégré ou GTX 1650
- 60 FPS sur 5000 éléments visibles

### Haut de gamme
- CPU : i7/Ryzen 7+
- GPU : dédié récente
- 144 FPS sur 10K éléments visibles

## Instrumentation

### tracing
```rust
use tracing::{info, debug, instrument};

#[instrument(skip(scene))]
fn render_frame(scene: &Scene, camera: &Camera) {
    let _span = tracing::info_span!("render").entered();
    // ...
}
```

### puffin / tracy
Pour le profilage continu en dev :
```rust
puffin::profile_function!();
puffin::profile_scope!("upload buffers");
```

UI Puffin attachable au runtime pour voir la flame chart.

### Métriques runtime
- Frame time (rolling average, p50, p99)
- Memory : `dhat` ou `tikv-jemallocator` pour profiler
- GPU : timing queries wgpu

## Budget par sous-système

À 60 FPS = 16,6 ms total :

| Sous-système | Budget |
|---|---|
| Input | 0.2 ms |
| State update | 0.5 ms |
| Spatial query | 0.5 ms |
| egui UI | 1.5 ms |
| Build batches | 1 ms |
| Render shapes | 2 ms |
| Render strokes | 2 ms |
| Render text | 1.5 ms |
| Render images | 1.5 ms |
| Other passes | 1 ms |
| Compose + present | 2 ms |
| Marge | 2.9 ms |

Si on dépasse dans un sous-système : profiler et optimiser ce sous-système précisément.

## Optimisations connues

### 1. Spatial index
Cf [03-spatial-indexing.md](./03-spatial-indexing.md). **Indispensable** au-delà de 500 éléments.

### 2. Instanced rendering
Regrouper les shapes similaires en draw calls instancés. Gain : 10-100×.

### 3. Texture atlas
Pour texte et icônes. Réduit le nombre de bind groups.

### 4. Reuse buffers
Ring buffers pour les uploads transitoires (vertex/index/instance).

### 5. CRDT incrementality
Ne pas relire l'état CRDT complet à chaque frame. Notifier `bse-canvas` des deltas.

### 6. Throttle awareness
30 Hz max pour les curseurs.

### 7. Cull aggressively
20% margin viewport pour pre-render lors du pan.

### 8. LOD
Pas de texte rendu sous 8 px ; quad coloré pour les éléments minuscules.

### 9. Async asset loading
Décodage image sur autre thread, GPU upload sur main.

### 10. Avoid alloc dans le hot path
Pré-allouer les Vec, réutiliser.

## Techniques pour démarrage rapide

- Lazy load : ne charger qu'un projet, pas tous
- Persistance SQLite légère, schema minimal
- Pas de splash screen 2s pour rien
- Pré-compile shaders au build (pas au runtime)

## Mémoire

### Sources de consommation
- CRDT Doc : ~50-200 KB pour 1000 éléments
- Spatial index : ~5-50 KB
- GPU buffers : variable, fonction de scène
- egui state : ~few MB
- Asset cache : config 256 MB max
- OS / runtime : ~50 MB

### Profiling
```bash
DHAT_OUT=dhat.json cargo run --release
# Puis ouvrir dhat.json dans Firefox profiler
```

### Stratégies
- Pas de cloning gratuit (rust borrow checker aide)
- Texture cache LRU
- Eviction des assets non visibles depuis 5 min

## Tests de performance

### Suite synthétique
```rust
#[test]
fn perf_scene_10k() {
    let scene = generate_scene(10_000);
    let bench = || render_frame(&scene, ...);
    let times = benchmark(bench, 100_iters);
    assert!(times.p99 < Duration::from_millis(16));
}
```

### Suite réaliste
- Scène test : mindmap de 200 nodes, 50 stickers, 30 images
- Stress : 10 peers connectés simulés en train d'éditer
- Soak : 1h continue, mesurer drift mémoire

### CI perf
Trends sur les release :
- Si frame time p99 augmente >10% → flag dans PR
- Tracking dans Grafana (ou dossier `perf/` versionné)

## Anti-patterns à éviter

### ❌ Re-rendre tout à chaque petit changement
Ex : un cursor move ne devrait pas recalculer le spatial index.

### ❌ Allocations dans le hot path
Pas de `Vec::new()` à chaque frame pour les visible elements. Réutiliser le Vec.

### ❌ Lock contention
Pas de `Arc<Mutex<Scene>>` lu par 5 threads. Préférer Arc<Scene> immutable + channels.

### ❌ Synchronous network IO
Tout async, jamais bloquer le thread main.

### ❌ Excessive logging
Logs `tracing::info` dans le hot path = perf killer. Logs en `debug` ou `trace`.

## Edge cases connus

### Très grand stroke (5000+ points)
- Triangulation incrémentale
- LOD : sous-échantillonnage si trop éloigné

### Très grande image (5000×5000)
- Mipmaps obligatoires
- Décodage progressif (JPEG progressif, WebP)

### Beaucoup de texte (1000+ post-its avec texte long)
- Cache glyphes
- LOD texte aggressif

## Pour finir

> **La performance native est notre différenciateur clé.** Chaque release doit inclure des tests perf qui ne régressent jamais.
