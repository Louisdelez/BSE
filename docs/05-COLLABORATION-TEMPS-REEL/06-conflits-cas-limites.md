# 05.06 — Conflits et cas limites

> Comment BSE gère les situations délicates en édition concurrente.

## Vue d'ensemble

Le CRDT garantit la **convergence**. Mais il ne garantit pas que le résultat soit **désirable**. Certains cas méritent une politique explicite ou une UX.

## Cas 1 — Édition concurrente d'une propriété simple

Alice change la couleur d'un rect en bleu. Bob en rouge. Quasi simultané.

- yrs LWW : le **dernier en horloge logique** gagne
- L'autre peer voit le changement appliqué chez lui
- Pas d'UI spéciale

C'est le cas trivial, marche par défaut.

## Cas 2 — Suppression + édition concurrente

Alice supprime un élément. Bob modifie la même propriété de l'élément.

- yrs : delete wins (l'élément n'est plus dans le map)
- Bob voit l'élément disparaître
- Pas d'UI spéciale (déjà géré par CRDT)

**Variante** : on peut afficher une notif « 1 élément que tu modifiais a été supprimé » côté Bob.

## Cas 3 — Modification de la même partie de texte

Alice et Bob tapent au même endroit d'un post-it.

- yrs::Text gère ça avec son CRDT texte (YATA)
- **Risque interleaving** : si vraiment au même caractère, les caractères peuvent se mélanger
- En pratique : extremement rare, mais possible

**Mitigation** : si on bascule sur Loro, l'algorithme Fugue garantit non-interleaving.

## Cas 4 — Déplacement concurrent

Alice déplace un rect à (100, 100). Bob le déplace à (200, 200). Simultané.

- yrs LWW : le dernier gagne (position finale = celle du peer avec horloge max)
- Pas de fusion : aucune sémantique de « position moyenne »
- L'autre peer voit son déplacement « écrasé »

**UX optionnelle** : si le rect bouge sous le curseur de Bob alors qu'il dragge, on pourrait pulse une notification. Mais c'est complexité gratuite — Figma ne le fait pas.

## Cas 5 — Resize concurrent

Alice resize en hauteur. Bob resize en largeur. Simultané.

- `width` et `height` sont **deux registres LWW indépendants**
- Donc Alice's height + Bob's width peuvent **coexister** !
- Résultat : un rectangle avec les deux modifs appliquées
- Pas de conflit

C'est un cas où le CRDT est *meilleur* que LWW global.

## Cas 6 — Sélection concurrente / move multiple

Alice sélectionne 5 éléments et les déplace. Bob sélectionne 2 des 5 (intersection) et les déplace dans la direction opposée.

- Chaque déplacement d'élément est une op CRDT séparée
- LWW : les éléments touchés par les deux peers prennent la position du dernier writer
- Les 3 éléments touchés seulement par Alice gardent sa position
- Les 2 touchés par les deux prennent la position du dernier

Cohérent. Pas de UI spéciale.

## Cas 7 — Création d'éléments avec UUID identique

Théoriquement impossible avec UUID v7/v4 (collision = 1 sur 2⁶⁴+). Mais imaginons.

- Le yrs::Map a 1 slot par clé → un seul gagne (LWW)
- L'autre élément est perdu

**Mitigation** : générer les UUID **localement, jamais coordonnés**. Le risque est mathématiquement nul.

## Cas 8 — Stroke en cours de dessin + autre peer

Alice dessine un trait (50 points appendés). Bob bouge un élément voisin pendant.

- Les ops sont indépendantes (pas de overlap)
- Aucun conflit

**Variante** : Bob tape sur la zone où Alice dessine. Aucun problème — ils créent deux éléments distincts.

## Cas 9 — Reparenting de mindmap concurrent

Alice fait du nœud N un enfant de A. Bob en fait un enfant de B.

### Avec yrs (LWW sur le champ parent_id)
- Le dernier write gagne
- N finit chez A OU chez B
- Risque de cycle : si A devient enfant de B et que B était enfant de A, cycle

**Détection de cycle côté client** :
```rust
fn detect_cycle(scene: &Scene, child: ElementId, parent: ElementId) -> bool {
    let mut current = parent;
    loop {
        if current == child { return true; }
        match scene.parent_of(current) {
            Some(p) => current = p,
            None => return false,
        }
    }
}
```

Si cycle détecté → casser en mettant `parent = None` (orphelin).

### Avec Loro
- Movable Tree CRDT empêche le cycle par construction
- Solution propre. **Argument pour Loro en v0.5**.

## Cas 10 — Net partition (split brain)

Alice et Bob perdent la connexion pendant 5 min. Chacun édite localement.

- Quand reconnectent : sync se déclenche
- Toutes les ops accumulées sont échangées
- CRDT garantit la convergence
- Aucune perte de données

Mais l'utilisateur peut être surpris par le merge. Solution UX :
- Indicateur clair de l'état de connexion
- Pop-up « X modifications de Bob viennent de se synchroniser »

## Cas 11 — Snapshot vs ops désynchronisés

Si un client a un snapshot obsolète et envoie des ops basées dessus, mais que le serveur a évolué :
- Les ops s'appliquent quand même (CRDT)
- Le client reçoit en retour les ops qu'il manquait
- Convergence garantie

Pas de cas pathologique.

## Cas 12 — Op massive (ex: paste de 1000 éléments)

Alice colle 1000 éléments.

- Volume d'ops : ~1000 add ops
- Traffic WS : ~150 KB (binaire compact)
- Côté serveur : flood
- Côté autres peers : application en batch

**Mitigation** :
- Rate limit serveur : refuse au-delà
- Batching de la transaction : 1 update binaire contenant tout
- Notification UX : « 1000 éléments ajoutés »

## Cas 13 — Verrou applicatif

Parfois on veut **interdire** que 2 peers éditent le même élément :
- Mode présentation : un présentateur a la main
- Verrou explicite : on lock un élément le temps de l'éditer

BSE en v1.0 : pas de verrou applicatif (par design : CRDT gère).
BSE en v1.x : option de **lock** explicite via UI (un peer peut verrouiller un élément temporairement).

## Cas 14 — Permissions changent au milieu

Bob est éditeur. Owner le passe en viewer pendant qu'il est connecté.

- Le serveur révoque les futures ops de Bob
- Bob voit son rôle changer (notification)
- Ses ops déjà envoyées restent appliquées
- Sa session WS reste active mais en read-only

## Cas 15 — Cleanup périodique

Une room qui tourne 24/7 accumule potentiellement :
- WAL entries (Postgres)
- Tombstones (CRDT)

Cleanup :
- WAL : tronqué après checkpoint (>1h)
- Tombstones : GC quand consensus de tous les peers actuels
- Snapshots : garder N derniers

## Cas 16 — Migration de schéma

Si BSE évolue le modèle (nouveau type d'élément), les vieux projets doivent rester ouvrables.

- Champ `model_version` au niveau du Doc
- Migration lazy à l'ouverture
- Tests round-trip pour chaque version

Cf. [../03-ARCHITECTURE/05-modele-donnees.md](../03-ARCHITECTURE/05-modele-donnees.md).

## Cas 17 — Échec d'envoi d'op

Le client génère une op, l'envoie, le serveur crashe avant d'appliquer.

- Le client a l'op dans son CRDT local (déjà appliquée)
- Le serveur ne l'a pas
- À la reconnexion, le client envoie son state vector → server demande les ops manquantes
- Convergence rétablie

Aucune perte.

## Cas 18 — Replay attack

Un attaquant rejoue un message WS.

- yrs : les ops sont idempotentes (déjà appliquées = no-op)
- Pas de risque de corruption
- Mais : bande passante gâchée → rate limit serveur côté entrée

## Synthèse — politique BSE

| Conflit | Politique | UX feedback |
|---|---|---|
| Édit concurrente propriété | LWW | Aucun |
| Delete + edit | Delete wins | Notif optionnelle |
| Move concurrent | LWW | Aucun |
| Resize concurrent (différentes dims) | Merge automatique | Aucun |
| Text concurrent | CRDT text | Aucun |
| Reparenting | LWW + détection cycle | Notif si cycle |
| Net partition | Resync auto | Notif « X synchronisé » |
| Op massive | Rate limit | Notif « N éléments ajoutés » |
| Permission change | Révocation soft | Notif rôle |

## Tests à automatiser

Property-based tests (`proptest`) :
- Pour N peers, K opérations aléatoires → tous les peers convergent au même état
- Round-trip serialization de tous les types d'opérations
- Replay random d'ops dans tous les ordres → état identique
