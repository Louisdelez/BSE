# 07.04 — Images et médias

> Import et affichage de fichiers images sur la toile.

## Cas d'usage

- Moodboards
- Captures d'écran d'analyse
- Photos de prototypes
- Diagrammes externes (PNG/SVG)
- Icônes de bibliothèque

## Formats supportés

### v1.0
- **PNG** ✅
- **JPEG / JPG** ✅
- **WebP** ✅
- **GIF** ✅ (statique ou animé)
- **SVG** ⚠️ (importé comme PNG rendu côté client)

### v1.x
- **AVIF** (décodage Rust mûr)
- **HEIC** (iPhone)
- **PDF** (1 page = 1 image, multi-page = embed)

### Hors scope v1
- Vidéo (mp4)
- Audio
- 3D models

## Workflow d'import

### Méthodes
1. **Drag & drop** depuis le bureau / explorateur
2. **Copy-paste** depuis presse-papier
3. **Bouton « Insérer image »** dans la toolbar
4. **Drag depuis un site web** (URL)

### Pipeline backend

```
1. Capture du fichier (drop event, paste, ...)
2. Validation MIME + magic bytes
3. Vérification taille (< 20 MB v1.0)
4. Calcul SHA-256
5. Check side-server : déjà uploadé ?
   ─► Oui : skip upload, juste créer l'Element
   ─► Non : upload (multipart POST)
6. Server génère variantes (thumb, medium)
7. Server enregistre metadata
8. Client crée l'Element { kind: Image { asset_id, width, height } }
9. Op CRDT broadcast
```

## Modèle

```rust
pub struct Image {
    pub asset_id: AssetId,        // SHA-256 du fichier
    pub width: f32,                // taille en coords monde
    pub height: f32,
    pub crop: Option<Rect>,        // crop UV (0..1)
    pub fit: ImageFit,             // contain / cover / fill
}

pub enum ImageFit {
    Original,    // taille originale
    Contain,    // fit dans le bbox sans déformer
    Cover,      // remplit le bbox sans déformer (crop)
    Stretch,    // étire au bbox
}
```

## Affichage

- Tex texture array (cf [../06/05-pipeline-rendu.md](../06-CANVAS-INFINI/05-pipeline-rendu.md))
- Variant selection selon zoom :
  - thumb → cover view, zoom out
  - medium → vue normale
  - original → vue zoom in
- Mipmaps activées pour le filtering

## Manipulations

### Resize
- Resize aux 8 handles
- Shift drag → préserve aspect ratio (par défaut activé sur images)

### Crop
- Outil dédié (raccourci C ?)
- Affiche les 8 handles internes
- Drag → ajuste le crop UV

### Rotate
- Handle de rotation
- Image rotated, mais la bbox AABB est calculée pour le hit test

### Tint / filtres (v1.x)
- Saturation, luminosité, contraste
- Noir et blanc, sépia
- Implementation : shader avec uniforms

## SVG

Le SVG est complexe à rendre vectoriellement en WGSL. Options :

### Option A (v1.0) : rasterize au moment de l'import
- `resvg` ou `usvg` pour parser
- Render en PNG haute résolution (2x ou 4x du affichage)
- Stocker comme image PNG
- Limite : pas de zoom infini

### Option B (v1.x) : rendre via vello
- `vello` peut rendre des paths SVG
- Vrai vectoriel infini
- Plus complexe

V1 : option A. V1.x : option B avec vello.

## GIF animé

- Décoder en boucle de frames
- Updater la texture GPU à chaque frame du GIF
- Optimisation : pause hors viewport (économie CPU)

## Liens externes

Quand on drag une URL d'image :
1. Fetch l'image
2. Stocker localement
3. Créer un Element Image standard

L'utilisateur peut ne pas vouloir l'embed → option « lien externe » avec preview.

## Performance

### Décodage
- Background thread (tokio task)
- Crate `image` (PNG, JPG, GIF, WebP)
- Crate `resvg` (SVG)
- GPU upload une fois décodé

### Mémoire
- Cache LRU 256 MB GPU + 1 GB disque
- Eviction des textures non utilisées (hors viewport >5 min)
- Format intermédiaire : RGBA8

### Bandwidth
- Variant thumb (~10 KB) chargée en premier
- Variant medium chargée pour le viewport actuel
- Variant original chargée si zoom in (>200%)

## UX

### Indicateur de chargement
- Placeholder gris pendant le download
- Progress bar discrète si fichier lourd

### Image manquante
Si l'asset n'existe plus côté serveur (ex: import / export, ou cleanup) :
- Placeholder « image not found »
- Option de remplacer

### Compression info
Tooltip sur l'image : taille originale, taille stockée, dimensions.

## Sécurité

- Validation MIME stricte (anti-spoofing)
- SVG sanitization (anti-XSS si on rend via WebView un jour)
- Limites de taille
- Pas d'exécution de code

## Tests

- Import PNG, JPG, GIF, WebP, SVG
- Drag depuis bureau, paste, drag depuis URL
- Resize, crop, rotate
- Persistance après reload
- Dé-duplication : 2 imports même fichier → 1 asset stocké

## Liens

- Stockage assets → [../04-STACK-TECHNIQUE/06-stockage-assets.md](../04-STACK-TECHNIQUE/06-stockage-assets.md)
- Modèle → [../03-ARCHITECTURE/05-modele-donnees.md](../03-ARCHITECTURE/05-modele-donnees.md)
