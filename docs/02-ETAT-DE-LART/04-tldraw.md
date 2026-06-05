# 02.04 — Étude : tldraw

> Le SDK d'infinite canvas le plus avancé techniquement en 2026. tldraw sync est *la* référence pour le multiplayer canvas.

## Vue d'ensemble

- **SDK React** pour bâtir des apps de type whiteboard
- Engine particulièrement performant (60 FPS soutenu sur boards de milliers d'éléments)
- **tldraw sync** : leur engine de synchro multi-user
- **Modèle économique** : SDK libre pour usages non commerciaux + licence commerciale + SaaS tldraw.com

## tldraw sync — l'architecture

### Schéma

```
   ┌────────────┐
   │  Client A  │──┐
   └────────────┘  │     WebSocket
                   ▼
   ┌────────────┐  ┌─────────────────────┐    ┌──────────┐
   │  Client B  │──┤ Cloudflare Durable  ├───►│   R2     │
   └────────────┘  │   Object (1/room)   │    │ (binaire)│
                   │                     │    └──────────┘
   ┌────────────┐  │  - Mémoire serveur  │
   │  Client C  │──┤  - WebSocket pool   │    ┌──────────┐
   └────────────┘  │  - Persistance KV   │───►│  Workers │
                   └─────────────────────┘    │   KV     │
                                              └──────────┘
```

### Cloudflare Durable Objects

C'est le **choix architectural fondateur** :

- **1 board = 1 Durable Object** = 1 mini-serveur dédié
- Chaque DO a son propre stockage clé/valeur persistant
- Chaque DO peut gérer jusqu'à **50 collaborateurs simultanés**
- Cloudflare gère le routage géographique automatiquement
- **Consistance forte** : une seule copie authoritative

C'est l'évolution moderne du pattern *« one-process-per-room »* de Figma. Avantage : Cloudflare s'occupe du scaling et de la distribution géo.

**Implication pour BSE** : on peut s'inspirer de ce pattern mais on n'a pas besoin de Cloudflare. Un serveur classique avec une *task tokio par room* suffit à notre échelle.

### Le moteur de synchro

D'après l'annonce *« Announcing tldraw sync »* :

- Conçu spécifiquement pour les interactions canvas (drag, draw, transform)
- Optimisé pour envoyer le **minimum** de messages nécessaires
- Résolution de conflits avec **peu d'overhead** (pas CRDT pur — leur propre protocole)
- Évite les duplications avec coordination client-server

> *« Powered collaboration for more than 400,000 users across 200,000 shared projects »*

C'est éprouvé en production à grande échelle.

### Modèle de données

Chaque objet sur le canvas est un **record** (au sens base de données) :
- Un `id` unique
- Un `typeName` (shape, binding, document, …)
- Des propriétés versionnées

Les *records* sont stockés dans une *store* unifiée. Le sync se fait au niveau record (delta de record). Très proche d'un schéma SQL/NoSQL.

## Le starter kit multiplayer

tldraw fournit `tldraw-sync-cloudflare` open-source : un exemple complet, déployable en quelques minutes.

```typescript
// extrait simplifié
export class TldrawDurableObject {
  constructor(state: DurableObjectState) {
    this.room = new TLSocketRoom({...})
  }
  fetch(req: Request) {
    return this.room.handleConnection(req, ...)
  }
}
```

C'est de l'art. Mais c'est web/JS. BSE l'adaptera à Rust + tokio.

## Forces

1. **Performance** : référence absolue de l'infinite canvas web.
2. **Architecture sync** : claire, scalable, éprouvée.
3. **API SDK** : ergonomique, bien documentée.
4. **Communauté** : très active, Discord vivant.

## Faiblesses

1. **Web only** : pas d'app desktop native.
2. **Licence** : SDK pro non MIT, contraintes commerciales.
3. **Couplé Cloudflare** pour le sync officiel (auto-host possible mais moins prouvé).
4. **Pas un produit fini** : c'est un SDK avant tout.

## Leçons pour BSE

### Inspirations directes
1. **One-DO-per-room** → **one-tokio-task-per-room** côté BSE.
2. **Records / delta records** comme modèle de données interne.
3. **Sync optimisé pour le canvas** (pas un CRDT générique surdimensionné).
4. **Starter kit déployable rapidement**.

### Différenciateurs BSE
1. **Desktop natif** au lieu de React/web.
2. **Engine en Rust** (potentiellement plus rapide encore que tldraw qui est TS+canvas).
3. **Pas de dépendance Cloudflare**.

## Comparaison perf : tldraw vs cible BSE

| Métrique | tldraw v3 (2026) | Cible BSE v1.0 |
|---|---|---|
| FPS sur 1000 éléments | 60 FPS | 60+ FPS |
| FPS sur 10K éléments | ~30-40 FPS | 60 FPS |
| Démarrage cold | ~1.5 s (web) | <500 ms (natif) |
| Memory footprint | ~250 MB (Chrome) | <100 MB (natif) |
| Latence sync LAN p95 | ~100 ms | <50 ms |
| Max peers/room | 50 (1 DO) | Cible 100 |

## Pourquoi pas juste réutiliser tldraw ?

Question légitime. Réponses :
1. **Licence** : restrictive pour usage commercial.
2. **Web** : pas notre cible (on veut natif).
3. **Stack** : React + TS, on veut Rust.
4. **Souveraineté** : on veut tout contrôler.
5. **Apprentissage** : faire son propre engine donne une meilleure compréhension et évolutivité.

C'est la même logique que Figma qui a réécrit en Rust — on contrôle, on optimise.

## Sources clés

- github.com/tldraw/tldraw
- tldraw.dev (docs)
- tldraw.substack.com — *Announcing tldraw sync*
- github.com/tldraw/tldraw-sync-cloudflare — starter kit
