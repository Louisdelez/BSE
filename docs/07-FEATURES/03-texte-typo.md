# 07.03 — Texte et typographie

> Édition de texte directement sur la toile, avec support collaboratif.

## Cas d'usage

- Annotation libre sur la toile
- Titres de zones
- Notes à plusieurs lignes
- Édition simultanée par plusieurs peers (CRDT text)

## Outil texte

Activé par `T`. Modes :
1. **Click** → crée un texte ponctuel, largeur auto (wrap au niveau caractère)
2. **Click + drag** → crée un texte avec largeur fixée (wrap selon la largeur)

## Modèle de données

```rust
pub struct Text {
    pub content: String,           // dans yrs::Text
    pub width: Option<f32>,        // None = auto-width, Some = fixed-width avec wrap
    pub font: FontStyle,
    pub align: TextAlign,
    pub vertical_align: VerticalAlign,
}

pub struct FontStyle {
    pub family: String,            // "Inter", "Roboto", ...
    pub size: f32,                 // en pixels world
    pub weight: u16,               // 100..900
    pub style: FontStyleKind,      // Normal / Italic
    pub color: Color,
    pub line_height: f32,
    pub letter_spacing: f32,
}

pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}
```

## Polices

### Polices intégrées
Bundle dans l'app :
- **Inter** (sans-serif, défaut)
- **JetBrains Mono** (monospace)
- **Caveat** (handwriting, pour le mode hand-drawn)
- **Roboto** (sans-serif alternative)

Total bundle : ~5-10 MB de fichiers TTF/OTF.

### Polices système
Optionnellement, charger les polices système :
- Lib : `system-fonts` ou `font-kit`
- Performance : indexer au démarrage (cache)

### Polices custom
v1.x : import de polices custom par projet (.ttf, .otf).

## Édition collaborative

Le `content` est un `yrs::Text` — CRDT texte natif.

Quand Bob tape pendant qu'Alice tape :
- Chaque caractère = une op CRDT
- Position de curseur = awareness (cf [../05-COLLABORATION-TEMPS-REEL/04-presence-cursors.md](../05-COLLABORATION-TEMPS-REEL/04-presence-cursors.md))
- Pas d'interleaving anomaly (avec yrs YATA, ou Fugue si Loro)

### Curseur dans le texte
Alice voit le curseur de Bob dans le texte (couleur de Bob).

```
"Bonjour |Bob "
         ↑
         curseur de Bob
```

### Sélection dans le texte
Idem pour les sélections de plage. Bob highlight des mots, Alice voit en surbrillance colorée.

## Édition

### Modes
- **Hors édition** : le bloc texte est un élément standard (sélection, move, etc.)
- **En édition** : double-click → entre dans le bloc, curseur clavier, on tape

### Sortir d'édition
- Echap
- Click hors du bloc
- Ctrl+Enter

### Formatage (v1.x — rich text)

V1.0 : texte plain only. Tout le bloc partage le même style.

V1.x : rich text avec marqueurs sur ranges (bold, italic, underline, color). Nécessite yrs::Text avec attributs ou bascule Loro.

## Rendu

### Pipeline texte (cf [../06/05-pipeline-rendu.md](../06-CANVAS-INFINI/05-pipeline-rendu.md))
- `glyphon` + `cosmic-text`
- Atlas raster de glyphes
- Anti-aliasing via subpixel

### Performance
- Cache des layouts (text → glyph positions)
- Invalidation seulement si content/style change
- Glyph atlas LRU

## Layout

### Wrap
- Si `width` est None : pas de wrap automatique (newline manuel `\n`)
- Si `width` est défini : word wrap automatique

### Line height
- Par défaut : 1.2 × font size
- Configurable dans le panel

### Letter spacing
- 0 par défaut
- Range -10 .. +20 px

## Empty text behavior

Quand on crée un texte et qu'il reste vide à la fin de l'édition :
- L'élément est **supprimé** automatiquement
- Évite le canvas pollué de blocs vides

## Hit testing

- En mode sélection : hit test sur la bbox du bloc texte
- En mode édition : click positionne le curseur dans le texte (calcul via layout)

## Tests

- Édition simple : taper, supprimer, naviguer (arrows, home, end)
- Édition collaborative : 2 peers tapent dans le même bloc → convergence
- Wrap : changer la largeur change le layout
- Polices : switch d'une famille à l'autre
- Performance : 100 blocs de texte sur scène → 60 FPS

## Limites v1.0

- Pas de rich text (couleurs/styles inline)
- Pas de listes à puces
- Pas de tableaux
- Pas de Markdown rendering
- Pas de spell check

→ V1.x ou v2.

## Liens

- Modèle → [../03-ARCHITECTURE/05-modele-donnees.md](../03-ARCHITECTURE/05-modele-donnees.md)
- Awareness curseur → [../05-COLLABORATION-TEMPS-REEL/04-presence-cursors.md](../05-COLLABORATION-TEMPS-REEL/04-presence-cursors.md)
- Pipeline rendu texte → [../06-CANVAS-INFINI/05-pipeline-rendu.md](../06-CANVAS-INFINI/05-pipeline-rendu.md)
