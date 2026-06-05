# 02.03 — Étude : Excalidraw

> Référence open-source du whiteboarding minimaliste. Architecture *« pseudo-P2P »* élégante avec chiffrement bout-en-bout.

## Vue d'ensemble

- **Open-source** sous licence MIT (mais le SaaS Excalidraw+ est propriétaire)
- ~80K stars GitHub (2026)
- Web app React, embed possible
- Esthétique « croquis main levée » (style hand-drawn via Rough.js)

## Architecture de la collaboration

### Schéma

```
   Client A ─┐                              ┌─ Client C
            ├─► WebSocket relay server ◄───┤
   Client B ─┘   (chiffrement E2E)          └─ Client D
                       │
                       ▼
              Firebase (Firestore + Storage)
              pour la persistance des snapshots
```

### Modèle « pseudo-P2P »

Citation des docs : *« Excalidraw utilise un modèle pseudo-P2P : un serveur central relaie des messages chiffrés bout-en-bout entre les peers, sans faire de coordination centralisée. »*

Concrètement :
- Le serveur **relaie** les messages WebSocket
- Le serveur ne **lit jamais** le contenu (chiffrement room key côté clients)
- Le serveur ne **résout pas** les conflits — chaque client applique une stratégie LWW

### Quatre composants principaux

1. **Collab component** — orchestrateur côté client
2. **Portal** — couche d'abstraction WebSocket (wraps Socket.IO)
3. **Firebase services** — persistance (Firestore pour scènes, Storage pour images)
4. **WebSocket server** — relais temps réel

### Stratégie de sync

- **Par version d'élément** : chaque élément a une version monotone. Seuls les éléments dont la version a changé sont broadcast.
- **Sync périodique complet** : régulièrement, le client envoie l'état complet pour réparer toute désynchro.
- Pas de CRDT proprement dit — c'est du LWW sur des objets identifiés.

### Chiffrement E2E

- À la création d'une room, le client génère une **room key AES** (jamais envoyée au serveur).
- L'URL de partage contient la clé dans le fragment (`#`) → jamais transmise au serveur HTTP.
- Tous les messages WebSocket sont chiffrés avec cette clé.
- Le serveur voit du chiffrement, ne peut pas lire.

**C'est élégant et sécurisé. C'est aussi une inspiration majeure pour BSE en mode E2E optionnel.**

## Persistance

- **Firestore** (Google Firebase) pour les scènes
- **Firebase Storage** pour les binaires (images)
- Configuration via JSON-encoded creds
- Le serveur de room est dans un repo séparé : github.com/excalidraw/excalidraw-room

## Forces

1. **Simplicité** : install Docker en 1 ligne, ça marche.
2. **E2E natif** : différenciateur sécurité.
3. **Esthétique** unique (style hand-drawn).
4. **Embeddable** : intégrable dans n'importe quel produit.
5. **Communauté active** : contributions saines.

## Faiblesses (que BSE doit corriger)

1. **Pas de multi-projet** : chaque scène est isolée, pas de notion de « workspace ».
2. **Persistance imposée Firebase** (sauf self-host serveur de room sans persistance).
3. **Features limitées** : pas de mindmap, pas de templates, pas de post-its colorés natifs.
4. **Web-only** : pas d'app desktop native.
5. **Performance** : dégradée au-delà de 500 éléments.
6. **Pas de rôles utilisateurs** : tout le monde dans une room a les mêmes droits.

## Le serveur de room (excalidraw-room)

C'est un service Node.js Socket.IO qui :
- Gère les rooms (création, jointure)
- Relaie les messages
- N'a aucune logique de résolution (E2E oblige)
- Peut tourner en quelques lignes Docker

```yaml
# Exemple compose Excalidraw self-hosted
services:
  excalidraw:
    image: excalidraw/excalidraw
    ports: [80:80]
  excalidraw-room:
    image: excalidraw/excalidraw-room
    ports: [3002:80]
```

C'est simple. BSE peut viser la même simplicité de déploiement.

## Leçons retenir pour BSE

### Inspirations directes
1. **Chiffrement E2E optionnel via room key** : BSE doit le proposer pour les contextes sensibles.
2. **Simplicité de déploiement** : `docker compose up` doit suffire.
3. **Style hand-drawn** : à proposer en option (mode esthétique).
4. **Séparation serveur de relais / persistance** : architecture propre.

### Différenciateurs BSE
1. **Multi-projet natif** dès le début.
2. **CRDT mature** (yrs, Loro) au lieu de LWW custom — plus robuste.
3. **Desktop natif performant**.
4. **Features riches** (mindmap, post-its, templates).
5. **Auth + rôles** dès v0.5.

## Code source clé à étudier

| Repo | Apport |
|---|---|
| github.com/excalidraw/excalidraw | Front-end React + canvas logic |
| github.com/excalidraw/excalidraw-room | Serveur de relais |

## Sources

- *Excalidraw Collaboration System* — deepwiki.com/excalidraw/excalidraw/7
- *Building Excalidraw's P2P Collaboration Feature* — plus.excalidraw.com/blog
- *Selfhosting Excalidraw with Collaboration Support* — blog.lrvt.de
