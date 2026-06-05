# 07.06 — Post-its et cartes

> L'élément emblématique du brainstorm. Indispensable.

## Vision

Le post-it est l'élément le plus utilisé en session de brainstorm. UI optimisée pour : création rapide, identification visuelle, regroupement (affinity).

## Modèle

```rust
pub struct Postit {
    pub text: String,              // yrs::Text
    pub color: PostitColor,
    pub size: PostitSize,
    pub author_label: Option<String>,
    pub votes: HashMap<UserId, u8>,  // dot votes
}

pub enum PostitColor {
    Yellow,
    Pink,
    Blue,
    Green,
    Orange,
    Purple,
    White,
}

pub enum PostitSize {
    Small,    // 100×100
    Medium,   // 200×200 (default)
    Large,    // 300×300
}
```

## Création

### Méthodes
1. **Outil post-it actif** (`N` raccourci) + click → crée à la position du click
2. **Double-click vide** → crée un post-it à cet endroit (configurable)
3. **Drag depuis la sidebar des couleurs** → drag-drop sur la toile
4. **Paste de texte** → option « créer un post-it » dans le menu contextuel
5. **Bulk create** : paste de multi-lignes → 1 post-it par ligne

### Auto-resize
Le texte est centré. La taille du post-it grandit avec le texte (jusqu'à `PostitSize::Large`), puis le texte commence à wrap.

### Couleur par utilisateur (option)
Setting : « ma couleur de post-it ». Si activé, tous mes posts-its naissent dans ma couleur.

## Édition

- **Click** → sélectionne le post-it (peut le déplacer)
- **Double-click** → entre dans le mode édition texte
- **Enter** (en sélection) → édite texte
- **Echap** → quitte édition

## Style

### Visuel
- Coin légèrement arrondi (5-8 px)
- Ombre douce
- Inclinaison aléatoire de ±2° (effet "vrai post-it") — option

### Author label
Option : afficher initiales du créateur en bas droite, dans sa couleur.

```
┌───────────────┐
│ Optimiser     │
│ l'onboarding  │
│               │
│            AD │ ← initiales en couleur du peer
└───────────────┘
```

### Vote pills
Quand des votes existent (dot voting), affichage en bas :

```
┌───────────────┐
│ Mon idée      │
│               │
│ ● ● ● 3 votes │
└───────────────┘
```

## Affinity mapping

Quand l'utilisateur drag plusieurs post-its côte à côte :
- BSE détecte automatiquement la proximité (<50 px)
- Suggère un **group** par couleur dominante ou pattern
- Option pour valider en groupe nommé

Cluster manuel via : Ctrl+G (group) après sélection multiple.

## Bulk operations

### Bulk paste
Coller un texte multi-ligne :
```
Idée 1
Idée 2
Idée 3
```
→ Crée 3 post-its (alignés en grille ou colonne)

### Bulk import depuis CSV
v1.x : drag d'un CSV → 1 post-it par ligne (text de la première colonne).

### Bulk export
Sélection → Ctrl+C → texte multi-ligne (1 post-it par ligne).

## Voting (dot voting)

### Setup
Le facilitateur déclenche un vote :
- Choisit le budget (3 votes par défaut)
- Choisit la durée (5 min par défaut)
- Optionnel : anonyme ou nominatif

### Action user
Pendant la phase de vote :
- Click sur post-it → +1 vote
- Click droit → -1 vote
- UI affiche votes restants

### Reveal
À la fin du timer (ou trigger manuel) :
- Les votes sont affichés
- Top-N peuvent être mis en avant (zoom + highlight)
- Snapshot avant vote sauvé (rollback possible)

### Modèle CRDT
```rust
// HashMap<UserId, u8> dans le Postit
// CRDT-compatible : conflict-free par user_id
```

## Mode anonyme

Si activé pour le projet :
- author_label remplace par anonyme
- mais user_id réel stocké en metadata pour modération éventuelle

## Stylisation par template

Différents templates peuvent imposer des couleurs :
- **Rétrospective** : Vert/Orange/Rouge pour Start/Continue/Stop
- **SCAMPER** : couleurs par verbe
- **Six chapeaux** : couleurs imposées (blanc/rouge/noir/jaune/vert/bleu)

## Snap entre post-its

Quand on déplace un post-it, snap optionnel :
- Aligne sur le bord d'un autre post-it
- Espacement régulier suggéré

## Performance

- Post-it = élément léger (text + 1 quad SDF + ombre)
- 500 post-its sur scène → no problem
- 5000 post-its → LOD agressif (cf [../06-CANVAS-INFINI/04-culling-lod.md](../06-CANVAS-INFINI/04-culling-lod.md))

## Recherche

Sur la toile, on peut rechercher (`Ctrl+F`) :
- Le texte des post-its est indexé
- Match → zoom auto sur le post-it + highlight

## Tests

- Création rapide à plusieurs (10 peers créent 10 post-its chacun)
- Bulk paste fonctionne
- Voting flow complet (start, vote, reveal, rollback)
- Anonyme : author_label invisible mais distinct

## Liens

- Voting features → [../01-BRAINSTORMING-RECHERCHE/04-facilitation.md](../01-BRAINSTORMING-RECHERCHE/04-facilitation.md)
- Affinity mapping → cf brainstorming methods
- Templates → [07-templates.md](./07-templates.md)
