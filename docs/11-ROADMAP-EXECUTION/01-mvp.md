# 11.01 — MVP (3 mois)

> Le minimum viable pour valider l'idée techniquement et démontrer la faisabilité.

## Objectif du MVP

> *« Deux personnes peuvent dessiner en temps réel sur la même toile. L'une voit le curseur de l'autre. Les modifications sont fluides. »*

Pas de polish UI, pas d'auth, pas de cloud. Juste **la preuve technique**.

## Périmètre

### IN (à livrer)
- ✅ Fenêtre desktop fonctionnelle (Windows + macOS + Linux)
- ✅ Canvas infini avec pan/zoom à 60 FPS
- ✅ Outil **Pen** (dessin libre avec perfect-freehand)
- ✅ Outil **Rectangle** et **Ellipse**
- ✅ Outil **Select** (clic, drag, delete)
- ✅ Serveur Rust qui héberge **1 room**
- ✅ Multi-user via WebSocket en LAN (2-5 peers)
- ✅ Curseurs distants visibles
- ✅ Persistance disque locale (1 projet)
- ✅ CRDT basique (yrs) pour le sync

### OUT (pas dans le MVP)
- ❌ Auth, multi-projet
- ❌ Images, texte, post-its, mindmap
- ❌ Templates
- ❌ Voting, facilitation
- ❌ Permissions, rôles
- ❌ E2EE
- ❌ Auto-update, packaging
- ❌ Documentation utilisateur
- ❌ Tout ce qui touche au cloud

## Découpage en sprints (12 semaines)

### Sprint 1-2 (semaines 1-2) — Setup & Hello World
- Repo Git, structure workspace Cargo
- CI Windows/macOS/Linux
- Fenêtre vide avec eframe + wgpu
- "Hello world" via egui dans la fenêtre

### Sprint 3 (semaine 3) — Pan & Zoom
- Caméra implementée
- Pan via espace + drag
- Zoom via molette centré sur le curseur
- Background blanc (sans grille)

### Sprint 4 (semaine 4) — Premier outil : Rectangle
- Modèle `Element` et `Scene`
- Outil Select et Rectangle dans la toolbar egui
- Rendu rectangle via wgpu pipeline SDF
- Création, sélection, déplacement, suppression

### Sprint 5 (semaine 5) — Ellipse + Style basique
- Ellipse identique au rectangle
- Properties panel basique (couleur fill, stroke)
- Color picker minimal

### Sprint 6 (semaine 6) — Pen (dessin libre)
- Capture des points (souris)
- Implem perfect-freehand en Rust
- Triangulation et rendu
- Stress test : 100 strokes sur scène

### Sprint 7 (semaine 7) — Spatial index + culling
- Quadtree implementé
- Viewport culling activé
- Test : scène 1000 éléments → 60 FPS

### Sprint 8 (semaine 8) — Serveur basique
- Axum + tokio server
- Route `/ws/rooms/default`
- WS upgrade
- Echo simple pour tester

### Sprint 9 (semaine 9) — CRDT integration
- yrs intégré au modèle Scene
- Sérialisation ops binaires
- Apply remote update fonctionnel
- 1 peer → server → broadcast → 2nd peer

### Sprint 10 (semaine 10) — Curseurs distants
- Awareness avec yrs::sync::Awareness
- Cursor messages throttle 30 Hz
- Rendu des curseurs distants
- Test : 2 instances voient le curseur l'une de l'autre

### Sprint 11 (semaine 11) — Persistance locale
- SQLite locale
- Sauvegarde du snapshot à intervalle régulier
- Reload du projet au démarrage
- Resync après reconnect

### Sprint 12 (semaine 12) — Polish, tests, démo
- Bugfixes critiques
- Smoke tests
- Vidéo démo
- Documentation MVP

## Critères de sortie

Pour valider le MVP :

1. ✅ Démarrage app en <2 s
2. ✅ Pan/zoom à 60 FPS sur scène vide
3. ✅ Pan/zoom à 60 FPS sur scène 500 éléments
4. ✅ Trace pen visible à <50 ms de latence input → écran
5. ✅ 2 instances en LAN voient les modifs en <100 ms
6. ✅ Reconnect après coupure réseau resync correct
7. ✅ Aucun crash sur 30 min d'usage continu
8. ✅ Sauvegarde + reload identique (round-trip)

## Démo cible

À la fin des 12 semaines, **vidéo démo de 2 min** :
- Lancement de 2 instances BSE sur 2 ordis du LAN
- Création d'un dessin par chacun
- Édition simultanée du même rectangle
- Coupure réseau → reconnect → tout est cohérent
- Performance 60 FPS visible (compteur affiché)

## Ressources

### Profils
- **1 dev Rust senior** (full-time) : architecture + serveur
- **1 dev Rust mid** (full-time) : client + canvas
- **1 dev Rust junior** (mid-time) : UI + outils

Total : ~2.5 ETP × 3 mois = **7.5 mois.dev**.

### Matériel
- 3 laptops dev (Win/Mac/Linux)
- VPS test pour le serveur (5 €/mois)

### Coût estimé
- Salaires : selon location, ~30-60 K€
- Infra : <100 €
- Cert : pas encore (post-MVP)

## Risques MVP

| Risque | Probabilité | Impact | Mitigation |
|---|---|---|---|
| Perfect-freehand Rust difficile | Moyenne | Moyen | Plan B : algo simplifié |
| CRDT yrs API plus complexe que prévu | Faible | Moyen | Ressources docs/exemples |
| Performance pas atteinte | Moyenne | Élevé | Profiling early, ajuster |
| Multi-platform issues | Forte | Moyen | CI multi-OS dès semaine 1 |

## Après le MVP

→ Cf [02-jalons-v0-v1.md](./02-jalons-v0-v1.md) pour les jalons v0.1 → v1.0.

## Liens

- Architecture → [../03-ARCHITECTURE/](../03-ARCHITECTURE/)
- Stack → [../04-STACK-TECHNIQUE/](../04-STACK-TECHNIQUE/)
- Risques détaillés → [03-risques.md](./03-risques.md)
