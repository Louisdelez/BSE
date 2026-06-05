# 05.05 — Undo / Redo en multi-user

> Le undo collaboratif est un **problème délicat**. Voici comment BSE le résout.

## Le problème

Alice fait :
1. Crée un rectangle
2. Crée un cercle

Bob fait (entre-temps) :
3. Crée un texte

Alice fait Ctrl+Z. Qu'est-ce qu'on annule ?

### Option A — Undo global
On annule la dernière op chronologique. Mais ça pourrait annuler l'op de Bob, ce qui surprendrait Alice (« je n'ai pas fait ça »).

### Option B — Undo local (recommandé)
On annule la **dernière op d'Alice spécifiquement**. C'est ce que font Figma, Google Docs.

**BSE choisit l'option B**.

## L'algorithme : selective undo

yrs fournit un mécanisme natif :

```rust
use yrs::UndoManager;

let mut undo = UndoManager::new(&doc, &[
    elements_map.into(),
    layers_array.into(),
    settings_map.into(),
]);

undo.include_origin(my_peer_id);  // ne capture que mes ops

// Faire des modifications
do_some_edits();

// Undo
undo.undo()?;
// Redo
undo.redo()?;
```

L'undo manager :
- Capture les modifications qui passent par les containers spécifiés
- Filtre par origine (`peer_id`) pour ne capturer que les ops locales
- Produit une op CRDT inverse qui est appliquée et broadcast comme une op normale

## Granularité des transactions

Une transaction = un undo step.

```rust
// Bon : 1 transaction = 1 undo step
{
    let mut txn = doc.transact_mut();
    create_rectangle(&mut txn, ...);
    set_style(&mut txn, ...);
}  // 1 undo

// Mauvais : 2 transactions = 2 undos
create_rectangle();  // 1 undo step
set_style();         // 2nd undo step
```

Cas BSE :
- **Drag of element** : 1 transaction qui contient le delta de position (pas tous les frames intermédiaires)
- **Stroke de pen** : 1 transaction par stroke complet
- **Création + style initial** : 1 transaction

Pour la fluidité, on commit la transaction au `mouse_up`, pas pendant le drag.

## Cas particuliers

### Undo sur élément supprimé par autrui
Alice crée un rectangle. Bob le supprime. Alice fait Ctrl+Z (qui devait annuler une autre op récente).

Si la pile undo d'Alice contient toujours la création du rectangle :
- Undo va « créer un autre rectangle » via une op CRDT — mais on undo *plus loin* dans la pile
- Aucun conflit, juste cohérent

### Undo sur élément modifié par autrui
Alice crée un rectangle à (10, 10). Bob le déplace à (50, 50). Alice fait Undo de sa création.

- Undo CRDT supprime le rectangle (l'inverse de create = delete)
- Le rectangle disparait pour tout le monde — incluant la modification de Bob
- C'est cohérent : si on annule la cause (création), l'effet (modification) disparaît

### Undo après long offline
Alice fait 10 modifications offline. Reconnecte. Pile undo conservée localement.

- L'undo s'applique normalement après resync
- Les ops d'inverse sont broadcast comme nouvelles ops

## UI

- **Ctrl+Z** : undo
- **Ctrl+Shift+Z** / **Ctrl+Y** : redo
- Boutons dans la toolbar : ←↑ undo, →↓ redo
- États grisés si la pile est vide
- Tooltip : « Annuler [description de l'action] »

## Limites de la pile

- **Capacité** : 100 actions par défaut
- **Reset** : à la fermeture du projet, pile vidée (non persistée)
- **Optionnel** : persistance de la pile dans SQLite local (v1.x)

## Undo de plusieurs peers ?

Mural et certains outils proposent un « undo team-wide ». BSE **ne le fait pas** :
- Trop confusant : qui décide ?
- Cas d'usage marginal
- Risque de destruction du travail d'autrui

Si vraiment besoin : revenir à un snapshot antérieur (feature de **versioning** v1.0+).

## Versioning (à part du undo)

Pour permettre « revenir à hier » à l'échelle du projet :
- Snapshots quotidiens persistés (cf [../03/03-serveur.md](../03-ARCHITECTURE/03-serveur.md))
- UI : sidebar « Historique » avec timeline
- Restoration = créer un nouveau projet à partir du snapshot, ou écraser

Cette feature relève de la v1.0, distincte du undo local.

## Bugs classiques à éviter

### Drift entre pile undo et état CRDT
Si l'utilisateur sauve un snapshot puis reload, sa pile undo doit-elle persister ? Non par défaut, sinon on risque des incohérences.

### Loops de undo/redo
On veut que `redo(undo(op)) == op`. yrs garantit ça.

### Multiple opérations dans une transaction = un undo
Confirmé par tests. À ne pas oublier dans le design des outils.

### Undo qui resuscite des éléments supprimés par autrui
Comme vu, c'est ok techniquement, mais peut surprendre. Solution : pop-up de confirmation si l'undo affecte un élément touché par un autre peer.

## Test de validation

```
1. A crée rect → A undo → rect disparait
2. A crée rect → B supprime → A undo → rien (déjà supprimé)
3. A crée rect → B édite → A undo → rect supprimé + édit perdu
4. A crée 100 rects → A 100x undo → vide
5. A et B éditent en parallèle → A undo n'affecte que ses ops
```

## Liens

- CRDT fundamentals → [01-crdt-fondamentaux.md](./01-crdt-fondamentaux.md)
- Conflits → [06-conflits-cas-limites.md](./06-conflits-cas-limites.md)
- Versioning → roadmap v1.0+
