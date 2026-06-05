# BSE — Brainstorm Shared Environment

> Une application collaborative en Rust pour brainstormer en équipe sur une toile infinie : dessin libre, formes, texte, images, mindmaps, post-its — édités à plusieurs en temps réel.

## Vision en une phrase

> *« Un Figma/Miro/Excalidraw open-source, performant, auto-hébergeable, écrit en Rust, où une équipe peut "build" ensemble sur une grande carte vierge, comme on construirait à plusieurs sur un serveur Minecraft. »*

---

## Index de la documentation

Cette documentation est organisée en 13 dossiers thématiques. Chaque dossier traite un aspect du projet — du *pourquoi* (recherche brainstorming, état de l'art) au *comment* (architecture, stack, features, déploiement).

### [00 — Vue d'ensemble](./00-VUE-ENSEMBLE/)
Introduction, vision produit, glossaire, roadmap haut niveau.

| Fichier | Sujet |
|---|---|
| [01-introduction.md](./00-VUE-ENSEMBLE/01-introduction.md) | Présentation générale du projet BSE |
| [02-vision-produit.md](./00-VUE-ENSEMBLE/02-vision-produit.md) | Vision, valeurs, principes directeurs |
| [03-glossaire.md](./00-VUE-ENSEMBLE/03-glossaire.md) | Lexique technique et produit |
| [04-roadmap.md](./00-VUE-ENSEMBLE/04-roadmap.md) | Feuille de route MVP → V1 → V2 |

### [01 — Recherche sur le brainstorming](./01-BRAINSTORMING-RECHERCHE/)
La théorie, la science, les méthodes et outils du brainstorming en équipe — la fondation conceptuelle de BSE.

| Fichier | Sujet |
|---|---|
| [01-theorie-et-science.md](./01-BRAINSTORMING-RECHERCHE/01-theorie-et-science.md) | Histoire, principes d'Osborn, perte de productivité |
| [02-methodes-techniques.md](./01-BRAINSTORMING-RECHERCHE/02-methodes-techniques.md) | 12+ techniques (brainwriting, 6-3-5, SCAMPER…) |
| [03-outils-existants.md](./01-BRAINSTORMING-RECHERCHE/03-outils-existants.md) | Panorama Miro/Mural/FigJam/Excalidraw/tldraw |
| [04-facilitation.md](./01-BRAINSTORMING-RECHERCHE/04-facilitation.md) | Comment animer une session efficace |
| [05-brainstorming-distant.md](./01-BRAINSTORMING-RECHERCHE/05-brainstorming-distant.md) | Async / remote / distributed |
| [06-ia-brainstorming.md](./01-BRAINSTORMING-RECHERCHE/06-ia-brainstorming.md) | IA comme co-équipier d'idéation |

### [02 — État de l'art](./02-ETAT-DE-LART/)
Étude des solutions existantes : ce qu'on en apprend pour BSE.

| Fichier | Sujet |
|---|---|
| [01-figma-figjam.md](./02-ETAT-DE-LART/01-figma-figjam.md) | Architecture multiplayer de Figma |
| [02-miro-mural.md](./02-ETAT-DE-LART/02-miro-mural.md) | Modèle SaaS enterprise |
| [03-excalidraw.md](./02-ETAT-DE-LART/03-excalidraw.md) | Architecture open-source / E2E |
| [04-tldraw.md](./02-ETAT-DE-LART/04-tldraw.md) | tldraw-sync + Durable Objects |
| [05-tableau-comparatif.md](./02-ETAT-DE-LART/05-tableau-comparatif.md) | Comparaison synthétique |
| [06-positionnement-bse.md](./02-ETAT-DE-LART/06-positionnement-bse.md) | Différenciateurs de BSE |

### [03 — Architecture](./03-ARCHITECTURE/)
Architecture haut niveau du système BSE.

| Fichier | Sujet |
|---|---|
| [01-vue-ensemble.md](./03-ARCHITECTURE/01-vue-ensemble.md) | Schéma global client/serveur |
| [02-client.md](./03-ARCHITECTURE/02-client.md) | Architecture interne de l'app desktop |
| [03-serveur.md](./03-ARCHITECTURE/03-serveur.md) | Architecture du backend (rooms, services) |
| [04-protocole-reseau.md](./03-ARCHITECTURE/04-protocole-reseau.md) | Format des messages, transport |
| [05-modele-donnees.md](./03-ARCHITECTURE/05-modele-donnees.md) | Schéma des projets, scènes, éléments |
| [06-diagrammes.md](./03-ARCHITECTURE/06-diagrammes.md) | Diagrammes ASCII de l'architecture |

### [04 — Stack technique](./04-STACK-TECHNIQUE/)
Choix de bibliothèques et justifications.

| Fichier | Sujet |
|---|---|
| [01-rust-pourquoi.md](./04-STACK-TECHNIQUE/01-rust-pourquoi.md) | Pourquoi Rust pour ce projet |
| [02-gui-framework.md](./04-STACK-TECHNIQUE/02-gui-framework.md) | egui vs iced vs slint vs gpui… |
| [03-rendu-canvas.md](./04-STACK-TECHNIQUE/03-rendu-canvas.md) | wgpu, vello, skia |
| [04-networking.md](./04-STACK-TECHNIQUE/04-networking.md) | axum + tokio + WebSocket / QUIC / iroh |
| [05-base-donnees.md](./04-STACK-TECHNIQUE/05-base-donnees.md) | Postgres + SQLite client + objet S3 |
| [06-stockage-assets.md](./04-STACK-TECHNIQUE/06-stockage-assets.md) | Images, fichiers binaires |
| [07-recommandations-finales.md](./04-STACK-TECHNIQUE/07-recommandations-finales.md) | Stack BSE recommandée |

### [05 — Collaboration temps réel](./05-COLLABORATION-TEMPS-REEL/)
Le cœur du projet : comment N utilisateurs peuvent éditer la même toile sans conflit.

| Fichier | Sujet |
|---|---|
| [01-crdt-fondamentaux.md](./05-COLLABORATION-TEMPS-REEL/01-crdt-fondamentaux.md) | Théorie des CRDT |
| [02-yrs-loro-automerge.md](./05-COLLABORATION-TEMPS-REEL/02-yrs-loro-automerge.md) | Comparaison Rust CRDT libs |
| [03-choix-bse.md](./05-COLLABORATION-TEMPS-REEL/03-choix-bse.md) | Choix retenu et justification |
| [04-presence-cursors.md](./05-COLLABORATION-TEMPS-REEL/04-presence-cursors.md) | Curseurs distants, awareness |
| [05-undo-redo.md](./05-COLLABORATION-TEMPS-REEL/05-undo-redo.md) | Annulation locale en multi-user |
| [06-conflits-cas-limites.md](./05-COLLABORATION-TEMPS-REEL/06-conflits-cas-limites.md) | Cas limites et résolutions |

### [06 — Canvas infini](./06-CANVAS-INFINI/)
La toile infinie : système de coordonnées, caméra, performance.

| Fichier | Sujet |
|---|---|
| [01-systeme-coordonnees.md](./06-CANVAS-INFINI/01-systeme-coordonnees.md) | Repère monde / écran |
| [02-camera-zoom-pan.md](./06-CANVAS-INFINI/02-camera-zoom-pan.md) | Caméra, transformations |
| [03-spatial-indexing.md](./06-CANVAS-INFINI/03-spatial-indexing.md) | Quadtree, R-tree |
| [04-culling-lod.md](./06-CANVAS-INFINI/04-culling-lod.md) | Viewport culling, level-of-detail |
| [05-pipeline-rendu.md](./06-CANVAS-INFINI/05-pipeline-rendu.md) | Pipeline de rendu GPU |
| [06-performance.md](./06-CANVAS-INFINI/06-performance.md) | Cibles, profiling, optimisations |

### [07 — Fonctionnalités](./07-FEATURES/)
Détail de chaque feature.

| Fichier | Sujet |
|---|---|
| [01-dessin-libre.md](./07-FEATURES/01-dessin-libre.md) | Stylo, pression, perfect-freehand |
| [02-formes-geometriques.md](./07-FEATURES/02-formes-geometriques.md) | Rectangle, ellipse, polygone, flèche |
| [03-texte-typo.md](./07-FEATURES/03-texte-typo.md) | Édition de texte sur canvas |
| [04-images-medias.md](./07-FEATURES/04-images-medias.md) | Import images, vidéo, embed |
| [05-mindmap-connecteurs.md](./07-FEATURES/05-mindmap-connecteurs.md) | Mindmap, connecteurs intelligents |
| [06-post-its-cards.md](./07-FEATURES/06-post-its-cards.md) | Post-its colorés |
| [07-templates.md](./07-FEATURES/07-templates.md) | Templates de session |
| [08-export-import.md](./07-FEATURES/08-export-import.md) | PNG, PDF, SVG, JSON |

### [08 — UX / UI](./08-UX-UI/)
Principes d'interface et design system.

| Fichier | Sujet |
|---|---|
| [01-principes-design.md](./08-UX-UI/01-principes-design.md) | Principes directeurs UX |
| [02-toolbar-outils.md](./08-UX-UI/02-toolbar-outils.md) | Barre d'outils, palette |
| [03-multi-curseurs-presence.md](./08-UX-UI/03-multi-curseurs-presence.md) | Indicateurs de présence |
| [04-raccourcis-clavier.md](./08-UX-UI/04-raccourcis-clavier.md) | Liste des raccourcis |
| [05-design-system.md](./08-UX-UI/05-design-system.md) | Design system Miro (via getdesign) |

> 📌 **Tokens visuels (couleurs, typographie, composants)** : voir [`/DESIGN.md`](../DESIGN.md) à la racine du repo.

### [09 — Sécurité](./09-SECURITE/)
Authentification, permissions, chiffrement.

| Fichier | Sujet |
|---|---|
| [01-authentification.md](./09-SECURITE/01-authentification.md) | OAuth / JWT / sessions |
| [02-permissions-rbac.md](./09-SECURITE/02-permissions-rbac.md) | Rôles, ACL par projet |
| [03-chiffrement.md](./09-SECURITE/03-chiffrement.md) | TLS, E2E optionnel |
| [04-modele-de-menace.md](./09-SECURITE/04-modele-de-menace.md) | Threat model |

### [10 — Déploiement](./10-DEPLOIEMENT/)
Comment installer et faire tourner BSE.

| Fichier | Sujet |
|---|---|
| [01-architectures-cibles.md](./10-DEPLOIEMENT/01-architectures-cibles.md) | Self-host / cloud / hybride |
| [02-self-hosted.md](./10-DEPLOIEMENT/02-self-hosted.md) | Docker, compose, K8s |
| [03-cloud.md](./10-DEPLOIEMENT/03-cloud.md) | SaaS BSE Cloud |
| [04-distribution-binaires.md](./10-DEPLOIEMENT/04-distribution-binaires.md) | Builds Windows/macOS/Linux |

### [11 — Roadmap et exécution](./11-ROADMAP-EXECUTION/)
Planification.

| Fichier | Sujet |
|---|---|
| [01-mvp.md](./11-ROADMAP-EXECUTION/01-mvp.md) | Périmètre MVP (3 mois) |
| [02-jalons-v0-v1.md](./11-ROADMAP-EXECUTION/02-jalons-v0-v1.md) | Jalons v0.1 → v1.0 |
| [03-risques.md](./11-ROADMAP-EXECUTION/03-risques.md) | Risques techniques et mitigation |
| [04-equipe.md](./11-ROADMAP-EXECUTION/04-equipe.md) | Profils nécessaires |

### [12 — Références](./12-REFERENCES/)
Bibliographie.

| Fichier | Sujet |
|---|---|
| [01-crates-rust.md](./12-REFERENCES/01-crates-rust.md) | Liste exhaustive des crates utiles |
| [02-projets-open-source.md](./12-REFERENCES/02-projets-open-source.md) | Projets à étudier |
| [03-papers-academiques.md](./12-REFERENCES/03-papers-academiques.md) | Papers CRDT, OT, brainstorming |
| [04-liens-externes.md](./12-REFERENCES/04-liens-externes.md) | Tous les liens externes |

---

## Lecture recommandée selon ton profil

- **Vue d'ensemble rapide (30 min)** : 00-VUE-ENSEMBLE/, puis 02-ETAT-DE-LART/05, puis 03-ARCHITECTURE/01.
- **Pour démarrer l'implémentation (2-3 h)** : 04-STACK-TECHNIQUE/ en entier + 05-COLLABORATION-TEMPS-REEL/03 + 06-CANVAS-INFINI/05 + 11-ROADMAP-EXECUTION/01.
- **Pour comprendre le *pourquoi* brainstorming** : 01-BRAINSTORMING-RECHERCHE/ en entier.
- **Pour vendre / pitcher** : 00-VUE-ENSEMBLE/02 + 02-ETAT-DE-LART/06.

---

## Convention de versionnement de la doc

- Cette documentation est versionnée avec le code dans `D:\BSE\`.
- Toute décision technique majeure doit être tracée en bas du fichier concerné dans une section *« Décisions / changements »*.
- Date de la version initiale : **2026-06-05**.
