# 08.01 — Principes de design UX

> Les règles directrices pour toutes les décisions UI de BSE.

## Les 7 principes

### 1. Le canvas est le héros
Tout ce qui n'est pas le canvas (toolbar, panels, dialogs) doit être **discret et collapsible**. Au démarrage, l'utilisateur doit voir 90% canvas, 10% chrome UI.

### 2. Mode focus par défaut
- Pas de panels ouverts par défaut
- Toolbar minimale en haut
- Tout autre élément à la demande (Tab, F-keys)

### 3. Tout au clavier accessible
Pour les power users, **chaque action doit avoir un raccourci**. La souris est un accélérateur, pas une obligation.

### 4. Une seule manière de faire (par feature)
Pas 3 menus différents pour la même action. Évite la fatigue décisionnelle.

### 5. Pas de modaux bloquants
Les dialogs sont **non-bloquants** quand possible. Les actions critiques (delete projet) ont confirmation mais ne bloquent pas le reste.

### 6. Feedback immédiat
Toute action a un retour visuel **dans la frame** :
- Hover : highlight subtil
- Click : flash
- Long action : spinner / progress
- Erreur : message non intrusive (toast)

### 7. Cohérence avant nouveauté
Quand on rencontre un cas inconnu, on regarde Figma/Miro et on copie les patterns familiers. Pas de réinvention.

## Layout général

```
┌──────────────────────────────────────────────────────────────┐
│ [≡][📁 Mon projet][👥 3]                          [🔍][⚙][❔]│  ← Top bar
├──────────────────────────────────────────────────────────────┤
│ ●                                                            │
│ │     ┌───┐                                          ┌────┐  │
│ │     │T  │ ←─ left toolbar (tools)                  │    │  │
│ │     │R  │                                          │Side│  │
│ │     │O  │                                          │bar │  │
│ │     │P  │                                          │    │  │
│ │     │I  │           THE CANVAS                     │    │  │
│ │     │M  │                                          │    │  │
│ │     │A  │                                          │    │  │
│ │     │G  │                                          │    │  │
│ │     │S  │                                          │    │  │
│ │     └───┘                                          └────┘  │
│ │                                                            │
│ ●─── [Zoom: 100%] [Sync ●] [Bob, Charlie] [⏱]    [Mini map]  │
└──────────────────────────────────────────────────────────────┘
```

### Top bar
- Menu hamburger (réservé)
- Nom du projet (cliquable pour renommer)
- Présence : avatars des peers connectés
- Search (Ctrl+F)
- Settings
- Help

### Left toolbar
Outils canvas. Petite (40-50 px width). Verticale.

### Right sidebar (collapsible)
- Properties panel quand sélection
- Layers panel
- Comments
- Templates browser
Tab pour switch.

### Bottom bar
- Zoom indicator
- Connection status
- Quick presence
- Timer (si actif)
- Mini-map (toggleable)

## Couleurs et typographie : voir DESIGN.md

> 📌 **Les tokens visuels précis (couleurs, typographie, espacements) sont définis dans le fichier [`/DESIGN.md`](../../DESIGN.md) à la racine du repo.**
>
> Ce fichier est généré via `npx getdesign@latest add miro` et constitue la **source de vérité** du design system BSE (inspiré de Miro).
>
> Détails du choix et adaptation pour BSE : [05-design-system.md](./05-design-system.md).

### Principes de couleur
- Palette inspirée de **Miro** : brand yellow `#ffd02f`, accents pastel (rose, teal, coral, orange, mint) — exactement les couleurs naturelles des post-its
- **Mode sombre** : variantes inverses, supporté dès v0.5
- **WCAG AA** minimum sur tous les contrastes texte/fond
- **Couleurs peers** : palette de 8 distinctes pour la collaboration (cf [../05-COLLABORATION-TEMPS-REEL/04-presence-cursors.md](../05-COLLABORATION-TEMPS-REEL/04-presence-cursors.md))

### Principes typographiques
- **Famille principale** : **Inter** (open-source, en remplacement de Roobert PRO commerciale de Miro)
- **Hiérarchie par graisse** plus que par taille pour le compact
- **UI compact par défaut** (densité « pro »), modes confort et spacious disponibles

## Espacements

Grille 4 px. Tout est multiple de 4.

```
xs  = 4
sm  = 8
md  = 12
lg  = 16
xl  = 24
2xl = 32
```

## Animations

### Durations
- Micro (hover, focus) : 100 ms
- Standard (slide, fade) : 200 ms
- Macro (page transition) : 300 ms

### Easing
- Default : `cubic-bezier(0.4, 0.0, 0.2, 1)` (Material standard)
- Spring pour les éléments interactifs

### Performance
Aucune animation ne doit faire chuter le frame rate. Si nécessaire, désactivables pour profils "performance".

## Densité

L'UI peut être à 3 densités :
- **Compact** : pour les pros (default)
- **Confortable** : pour les nouveaux
- **Spacious** : pour les écrans haute résolution

## Accessibilité (a11y)

- Toutes les couleurs respectent WCAG AA (contraste 4.5:1 min)
- Tab navigation complète
- Focus visible
- Labels ARIA via AccessKit
- Réduction des animations (`prefers-reduced-motion`)
- Mode haute contraste

## Internationalisation

- Strings en `i18n/{lang}.toml`
- Langues v1.0 : Français, Anglais
- Plus dans v1.x (Espagnol, Allemand, Japonais, Chinois)

## Anti-patterns explicitement bannis

- ❌ **Tutoriel intrusif au démarrage**
- ❌ **Popup « ajoutez votre carte de crédit »** (pas de pub, BSE est libre)
- ❌ **Cookies bannière** (app desktop, pas de cookies)
- ❌ **Dark patterns** : aucun
- ❌ **Notification push pour engager** : non
- ❌ **Gamification** (badges, streaks) : non

## Tests UX

- Onboarding : nouvel utilisateur crée un projet et fait un dessin en <60 s
- Power user : actions principales accessible en <2 clics ou 1 raccourci
- Accessibilité : screen reader basique fonctionne (NVDA, VoiceOver)
- Performance perçue : aucune action ne semble laggy

## Liens

- **Design system (tokens)** → [05-design-system.md](./05-design-system.md) + [`/DESIGN.md`](../../DESIGN.md)
- Toolbar → [02-toolbar-outils.md](./02-toolbar-outils.md)
- Multi-curseurs → [03-multi-curseurs-presence.md](./03-multi-curseurs-presence.md)
- Raccourcis → [04-raccourcis-clavier.md](./04-raccourcis-clavier.md)
