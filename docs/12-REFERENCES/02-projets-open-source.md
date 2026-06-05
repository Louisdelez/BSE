# 12.02 — Projets open-source à étudier

> Repos à lire pour s'inspirer, comprendre les patterns, et éventuellement réutiliser.

## Canvas / whiteboard

### Excalidraw
- **Repo** : github.com/excalidraw/excalidraw
- **Lang** : TypeScript
- **Lic** : MIT
- **Étudier** : pattern P2P E2E, simplicité du serveur de relais

### Excalidraw-room
- **Repo** : github.com/excalidraw/excalidraw-room
- **Lang** : Node.js
- **Étudier** : minimal WebSocket relay server

### tldraw
- **Repo** : github.com/tldraw/tldraw
- **Lang** : TypeScript
- **Lic** : MIT (free tier) / paid SDK
- **Étudier** : engine de canvas hyper-performant, sync architecture

### tldraw-sync-cloudflare
- **Repo** : github.com/tldraw/tldraw-sync-cloudflare
- **Étudier** : starter kit Durable Objects

### Drawpile
- **Repo** : github.com/drawpile/Drawpile
- **Lang** : C++/Qt
- **Étudier** : protocole de sync raster, multi-user dessin

### Wave (Google)
- Discontinué mais doc historique sur le collaborative editing

## CRDT

### y-crdt / yrs
- **Repo** : github.com/y-crdt/y-crdt
- **Étudier** : implementation des CRDT, API, providers

### Yjs (référence JS)
- **Repo** : github.com/yjs/yjs
- **Étudier** : architecture, ecosystem (y-websocket, y-leveldb, etc.)

### Loro
- **Repo** : github.com/loro-dev/loro
- **Étudier** : algorithm Fugue, Movable Tree, time travel

### Automerge
- **Repo** : github.com/automerge/automerge
- **Étudier** : versioning Git-like

### Diamond Types
- **Repo** : github.com/josephg/diamond-types
- **Étudier** : Eg-walker algorithm, perf extrême

### Crdts (Rust toolkit)
- **Repo** : github.com/rust-crdt/rust-crdt
- **Étudier** : implems de différents CRDT classiques

## Rust Desktop apps

### Zed editor
- **Repo** : github.com/zed-industries/zed
- **Étudier** : gpui framework, performance, collaboration (CRDTs intégrés)
- **Inspiration** : architecture général, choix techniques

### Helix editor
- **Repo** : github.com/helix-editor/helix
- **Étudier** : TUI Rust (pas notre cas mais bon code)

### Lapce
- **Repo** : github.com/lapce/lapce
- **Étudier** : Rust + floem framework

### Bevy
- **Repo** : github.com/bevyengine/bevy
- **Étudier** : ECS, game engine en Rust

### Linebender / Druid / Xilem
- **Repo** : github.com/linebender/xilem
- **Étudier** : framework UI alternatif

## Rendu 2D

### Vello
- **Repo** : github.com/linebender/vello
- **Étudier** : GPU compute 2D rendering

### Skia
- **Repo** : github.com/google/skia
- **Étudier** : référence absolue 2D (C++)

### Cairo
- **Repo** : gitlab.freedesktop.org/cairo
- **Étudier** : référence CPU 2D

### Pathfinder
- **Repo** : github.com/servo/pathfinder
- **Étudier** : Rust GPU vector rendering

## Networking Rust

### Axum examples
- **Repo** : github.com/tokio-rs/axum/tree/main/examples
- **Étudier** : WebSocket, websockets-chat

### Iroh examples
- **Repo** : github.com/n0-computer/iroh
- **Étudier** : P2P patterns

### Hyper
- **Repo** : github.com/hyperium/hyper
- **Étudier** : HTTP bas niveau Rust

### Tokio examples
- **Repo** : github.com/tokio-rs/tokio
- **Étudier** : patterns async

## Auth

### Keycloak (Java mais référence OIDC)
- **Repo** : github.com/keycloak/keycloak

### Authelia
- **Repo** : github.com/authelia/authelia
- **Étudier** : OIDC en Go

### Authentik
- **Repo** : github.com/goauthentik/authentik

## Persistence

### Postgres
- **Repo** : github.com/postgres/postgres
- **Étudier** : LSN, WAL, MVCC

### MinIO
- **Repo** : github.com/minio/minio
- **Étudier** : S3-compatible self-host

## Outils similaires

### Affine
- **Repo** : github.com/toeverything/AFFiNE
- **Étudier** : workspace écrit en TS, canvas + docs

### Obsidian Canvas
- Propriétaire mais inspiration UX

### Heptabase
- Propriétaire, UX visual notes

### NotebookLM (Google)
- Propriétaire, IA + notes

### Logseq
- **Repo** : github.com/logseq/logseq
- **Étudier** : notes app Clojure

### Anytype
- **Repo** : github.com/anyproto
- **Étudier** : approche P2P local-first

## Brainstorming OSS

### Stormboard (closed)
### Klaxoon (closed)
### Conceptboard (closed)

Pas d'équivalent OSS proche → opportunité BSE.

## Local-first

### Ink & Switch
- **Web** : inkandswitch.com
- Recherche fondamentale, papers sur local-first et CRDTs

### Local-First Web (LoFi)
- Communauté active autour de ce paradigme

## Liens

- Crates Rust → [01-crates-rust.md](./01-crates-rust.md)
- Papers académiques → [03-papers-academiques.md](./03-papers-academiques.md)
- Liens externes → [04-liens-externes.md](./04-liens-externes.md)
