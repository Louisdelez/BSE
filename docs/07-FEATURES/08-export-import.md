# 07.08 — Export et import

> Faire entrer et sortir des données de BSE.

## Vision

Un projet BSE ne doit jamais être prisonnier. Exports multiples, formats ouverts.

## Exports

### v1.0
- **PNG** : viewport courant ou scène complète
- **SVG** : vectoriel propre
- **PDF** : multi-pages possibles (1 page = 1 zone définie)
- **JSON natif BSE** : format complet et lossless

### v1.x
- **PowerPoint (.pptx)** : zones → slides
- **Markdown** : texte des éléments (post-its, texte) en MD
- **CSV** : pour les post-its (1 ligne = 1 post-it)

### v2
- **Excalidraw (.excalidraw)** : compat
- **Figma** : export structuré (auto-layout, frames)
- **Webm/MP4** : enregistrement animé de la session

## Export PNG

### Options
- Zone : viewport courant / scène entière / sélection / zone définie
- Résolution : 1x, 2x, 4x (pour print)
- Background : couleur projet ou transparent
- Inclure les overlays (curseurs distants) : non par défaut

### Implementation
- Rendre la scène dans une texture offscreen (wgpu)
- Lire la texture en RGBA8
- Encoder PNG via crate `image`
- Save sur disque

## Export SVG

- Génération XML à partir des éléments
- Chaque type d'élément → balise SVG correspondante
  - Rectangle → `<rect>`
  - Ellipse → `<ellipse>`
  - Path/Pen → `<path>` avec courbes Bézier
  - Text → `<text>`
  - Image → `<image>` (base64 inline ou href)

### Spécificités
- Préserve la vectoriel pour les formes simples
- Image rasters embedées en base64
- Polices : embed ou keyword (Arial, sans-serif)

## Export PDF

V1.0 : multi-page = 1 page par "frame" (zone définie).

Implementation : crate `printpdf` ou `genpdf`.

## Export JSON natif

Le format BSE natif. Lossless. Utile pour :
- Backup
- Export → import dans un autre instance BSE
- Versioning (Git-friendly)

```json
{
  "version": "1.0",
  "project": { "id": "...", "name": "..." },
  "elements": [
    { "id": "...", "kind": { "type": "Rectangle", "width": 100, "height": 50, ... }, ... },
    ...
  ],
  "settings": { ... }
}
```

## Imports

### Du JSON natif
Trivial : déserialize et apply ops CRDT.

### Du Excalidraw
- Parse JSON Excalidraw
- Map des types : `excalidraw.rect → BSE.Rectangle`, etc.
- Quelques pertes (style hand-drawn rendu différent)

### De Miro / FigJam (v2)
Pas d'API publique d'export. Limite.

### Drag-drop d'images
Cf [04-images-medias.md](./04-images-medias.md).

### Paste de texte
Crée des post-its ou des éléments texte.

### Paste d'image (presse-papier)
Crée un Element Image, upload comme un import standard.

## Format `.bse` (futur)

Un fichier `.bse` est un **container** :
- JSON du projet
- Dossier `assets/` avec les binaires
- Compressé en zip

```
mon-projet.bse (zip)
├── project.json
├── assets/
│   ├── sha256_abc...png
│   ├── sha256_def...jpg
│   └── ...
└── metadata.json
```

Permet de partager un projet complet en 1 fichier.

## Sécurité de l'import

- Validation du JSON (taille, structure)
- Pas d'exécution
- Pas de chargement de polices arbitraires (whitelist)
- Sanitize SVG importés

## UX

### Menu
```
Fichier
├── Nouveau projet... (Ctrl+N)
├── Ouvrir... (Ctrl+O)
├── Sauvegarder (Ctrl+S)  ← local snapshot
├── Sauvegarder sous... (Ctrl+Shift+S)
├── ────────
├── Importer
│   ├── Depuis JSON BSE...
│   ├── Depuis Excalidraw...
│   └── Depuis image...
├── Exporter
│   ├── PNG... (Ctrl+E)
│   ├── SVG...
│   ├── PDF...
│   └── JSON...
└── Quitter
```

### Dialog d'export
```
┌────────────────────────────┐
│ Exporter en PNG            │
├────────────────────────────┤
│ Zone :                     │
│ ○ Viewport courant         │
│ ● Scène entière            │
│ ○ Sélection                │
│ ○ Zone définie...          │
│                            │
│ Résolution :  [2x ▼]       │
│                            │
│ ☑ Fond transparent         │
│ ☐ Inclure curseurs distants│
│                            │
│ [Exporter] [Annuler]       │
└────────────────────────────┘
```

## Tests

- Round-trip JSON : export → import → identique
- Export PNG : pixel-perfect avec rendu écran
- Export SVG ouvrable dans Inkscape sans erreur
- Excalidraw import : majorité des éléments OK

## Liens

- Modèle → [../03-ARCHITECTURE/05-modele-donnees.md](../03-ARCHITECTURE/05-modele-donnees.md)
- Assets → [04-images-medias.md](./04-images-medias.md)
