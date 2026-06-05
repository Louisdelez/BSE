# 12.03 — Papers académiques

> Les papers de référence pour comprendre les choix techniques de BSE.

## CRDTs — fondamentaux

### Shapiro et al., 2011
- **Titre** : *Conflict-free Replicated Data Types*
- **Référence** : INRIA Tech Report RR-7687, 2011
- **URL** : hal.inria.fr/inria-00609399
- **Apport** : pose le terme CRDT, classification CvRDT/CmRDT, types canoniques (counters, sets, etc.)

### Shapiro et al., 2011 (extended)
- **Titre** : *A comprehensive study of Convergent and Commutative Replicated Data Types*
- **URL** : hal.inria.fr/inria-00555588
- **Apport** : taxonomie complète

### Bauwens & Boix, 2024+
- Papers récents sur les *Push-based CRDTs* et applications

## CRDTs — textes collaboratifs

### Preguiça et al., 2009 — WOOT
- **Titre** : *WOOT: An Effective Model for Real-Time Cooperative Editing*

### Weiss et al., 2010 — Logoot
- **Titre** : *Logoot: A Scalable Optimistic Replication Algorithm for Collaborative Editing*

### Roh et al., 2011 — RGA
- **Titre** : *Replicated abstract data types: Building blocks for collaborative applications*

### Yjs / YATA
- **Titre** : *Near Real-Time Peer-to-Peer Shared Editing on Extensible Data Types*
- **Auteur** : Kevin Jahns
- **URL** : github.com/yjs/yjs (références)

### Fugue (2023)
- **Titre** : *The Art of the Fugue: Minimizing Interleaving in Collaborative Text Editing*
- **Auteurs** : Weidner, Gentle, Kleppmann
- **URL** : arxiv.org/abs/2305.00583
- **Apport** : algorithm pour maximal non-interleaving — implémenté dans Loro

### Eg-walker (2024)
- **Titre** : *Collaborative Text Editing with Eg-walker: Better, Faster, Smaller*
- **Auteurs** : Gentle, Kleppmann
- **URL** : arxiv.org/abs/2409.14252
- **Apport** : optimisation mémoire et perf — implémenté dans Diamond Types

## Operational Transformation

### Ellis & Gibbs, 1989
- **Titre** : *Concurrency Control in Groupware Systems*
- **Apport** : OT initial

### Ressel et al., 1996
- **Titre** : *An Integrating, Transformation-Oriented Approach to Concurrency Control and Undo in Group Editors*

### Sun et al., 1998
- **Titre** : *Achieving Convergence, Causality Preservation, and Intention Preservation*

## Brainstorming — science

### Diehl & Stroebe, 1987 (clé)
- **Titre** : *Productivity Loss in Brainstorming Groups: Toward the Solution of a Riddle*
- **Journal** : Journal of Personality and Social Psychology, 53(3), 497–509
- **URL** : tandfonline.com/doi/abs/10.1207/s15324834basp1201_1
- **Apport** : identification du **production blocking** comme cause principale

### Mullen, Johnson & Salas, 1991
- **Titre** : *Productivity Loss in Brainstorming Groups: A Meta-Analytic Integration*
- **Journal** : Basic and Applied Social Psychology, 12(1)
- **Apport** : méta-analyse confirme la perte de productivité

### Paulus, 2000
- **Titre** : *Groups, teams, and creativity: The creative potential of idea-generating groups*
- **Journal** : Applied Psychology Review

### Paulus & Brown, 2007
- **Titre** : *Toward more creative and innovative group idea generation*
- **Apport** : stimulation cognitive vs production blocking — base du brainwriting

### Sutton & Hargadon, 1996
- **Titre** : *Brainstorming Groups in Context: Effectiveness in a Product Design Firm*
- **Journal** : Administrative Science Quarterly, 41(4)
- **Apport** : brainstorming à IDEO, fonctions sociales du brainstorming

### Stroebe et al., 2010
- **Titre** : *Beyond Productivity Loss in Brainstorming Groups: The Evolution of a Question*
- **Journal** : Advances in Experimental Social Psychology, Vol 43
- **URL** : sciencedirect.com/science/article/abs/pii/S006526011043004X

### Putman & Paulus, 2009
- **Titre** : *Brainstorming, Brainstorming Rules and Decision Making*
- **Journal** : Journal of Creative Behavior, 43(1)

## Brainwriting

### Rohrbach, 1969
- **Titre** : *Kreativ nach Regeln — Methode 635, eine neue Technik zum Lösen von Problemen*
- **Apport** : invention de la méthode 6-3-5

### Heslin, 2009
- **Titre** : *Better than brainstorming? Potential contextual boundary conditions to brainwriting*
- **Apport** : étude comparative

### Pamela et al., 2011
- **Titre** : *Effectiveness of Brainwriting Techniques: Comparing Nominal Groups to Real Teams*
- **URL** : link.springer.com/content/pdf/10.1007/978-0-85729-224-7_22.pdf

## Group / nominal

### Diehl & Stroebe, 1991
- **Titre** : *Productivity Loss in Idea-Generating Groups: Tracking Down the Blocking Effect*
- **Journal** : JPSP, 61(3)

### Paulus & Yang, 2000
- **Titre** : *Idea generation in groups: a basis for creativity in organizations*

## Design Thinking

### Brown, 2008
- **Titre** : *Design Thinking*
- **Journal** : Harvard Business Review
- **Apport** : pose le design thinking en management

### Kelley & Kelley, 2013
- **Titre** : *Creative Confidence*
- Livre IDEO sur la créativité

### Liedtka, 2014
- **Titre** : *Innovative Ways Companies Are Using Design Thinking*

## Edge cases CRDTs

### Kleppmann et al., 2018
- **Titre** : *Local-first software: You own your data, in spite of the cloud*
- **URL** : inkandswitch.com/local-first/
- **Apport** : manifeste local-first, lien direct avec philosophie BSE

### Kleppmann & Beresford, 2017
- **Titre** : *A Conflict-Free Replicated JSON Datatype*
- **URL** : arxiv.org/abs/1608.03960

### Nair et al., 2022
- **Titre** : *A Conflict-Free Replicated Data Type for Concurrent Movement of Subtrees in Trees*
- Loro a implémenté ça (Movable Tree)

## Performance / rendering

### Loop et al., 2005
- **Titre** : *Resolution Independent Curve Rendering using Programmable Graphics Hardware*
- Référence SDF / Bézier sur GPU

### Green, 2007
- **Titre** : *Improved Alpha-Tested Magnification for Vector Textures and Special Effects*
- Valve, SDF pour textures vectorielles

### Vello papers
- Linebender publie des posts détaillés sur le rendering moderne 2D GPU-compute

## Sites / blogs de référence

| Source | Type |
|---|---|
| [crdt.tech](https://crdt.tech) | Référence CRDT |
| [inkandswitch.com](https://inkandswitch.com) | Recherche local-first |
| [Martin Kleppmann's blog](https://martin.kleppmann.com) | CRDTs, distributed systems |
| [tldraw blog](https://tldraw.substack.com) | Canvas eng |
| [Figma engineering blog](https://www.figma.com/blog/section/inside-figma/) | Multiplayer architecture |
| [Linebender blog](https://linebender.org/blog) | Vello, GPU rendering |

## Liens

- Projets open-source → [02-projets-open-source.md](./02-projets-open-source.md)
- Outils existants → [../01-BRAINSTORMING-RECHERCHE/03-outils-existants.md](../01-BRAINSTORMING-RECHERCHE/03-outils-existants.md)
- CRDT fondamentaux → [../05-COLLABORATION-TEMPS-REEL/01-crdt-fondamentaux.md](../05-COLLABORATION-TEMPS-REEL/01-crdt-fondamentaux.md)
