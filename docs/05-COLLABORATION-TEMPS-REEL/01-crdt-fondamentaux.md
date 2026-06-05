# 05.01 — Fondamentaux des CRDT

> Qu'est-ce qu'un CRDT et pourquoi c'est la fondation de BSE.

## Le problème à résoudre

Trois utilisateurs (Alice, Bob, Charlie) modifient en **même temps** le même projet. Sans coordination :

```
Alice ajoute un rectangle à (10, 10)
Bob déplace ce rectangle à (50, 50)        ← arrivés au serveur dans cet ordre
Charlie supprime ce rectangle              ← mais émis dans quel ordre ?
```

Comment garantir que **tous les peers convergent vers le même état final**, quel que soit l'ordre de réception des messages ?

## Les approches classiques

### Approche 1 — Verrou exclusif
Un seul peut éditer à la fois. Impossible en collaboration temps réel.

### Approche 2 — Last-Writer-Wins (LWW)
Le dernier qui écrit gagne, identifié par un timestamp + ID de peer.

- ✅ Simple
- ✅ Convergent (avec horloges logiques)
- ❌ Perte d'éditions concurrentes
- ❌ Mal adapté au texte (perdre un mot entier)

### Approche 3 — Operational Transformation (OT)
Chaque édition est une opération transformable. Quand deux opérations concurrentes arrivent, on les transforme pour qu'elles soient compatibles.

- ✅ Fine-grained (caractère par caractère)
- ✅ Utilisé par Google Docs
- ❌ Complexité explosive avec >2 types d'ops (fonction de transformation par paire)
- ❌ Demande un serveur central pour ordonner les ops
- ❌ Difficile à étendre à des structures riches

### Approche 4 — Conflict-free Replicated Data Types (CRDT)
Des structures de données conçues pour que **toute application d'opérations dans n'importe quel ordre donne le même résultat**.

- ✅ Convergence garantie mathématiquement
- ✅ Pas besoin de serveur central pour ordonner
- ✅ Support natif de l'offline
- ⚠️ Plus de mémoire (historique des ops, tombstones)
- ⚠️ Plus complexe à implémenter — mais des libs existent

**BSE choisit CRDT** pour la robustesse, l'offline-first et la flexibilité.

## Le principe mathématique

Un CRDT est une structure dont les opérations sont :
- **Commutatives** : `apply(op1, op2) == apply(op2, op1)`
- **Associatives** : `apply((op1+op2), op3) == apply(op1, (op2+op3))`
- **Idempotentes** : `apply(op, op) == apply(op)` (appliquer 2 fois = 1 fois)

Si ces propriétés tiennent, **tous les réplicas convergent** dès qu'ils ont reçu les mêmes opérations.

## Deux familles : state-based vs op-based

### State-based (CvRDT)
- Chaque réplica envoie son **état complet**
- Le merge se fait par une fonction `join(state1, state2) → state3`
- ✅ Simple, robuste au réseau peu fiable
- ❌ Bande passante élevée

### Op-based (CmRDT)
- Chaque réplica envoie ses **opérations**
- Chaque op est appliquée chez chaque peer
- ✅ Bande passante faible (incrémental)
- ❌ Besoin de fiabilité (chaque op doit arriver exactement 1 fois)

**Les libs modernes (yrs, Loro, Automerge) combinent les deux** : op-based en steady state, state-based pour les resyncs.

## Types de CRDT classiques

### Compteurs
- **G-Counter** : grow-only
- **PN-Counter** : positive/negative (deux G-counters)

### Sets
- **G-Set** : grow-only set
- **2P-Set** : 2-phase (add + remove avec tombstones)
- **OR-Set** : Observed-Remove set (élégant, sans tombstones)

### Registres
- **LWW-Register** : Last-Writer-Wins
- **MV-Register** : Multi-Value (conserve tous les concurrents, résolution applicative)

### Textes
- **WOOT** : référence académique
- **Logoot, RGA, YATA** : améliorations
- **Fugue** : récent (2023), résout les *interleaving anomalies*

### Maps & Documents
- Combinaisons des précédents
- Y.js, Automerge, Loro proposent des « JSON CRDT »

## Pour BSE : quels types ?

Notre modèle (cf [../03-ARCHITECTURE/05-modele-donnees.md](../03-ARCHITECTURE/05-modele-donnees.md)) :
- **`elements` map** : ajout / suppression / modification d'éléments
- Chaque **élément** est un sous-objet avec :
  - `transform` : position/rotation/scale → registres LWW
  - `style` : couleurs/épaisseur → registres LWW
  - `kind` : enum → registre LWW (les changements de kind sont rares)
  - `text` (pour Text/Postit) : CRDT text natif
  - `points` (pour Pen) : array append-only

### Cas spéciaux

**Strokes de dessin libre** : array de points avec append-only. C'est un cas trivial — les peers ne réordonnent pas les points dans un stroke.

**Texte de Postit/Text** : CRDT texte (yrs::Text) pour édition collaborative caractère par caractère.

**Ordre des layers** : array CRDT (`yrs::Array`) avec opérations move.

## Anomalies CRDT à connaître

### Interleaving
Quand 2 peers tapent au même endroit, certains algos (RGA, YATA) peuvent **mélanger les caractères** : `Hello` + `World` → `HWelolrlod`. L'algorithme **Fugue** (Loro) résout ça mathématiquement.

### Concurrent move
Si Alice déplace un élément du parent A au parent B, et Bob du parent A au parent C, **où finit l'élément** ? Aucune solution parfaite — Loro propose une réponse propre. BSE n'a pas de hiérarchie parent/enfant complexe à part Mindmap → cas peu fréquent.

### Concurrent delete + edit
Alice supprime un élément, Bob l'édite en parallèle. Politique BSE : **delete wins** (tombstone), Bob voit son edit disparaître. Une notification UI possible.

## Coût mémoire des CRDT

- **Tombstones** : trace des éléments supprimés (pour ne pas les ressusciter)
- **Vector clocks** : taille proportionnelle au nombre de peers historiques
- **Historique des ops** : potentiellement grand

Mitigations :
- **Garbage collection** des tombstones après quorum
- **Snapshots compactés** régulièrement
- **Compaction des vector clocks** quand tous les peers ont vu

Les libs modernes (Loro, yrs) gèrent ça automatiquement.

## Performance

Benchmarks 2026 sur des éditions text :
- **Loro** : le plus rapide, jusqu'à 2× yrs sur certains benchmarks
- **yrs** : très rapide, ecosystem mature
- **Automerge** : plus lent mais excellent versioning
- **Diamond Types** : fastest sur texte pur

Pour BSE, les opérations sont surtout :
- Add/move element : peu fréquent
- Drawing stroke : append intensif (mais simple)
- Cursor moves : pas CRDT (awareness)

Donc même Automerge serait suffisant. On choisit yrs ou Loro pour la **maturité écosystème + perfs**.

## Liens

- Comparaison libs Rust → [02-yrs-loro-automerge.md](./02-yrs-loro-automerge.md)
- Choix BSE → [03-choix-bse.md](./03-choix-bse.md)
- Awareness (non-CRDT) → [04-presence-cursors.md](./04-presence-cursors.md)
- Conflits → [06-conflits-cas-limites.md](./06-conflits-cas-limites.md)

## Bibliographie

- Shapiro et al., *Conflict-free Replicated Data Types*, 2011 (paper fondateur)
- Kleppmann, *Designing Data-Intensive Applications*, ch. CRDT
- Site de référence : [crdt.tech](https://crdt.tech)
