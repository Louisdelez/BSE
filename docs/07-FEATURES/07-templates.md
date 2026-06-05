# 07.07 — Templates de session

> Le catalogue de templates BSE — la valeur produit qui distingue d'un canvas vierge.

## Vision

Un nouveau projet ne devrait pas démarrer vide. L'utilisateur choisit un template selon sa session :
- Brainstorm classique
- Rétrospective
- SCAMPER
- Six chapeaux
- Empathy map
- ...

Et un canvas pré-structuré apparaît avec zones, instructions, scripts.

## Liste des templates v1.0

| Template | Catégorie | Description |
|---|---|---|
| **Vierge** | - | Démarre avec une toile blanche |
| **Brainstorm 6-3-5** | Idéation | 6 colonnes, timer 5 min, 6 rondes |
| **SCAMPER** | Idéation | 7 zones (S, C, A, M, P, E, R) |
| **Crazy 8s** | Idéation | Grille 2×4 |
| **Six chapeaux** | Analyse | 6 zones colorées |
| **Reverse brainstorm** | Idéation | 2 colonnes (Causer / Inverser) |
| **Starbursting** | Approfondir | Étoile 6 branches |
| **Empathy map** | UX research | 4 quadrants (Says/Thinks/Does/Feels) |
| **Customer journey** | UX | Timeline avec étapes |
| **Mindmap radial** | Structurer | Nœud central + 8 branches |
| **Rétrospective Start/Stop/Continue** | Équipe | 3 colonnes |
| **Rétrospective MadSadGlad** | Équipe | 3 colonnes humeurs |
| **Matrice impact/effort** | Convergence | Plan 2×2 |
| **Lean Canvas** | Product | 9 cases business |
| **Affinity map** | Convergence | Zone vide avec instructions |

## Modèle

```rust
pub struct Template {
    pub id: TemplateId,
    pub name: String,
    pub description: String,
    pub category: TemplateCategory,
    pub preview_url: String,
    pub elements: Vec<Element>,        // contenu initial
    pub script: Option<TemplateScript>, // optionnel
    pub camera: Camera,                 // caméra initiale
}

pub enum TemplateCategory {
    Ideation,
    Convergence,
    UxResearch,
    TeamRetro,
    Strategy,
    Other,
}
```

## Scripts de session

Un template peut définir un **script** : enchaînement de phases.

```rust
pub struct TemplateScript {
    pub phases: Vec<Phase>,
}

pub struct Phase {
    pub name: String,
    pub duration: Option<Duration>,
    pub instructions: String,             // markdown
    pub mode: PhaseMode,
    pub on_start: Vec<PhaseAction>,
    pub on_end: Vec<PhaseAction>,
}

pub enum PhaseMode {
    Free,
    PrivateGeneration,    // mode privé activé
    SharedView,
    Voting,
    Convergence,
}

pub enum PhaseAction {
    EnablePrivateMode,
    StartTimer(Duration),
    EnableVoting { budget: u8 },
    RevealPrivateContent,
    LockElements(Vec<ElementId>),
    ZoomToRegion(Rect),
}
```

### Exemple : script de Brainstorm 6-3-5

```yaml
phases:
  - name: "Introduction"
    duration: 5min
    instructions: |
      Bienvenue dans la session 6-3-5.
      Cadrage du problème : [...]
    
  - name: "Round 1 - Idéation silencieuse"
    duration: 5min
    mode: private_generation
    instructions: |
      Écris 3 idées sur des post-its dans ta colonne.
      Mode privé activé : les autres ne te voient pas.
    on_start:
      - enable_private_mode
      - start_timer: 5min
    
  - name: "Round 2 - Rebond"
    duration: 5min
    mode: free
    instructions: |
      Lis les 3 idées du voisin, ajoute 3 nouvelles.
    
  # ... 4 rondes de plus
    
  - name: "Convergence"
    duration: 15min
    mode: voting
    instructions: |
      Vote pour tes 3 idées préférées.
    on_start:
      - enable_voting:
          budget: 3
```

## Pre-built elements par template

Chaque template a un **payload d'éléments** prêts. Exemple SCAMPER :

```rust
fn build_scamper_template() -> Vec<Element> {
    let titles = [
        ("S - Substituer", "Remplace un composant"),
        ("C - Combiner", "Mix avec autre chose"),
        ("A - Adapter", "Imite un autre domaine"),
        ("M - Modifier", "Change taille, forme, perception"),
        ("P - Proposer autre usage", "Nouvel usage possible ?"),
        ("E - Éliminer", "Que peut-on retirer ?"),
        ("R - Réorganiser", "Inverser, réordonner"),
    ];
    
    let mut elements = Vec::new();
    for (i, (title, subtitle)) in titles.iter().enumerate() {
        let x = (i as f32) * 350.0;
        elements.push(text_element(title, x, 0.0));
        elements.push(text_element(subtitle, x, 50.0));
        elements.push(rectangle(x - 20.0, 100.0, 300.0, 600.0));  // zone
    }
    
    elements
}
```

## Catalog UI

À la création d'un projet :

```
┌──────────────────────────────────────────────────────────┐
│  Nouveau projet                                          │
├──────────────────────────────────────────────────────────┤
│ Filtrer : [Toutes ▼]      Recherche : [____________🔍]   │
│                                                          │
│ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐              │
│ │        │ │        │ │        │ │        │              │
│ │Vierge  │ │6-3-5   │ │SCAMPER │ │Crazy 8s│              │
│ │        │ │        │ │        │ │        │              │
│ └────────┘ └────────┘ └────────┘ └────────┘              │
│                                                          │
│ ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐              │
│ │        │ │        │ │        │ │        │              │
│ │6 chap. │ │Starbst.│ │Empathy │ │Journey │              │
│ │        │ │        │ │        │ │        │              │
│ └────────┘ └────────┘ └────────┘ └────────┘              │
│                                                          │
│                                  [Choisir] [Annuler]     │
└──────────────────────────────────────────────────────────┘
```

Chaque preview = capture PNG du template.

## Storage des templates

### Built-in
Les templates v1.0 sont **bundle dans l'app** (fichiers JSON dans `assets/templates/`).

### Custom (v1.x)
- Un utilisateur peut sauvegarder un projet courant comme template
- Templates custom dans `~/.local/share/bse/templates/`
- Partage : export en fichier `.bse-template`

### Marketplace (v2+)
- Un repo public ou un service de partage de templates
- Notation, téléchargement, contribution

## Mode présentation déclenché par template

Certains templates incluent le mode présentation par défaut :
- Le facilitateur navigue de phase en phase
- Les autres voient l'instruction de phase
- Le timer s'affiche en grand

## Tests

- Tous les templates v1.0 sont créés correctement
- Le script de phase change correctement entre les phases
- Custom templates peuvent être sauvés et rechargés
- Templates rendent visuellement comme prévu

## Liens

- Méthodes brainstorm → [../01-BRAINSTORMING-RECHERCHE/02-methodes-techniques.md](../01-BRAINSTORMING-RECHERCHE/02-methodes-techniques.md)
- Facilitation → [../01-BRAINSTORMING-RECHERCHE/04-facilitation.md](../01-BRAINSTORMING-RECHERCHE/04-facilitation.md)
- Post-its → [06-post-its-cards.md](./06-post-its-cards.md)
