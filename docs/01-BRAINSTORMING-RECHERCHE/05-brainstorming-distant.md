# 01.05 — Brainstorming distant / asynchrone

> Pourquoi le distanciel **n'est pas qu'un palliatif** — il peut être *meilleur* que le présentiel. Et ce que BSE doit livrer pour bien le supporter.

## La grande surprise du remote brainstorming

Contre-intuitif mais documenté : **le brainstorming distant via outil collaboratif tend à produire plus d'idées** qu'en présentiel verbal. Les raisons :

1. **Réduit le production blocking** : tout le monde « parle » en même temps via post-its numériques.
2. **Réduit l'évaluation apprehension** : la distance physique baisse la pression sociale.
3. **Donne du temps de réflexion** aux profils introvertis ou non-natifs de la langue.
4. **Permet le mode anonyme** réel.
5. **Inclut plusieurs fuseaux horaires**.

## Sync vs Async vs Hybride

### Synchrone distant (visio + canvas)
Tout le monde est connecté au même moment, sur le même canvas, avec une visio en parallèle.

- ✅ Énergie de groupe préservée
- ✅ Rebond rapide
- ✅ Possibilité de facilitation forte
- ❌ Fuseaux horaires problématiques
- ❌ Saturation de canaux (visio + canvas + chat)

**Mode dominant pour les équipes co-localisées en télétravail.**

### Asynchrone pur
Pas de réunion. Une question est posée, chacun contribue quand il peut, sur une fenêtre de 24-72 h.

- ✅ Réfléchit en profondeur
- ✅ Fuseaux gérés
- ✅ Inclut les non-natifs et introvertis
- ❌ Perd l'énergie de groupe
- ❌ Manque de stimulation immédiate
- ❌ Demande discipline collective

**Idéal pour des équipes globales, ou pré-session synchrone (pré-trempage).**

### Hybride (recommandé)
Le pattern qui marche le mieux :

```
J-7  ─► Envoi de la problématique + canvas template
J-3  ─► Fenêtre async : chacun ajoute 5-10 idées sur le canvas
J 0  ─► Session synchrone 60 min de convergence
        - Tour de présentation des idées async
        - Affinity mapping
        - Vote
        - Actions
J+1  ─► Snapshot exporté envoyé à tous
```

**BSE doit nativement supporter ce flow hybride.**

## Spécificités UX du distant

### 1. Identification visuelle forte
Chaque utilisateur a :
- Une **couleur dédiée** auto-assignée (palette accessible WCAG AA)
- Un **avatar / initiales** sur son curseur
- Un **nom** affiché au survol de ses contributions

### 2. Présence riche
- Liste des participants connectés en haut à droite
- Indication « X est en train d'écrire »
- Mini-map indiquant où sont les autres caméras
- Bouton **« Aller voir »** pour téléporter la caméra à la position d'un peer

### 3. Communication asynchrone intégrée
- **Commentaires** ancrés sur n'importe quel élément
- Notifications quand quelqu'un répond
- Mode « ⚠️ Cette idée demande feedback »

### 4. Tolérance à la déconnexion
- L'app continue à fonctionner offline
- Reconnexion automatique avec resync CRDT
- Indicateur clair de l'état de connexion

### 5. Gestion fuseaux
- Timestamps relatifs (« il y a 2 h ») mais avec hover absolu
- Horaire de session affiché dans le fuseau local de chaque peer

## Anti-patterns du remote brainstorming

- ❌ **Mode présentiel transposé** : un facilitateur parle, les autres écoutent. Tue le brainstorm.
- ❌ **Trop de monde en visio** : >6 personnes en visio, c'est un meeting, pas un brainstorm.
- ❌ **Pas de phase silencieuse** : encore plus critique en remote (le facilitateur a moins de signaux).
- ❌ **Outils éparpillés** : canvas + chat + visio + doc + tableur = chaos. Concentrer sur 1-2 outils.
- ❌ **Pas de mode anonyme** : le distanciel doit *amplifier* la sécurité psychologique, pas la rejouer.

## Patterns spécifiques à BSE

### Pattern : « Async-first »
Le facilitateur crée le projet, configure le template, partage le lien.
Les participants reçoivent une notification avec délai (« avant vendredi »).
Le facilitateur reçoit un récap des contributions avant la session synchrone.

### Pattern : « Vote silencieux »
Pendant une session sync :
- Facilitateur active le **mode vote anonyme**
- Chaque participant a 3 dots à distribuer
- Les votes ne sont visibles **qu'après reveal**
- Évite l'effet cascade

### Pattern : « Zone privée »
Un participant peut activer une zone personnelle pendant la divergence.
Sa contribution n'est révélée aux autres qu'après la phase.

## Recommandations de durée pour le distant

| Type de session | Durée recommandée distant | Notes |
|---|---|---|
| Brainstorm court | 45 min sync | Plus court qu'en présentiel |
| Brainstorm long | 60-75 min sync | Avec pause 5 min à 30 min |
| Workshop complet | 2× 60 min séparées | Ne pas faire 3 h sync remote |
| Rétro d'équipe | 60 min | Standard |
| Hybride | 30 min async + 45 min sync | Le sweet spot |

## Inspirations / outils

- **Miro** : référence sur la facilitation distante
- **Loom** : vidéos courtes pour expliquer une idée complexe en async
- **Notion / Linear** : pré-canvas pour cadrer
- **GitHub Discussions / Linear** : suivi des actions post-session

## Synthèse

> Le brainstorming distant n'est pas une dégradation du présentiel. C'est une **modalité différente** avec ses propres règles. BSE doit être pensé **async-first**, le synchrone étant un cas particulier où plusieurs peers sont en ligne en même temps.
