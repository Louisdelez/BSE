# 05.03 — Choix CRDT pour BSE

> Décision finale après évaluation.

## Recommandation

> **Choix de départ : `yrs`** pour la v1.0.  
> **Évaluation Loro au MVP** ; switch possible avant v0.5 si benchmarks le justifient.

## Le raisonnement

### Pourquoi yrs en first choice

1. **Maturité prod** : utilisé par Atlassian, Notion-likes, etc. en production. Pas de risque de bugs structurels.
2. **Écosystème** : `y-websocket`, `y-redis`, `y-leveldb`, bindings multiples. Si on veut intégrer un service tiers, c'est dispo.
3. **Compat Yjs binaire** : si un jour BSE veut un client web léger en TypeScript, on peut le faire avec Yjs et il sera compatible avec nos clients Rust.
4. **Doc et exemples** : très abondants.
5. **Risque inverse** : Loro est très prometteur mais moins éprouvé. Pour un produit qui démarre, mieux vaut la sécurité.

### Pourquoi *envisager* Loro

1. **Perf** : si les benchmarks BSE montrent un avantage net (>2× sur nos cas d'usage)
2. **Fugue** : si on a des bugs interleaving avec yrs sur du texte multi-user
3. **Movable Tree** : utile pour le mindmap structuré
4. **Time travel** : déjà préparé pour le versioning v1.0+

### Quand brancher au passage à Loro
On évalue **avant la v0.5**. Critères go/no-go :
- Loro encore mainteinu / actif ?
- Bench BSE :  >2× perf sur opérations canvas typiques ?
- Bugs critiques bloquants sur yrs identifiés ?
- Ecosystem providers prêts pour Loro ?

Si oui → migration. Sinon → on reste sur yrs.

## Modélisation BSE avec yrs

### Structure de Doc

```rust
struct ProjectDoc {
    doc: yrs::Doc,
    
    // Containers
    elements: yrs::Map,          // ElementId → Element data
    layers: yrs::Array,          // ordre des layers
    settings: yrs::Map,          // settings du projet
    comments: yrs::Map,          // CommentId → Comment data
}
```

### Représentation d'un Element

Chaque élément est lui-même un yrs::Map :

```rust
fn element_to_yrs(elem: &Element, txn: &mut Transaction, elements: &Map) {
    let el_map = elements.insert_with(txn, elem.id.to_string(), MapPrelim::default());
    
    el_map.insert(txn, "kind_tag", elem.kind.tag());  // "Rectangle", "Pen", ...
    el_map.insert(txn, "kind_data", serialize(&elem.kind));  // payload
    el_map.insert(txn, "x", elem.transform.x);
    el_map.insert(txn, "y", elem.transform.y);
    el_map.insert(txn, "rotation", elem.transform.rotation);
    el_map.insert(txn, "scale_x", elem.transform.scale_x);
    el_map.insert(txn, "scale_y", elem.transform.scale_y);
    
    // Style
    let style_map = el_map.insert(txn, "style", MapPrelim::default());
    style_map.insert(txn, "stroke", serialize(elem.style.stroke));
    style_map.insert(txn, "fill", serialize(elem.style.fill));
    // ...
    
    el_map.insert(txn, "z", elem.z);
    el_map.insert(txn, "locked", elem.locked);
}
```

### Cas particulier — Texte

Pour les `Text` et `Postit`, on utilise `yrs::Text` pour le contenu :

```rust
// Pour un Postit
let postit_map = ...;
let text = postit_map.insert(txn, "text", TextPrelim::new(""));
// Modifications sur le texte = ops CRDT fines
text.insert(txn, 0, "Bonjour");
```

Permet à 2 utilisateurs d'éditer le texte du même postit simultanément.

### Cas particulier — Stroke de dessin

Pour un `Pen`, les points sont un `yrs::Array<StrokePoint>` :

```rust
let pen_map = ...;
let points = pen_map.insert(txn, "points", ArrayPrelim::default());
// Pendant le dessin, on append au fil de l'eau
points.push(txn, point_data);
```

**Pas de réordonnancement de points dans un stroke** → pas de problème CRDT.

### Cas particulier — Mindmap (movable tree)

yrs **n'a pas de Movable Tree natif**. Pour le mindmap :

**Approche A** (avec yrs) :
- Chaque MindmapNode stocke son `parent_id`
- Si 2 peers reparentent simultanément, LWW résout (le dernier gagne)
- Risque : cycle (A devient enfant de B qui était enfant de A)
- Mitigation : détection de cycle côté client + suggest manuel

**Approche B** (avec Loro) :
- Movable Tree natif, pas de cycle possible

C'est un **argument fort en faveur de Loro si le mindmap est prioritaire**. À évaluer en jalon v0.5.

## Transactions

yrs requiert d'envelopper les modifications dans des transactions :

```rust
// Application d'une op locale
fn apply_local_op(doc: &Doc, op: LocalOp) {
    let mut txn = doc.transact_mut();
    match op {
        LocalOp::AddElement(e) => add_element_to_yrs(&e, &mut txn, &elements),
        LocalOp::Move { id, x, y } => move_element(id, x, y, &mut txn, &elements),
        ...
    }
    // txn drop → commit + génération d'updates pour les observers
}

// Réception d'un update distant
fn apply_remote_update(doc: &Doc, update: &[u8]) {
    let mut txn = doc.transact_mut();
    let upd = Update::decode_v2(update)?;
    txn.apply_update(upd)?;
}
```

## Sync incrémental via state vector

yrs utilise le pattern « state vector » pour resync :

```rust
// 1. Client demande state vector du serveur
let sv = server_doc.transact().state_vector();
// 2. Client envoie son update à partir de ce SV
let update = client_doc.transact().encode_state_as_update_v2(&sv);
// 3. Server applique
server_doc.transact_mut().apply_update(Update::decode_v2(&update)?)?;
// 4. Server répond avec son delta inverse
let server_delta = server_doc.transact().encode_state_as_update_v2(&client_doc.state_vector());
// 5. Client applique
client_doc.transact_mut().apply_update(Update::decode_v2(&server_delta)?)?;
```

Tout est binaire et compact.

## Persistance du Doc

Pour persister un Doc yrs :

```rust
// Snapshot complet
let snapshot = doc.transact().encode_state_as_update_v2(&StateVector::default());
// → écrire dans S3 sous projects/{pid}/snapshots/{version}.crdt

// Restauration
let new_doc = Doc::new();
new_doc.transact_mut().apply_update(Update::decode_v2(&snapshot)?)?;
```

Pour le WAL, chaque op est aussi un `Update` binaire qu'on sauve.

## API d'abstraction BSE

Pour pouvoir éventuellement switcher de yrs à Loro (ou autre) plus tard, on **encapsule** dans une lib `bse-crdt` :

```rust
// crates/bse-crdt/src/lib.rs

pub trait CrdtBackend {
    type Doc;
    type Update;
    
    fn new_doc() -> Self::Doc;
    fn apply_op(doc: &Self::Doc, op: LocalOp);
    fn apply_remote(doc: &Self::Doc, update: &Self::Update);
    fn export_diff(doc: &Self::Doc, base: StateVector) -> Self::Update;
    fn snapshot(doc: &Self::Doc) -> Vec<u8>;
    // ...
}

pub struct YrsBackend;
impl CrdtBackend for YrsBackend { ... }

// pub struct LoroBackend; impl CrdtBackend for LoroBackend { ... }

// Dans le code BSE
type CurrentBackend = YrsBackend;
```

Migrer = changer un type alias + tester.

## Garbage collection des tombstones

yrs gère automatiquement la GC :
- Les tombstones (éléments supprimés) sont conservés tant qu'un peer pourrait ne pas les avoir vus
- Périodiquement, après consensus de tous les peers actifs, on compacte
- En pratique pour BSE : on déclenche un GC pass après chaque checkpoint

## Conflit resolution policies

### Pour les champs simples (transform, style)
LWW automatique via yrs. Le dernier write gagne, identifié par yrs Clock (logique).

### Pour les éléments dupliqués (peu probable)
Si 2 peers créent un élément avec le même UUID (très improbable avec UUID v7), les 2 coexistent dans le map (un seul UUID = un seul slot). Mais on génère des UUID v4 ou v7 garantis uniques côté client.

### Pour les suppressions concurrentes
Delete + edit concurrent : delete wins. L'edit est appliqué sur un élément qui est marqué deleted → ignoré.

### Pour les déplacements concurrents (move)
Two peers move the same element to different positions → LWW. Pas de fusion (ça n'aurait pas de sens géométriquement).

## Liens

- Fondamentaux → [01-crdt-fondamentaux.md](./01-crdt-fondamentaux.md)
- Comparaison libs → [02-yrs-loro-automerge.md](./02-yrs-loro-automerge.md)
- Awareness → [04-presence-cursors.md](./04-presence-cursors.md)
- Cas limites → [06-conflits-cas-limites.md](./06-conflits-cas-limites.md)
