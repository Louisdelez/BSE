# BSE — Brainstorm Shared Environment

> **Application collaborative de brainstorming en temps réel sur toile infinie.**
> Desktop natif en **Rust**. Performant. Auto-hébergeable. Open-source.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)
[![Status: Scaffolding](https://img.shields.io/badge/Status-Scaffolding-blue.svg)](./docs)
[![Version: v004](https://img.shields.io/badge/Version-v004-green.svg)](./CHANGELOG.md)
[![CI](https://github.com/Louisdelez/BSE/actions/workflows/ci.yml/badge.svg)](https://github.com/Louisdelez/BSE/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/lang-Rust-orange.svg)](https://www.rust-lang.org)

---

## ✨ Vision

> *« Un Figma/Miro/Excalidraw open-source, performant, auto-hébergeable, écrit en Rust, où une équipe peut "build" ensemble sur une grande carte vierge — comme on construirait à plusieurs sur un serveur Minecraft. »*

BSE permet à plusieurs personnes de **brainstormer ensemble en temps réel** sur une **toile infinie partagée** : dessin libre, formes, texte, images, post-its, mindmaps. Tout est édité simultanément, comme dans Figma — mais **en application desktop native** (pas une web app).

## 🎯 Différenciateurs

| Axe | BSE | Concurrents |
|---|---|---|
| **Performance** | 60-144 FPS natif, démarrage <500 ms, <100 MB RAM | Web/Electron, lourds |
| **Souveraineté** | Auto-hébergeable en `docker compose up`, MIT, E2EE opt. | SaaS US verrouillé |
| **Brainstorming-first** | Templates dédiés, mode facilitation complet | Canvas généraliste |
| **Multi-utilisateur** | CRDT mature, offline-first, async/sync hybride | Variable |

## 📦 État du projet

> **v004 — Canvas pan / zoom + grille adaptative.**
> Vraie navigation infinie : Espace + drag pour pan, molette pour zoom centré sur le curseur.
> Premier outil interactif (Rectangle) arrive en v005.

Voir la [roadmap complète](./docs/00-VUE-ENSEMBLE/04-roadmap.md) et le [CHANGELOG](./CHANGELOG.md).

### 🚀 Compiler et lancer

```bash
# Requis : Rust 1.94+ (rustup recommandé)
cargo build --release
cargo run --release -p bse-app       # binaire client (smoke test en v002)
cargo run --release -p bse-server    # binaire serveur (smoke test en v002)
```

### 🧪 Vérifier la qualité

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## 🗂️ Documentation

La documentation complète se trouve dans [`/docs`](./docs/) — **70 fichiers répartis sur 13 sections** :

| Section | Description |
|---|---|
| [00-VUE-ENSEMBLE](./docs/00-VUE-ENSEMBLE/) | Introduction, vision produit, glossaire, roadmap |
| [01-BRAINSTORMING-RECHERCHE](./docs/01-BRAINSTORMING-RECHERCHE/) | Théorie, méthodes, outils, facilitation, IA |
| [02-ETAT-DE-LART](./docs/02-ETAT-DE-LART/) | Étude Figma, Miro, Mural, Excalidraw, tldraw |
| [03-ARCHITECTURE](./docs/03-ARCHITECTURE/) | Architecture client/serveur, protocole, modèle de données |
| [04-STACK-TECHNIQUE](./docs/04-STACK-TECHNIQUE/) | Rust, egui, wgpu, axum, Postgres, S3… |
| [05-COLLABORATION-TEMPS-REEL](./docs/05-COLLABORATION-TEMPS-REEL/) | CRDT, présence, undo, cas limites |
| [06-CANVAS-INFINI](./docs/06-CANVAS-INFINI/) | Coordonnées, caméra, spatial index, pipeline GPU |
| [07-FEATURES](./docs/07-FEATURES/) | Dessin, formes, texte, images, mindmap, post-its, templates |
| [08-UX-UI](./docs/08-UX-UI/) | Principes design, toolbar, multi-curseurs, raccourcis |
| [09-SECURITE](./docs/09-SECURITE/) | Auth, RBAC, chiffrement, threat model |
| [10-DEPLOIEMENT](./docs/10-DEPLOIEMENT/) | Self-host, cloud, distribution binaires |
| [11-ROADMAP-EXECUTION](./docs/11-ROADMAP-EXECUTION/) | MVP, jalons, risques, équipe |
| [12-REFERENCES](./docs/12-REFERENCES/) | Crates Rust, projets OSS, papers, liens |

**Démarrer ici** : [docs/README.md](./docs/README.md) pour la navigation, ou directement [00-VUE-ENSEMBLE/01-introduction.md](./docs/00-VUE-ENSEMBLE/01-introduction.md).

## 🎨 Design system

BSE utilise le design system **Miro** comme référence visuelle, généré via [`getdesign`](https://www.getdesign.app/) :

```bash
npx getdesign@latest add miro
```

Le fichier [`DESIGN.md`](./DESIGN.md) à la racine contient tous les tokens (couleurs, typographie, espacements, composants). C'est la **source de vérité** pour toute UI à écrire.

> 📌 Documentation du choix : [docs/08-UX-UI/05-design-system.md](./docs/08-UX-UI/05-design-system.md).

## 🛠️ Stack technique (résumé)

| Couche | Choix |
|---|---|
| Langage | **Rust** (client + serveur) |
| GUI | **egui** + **wgpu** + **winit** (via eframe) |
| Rendu 2D | **wgpu** + **vello** (pour vectoriel complexe) |
| CRDT | **yrs** (Y-CRDT Rust) — évaluation Loro en v0.5 |
| Serveur HTTP | **axum** + **tokio** |
| Transport | **WebSocket** (TLS) — option QUIC via **iroh** plus tard |
| DB serveur | **PostgreSQL** + **S3-compatible** (MinIO) |
| DB client | **SQLite** |
| Auth | OIDC + JWT |

Détails dans [04-STACK-TECHNIQUE](./docs/04-STACK-TECHNIQUE/).

## 🗺️ Roadmap macro

```
M0    M3    M6    M9   M12   M15   M18   M21   M24
│     │     │     │     │     │     │     │     │
│  MVP    v0.1   v0.5  v1.0       v1.1       v2.0
   POC    Beta   Préco Stable    Polish     Plugins
```

Détail : [11-ROADMAP-EXECUTION/](./docs/11-ROADMAP-EXECUTION/).

## 🔢 Système de versionning

Ce repo suit un système de **paliers numérotés `v001`, `v002`, `v003`…** alignés sur la roadmap.

Voir [VERSIONING.md](./VERSIONING.md) pour la convention complète.

- **`main`** : toujours stable, reflète le dernier palier
- **Tags `v0XX`** : milestone interne (1 par étape de la roadmap)
- **Branches `feat/0XX-nom`** : travail en cours avant merge

## 🚀 Démarrage rapide

> ⚠️ **Pas encore de code.** Cette version `v001` est la documentation fondatrice. Le code viendra en `v002`.

Pour suivre l'avancement :
- ⭐ Star le repo
- 👀 Watch les releases pour être notifié des nouveaux paliers
- 💬 [Discussions GitHub](../../discussions) pour échanger sur le design

## 🤝 Contribuer

À ce stade (`v001` documentation), les contributions bienvenues :

- Relecture et critique constructive de la documentation
- Suggestions sur l'architecture (issues GitHub)
- Proposition de cas d'usage ou de features manquantes

Le code arrivera en `v002`. À ce moment, un `CONTRIBUTING.md` sera ajouté avec les règles de PR.

## 📜 Licence

**MIT** — voir [LICENSE](./LICENSE).

Vous pouvez utiliser, modifier, redistribuer ce projet librement, y compris à des fins commerciales.

## 👤 Auteur

**Louis Delez** — 2026

## 🙏 Remerciements / inspirations

Ce projet s'inspire des travaux remarquables de :
- [**Figma**](https://figma.com) — pour l'architecture multiplayer
- [**Excalidraw**](https://excalidraw.com) — pour la simplicité et le chiffrement E2E
- [**tldraw**](https://tldraw.com) — pour l'engine de canvas
- [**Y-CRDT / Yjs**](https://github.com/yjs/yjs) — pour la fondation CRDT
- [**Loro**](https://loro.dev) — pour l'algorithm Fugue
- [**Linebender / Vello**](https://github.com/linebender/vello) — pour le rendu 2D GPU
- Toute la communauté **Rust** qui rend ce projet possible

---

> *« Imaginez Miro, mais qui démarre en moins d'une seconde, tourne à 144 FPS, vous appartient totalement, et que vous installez sur votre serveur en une commande. C'est BSE. »*
