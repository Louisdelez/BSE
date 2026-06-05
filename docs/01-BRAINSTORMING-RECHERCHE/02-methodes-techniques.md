# 01.02 — Méthodes et techniques de brainstorming

> 12+ techniques classées par usage. Pour chacune : principe, force, faiblesse, et comment BSE peut la supporter via des **templates**.

## A. Techniques de génération (divergence)

### 1. Brainstorming classique (verbal)
- **Principe** : tour de table libre, on dit ses idées à voix haute.
- **Force** : simple, intuitif, energisant.
- **Faiblesse** : *production blocking*, voix fortes dominent.
- **Support BSE** : pas spécifiquement supporté — déconseillé d'ailleurs.

### 2. Brainwriting
- **Principe** : chacun écrit ses idées **en silence**, puis on partage.
- **Force** : élimine le blocking, égalise la parole.
- **Faiblesse** : moins « social » qu'à l'oral.
- **Support BSE** : phase 1 = chacun ajoute des post-its dans **sa zone privée** (mode focus). Phase 2 = on révèle.

### 3. Méthode 6-3-5
- **Principe** : 6 personnes × 3 idées × 5 minutes × 6 rondes = **108 idées en 30 minutes**. À chaque ronde, on échange sa feuille avec son voisin et on rebondit.
- **Force** : production massive, structurée, stimulation cognitive intégrée.
- **Faiblesse** : optimal exactement à 6 personnes.
- **Support BSE** : **template dédié** avec 6 colonnes pré-créées, timer intégré, rotation automatique des zones.

### 4. Mind mapping
- **Principe** : carte mentale autour d'un nœud central, expansion par branches associatives.
- **Force** : explore non-linéairement, visualise les relations.
- **Faiblesse** : devient illisible quand la carte grossit.
- **Support BSE** : **feature native** (cf. [../07-FEATURES/05-mindmap-connecteurs.md](../07-FEATURES/05-mindmap-connecteurs.md)).

### 5. SCAMPER
- **Principe** : appliquer 7 verbes à un produit/idée existant — **S**ubstituer, **C**ombiner, **A**dapter, **M**odifier, **P**roposer un autre usage, **É**liminer, **R**éorganiser.
- **Force** : génère des améliorations systématiques.
- **Faiblesse** : oriente vers l'incrémental, pas le disruptif.
- **Support BSE** : **template** avec 7 zones étiquetées.

### 6. Reverse brainstorming
- **Principe** : *« Comment **causer** le problème ? »* au lieu de *« comment le résoudre ? »*. Puis on inverse les réponses.
- **Force** : débloque les équipes coincées, casse les biais.
- **Faiblesse** : peut sembler frivole, demande un cadrage clair.
- **Support BSE** : template avec deux colonnes (« Comment empirer ? » → « Inverser »).

### 7. Worst possible idea
- **Principe** : forcer les *pires* idées. Lever la peur du jugement.
- **Force** : excellent icebreaker, libère la créativité.
- **Faiblesse** : ne produit pas directement de solution.
- **Support BSE** : template d'échauffement.

### 8. Six chapeaux de la pensée (De Bono)
- **Principe** : adopter successivement 6 angles : faits (blanc), émotions (rouge), critique (noir), optimisme (jaune), créativité (vert), processus (bleu).
- **Force** : analyse multi-dimensionnelle d'une même idée.
- **Faiblesse** : demande une discipline collective ; long.
- **Support BSE** : template avec 6 zones colorées correspondantes.

### 9. Rapid ideation
- **Principe** : écrire un *max* d'idées en temps limité (généralement 5 min).
- **Force** : énergise, démarre une session, casse la procrastination.
- **Faiblesse** : qualité variable.
- **Support BSE** : timer intégré + compteur de post-its.

### 10. Round robin
- **Principe** : chacun parle à tour de rôle, exactement une idée par tour.
- **Force** : égalité de parole garantie.
- **Faiblesse** : version verbale = production blocking ; version écrite = brainwriting.
- **Support BSE** : pas de support spécifique, mais le mode présentation peut indiquer le tour.

### 11. Starbursting
- **Principe** : étoile à 6 branches autour d'une idée — **Qui**, **Quoi**, **Où**, **Quand**, **Pourquoi**, **Comment**. Générer un max de questions pour chaque.
- **Force** : approfondit une idée avant exécution.
- **Faiblesse** : convergent, pas divergent.
- **Support BSE** : template étoile 6 branches.

### 12. Brain-netting / brainstorming asynchrone
- **Principe** : pas de réunion, contributions étalées dans le temps via outil partagé.
- **Force** : équipes distantes, fuseaux différents, profils introvertis.
- **Faiblesse** : perd l'énergie de groupe, manque de stimulation immédiate.
- **Support BSE** : par construction. Cf. [05-brainstorming-distant.md](./05-brainstorming-distant.md).

### 13. Crazy 8s (Design Sprint)
- **Principe** : 8 idées en 8 minutes, dessinées dans 8 cases d'une feuille pliée.
- **Force** : force la quantité et la visualisation rapide.
- **Faiblesse** : profils non-visuels frustrés.
- **Support BSE** : template grille 8 cases.

## B. Techniques de convergence (souvent oubliées)

### 14. Affinity mapping
- **Principe** : regrouper les post-its par thèmes émergents.
- **Force** : fait émerger des clusters non anticipés.
- **Faiblesse** : subjectif.
- **Support BSE** : feature de *grouping* manuel ; sélection multi → group.

### 15. Dot voting
- **Principe** : chacun a N votes (pastilles) à distribuer.
- **Force** : démocratique, rapide.
- **Faiblesse** : effet de cascade (les premiers votes biaisent).
- **Support BSE** : feature *dot voting* intégrée avec budget configurable.

### 16. Matrice impact/effort
- **Principe** : placer chaque idée sur un plan 2D (impact, effort).
- **Force** : priorise visuellement.
- **Faiblesse** : évaluation subjective.
- **Support BSE** : template matrice 2×2.

### 17. Nominal Group Technique (NGT)
- **Principe** : génération individuelle silencieuse, partage, ranking individuel, agrégation.
- **Force** : reconnue comme la *plus efficace* en recherche.
- **Faiblesse** : peu connue, perçue comme rigide.
- **Support BSE** : peut être scriptée via un template multi-phases.

## C. Synthèse — choix d'une technique

```
Sujet large / vague          →  Mind mapping, Starbursting
Améliorer existant           →  SCAMPER
Production max               →  6-3-5, Crazy 8s, Rapid ideation
Équipe bloquée               →  Reverse brainstorming, Worst idea
Analyse multidim             →  Six chapeaux
Équipe distante / async      →  Brain-netting, Brainwriting async
Choisir parmi N options      →  Dot voting, Matrice impact/effort
Sortir le meilleur du groupe →  NGT
```

## D. Templates BSE livrés par défaut

Pour la v0.5, BSE livrera au minimum ces templates :

- ⭐ 6-3-5 (matrice 6 colonnes)
- ⭐ SCAMPER (7 zones)
- ⭐ Six chapeaux (6 zones colorées)
- ⭐ Starbursting (étoile 6 branches)
- ⭐ Crazy 8s (grille 2×4)
- ⭐ Affinity map (zone vide structurée)
- ⭐ Matrice impact/effort
- ⭐ Rétrospective Start/Stop/Continue
- ⭐ Empathy map (4 quadrants)
- ⭐ Customer journey

Détail dans [../07-FEATURES/07-templates.md](../07-FEATURES/07-templates.md).
