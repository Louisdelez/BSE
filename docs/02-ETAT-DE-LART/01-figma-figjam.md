# 02.01 — Étude : Figma / FigJam

> Figma est *la* référence en multiplayer canvas. Étudier son architecture est obligatoire.

## Vue d'ensemble

Figma a publié un article fondateur en 2019 : *« How Figma's multiplayer technology works »*. Il décrit une architecture client/serveur WebSocket avec un compromis original entre OT et CRDT.

## Architecture haut niveau

```
┌────────────┐     WebSocket      ┌─────────────────────┐
│  Browser   │ ◄─────────────────►│  Multiplayer Server │
│   client   │                    │   (par document)    │
└────────────┘                    └─────────────────────┘
                                          │
                                          │
                              ┌───────────┼────────────┐
                              ▼           ▼            ▼
                          ┌──────┐  ┌───────────┐  ┌──────────┐
                          │  S3  │  │ DynamoDB  │  │ Postgres │
                          │ files│  │  log WAL  │  │ métadata │
                          └──────┘  └───────────┘  └──────────┘
```

**Points clés** :

- Chaque document = **un processus serveur** dédié (one-doc-per-process)
- Tous les clients d'un même document parlent à ce processus via WebSocket
- Le client télécharge le doc complet à l'ouverture, puis ne reçoit que des deltas
- Le serveur est responsable de : validation, ordonnancement, résolution de conflits, broadcast

## Le compromis OT-like / CRDT-like

Figma a choisi **ni OT pur ni CRDT pur**. Ils ont construit un système custom **inspiré des CRDT mais simplifié** :

- Chaque document est un **arbre d'objets** avec une racine
- Chaque propriété de chaque objet utilise **last-writer-wins** (LWW) avec un timestamp logique
- Les opérations de structure (création, parentage) ont une logique plus fine pour éviter les anomalies

**Pourquoi pas pur CRDT ?** Les CRDT complets ont un coût mémoire élevé (tombstones, historique). Figma ne voulait pas ça.

**Pourquoi pas pur OT ?** OT requiert une fonction de transformation par paire d'opérations, ce qui devient ingérable avec des dizaines de types d'opérations.

## Persistance

- **Amazon S3** pour les *file checkpoints* (snapshots binaires des documents)
- **DynamoDB** pour le **write-ahead log** (toutes les modifications)
- **PostgreSQL** horizontalement sharded pour les **métadonnées** (utilisateurs, équipes, permissions)

## La réécriture Rust

L'événement marquant : le serveur multiplayer était **initialement en TypeScript**. Figma l'a **réécrit en Rust** et a constaté **un ordre de grandeur d'amélioration de performance**.

Citation du blog Figma : *« Rust's ownership model helped audit file update paths and ensure journal consistency »*.

**Implication pour BSE** : choisir Rust dès le départ évite la dette technique de migration ; on a *out of the box* la performance que Figma a payée 10× chère à atteindre.

## Scaling

Figma scale par :
- **Sharding par document** (one process per doc)
- **Routage** des clients d'un document vers le bon process
- Persistance off-process (S3 + DynamoDB sont indépendants)

## FigJam vs Figma

FigJam est le « cousin whiteboarding » de Figma. Architecture similaire (même infra), mais :
- UX simplifiée et ludique
- Templates intégrés (workshop, retrospective, brainstorm)
- Curseurs animés, stickers, emojis
- Less features de design (pas de variants, pas d'auto-layout complexe)

**FigJam = ce que BSE doit imiter en UX**, sur stack technique différente.

## Limites de Figma vis-à-vis de BSE

- ❌ Web only (Electron-style desktop)
- ❌ Closed-source
- ❌ Cloud only (pas self-host)
- ❌ Le serveur multiplayer est propriétaire — on apprend de leurs articles, on ne peut pas le réutiliser
- ❌ Modèle économique SaaS dépendant d'Adobe (depuis 2023)

## Leçons à retenir pour BSE

1. **One-room-per-process** est une architecture éprouvée à grande échelle.
2. **Rust pour le serveur multiplayer** = bon choix selon Figma eux-mêmes.
3. **WebSocket** est suffisant pour ce type de charge.
4. **LWW + arbre d'objets** est plus simple que CRDT pur, et marche.
5. **Séparer fichier (S3) / log (DynamoDB) / métadonnées (Postgres)** est un pattern à imiter (adapté à notre échelle : Postgres + objet storage suffit).

## Décisions BSE inspirées de Figma

| Décision | Inspirée de Figma | Adaptation BSE |
|---|---|---|
| Une room = un processus / task | ✅ | ✅ |
| Rust pour le serveur | ✅ | ✅ |
| Snapshot + delta log | ✅ | ✅ |
| LWW pour propriétés | Inspiration | Mais on prend un CRDT mature (Loro ou yrs) |
| S3 pour binaires | ✅ | MinIO / S3 / disque selon déploiement |
| WebSocket | ✅ | ✅ (option QUIC plus tard) |

## Sources clés

- *How Figma's multiplayer technology works* — figma.com/blog/how-figmas-multiplayer-technology-works
- *Making multiplayer more reliable* — figma.com/blog/making-multiplayer-more-reliable
- *Figma: Building Multiplayer Infrastructure* — Sujeet Jaiswal
- *Inside Figma's multiplayer infrastructure* — Runtime.news
