# 08.03 — Multi-curseurs et présence

> Comment voir les autres et être vu.

## Vue d'ensemble

La présence est un signal **continu** que le brainstorm est vivant. Voir un curseur bouger côté Alice motive Bob à contribuer. C'est psychologique.

## Le curseur distant

### Affichage
Chaque peer connecté est représenté par :

```
        ↗
        ↑
       Alice
```

- Flèche dans la couleur du peer
- Label « Alice » juste à côté
- Légère ombre pour la lisibilité sur tout fond
- Animation d'interpolation entre les positions reçues

### Couleur
- Assignée par le serveur à la connexion
- Palette de 8 couleurs WCAG AA distinctes
- Personnalisable dans les settings utilisateur (override)

### Estompage
- Si le peer est inactif depuis 5 s : curseur s'estompe (opacity 50%)
- Si 30 s : disparaît (jusqu'à un nouveau move)
- Évite la pollution visuelle

## Indicateur d'outil

L'icône du curseur **change selon l'outil actif** du peer :
- ↗ Select
- ✏ Pen (avec un petit dot quand il dessine)
- ▢ Rectangle / shapes
- T Text
- 📋 Postit

Permet de comprendre *ce que* fait le peer, pas juste *où* il est.

## Sélections distantes

Quand Bob sélectionne un élément, Alice voit :

```
       ▄▄▄▄▄▄▄▄▄▄
       █ Element │
       █ X       │  ← bordure bleue (couleur Bob)
       ▀▀▀▀▀▀▀▀▀▀
          ↳ Bob
```

- Bordure colorée (couleur Bob)
- Label « Bob » dans un coin
- Pas d'animation (statique)

## Liste des participants

Top bar (en haut à droite) :

```
┌─────────────────────────────────────┐
│ ... 👤 Alice (vous) 👤 Bob 👤 Charlie │
└─────────────────────────────────────┘
```

- Avatars des peers connectés
- Hover → tooltip avec nom complet
- Click sur un avatar → ouvre menu :
  - « Aller voir » (téléport caméra à son curseur)
  - « Suivre » (follow camera)
  - « Mute notifications » (pour ce peer)

## Mini-map

Optionnelle (toggle), en bas à droite :

```
┌────────────────┐
│      ●         │ ← Alice
│   ●            │ ← Bob (couleur Bob)
│           ●    │ ← Charlie (couleur Charlie)
│   ┌──┐         │ ← viewport courant
│   │  │         │
│   └──┘         │
└────────────────┘
```

- Vue d'ensemble de la scène
- Point coloré par peer (position de son curseur)
- Rectangle de viewport courant
- Click → téléport caméra

## Follow user

Activable depuis le menu d'avatar :
- La caméra reprend celle du peer en continu
- Animation lerp pour fluidité
- Au moindre input local → désactivation auto

## Statut « en train de taper »

Quand un peer édite un texte (post-it, text, mindmap) :
- Près du curseur : « ⌨ en train de taper »
- Dans la liste des participants : petit point vert clignotant
- Discrete, pas envahissant

## Notification de jointure / sortie

Discrète, en bas :

```
┌────────────────────────────┐
│ ● Bob a rejoint le projet  │  ← s'estompe en 3 s
└────────────────────────────┘
```

Pas de son par défaut (option setting).

## Mode présentation

Quand un peer active le mode présentation (facilitateur) :
- Sa caméra **est imposée** aux autres
- Notification : « Charlie présente. Suivez sa caméra. »
- Bouton « Quitter le mode follow » bien visible
- Curseur du présentateur en gros, clairement visible

## Mode anonyme

Si projet en mode anonyme :
- Les noms sont remplacés par « Anonyme #N » (N stable session)
- Couleurs toujours attribuées
- Aucune révélation visible

## Performance

### Throttle
- Curseurs envoyés à 30 Hz max
- Si très peu d'activité (>1 s sans bouger) : pas d'envoi du tout

### Interpolation
Pour la fluidité visuelle :
- Position reçue à 30 Hz mais affichage rendu à 60+ FPS
- Lerp entre la position précédente et la nouvelle
- Pas de saccades

### Au-delà de N peers
À 20+ peers actifs simultanément, les curseurs deviennent du bruit. Solutions :
- **Mode heatmap** : remplace les curseurs individuels par une heatmap d'activité
- **Filter** : option pour cacher les curseurs hors viewport
- **Pulse mode** : ne pulse que les peers actifs récemment

## Réactions

Bouton « Réaction » (v1.x) : émojis flottants comme dans Zoom/Teams.

```
        🎉
       ↗  ← Alice envoie un confetti
      Alice
```

Léger, ludique, non persistant.

## Accessibilité

- Couleurs respectent WCAG AA contrast
- Mode haute contraste : labels plus prononcés
- Screen reader : annonce les arrivées/sorties (configurable)
- Reduced motion : pas d'animation de curseur (jump direct)

## Tests

- 2 peers : voient leurs curseurs réciproques
- 10 peers : pas de lag, pas de bug d'affichage
- Reconnect : curseur réapparait
- Disconnect brutal : curseur disparait après 5 s

## Liens

- Awareness technique → [../05-COLLABORATION-TEMPS-REEL/04-presence-cursors.md](../05-COLLABORATION-TEMPS-REEL/04-presence-cursors.md)
- Principes UX → [01-principes-design.md](./01-principes-design.md)
- Facilitation → [../01-BRAINSTORMING-RECHERCHE/04-facilitation.md](../01-BRAINSTORMING-RECHERCHE/04-facilitation.md)
