# 08.02 — Toolbar et palette d'outils

> Tous les outils canvas et leurs interactions.

## Toolbar gauche

Position : verticale à gauche, 48 px de large.

```
┌────┐
│ ↖ │ ← Select (V)
├────┤
│ ✏ │ ← Pen (P)
├────┤
│ ▢ │ ← Rectangle (R)
├────┤
│ ⬭ │ ← Ellipse (O)
├────┤
│ ╱ │ ← Line (L)
├────┤
│ ↗ │ ← Arrow (A)
├────┤
│ T │ ← Text (T)
├────┤
│ 📋 │ ← Postit (N)
├────┤
│ 🖼 │ ← Image (I)
├────┤
│ 🌳 │ ← Mindmap (M)
├────┤
│ ✂ │ ← Eraser (E)
└────┘
```

Chaque bouton :
- Icône
- Tooltip avec nom + raccourci
- Highlight si actif
- Sub-menu sur long-click (si variantes)

## Outil par défaut

**Select (V)** au démarrage. C'est le mode le plus fréquent (sélectionner, déplacer, modifier).

## Comportement Spacebar

**Tenir Espace** = activate temporary Pan mode. Toolbar n'est pas modifiée. Au relâchement, retour à l'outil actif.

## Liste complète des outils

### Select (V)
- Click → sélectionne un élément (ou rien)
- Click + drag (sur vide) → box selection
- Drag sur élément → move
- Double-click → mode édition spécifique (texte, mindmap node)
- Right-click → context menu

### Pen (P)
- Pen, Marker, Highlighter dans sous-menu
- Variante : Pencil (v1.x)
- Click + drag → trace
- Pression supportée

### Rectangle (R)
- Click + drag → crée
- Shift → carré
- Alt → centré sur point de départ

### Ellipse (O)
- Identique à Rectangle, en ellipse

### Line (L)
- Click → début
- Click again → fin
- Click + drag également supporté

### Arrow (A)
- Identique à Line
- Embouts automatiques
- Si on click sur un élément : ancre automatique (smart connector)

### Text (T)
- Click → crée un texte (auto-width)
- Click + drag → crée avec largeur fixée
- Entre directement en mode édition

### Postit (N)
- Click → crée un post-it taille medium à la position
- Auto-select de la couleur (par utilisateur)
- Entre directement en mode édition

### Image (I)
- Click → ouvre file picker
- Drop d'image bypass l'outil

### Mindmap (M)
- Click → crée un nœud racine
- Tab/Enter pour étendre
- Drag entre nœuds → reparent

### Eraser (E)
- Click sur élément → supprime
- Drag → supprime tous les éléments traversés

## Sous-menus / variantes

Long-click ou right-click sur un bouton outil → sous-menu :

```
Pen ───►
       ├── Pen
       ├── Marker
       ├── Highlighter
       └── Pencil  (post-v1)

Rectangle ───►
              ├── Rectangle
              ├── Rounded rectangle
              └── Square (Shift forced)

Ellipse ───►
            ├── Ellipse
            ├── Circle (Shift forced)
            └── ...
```

## Properties panel (à droite)

Apparaît automatiquement quand un élément est sélectionné.

```
┌─────────────────────┐
│ Rectangle           │
├─────────────────────┤
│ Position            │
│ X: [100  ] Y: [200] │
│ Taille              │
│ W: [150  ] H: [80 ] │
├─────────────────────┤
│ Style               │
│ Fill   [▓ #FF6B6B ▼]│
│ Stroke [─ #000000 ▼]│
│ Width  [─2 px────●]│
│ Dash   [─── solid ▼]│
│ Radius [──── 8 px ●]│
│ Opacity[▓▓▓▓▓░░░ 80%]│
├─────────────────────┤
│ Rotation : 0°       │
└─────────────────────┘
```

Multi-select : valeurs communes ou « mixed ».

## Color picker

Modal :
```
┌──────────────────────────────────┐
│ Couleur                          │
├──────────────────────────────────┤
│ ┌──────────────┐  ┌────┐         │
│ │              │  │    │ ← preview│
│ │  Color wheel │  └────┘         │
│ │              │                 │
│ └──────────────┘                 │
│ Hue: [────●─────────] 0°         │
│ Sat: [──────●───────] 70%        │
│ Lum: [─────●────────] 50%        │
│ Hex: [#FF6B6B]                   │
│                                  │
│ Récentes : ▓ ▓ ▓ ▓ ▓ ▓ ▓         │
│ Palette  : ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓       │
└──────────────────────────────────┘
```

- Récentes : 7 dernières couleurs utilisées
- Palette : couleurs de marque + presets

## Right-click context menu

Sur un élément :
```
─────────────────────
Cut         Ctrl+X
Copy        Ctrl+C
Duplicate   Ctrl+D
Delete      Del
─────────────────────
Bring forward    Ctrl+]
Send backward    Ctrl+[
Lock             Ctrl+Shift+L
─────────────────────
Add comment      Ctrl+Alt+M
Convert to...    ▶
─────────────────────
```

Sur vide :
```
─────────────────────
Paste     Ctrl+V
─────────────────────
Add postit    N
Add text      T
Add image...  I
─────────────────────
Zoom to fit   Ctrl+3
Reset zoom    Ctrl+0
─────────────────────
```

## Facilitator tools (mode facilitateur)

Quand l'utilisateur a le rôle facilitator, des outils additionnels apparaissent en bas :

```
┌────────────────────────────────────────┐
│ Timer [▶ 05:00 ] [Reset] [Vote] [Summon]│
└────────────────────────────────────────┘
```

Détail dans [../01-BRAINSTORMING-RECHERCHE/04-facilitation.md](../01-BRAINSTORMING-RECHERCHE/04-facilitation.md).

## Accessibilité toolbar

- Toolbar focusable au Tab
- Flèches haut/bas pour naviguer
- Enter pour activer
- Labels ARIA

## Personnalisation (v1.x)

- Réordonner les outils
- Cacher certains
- Définir des outils favoris

## Liens

- Raccourcis complets → [04-raccourcis-clavier.md](./04-raccourcis-clavier.md)
- Principes → [01-principes-design.md](./01-principes-design.md)
- Features → [../07-FEATURES/](../07-FEATURES/)
