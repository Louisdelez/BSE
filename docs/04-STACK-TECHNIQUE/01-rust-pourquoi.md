# 04.01 — Pourquoi Rust pour BSE

> Justification du choix de Rust comme langage **client ET serveur**.

## Le résumé en 3 phrases

> 1. **Performance native** : 60-144 FPS soutenu, démarrage <500 ms, mémoire <100 MB — impossibles à atteindre en JS/Electron sur ce cas d'usage.
> 2. **Sécurité mémoire sans GC** : pas de pauses GC qui font sauter une frame ; pas de buffer overflow ni race conditions imprévisibles.
> 3. **Ecosystem mature en 2026** : GUI, GPU rendering, async networking, CRDT — toutes les briques sont là.

## Les arguments en détail

### 1. Pas de garbage collector

Pour une app de canvas à 60+ FPS, **chaque pause GC est un frame sauté**. Java/.NET/Go ont des GC modernes mais imprévisibles. Rust gère la mémoire à la compilation → **latence prévisible**.

### 2. Performance ~= C/C++ sans piège de mémoire

Rust égale ou dépasse C++ sur la plupart des benchmarks de rendu et networking, **sans** les classes de bugs (segfault, use-after-free, data race).

### 3. Cross-platform natif

Compile vers Windows, macOS, Linux, iOS, Android, **et WebAssembly**. Donc on peut imaginer plus tard un client web léger qui partage le moteur Rust avec le desktop.

### 4. Backend et client en même langage

Logique partagée (modèles, sérialisation, protocole) dans des crates `bse-protocol`, `bse-types`. **Pas de duplication entre TS et autre chose**.

### 5. Ecosystem clé en 2026

| Domaine | Crate(s) clé | Maturité |
|---|---|---|
| GUI immediate-mode | egui | ⭐⭐⭐⭐⭐ |
| Rendu GPU | wgpu | ⭐⭐⭐⭐⭐ |
| Rendu vectoriel 2D | vello | ⭐⭐⭐ (alpha mature) |
| Async runtime | tokio | ⭐⭐⭐⭐⭐ |
| HTTP serveur | axum | ⭐⭐⭐⭐⭐ |
| WebSocket | tokio-tungstenite | ⭐⭐⭐⭐⭐ |
| CRDT | yrs, loro | ⭐⭐⭐⭐ |
| DB client | sqlx, sea-orm | ⭐⭐⭐⭐ |
| Serde | serde, rmp-serde | ⭐⭐⭐⭐⭐ |
| P2P | iroh, libp2p | ⭐⭐⭐⭐ |

### 6. Précédent : Figma a réécrit en Rust

Le serveur multiplayer de Figma a été réécrit de TypeScript vers Rust en 2023, avec un **gain de performance d'un ordre de grandeur** (citation directe). On part d'emblée à l'arrivée.

### 7. Compile-time safety pour la concurrence

Rust force à penser la concurrence à la compilation (lifetimes, Send/Sync). Sur un projet multi-threaded comme BSE, c'est inestimable. Les bugs de concurrence sont parmi les plus coûteux à débugger.

### 8. Communauté et trajectoire

Rust est en 2026 :
- **Top 5 mondial** des langages préférés des devs (Stack Overflow survey 7e année)
- **Adoption massive** chez Microsoft (Windows kernel), Google (Android), Meta, Discord, Cloudflare
- **Stabilité de l'écosystème** : breaking changes rares, async/await stable depuis 2019

## Arguments contre Rust (et pourquoi ils ne tiennent pas)

### « Rust est trop dur »
- Vrai pour les débutants. Mais pour un projet sérieux, l'investissement initial est largement payé sur la maintenabilité long terme.
- Les types et le borrow checker préviennent des classes entières de bugs qu'on chasserait en prod.

### « Le compilateur est lent »
- Vrai, surtout en debug avec dépendances. Mitigation :
  - `cargo check` est rapide
  - LTO seulement en release
  - Sccache / mold (linker rapide)
  - En 2026, les améliorations du compilateur ont réduit drastiquement
- Pas un blocant pour la productivité après setup correct.

### « L'écosystème GUI n'est pas mûr »
- Vrai en 2022. **Plus en 2026.** egui, iced, slint sont production-ready pour bcp de cas d'usage. cf [02-gui-framework.md](./02-gui-framework.md).

### « Pour itérer rapidement, mieux vaut TS »
- Sur un PoC <2 mois, peut-être. Sur un projet 2 ans, l'avantage Rust se cumule.
- Et BSE est précisément un projet où la performance native est un *différenciateur* — donc Rust est *le moyen* du produit, pas un détail.

## Comparaison rapide à des alternatives

### Rust vs Go (côté serveur)
- Go : plus simple, GC, écosystème solide
- Rust : plus performant, contrôle mémoire, partage de code avec client
- **Choix BSE** : Rust pour partager le code

### Rust vs C++ (côté client)
- C++ : plus mature pour le GPU, plus de bibliothèques 2D
- Rust : sécurité mémoire, tooling moderne, ergonomie
- **Choix BSE** : Rust ; gap d'écosystème comblé en 2026

### Rust vs TypeScript + Electron
- TS + Electron : itération rapide, vaste écosystème, mais perf catastrophique (Chrome embedded)
- Rust : compilation à apprendre, mais perf native
- **Choix BSE** : Rust ; on parie sur la performance différenciatrice

### Rust vs Swift/Kotlin natifs
- Swift/Kotlin : excellent sur leur plateforme, mais **non cross-platform**
- Rust : cross-platform un seul codebase
- **Choix BSE** : Rust pour cross-platform desktop

### Rust + Tauri ?
- Tauri = back-end Rust + front-end web (WebView)
- Plus simple à démarrer (UI HTML/CSS familière)
- Mais perdrait l'avantage perf canvas (WebView ralentit le rendu)
- **Choix BSE** : Rust **natif** (sans WebView), pour vraie perf canvas

## Risques liés au choix Rust

| Risque | Mitigation |
|---|---|
| Difficulté de recruter des devs Rust | Communauté en croissance ; OSS attire |
| Bibliothèque GUI immature pour cas extrêmes | On contribute upstream ; alternative iced disponible |
| Compile times longs | Sccache, mold, builds CI cache |
| Apprentissage initial | Onboarding 2-3 semaines pour dev expérimenté |

## Décision finale

> **Rust est le bon choix pour BSE.** L'écosystème de 2026 est mûr, la performance est différenciatrice, le partage client/serveur est précieux. L'investissement initial est largement amorti sur 2 ans.

## Liens

- GUI framework détaillé → [02-gui-framework.md](./02-gui-framework.md)
- Networking → [04-networking.md](./04-networking.md)
- Crates clés → [../12-REFERENCES/01-crates-rust.md](../12-REFERENCES/01-crates-rust.md)
