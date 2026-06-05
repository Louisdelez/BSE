# 00.04 — Roadmap haut niveau

Cette roadmap est **macro**. Le détail par jalon se trouve dans [11-ROADMAP-EXECUTION/](../11-ROADMAP-EXECUTION/).

## Vue d'ensemble (24 mois)

```
   M0    M3    M6    M9   M12   M15   M18   M21   M24
   │     │     │     │     │     │     │     │     │
   │  MVP    v0.1   v0.5  v1.0       v1.1       v2.0
   │  ────►  ────►  ────► ─────►     ─────►    ──────►
   │  POC    Beta   Préco Stable    Polish     Plugins
   │  fermée publique
```

## M0 — Setup (semaines 1-2)
- Repo Git, CI Windows/macOS/Linux
- Squelette projet (`cargo new`, structure des crates)
- Choix gel de GUI framework et CRDT lib (cf. [04-STACK-TECHNIQUE/](../04-STACK-TECHNIQUE/))
- Hello world fenêtre wgpu

## M1-M3 — MVP fermé
**Objectif** : démontrer la faisabilité technique.

- Canvas infini avec pan/zoom fluide à 60 FPS
- Dessin libre (perfect-freehand)
- Formes basiques : rectangle, ellipse, ligne
- Sélection, déplacement, suppression
- Multi-user en LAN via WebSocket, 1 room
- Persistance disque locale (1 projet)
- **Pas encore** : auth, multi-projet, images, texte avancé

**Critère de sortie** : 2 personnes peuvent dessiner en même temps sur la même toile, l'une voit le curseur de l'autre, les deux voient les traits en temps réel.

## M3-M5 — v0.1 publique (alpha)
- Multi-projets côté client
- Auth basique (email + password local, ou OIDC)
- Serveur déployable en Docker
- Texte sur canvas
- Import image (drag & drop, taille raisonnable)
- Awareness complète (curseur + sélection + nom)
- Undo/redo local
- Export PNG du viewport

**Critère** : un early adopter peut déployer un serveur, créer un projet, inviter 3 amis.

## M5-M8 — v0.5 (beta)
- Mindmap (nœuds + connecteurs)
- Post-its colorés
- Templates intégrés (rétro, SCAMPER, six chapeaux…)
- Permissions par projet (lecture/édition/admin)
- Recherche dans un projet
- Mode présentation
- Tablette graphique (pression stylet)

## M8-M12 — v1.0 (stable)
- Polish UX exhaustif
- Performance : profilage et optimisation
- Stabilité (zéro crash en session 4 h)
- Packaging : installateurs Windows/macOS/Linux
- Doc utilisateur complète
- Site web
- Tests automatisés sur multi-user

**Critère** : un utilisateur lambda peut faire un workshop d'1 h sans rencontrer de bug bloquant.

## M12-M18 — v1.x (croissance)
- Mobile compagnon (lecture + édition basique sur iPad/Android tablet)
- IA assistant intégré (sparring partner)
- Améliorations communautaires
- Performance multi-room sur serveur (100+ rooms simultanées)
- Sauvegarde / versioning

## M18-M24 — v2.0 (plateformisation)
- Plugins ? (à valider — anti-vision peut-être)
- Composants réutilisables (library de templates communautaire)
- Export interactif (presentation share)
- Intégrations (Slack, Discord, Notion webhook…)

## Critères de re-priorisation

Cette roadmap peut être réordonnée selon ces signaux :

- Beaucoup de demande pour mobile → on monte
- Performance non atteinte sur GPU intégré → on freeze features et on optimise
- Concurrent open-source sort une feature majeure → étude d'opportunité
- Adoption explosive sur un cas d'usage particulier → on double dessus

## Hors scope explicite

Pour qu'on ne soit jamais tenté :

- ❌ Voice / vidéo intégrée (Discord/Teams le fait mieux)
- ❌ Code source dans le canvas (≠ IDE)
- ❌ Diagrammes UML stricts (≠ Lucidchart)
- ❌ Workflow / automation (≠ n8n / Zapier)
- ❌ Time tracking / billing
