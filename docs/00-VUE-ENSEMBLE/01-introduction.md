# 00.01 — Introduction au projet BSE

## Qu'est-ce que BSE ?

**BSE** (*Brainstorm Shared Environment*) est une application **desktop collaborative** écrite en **Rust** qui permet à plusieurs personnes de **brainstormer ensemble en temps réel** sur une **toile infinie partagée**.

Concrètement, BSE permet à un utilisateur de :

1. **Créer plusieurs projets** (chaque projet = une toile indépendante)
2. **Inviter des collaborateurs** à rejoindre un projet
3. **Naviguer librement** sur une toile sans limites (zoom, dézoom, pan)
4. **Ajouter du contenu** : dessins libres, formes géométriques, texte, images, post-its, mindmaps
5. **Voir les autres en temps réel** : leurs curseurs, leurs sélections, leurs modifications
6. **Travailler en parallèle** sur la même zone ou des zones différentes — comme on construirait à plusieurs sur un même serveur Minecraft

## Analogie fondatrice : le serveur Minecraft

L'analogie au serveur Minecraft est centrale dans la vision du produit :

> *Sur un serveur Minecraft, plusieurs joueurs partagent un même monde. Chacun peut être dans une région différente en train de construire son propre bâtiment, ou bien tous peuvent collaborer sur le même bâtiment en se répartissant les tâches. La carte est immense, librement explorable, et tout ce qui est construit reste persistent.*

BSE applique ce paradigme à la **créativité collaborative** :

| Minecraft | BSE |
|---|---|
| Un *monde* | Un *projet* |
| Une grande carte explorable | Une toile infinie |
| Téléportation / déplacement | Zoom / pan / recentrage |
| Blocs, items | Formes, dessins, post-its, images |
| Plusieurs joueurs simultanés | Plusieurs éditeurs simultanés |
| Inventaire d'outils | Barre d'outils du canvas |
| Sauvegarde du monde | Persistance du projet |
| Permissions (whitelist, op) | Rôles (owner, éditeur, lecteur) |

## Pourquoi maintenant ?

Trois constats motivent BSE :

1. **Les outils dominants sont propriétaires et cloud-only.** Figma, Miro, Mural enferment les données chez eux. Pour des équipes soucieuses de souveraineté (recherche, défense, secteur public, R&D industrielle), aucune alternative crédible.

2. **Les outils open-source existants sont limités.** Excalidraw est élégant mais minimaliste. tldraw est puissant mais SDK web sous licence non-libre. Aucun n'offre une expérience desktop native fluide.

3. **Rust est mûr pour ça en 2026.** L'écosystème GUI (egui, iced, slint, gpui), le rendu 2D GPU (wgpu, vello), les CRDT (yrs, loro), et les networking (axum, iroh) sont arrivés au point où une app desktop native multijoueurs est non seulement possible mais peut être *plus performante* que ses équivalents Electron / web.

## Public cible

| Persona | Cas d'usage |
|---|---|
| **Équipes produit** | Ateliers d'idéation, design sprints |
| **Équipes de recherche** | Cartographie d'idées, modélisation de concepts |
| **Enseignants** | Tableau blanc collaboratif pour la classe |
| **Studios créatifs** | Moodboards, storyboards |
| **Équipes distribuées** | Workshops à distance asynchrones |
| **Hackathons** | Coordination visuelle d'équipe |

## Ce que BSE n'est PAS

Pour clarifier le périmètre :

- **Pas un outil de design pixel-perfect** (≠ Figma) — pas de variants, design system complexe
- **Pas un outil de modélisation BPM/UML strict** (≠ Lucidchart) — orienté créativité, pas formalisme
- **Pas un IDE / éditeur de texte** — pas de pretension à remplacer VSCode
- **Pas une messagerie** — la communication parlée passe par tes outils habituels (Discord, Teams…)
- **Pas un outil de gestion de projet** (≠ Jira/Notion) — c'est un *canvas*, pas un *tracker*

## Trois principes directeurs

1. **Performance native** — 60 FPS minimum, démarrage en <500 ms, latence collaborative <100 ms en LAN.
2. **Souveraineté** — auto-hébergeable en une commande, données chez soi, code open-source.
3. **Simplicité d'usage** — un nouvel utilisateur doit pouvoir créer son premier dessin en <30 secondes après installation.

## Prochaines lectures

- [02-vision-produit.md](./02-vision-produit.md) — vision long terme
- [03-glossaire.md](./03-glossaire.md) — vocabulaire
- [../02-ETAT-DE-LART/06-positionnement-bse.md](../02-ETAT-DE-LART/06-positionnement-bse.md) — différenciation
