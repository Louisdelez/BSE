# 08.05 — Design system

> Source unique de vérité pour les tokens visuels de BSE. Inspiré de Miro.

## Décision

BSE adopte le design system de **Miro** comme référence visuelle, généré dans le fichier `DESIGN.md` à la racine du repo via l'outil [getdesign](https://www.getdesign.app/).

### Commande utilisée

```bash
npx getdesign@latest add miro
```

Cette commande génère un fichier `DESIGN.md` à la racine du projet contenant tous les tokens (couleurs, typographie, espacements, composants, états) du design system Miro, sous forme exploitable par un agent de codage ou un développeur humain.

## Pourquoi Miro ?

| Critère | Justification |
|---|---|
| **Cohérence produit** | BSE est dans la famille whiteboard collaboratif comme Miro |
| **Maturité** | Design system éprouvé sur des millions d'utilisateurs |
| **Lisibilité** | Palette claire, hiérarchie typographique solide |
| **Esthétique** | Jaune brand `#ffd02f` + accent multi-couleurs (rose, teal, coral, orange, mint) — *exactement* les couleurs naturelles des post-its sur la toile |
| **Composants** | Pricing grids, comparison tables, feature cards — tous patterns réutilisables pour le site marketing BSE |

## Où vivent les tokens ?

> 📌 **Fichier de référence unique** : [`/DESIGN.md`](../../DESIGN.md) à la racine du repo.

Ce fichier est **autoritaire**. Tout le code UI de BSE doit s'y conformer. En cas de divergence entre ce document et DESIGN.md, DESIGN.md gagne.

## Contenu de DESIGN.md (résumé)

### Colors
- **Primary** : `#1c1c1e` (ink) / `#ffffff` (on-primary)
- **Brand yellow** : `#ffd02f` (la signature Miro)
- **Brand blue** : `#4262ff`
- **Accents pastel** : coral `#ff9999`, rose `#ffd8f4`, teal `#0fbcb0`, orange light `#ffe6cd`
- **Surfaces** : canvas `#ffffff`, surface `#f7f8fa`, surface soft `#fafbfc`
- **Hairlines** : `#e0e2e8` à `#c7cad5`
- **Inks** : ink-deep `#050038` à muted `#a5a8b5`

### Typography
- **Famille principale** : **Roobert PRO**
- **Hero display** : 80 px / 500 weight / line-height 1.05 / letter-spacing -2 px
- **Display lg** : 60 px
- **Heading 1 à 4** : 48 → 22 px, weight 500
- **Body** : tailles standards (à compléter selon DESIGN.md)

### Composants notables (Miro)
- Boutons pills noirs primary
- Cartes feature avec aperçus de board réels
- Tables de comparaison pricing 4-tier
- Tints pastel pour les sections produit

## Adaptation pour BSE

Quelques différences à anticiper :

| Aspect | Miro | BSE |
|---|---|---|
| Cible | Web SaaS | Desktop natif |
| Densité | Marketing, large | Compact (app de travail) |
| Polices | Roobert PRO (commerciale) | **Inter** comme fallback OSS (Roobert PRO est sous licence) |
| Mode sombre | Pas mis en avant | Supporté dès v0.5 |
| Yellow brand | Très présent | Présent mais discret (curseurs, accents) |

### Choix sur la police
Roobert PRO étant une police commerciale, **BSE utilise Inter** (open-source, similaire visuellement) comme police principale, et garde Roobert PRO comme référence quand on contribue au site marketing.

### Choix sur les couleurs
On garde la palette Miro **telle quelle** pour les couleurs des post-its (cohérence d'écosystème mental). On adapte la palette des surfaces pour le mode sombre.

## Comment regénérer / mettre à jour

Si Miro fait évoluer son design system, on peut rafraîchir :

```bash
cd D:\BSE
npx getdesign@latest add miro
git add DESIGN.md
git commit -m "docs(design): refresh Miro design system tokens"
```

## Lien avec les principes

Le fichier [01-principes-design.md](./01-principes-design.md) définit les **principes** (focus canvas, accessibilité, performance perçue). Les **tokens** sont dans DESIGN.md. Les deux se complètent :

- Principes → *pourquoi* et *comment* nous décidons
- DESIGN.md → *quelles valeurs précises* nous utilisons

## Workflow pour les agents de codage

Quand on demande à un agent (Claude Code, Cursor, etc.) d'écrire de l'UI :

> *« Avant d'écrire du code UI, lis `/DESIGN.md` à la racine. Tous les tokens, couleurs, espacements et composants doivent suivre ce fichier. »*

C'est précisément le pattern recommandé par getdesign.

## Sources

- getdesign : [getdesign.app](https://www.getdesign.app/)
- npm : [npmjs.com/package/getdesign](https://www.npmjs.com/package/getdesign)
- GitHub : [MohtashamMurshid/getdesign](https://github.com/MohtashamMurshid/getdesign)
- Miro design analysis : [getdesign.md/miro/design-md](https://getdesign.md/miro/design-md)

## Liens

- [DESIGN.md](../../DESIGN.md) — le fichier source de vérité
- [01-principes-design.md](./01-principes-design.md) — principes UX (séparés des tokens)
- [02-toolbar-outils.md](./02-toolbar-outils.md) — composants UI concrets
