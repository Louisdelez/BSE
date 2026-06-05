# 07.01 — Dessin libre (Pen)

> Le tracé pression-sensible. Cœur d'un outil de brainstorming.

## Objectifs

- Tracé naturel, lisse, esthétique
- Pression de stylet supportée
- Performance : 60+ FPS pendant le dessin
- Collaboration : autres peers voient le tracé en quasi temps réel

## Algorithme : perfect-freehand

L'algorithme **perfect-freehand** (Steve Ruiz, créateur de tldraw) transforme un ensemble de points (avec pression) en un **polygone d'épaisseur variable**, anti-aliasé, naturel.

### Caractéristiques
- Pression → épaisseur
- Smoothing : adoucit les angles
- Tapering : début et fin effilés
- Streamline : lisse les mouvements brusques de la souris

### Paramètres de style
```rust
pub struct PenOptions {
    pub size: f32,            // épaisseur de base (px)
    pub thinning: f32,        // -1..1 (intensité effet pression)
    pub smoothing: f32,       // 0..1 (lissage des angles)
    pub streamline: f32,      // 0..1 (lissage du tracé)
    pub start_taper: f32,     // 0..1 (effilement début)
    pub end_taper: f32,       // 0..1 (effilement fin)
    pub last: bool,           // est-ce un stroke fini ?
}
```

## Port Rust

Pas de port officiel. Options :
1. Implémenter en Rust depuis l'original TS (~500 lignes, faisable)
2. Wrapper WASM (overhead inacceptable)
3. Algorithme maison équivalent

**Choix BSE** : implem Rust maison s'inspirant de perfect-freehand. Test contre une suite de comparaison TS pour validation.

## Pipeline de dessin

```
1. Input :
   - pointer_down → début de stroke
   - pointer_move → append point
   - pointer_up → fin de stroke

2. Pour chaque point :
   - Capter position (x, y), pression, tilt, timestamp
   - Append au stroke en cours

3. Throttle : si plus de 240 Hz (tablette extreme), on downsample

4. Régénération polygon (incrementale dans la mesure du possible)

5. Triangulation (lyon_tessellation)

6. GPU upload (ring buffer)

7. Render

8. Broadcast op CRDT (1 batch per ~33ms, pas par point)
```

## Capture de pression

### Windows
- API Pointer (Pointer ID) ou WinTab/WinPen
- winit expose `Touch` event avec `force` (pressure)

### macOS
- NSEvent → `pressure` field
- Trackpad force touch également

### Linux
- libinput (X11/Wayland)
- Stylet généralement reconnu

### Fallback (souris sans pression)
- Pression simulée : `simulatePressure = true`
- Basée sur la vitesse et l'accélération du mouvement
- Résultat correct mais moins naturel

## Optimisation incrementale

Pendant qu'un stroke est en cours, on **ne regénère pas le polygon complet** à chaque point. On :
1. Re-triangule seulement la queue (derniers ~10 points)
2. Append au buffer GPU
3. Le restant déjà uploaded reste tel quel

Gain de perf : x10 sur un long stroke.

## Smoothing temps réel

Pour réduire le jitter de la souris/stylet :
- Filtre Kalman ou moyenne pondérée
- Latence ajoutée : 1-2 frames (acceptable)

## Collaboration

### Stroke en cours d'un peer
Bob commence un stroke. Tant qu'il ne `pointer_up` :
- Alice voit les points apparaître progressivement (batched op CRDT toutes les 33ms)
- Léger délai (~50-100 ms WAN) mais naturel

### Stroke terminé
- Op finale envoyée avec `last: true`
- L'élément est figé (immutable hors undo)

### Conflits ?
Aucun. Chaque stroke est un élément distinct. Pas de conflit possible.

## Outils variantes

| Outil | Variante |
|---|---|
| **Pen** | Stroke noir/coloré standard |
| **Marker** | Stroke épais semi-transparent |
| **Highlighter** | Stroke jaune épais, blend mode multiply |
| **Pencil** | Stroke fin avec texture grain (post-v1) |
| **Eraser** | Trace de suppression (objet entier sous le passage) |

## Eraser

Deux modes :
- **Object eraser** : passe sur un élément → suppression entière
- **Pixel eraser** : (v1.x) découpe le tracé en passant — complexe à implem en vectoriel

V1 : object eraser uniquement.

## Cibles de performance

| Métrique | Cible |
|---|---|
| Latence input → affichage | <33 ms (2 frames @ 60 Hz) |
| FPS pendant dessin | 60 soutenu |
| Stroke max acceptable | 10K points |
| Throughput throttle | 240 Hz → 120 Hz décimé |

## Tests

- Tracé visuel proche de tldraw / Excalidraw (référence)
- Pression réelle vs simulée : comparaison side-by-side
- Stress : 10 peers dessinent en parallèle, FPS reste >60

## Liens

- Rendering details → [../06-CANVAS-INFINI/05-pipeline-rendu.md](../06-CANVAS-INFINI/05-pipeline-rendu.md)
- Modèle Pen → [../03-ARCHITECTURE/05-modele-donnees.md](../03-ARCHITECTURE/05-modele-donnees.md)
- Tablette stylet → [02-formes-geometriques.md](./02-formes-geometriques.md) pour les autres input modes
