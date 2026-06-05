# 05.02 — yrs vs Loro vs Automerge

> Les trois bibliothèques Rust de CRDT à considérer pour BSE.

## TL;DR

> **yrs** est le choix par défaut (mature, écosystème vaste). **Loro** est le challenger Rust-native, le plus performant et avec une API plus moderne. **Automerge** est plus lent mais excellent pour le versioning Git-like.

## yrs (Y-CRDT en Rust)

### Présentation
- **Repo** : github.com/y-crdt/y-crdt
- **Version** : ~0.20 (2026)
- **Style** : op-based + state-based hybride
- **Famille** : Yjs (the JS reference), porté en Rust
- **Origine** : Y.js par Kevin Jahns, port Rust par Bartosz Sypytkowski

### Forces
- ✅ **Compatible binaire avec Yjs** : un client web JS et un client Rust peuvent collaborer
- ✅ **Écosystème énorme** : providers (y-websocket, y-redis, y-leveldb), bindings (Python, Ruby, Java…), demos, docs
- ✅ **Production-proven** : ~920K downloads/semaine sur npm pour Yjs ; deployé par Atlassian, Evernote, Linear-clone… 
- ✅ **Modèle riche** : `Map`, `Array`, `Text`, `XmlFragment`
- ✅ **Sub-documents** (un Doc dans un Doc) — utile pour BSE potentiellement

### Faiblesses
- ⚠️ API parfois verbeuse (transactions explicites)
- ⚠️ Pas l'absolute fastest sur certains benchmarks (Loro le dépasse)
- ⚠️ Algorithme de texte (YATA) peut produire des interleaving anomalies en cas extrême

### Exemple
```rust
use yrs::{Doc, Map, Transact};

let doc = Doc::new();
{
    let mut txn = doc.transact_mut();
    let map = doc.get_or_insert_map("elements");
    map.insert(&mut txn, "el_xxx", element_data);
}

// Échange d'updates
let update = doc.transact().encode_state_as_update_v2(&StateVector::default());
// → envoyer via WS

let other_doc = Doc::new();
other_doc.transact_mut().apply_update(update);
// → other_doc converge
```

## Loro

### Présentation
- **Repo** : github.com/loro-dev/loro
- **Version** : 1.x stable (2026)
- **Style** : op-based avec Replayable Event Graph
- **Origine** : projet Rust-native, équipe Loro Inc

### Forces
- ✅ **Le plus performant** : benchmarks récents montrent 1.5-3× yrs sur édition texte
- ✅ **Algorithme Fugue** pour le texte : **maximal non-interleaving** garanti
- ✅ **Movable Tree CRDT** : déplacement d'arbres concurrent sans cycle (utile pour mindmap)
- ✅ **Time-travel** intégré : on peut visualiser l'état à n'importe quel point
- ✅ **API moderne**, Rust-first
- ✅ **Rich text** natif avec marqueurs (gras, italique sur ranges)

### Faiblesses
- ⚠️ Écosystème plus jeune (~12K npm downloads/semaine vs 920K Yjs)
- ⚠️ Moins de providers prêts (websocket, leveldb)
- ⚠️ Pas de compat binaire avec Yjs (si un jour on veut un client web léger Yjs, conflit)
- ⚠️ Moins de battle-testing en production massive

### Exemple
```rust
use loro::{LoroDoc, ContainerType};

let doc = LoroDoc::new();
let map = doc.get_map("elements");
map.insert("el_xxx", element_data)?;
doc.commit();

// Échange
let update = doc.export_from(&Default::default());
// → envoyer via WS

let other_doc = LoroDoc::new();
other_doc.import(&update)?;
```

## Automerge

### Présentation
- **Repo** : github.com/automerge/automerge
- **Version** : 2.x stable (2026)
- **Style** : op-based + Git-like history
- **Origine** : recherche Ink & Switch, équipe Martin Kleppmann

### Forces
- ✅ **Versioning Git-like** : branches, merges, undo natif granulaire
- ✅ **Très adapté à l'édition documentaire** (texte structuré, JSON)
- ✅ **Stable, mature**, base scientifique solide
- ✅ **Visualisation d'historique**

### Faiblesses
- ⚠️ Performances inférieures à yrs/Loro (objectif design différent : robustesse > vitesse)
- ⚠️ Pour un canvas interactif, le coût mémoire historique pèse
- ⚠️ ~85K downloads/semaine — moins que Yjs mais respectable

### Quand le préférer ?
Si BSE devait **prioriser le versioning** (timeline, branches d'idées, exploration de variantes), Automerge serait pertinent. Pour notre cas (canvas temps réel), c'est trop coûteux.

## Diamond Types (mention)

- **Repo** : github.com/josephg/diamond-types
- **Style** : Event Graph Walker (Eg-walker)
- **Forces** : le plus rapide sur texte pur, low memory
- **Limites** : focus texte, pas de Map/Array riches → trop limité pour BSE

## Tableau comparatif

| Critère | **yrs** | **Loro** | **Automerge** | **DT** |
|---|---|---|---|---|
| Maturité | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ |
| Écosystème | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐ |
| Perf édition texte | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| Perf édition Map/Array | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐ |
| Memory footprint | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Versioning | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ |
| Rich text | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ |
| Movable tree | ❌ | ✅ | ⭐⭐ | ❌ |
| Compat JS Yjs | ⭐⭐⭐⭐⭐ | ❌ | partial | ❌ |
| Time travel | ❌ | ✅ | ✅ | ⭐⭐ |
| API Rust ergonomie | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |

## Benchmarks réels (à reproduire en MVP)

À reproduire en début de projet (M1-M2) :

```rust
// Pseudo-bench
fn bench_concurrent_strokes() {
    let n_peers = 10;
    let n_points_per_stroke = 100;
    let n_strokes = 50;
    
    // Simuler n_peers dessinant chacun n_strokes strokes
    // Mesurer :
    //  - Temps pour appliquer tous les ops localement
    //  - Taille de l'update binaire final
    //  - Mémoire du Doc résultant
}
```

Critères :
- Throughput d'application d'ops > 10K ops/sec ?
- Taille d'un Doc pour 1000 éléments ?
- Compaction memory après GC ?

## Critères BSE pour le choix

### Prioritaires
1. Maturité prod (BSE pas un labo)
2. Performance avec milliers d'éléments
3. Memory footprint raisonnable
4. Support de Map et Array
5. Rich text si possible (pour mindmap notes)

### Bonus
6. Time travel (futur versioning v1.0+)
7. Movable tree (mindmap structuré)
8. Compat ecosystem (potentiel client web futur)

### Verdict
- **Choix safe** : **yrs** (maturité, écosystème, compatible Yjs si client web futur)
- **Choix moderne** : **Loro** (perf, Fugue, time travel, movable tree)

## Liens

- Choix final → [03-choix-bse.md](./03-choix-bse.md)
- Sources :
  - github.com/y-crdt/y-crdt
  - github.com/loro-dev/loro
  - github.com/automerge/automerge
  - crdt.tech (état de l'art)
