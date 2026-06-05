# 02.02 — Étude : Miro et Mural

> Les deux géants du whiteboarding *enterprise*. Architectures fermées, mais on peut déduire beaucoup de leurs choix produits.

## Miro

### Positionnement
Leader mondial du whiteboarding collaboratif enterprise. ~50M utilisateurs (2024). Valorisé à $17,5 mds en 2022.

### Stack technique (déductions publiques)

- **Front-end** : web app SPA (React + Canvas/WebGL)
- **Back-end** : services Java/Kotlin pour la majorité, Node.js pour le sync temps réel
- **Sync** : WebSocket avec leur protocole maison
- **Persistance** : MySQL + Cassandra + S3 + Elasticsearch
- **Infra** : AWS (multi-région), CDN CloudFront

Miro publie peu d'engineering blog. On en déduit (eng blogs sporadiques, presentations conf) qu'ils utilisent un mix :
- OT pour le texte
- LWW + résolution custom pour les objets
- Snapshots périodiques pour la persistance

### Ce qu'on apprend de Miro

1. **L'écosystème de templates est la moitié du produit.** Miro a des centaines de templates verticaux (rétro, OKR, design sprint, lean canvas, journey map). C'est ce qui les fait choisir par des PME et grandes entreprises.

2. **Les intégrations sont critiques.** Jira, Slack, Confluence, Teams, Zoom, Google Drive — c'est ce qui les fait rentrer en entreprise.

3. **Le modèle commercial est par siège.** Free pour 3 boards, payant ensuite. Modèle inadapté pour open-source.

4. **La performance est un point de friction.** Les utilisateurs de boards avec >1000 éléments se plaignent souvent de lenteur, surtout sur GPU intégrés. **C'est une opportunité directe pour BSE.**

5. **Le « facilitation mode » est sous-développé.** Miro a un mode présentation mais pas de scripts de session. C'est moins poussé que Mural.

### Forces à imiter pour BSE
- ✅ Catalogue de templates riche dès le départ
- ✅ Mode présentation propre
- ✅ Exports multiples (PNG, PDF, CSV pour post-its)

### Faiblesses à éviter pour BSE
- ❌ Performance dégradée à grande échelle
- ❌ UX devenue chargée avec les années (trop de panels)
- ❌ Onboarding long

## Mural

### Positionnement
Concurrent direct de Miro, plus focalisé sur **la facilitation et le consulting**. Adopté massivement par les agences d'innovation (IDEO, McKinsey…).

### Différenciateur clé : Facilitation Superpowers

Mural a inventé une catégorie de fonctionnalités appelées *Facilitation Superpowers* :

- **Timer** : visible par tous, démarrable par le facilitateur
- **Mode privé** : les ajouts d'un participant ne sont visibles que par lui jusqu'au reveal
- **Summon** : « ramener » tous les participants à une zone (téléport caméra)
- **Voting** : sessions de vote intégrées avec budget
- **Outline** : table des matières du board pour naviguer entre zones thématiques

**Cette catégorie est exactement ce que BSE doit imiter.**

### Stack technique (déductions)
- Web app similaire à Miro
- Moins d'infos publiques sur leur backend
- Probablement Node + WebSocket

### Ce qu'on apprend de Mural

1. **La facilitation est une couche produit à part entière.** Pas un afterthought. Mural a structuré son offre autour.

2. **Les rôles utilisateurs sont importants.** Facilitator / Member / Visitor — différents droits.

3. **L'audit trail est demandé par les entreprises.** Voir qui a fait quoi quand.

4. **Le mode présentation est utilisé pour livrer des workshops à des clients.**

### Forces à imiter pour BSE
- ✅ Facilitation Superpowers complets
- ✅ Rôles utilisateurs explicites
- ✅ Outline / mini-map pour navigation

### Faiblesses à éviter pour BSE
- ❌ Performance et fluidité considérées inférieures à Miro
- ❌ Adoption hors consulting plus faible

## Tableau de comparaison Miro vs Mural

| Critère | Miro | Mural | Cible BSE |
|---|---|---|---|
| Catalogue templates | ✅✅✅ | ✅✅ | ✅ (10 au lancement) |
| Performance perçue | ⚠️ | ⚠️ | ✅✅ (avantage natif) |
| Facilitation features | ⚠️ | ✅✅✅ | ✅✅ |
| Intégrations | ✅✅✅ | ✅ | ⚠️ (post-v1) |
| Self-host | ❌ | ❌ | ✅✅✅ |
| Open-source | ❌ | ❌ | ✅✅✅ |
| Prix entrée | Free, $8/u | $9/u | Free (self-host) |
| Adoption enterprise | ✅✅✅ | ✅✅ | À construire |

## Synthèse pour BSE

> **Miro est la référence en richesse fonctionnelle. Mural est la référence en facilitation. BSE doit prendre le meilleur des deux, en y ajoutant ses propres axes différenciants : performance native, open-source, self-host.**

Les fonctionnalités à *absolument* livrer en v0.5 pour être perçu comme un vrai concurrent :
- ⭐ 10 templates pré-installés
- ⭐ Timer facilitateur
- ⭐ Mode privé
- ⭐ Voting intégré
- ⭐ Summon all
- ⭐ Outline / mini-map
- ⭐ Rôles user (owner/editor/viewer/facilitator)
- ⭐ Export PNG/PDF
