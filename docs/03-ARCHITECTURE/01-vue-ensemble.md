# 03.01 — Vue d'ensemble de l'architecture

> Le schéma global, les principes directeurs, et les choix d'architecture macro.

## Schéma global

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                                  CLIENT (BSE Desktop)                        │
│                                                                              │
│   ┌─────────────┐    ┌───────────────┐    ┌──────────────┐                   │
│   │  GUI / UX   │◄──►│  État local   │◄──►│  CRDT engine │                   │
│   │  (egui)     │    │  (scène vec)  │    │  (yrs/Loro)  │                   │
│   └──────┬──────┘    └───────────────┘    └──────┬───────┘                   │
│          │                                       │                           │
│          │           ┌─────────────────┐         │                           │
│          └──────────►│  Renderer 2D    │         │                           │
│                      │  (wgpu/vello)   │         │                           │
│                      └─────────────────┘         │                           │
│                                                  │                           │
│              ┌────────────────────────┐  ┌───────▼───────┐                   │
│              │ Persistance locale     │  │ Sync client   │                   │
│              │ (SQLite + filesystem)  │  │ (tokio + ws)  │                   │
│              └────────────────────────┘  └───────┬───────┘                   │
│                                                  │                           │
└──────────────────────────────────────────────────┼───────────────────────────┘
                                                   │
                                       WebSocket (TLS, msgpack)
                                                   │
┌──────────────────────────────────────────────────▼───────────────────────────┐
│                                 SERVEUR (BSE Server)                         │
│                                                                              │
│   ┌──────────────┐    ┌────────────────────┐    ┌─────────────────────┐      │
│   │  Axum HTTP   │    │  Gateway WebSocket │    │   Auth / OIDC       │      │
│   │  REST API    │    │  (upgrade route)   │    │   (jwt verif)       │      │
│   └──────┬───────┘    └─────────┬──────────┘    └─────────────────────┘      │
│          │                      │                                            │
│          │                      ▼                                            │
│          │            ┌────────────────────┐                                 │
│          │            │  Room Manager      │                                 │
│          │            │  (HashMap rooms)   │                                 │
│          │            └─────────┬──────────┘                                 │
│          │                      │                                            │
│          │             ┌────────┴──────────────┐                             │
│          │             ▼                       ▼                             │
│          │       ┌──────────┐           ┌──────────┐                         │
│          │       │ Room A   │           │ Room B   │  (1 tokio task / room)  │
│          │       │ (CRDT)   │           │ (CRDT)   │                         │
│          │       └────┬─────┘           └────┬─────┘                         │
│          │            │                      │                               │
│          └────────────┴──────────┬───────────┘                               │
│                                  ▼                                           │
│              ┌────────────────────────────────────┐                          │
│              │   Persistance Layer                │                          │
│              │   ┌─────────┐  ┌────────────────┐  │                          │
│              │   │ Postgres│  │  S3 / MinIO    │  │                          │
│              │   │ (meta)  │  │  (snapshots,   │  │                          │
│              │   │         │  │   images)      │  │                          │
│              │   └─────────┘  └────────────────┘  │                          │
│              └────────────────────────────────────┘                          │
└──────────────────────────────────────────────────────────────────────────────┘
```

## Principes directeurs

### 1. Offline-first
Le client tourne **sans serveur**. La connexion serveur n'est qu'un **multiplicateur** : permet le multi-user et la persistance distante. Sans elle, tout marche en local.

### 2. CRDT au cœur
Le modèle de données central est un **CRDT** (Conflict-free Replicated Data Type). Cela permet :
- Édition concurrente sans coordination
- Resync après déconnexion sans conflit
- Réplication client ↔ serveur trivialiisée

### 3. One-room-per-task
Côté serveur, **chaque projet actif = 1 tokio task dédiée**. Inspiré de Figma et tldraw. Avantages :
- Isolation par room
- Scaling vertical naturel (multi-cœur)
- Crash d'une room n'affecte pas les autres

### 4. Découplage rendu / état / réseau
Trois domaines indépendants côté client :
- **Rendu** (wgpu) : lit l'état, ne le modifie pas
- **État** (CRDT local) : source de vérité
- **Réseau** (tokio) : synchronise l'état distant ↔ local

### 5. Persistance event-sourced
Le serveur ne stocke pas seulement l'état actuel, mais le **journal des opérations**. Permet :
- Reconstruction d'un état antérieur (versioning)
- Audit
- Migrations futures

## Stack technique synthétique

| Couche | Choix |
|---|---|
| Langage client | **Rust** |
| Langage serveur | **Rust** |
| GUI | **egui** (cf. [04-STACK/02](../04-STACK-TECHNIQUE/02-gui-framework.md)) |
| Rendu 2D | **wgpu** direct + helpers vello (cf. [04-STACK/03](../04-STACK-TECHNIQUE/03-rendu-canvas.md)) |
| CRDT | **yrs** (Y-CRDT en Rust) ou **Loro** (cf. [05/02](../05-COLLABORATION-TEMPS-REEL/02-yrs-loro-automerge.md)) |
| HTTP / WS serveur | **axum** + **tokio-tungstenite** |
| Async runtime | **tokio** |
| Sérialisation | **rmp-serde** (MessagePack) + serde JSON |
| Base de données | **PostgreSQL** (serveur) + **SQLite** (client cache) |
| Stockage binaire | **S3-compatible** (MinIO en self-host) |
| Auth | **OIDC** + **JWT** (libs : oauth2, jsonwebtoken) |
| Logs / observabilité | **tracing** + **opentelemetry** |

Détails et alternatives dans [../04-STACK-TECHNIQUE/](../04-STACK-TECHNIQUE/).

## Topologies de déploiement

### A. Tout local (mode demo / dev)
```
Client desktop ──► sqlite local
```
Pas de serveur. Pour tester / dessiner solo.

### B. LAN privé (PME, équipe)
```
Client 1 ─┐
Client 2 ─┼──► Serveur BSE ──► Postgres + MinIO
Client 3 ─┘     (1 machine)
```
Self-host complet. Idéal pour 5-50 users.

### C. Cloud privé (entreprise)
```
Clients ──► Reverse proxy ──► N instances Serveur BSE ─► Postgres HA + S3
                 │                       │
                 └── TLS / auth SSO       └── Sticky session par room
```
Multi-instance, persistance distribuée.

### D. SaaS BSE (futur)
Notre offre managée pour ceux qui ne veulent pas self-host.

## Diagramme de séquence : ouverture d'un projet

```
Client                        Serveur                        BD
  │                              │                            │
  │── GET /api/projects/{id} ──►│                            │
  │                              │── SELECT project ─────────►│
  │                              │◄── metadata ───────────────│
  │◄── 200 OK + metadata ──────│                            │
  │                              │                            │
  │── WS UPGRADE /rooms/{id} ──►│                            │
  │                              │── Lazy load room ─────────►│
  │                              │◄── snapshot ──────────────│
  │◄── snapshot + ACK ─────────│                            │
  │                              │                            │
  │── Edit op ─────────────────►│                            │
  │                              │── Apply to CRDT            │
  │                              │── Broadcast to peers       │
  │                              │── Append to WAL ──────────►│
  │◄── Confirm ─────────────────│                            │
```

## Limites et choix consciemment écartés

### Pas de microservices
Tant que BSE n'a pas prouvé son traction, monolithe Rust modulaire. Beaucoup plus simple à opérer.

### Pas de message broker (Kafka, NATS) en v1
Pas justifié à notre échelle. Communication directe via tokio channels.

### Pas de Redis en v1
Postgres + cache in-memory suffit pour la cible v1.

### Pas de microfrontend / plugins en v1
Anti-fragilité par contrainte. Plugins viendront éventuellement en v2.

## Diagrammes détaillés

- Architecture client → [02-client.md](./02-client.md)
- Architecture serveur → [03-serveur.md](./03-serveur.md)
- Protocole réseau → [04-protocole-reseau.md](./04-protocole-reseau.md)
- Modèle de données → [05-modele-donnees.md](./05-modele-donnees.md)
- Diagrammes ASCII complets → [06-diagrammes.md](./06-diagrammes.md)
