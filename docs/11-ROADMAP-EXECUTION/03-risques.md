# 11.03 — Risques

> Liste structurée des risques BSE et plans de mitigation.

## Classification

- **Probabilité** : Faible / Moyenne / Forte
- **Impact** : Mineur / Moyen / Critique
- **Score** = combinaison qualitative

## Risques techniques

### R1 — Perf canvas insuffisante sur hardware bas de gamme
- **Probabilité** : Moyenne
- **Impact** : Critique
- **Description** : Si BSE rame sur GPU intégré moyen, notre USP « performance native » s'effondre.
- **Mitigation** :
  - Profiling early (dès MVP)
  - Tests de benchmark sur 3 niveaux de hardware
  - LOD agressif
  - Plan B : option « low-perf mode » avec rendu simplifié

### R2 — yrs limites avec movable tree (mindmap)
- **Probabilité** : Moyenne
- **Impact** : Moyen
- **Description** : Cycles potentiels en reparenting concurrent.
- **Mitigation** :
  - Détection de cycle côté client (workaround acceptable)
  - Evaluation de migration à Loro en v0.5
  - Tests de cas limites en stress concurrent

### R3 — eframe/egui limites pour le canvas custom
- **Probabilité** : Faible
- **Impact** : Moyen
- **Description** : Intégration wgpu paint callback peut être fragile.
- **Mitigation** :
  - PoC dès le sprint 4 du MVP
  - Plan B : passer en winit pur + wgpu, sans eframe (égui en overlay)

### R4 — Cross-platform issues (macOS notarization)
- **Probabilité** : Forte (probable bugs)
- **Impact** : Moyen
- **Description** : macOS code signing + notarization est notoirement difficile.
- **Mitigation** :
  - CI test dès le début pour produire des builds non-signed
  - Investissement spécifique sur le signing v0.8
  - Doc + scripts dédiés

### R5 — Memory leaks ou drift
- **Probabilité** : Moyenne
- **Impact** : Critique
- **Description** : Une app desktop qui consomme 10 GB après 4 h tue le projet.
- **Mitigation** :
  - Soak tests automatisés
  - Profiling régulier avec dhat
  - Strict Rust ownership minimise les fuites

### R6 — Compatibilité Wayland / X11 sur Linux
- **Probabilité** : Forte
- **Impact** : Moyen
- **Description** : Comportements différents sous X11 vs Wayland (notamment pour les fenêtres flottantes, clipboard).
- **Mitigation** :
  - winit gère ça en grande partie
  - CI Linux multi-DE (Ubuntu Wayland, Fedora X11)
  - Bug reports actifs early

## Risques projet/produit

### R7 — Sous-estimation du périmètre v1.0
- **Probabilité** : Forte
- **Impact** : Critique
- **Description** : Le scope v1.0 est ambitieux. Risque de glissement de 6+ mois.
- **Mitigation** :
  - Découpage strict en jalons
  - Critères de sortie par release
  - Pas d'ajout de feature post-jalon
  - Communication transparente du retard si advient

### R8 — Pas d'adoption initiale (pas de traction)
- **Probabilité** : Forte
- **Impact** : Moyen
- **Description** : Beaucoup de projets open-source disparaissent par manque d'attention.
- **Mitigation** :
  - Build in public (devlogs, vidéos)
  - Cibler une niche claire (cf [../02-ETAT-DE-LART/06-positionnement-bse.md](../02-ETAT-DE-LART/06-positionnement-bse.md))
  - Lancement Hacker News, ProductHunt
  - Première démo très visuelle (vidéo de qualité)

### R9 — Concurrence : tldraw devient open-source de la même catégorie
- **Probabilité** : Faible
- **Impact** : Moyen
- **Description** : tldraw pourrait pivoter en MIT et faire de l'app native.
- **Mitigation** :
  - Notre différentiation reste : Rust natif desktop, souveraineté
  - Réagir avec qualité, pas avec features

### R10 — Concurrence : Figma sort FigJam Desktop natif
- **Probabilité** : Très faible
- **Impact** : Moyen
- **Description** : Adobe pourrait pivoter.
- **Mitigation** :
  - Différentiation prix (free vs SaaS)
  - Différentiation OSS

### R11 — Burn-out équipe
- **Probabilité** : Forte si pas de gestion
- **Impact** : Critique
- **Description** : Projet ambitieux solo ou small team → risque réel.
- **Mitigation** :
  - Pas de rush sprint long
  - Vacances respectées
  - Communication des limites
  - Open-source = contributors externes (à terme)

### R12 — Manque de bandwidth pour la documentation
- **Probabilité** : Forte
- **Impact** : Moyen
- **Description** : La doc utilisateur n'est jamais terminée et est négligée.
- **Mitigation** :
  - Doc dans le sprint, pas après
  - Templates de doc standard
  - Considérer un tech writer freelance pour v1.0

## Risques sécurité

### R13 — Vuln critique découverte post-release
- **Probabilité** : Forte (à terme)
- **Impact** : Critique
- **Description** : Une CVE majeure sur un composant central (auth, CRDT).
- **Mitigation** :
  - Process de disclosure documenté
  - Capacité de patch + release en 48 h
  - Tests de régression

### R14 — Compromise de la pipeline de release
- **Probabilité** : Faible
- **Impact** : Critique
- **Description** : Un attaquant signe un binaire malveillant.
- **Mitigation** :
  - 2FA sur tous les comptes GitHub release
  - Signing keys hardware (YubiKey)
  - Reproducible builds
  - Cosign signatures

## Risques économiques (si projet professionnalisé)

### R15 — Pas de revenue model viable
- **Probabilité** : Moyenne
- **Impact** : Critique pour la sustainability long terme
- **Description** : Si BSE veut payer des mainteneurs.
- **Mitigation** :
  - Modèle BSE Cloud à valider (post-v1.0)
  - Sponsors GitHub
  - Subventions (NLnet, Open Tech Fund)
  - Support entreprise sur self-host

### R16 — Coût d'infra cloud explosé
- **Probabilité** : Moyenne (si BSE Cloud lancé)
- **Impact** : Moyen
- **Description** : Croissance utilisateurs > revenu.
- **Mitigation** :
  - Free tier limité
  - Alertes budget
  - Réversibilité (close cloud si pas viable)

## Risques légaux

### R17 — Patent troll sur CRDT ou canvas
- **Probabilité** : Faible
- **Impact** : Moyen
- **Description** : Brevets dormants sur des techniques utilisées.
- **Mitigation** :
  - Pas notre problème principal au début
  - OIN membership pour la défense (si entreprise)

### R18 — Licence d'une dépendance change
- **Probabilité** : Faible
- **Impact** : Moyen
- **Description** : Si une crate clé passe en GPL.
- **Mitigation** :
  - `cargo-deny` check licences à chaque PR
  - Surveillance manuelle des deps majeures
  - Fork si nécessaire

## Synthèse

### Risques top-3 à surveiller en priorité
1. **R1** — Performance (mitigation : profiling early)
2. **R7** — Périmètre (mitigation : jalons stricts)
3. **R11** — Burn-out (mitigation : rythme soutenable)

### Risques à accepter
- R10 (concurrence Adobe) — improbable
- R17 (patent troll) — coût de mitigation > probabilité × impact

## Process de revue

- Revue trimestrielle de cette liste
- Mises à jour basées sur réalité du terrain
- Nouveaux risques détectés → ajout

## Liens

- Roadmap → [01-mvp.md](./01-mvp.md)
- Équipe → [04-equipe.md](./04-equipe.md)
