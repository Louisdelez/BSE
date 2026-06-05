# 02.06 — Positionnement de BSE

> Le *pitch* différenciateur. Pour qui, contre qui, pourquoi maintenant.

## Le pitch en 30 secondes

> *« BSE est l'application desktop open-source de brainstorming collaboratif en temps réel. Construite en Rust pour la performance native, sur une toile infinie partagée — comme un Miro qui démarre en moins d'une seconde, qu'on installe chez soi en une commande, et qu'on contrôle totalement. »*

## La matrice de positionnement

```
                  Performance native
                         ▲
                         │
              [BSE] ●    │
                         │
                         │
                         │
  Open-source ◄──────────┼──────────► Propriétaire
                         │
                         │
    Excalidraw ●         │       ● Figma/FigJam
                         │       ● Miro
       tldraw ●          │       ● Mural
                         │
                         ▼
                    Web / Electron
```

BSE est seul dans le quadrant **open-source + native**.

## La proposition de valeur

### Pour le décideur (CTO, CIO, head of innovation)

> *« Vous avez besoin d'un outil de brainstorming. Les options actuelles sont soit SaaS US (Figma, Miro) avec données hors UE, soit OSS limités (Excalidraw) qui n'ont pas la richesse fonctionnelle. BSE vous donne le meilleur des deux : richesse Miro + souveraineté Excalidraw + performance qu'aucun d'eux n'offre. »*

### Pour l'utilisateur final (animateur, facilitateur, designer)

> *« Vous animez des sessions. Vous voulez que ça soit fluide, beau, et que ça ne plante pas en plein workshop client. BSE démarre instantanément, ne rame jamais, et propose un mode facilitation complet (timer, mode privé, voting). »*

### Pour le développeur (open-source enthusiast, contributor)

> *« Une code base Rust moderne, propre, libre, sur des sujets passionnants : CRDT, GPU rendering, networking. Un projet ambitieux mais réaliste. »*

## Différenciateurs (les 4 piliers)

### 1. Performance native
- **60+ FPS** soutenu, vise 144 FPS sur display compatible
- **<500 ms démarrage** cold
- **<100 MB mémoire** au repos
- GPU-accelerated rendering (wgpu/vello)

### 2. Souveraineté
- **Self-host** en `docker compose up`
- **Open-source** licence permissive (Apache-2)
- **E2E encryption** optionnelle (room key)
- **Aucune dépendance** cloud propriétaire
- **RGPD by design**

### 3. Brainstorming-first
- **Templates** de session pré-installés (10 au lancement)
- **Mode facilitation** complet (timer, privé, vote, summon)
- **Phases scriptées** dans les templates
- **Mode anonyme** pour réduire la peur du jugement
- **Mindmap** structurée native

### 4. Multi-utilisateur fluide
- **CRDT mature** (yrs ou Loro)
- **Présence riche** (curseurs, sélections, noms)
- **Offline-first** avec resync auto
- **Hybride sync/async**

## Anti-positionnement

Pour clarifier ce que BSE *n'est pas* :

- ❌ Pas un **Figma** : on ne fait pas de design pixel-perfect avec components et variants
- ❌ Pas un **Lucidchart** : pas de diagrammes UML/BPM formels
- ❌ Pas un **Notion** : pas de documents structurés
- ❌ Pas un **Drawpile** : pas de raster art numérique
- ❌ Pas un **Excalidraw avec plus de features** : on a aussi un mode esthétique différent
- ❌ Pas un **wrap web** Rust : on est natif de bout en bout

## La cible primaire (TAM/SAM/SOM)

### Total Addressable Market
Toute équipe qui collabore créativement. Le marché « visual collaboration » = ~$15-20B en 2026.

### Serviceable Addressable Market
Équipes avec **contraintes de souveraineté** ou **sensibles à la performance** :
- R&D et innovation en grandes entreprises EU
- Secteur public / défense / pharma
- Studios créatifs (design, advertising) avec NDA clients
- Établissements d'enseignement supérieur
- Communautés open-source

### Serviceable Obtainable Market (3 ans)
- **5 000 organisations** en self-host
- **50 000 utilisateurs actifs mensuels**

Modeste mais réaliste pour un projet OSS.

## Les concurrents à neutraliser par fonctionnalité

| Concurrent | Feature à neutraliser | Comment |
|---|---|---|
| Figma | Performance | On va plus vite par design |
| Miro | Templates | 10 livrés, marketplace plus tard |
| Mural | Facilitation Superpowers | On les copie tous |
| Excalidraw | E2E + simplicité install | On copie, en mieux |
| tldraw | Engine sync | CRDT > leur custom |

## Risques de marché

| Risque | Probabilité | Mitigation |
|---|---|---|
| Excalidraw ajoute multi-projet | Moyenne | On a plus de features de toutes manières |
| Figma sort un client desktop natif | Faible | Trop d'investissement Electron déjà |
| Miro réduit ses prix | Forte | Self-host = $0, on gagne sur souveraineté |
| Un Rust dev sort un BSE-clone | Moyenne | First mover advantage si bien exécuté |
| Désintérêt général | Faible | Le marché est en croissance, demande EU souveraineté forte |

## Phrase de pitch finale

> *« Imaginez Miro, mais qui démarre en moins d'une seconde, tourne à 144 FPS, vous appartient totalement, et que vous installez sur votre serveur en une commande. C'est BSE. »*
