# 11.04 — Équipe

> Profils nécessaires, organisation, recrutement.

## Profils par phase

### MVP (3 mois)
- **2.5 ETP Rust** :
  - 1 senior (architecture, serveur, CRDT)
  - 1 mid (client, canvas, rendu)
  - 0.5 junior (outils, UI, tests)

### v0.1 → v0.5 (5 mois post-MVP)
- **3-4 ETP** :
  - 1 senior Rust (lead tech)
  - 2 dev Rust (client, serveur)
  - 1 UX/UI designer (mi-temps minimum)
  - 0.5 dev a11y / i18n

### v0.5 → v1.0 (4 mois)
- **4-5 ETP** :
  - 1 senior Rust (lead)
  - 2-3 dev Rust
  - 1 UX/UI designer (full time)
  - 1 QA / tech writer (mi-temps)
  - Consultants : audit sécurité, perf

### v1.0+ (continu)
- **5-8 ETP** :
  - Maintainers core
  - Croissance équipe selon adoption
  - Contributors open-source externes (croissance organique)

## Compétences nécessaires

### Indispensables (au moins 1 personne par compétence)
- ⭐ Rust avancé (async, traits, lifetimes)
- ⭐ Architecture client/serveur distribuée
- ⭐ Programmation graphique 2D (wgpu/Vulkan/Metal)
- ⭐ CRDTs (théorie et pratique)
- ⭐ UX/UI design produit
- ⭐ Test automatisé (unit, integration, property-based)

### Importantes (utiles dans l'équipe)
- WebSocket, networking
- PostgreSQL avancé
- DevOps (Docker, K8s, CI/CD)
- Sécurité applicative (auth, OIDC, threat modeling)
- Cross-platform Windows/macOS/Linux
- Accessibilité a11y
- i18n

### Bonus
- C/C++ pour debug des libs natives
- GPU compute / shaders
- ML / IA (pour les features futures)
- Marketing / community building

## Organisation

### Structure plate jusqu'à 5 personnes
- Tech lead unique
- Décisions par consensus
- Daily async (5 min)
- Hebdo sync (30 min)

### Au-delà
- Équipe Frontend (client)
- Équipe Backend (serveur, infra)
- Équipe Produit (UX, doc, support)

## Recrutement

### Sources
- **GitHub** : profils Rust actifs sur des projets open-source
- **Discord Rust** : très bonne communauté
- **HN « Who's hiring »**
- **Rust meetups locaux**
- **LinkedIn** (moins efficace pour Rust mais OK pour senior)

### Process d'embauche
1. Application + lettre + portfolio (open-source idéalement)
2. Take-home : petite tâche Rust (4 h max), payée
3. Entretien tech : pair-programming
4. Entretien équipe / culture fit
5. Reference checks
6. Offre

Process en 2-3 semaines max. Pas de bullshit type 6 rounds.

## Culture d'équipe

### Valeurs
- **Honnêteté technique** : on dit ce qui marche pas
- **Slow is smooth, smooth is fast** : pas de rush, code propre
- **Open by default** : on documente, on partage
- **Async-friendly** : pas de meetings inutiles
- **Apprentissage continu** : 10% du temps sur le learning

### Communication
- **GitHub** pour le code et les issues
- **Discord** pour le chat équipe + communauté
- **Notion ou GitHub Wiki** pour la doc interne
- **Loom** pour les explications complexes async

### Rythme
- 35-40 h/semaine
- Pas de weekend par défaut
- Vacances respectées
- Sprint 2 semaines (jamais 4+)

## Open-source community

### Pré-v1.0
- Focus core team
- Issues bienvenues mais pas de PR externes attendues
- Discord ouvert

### Post-v1.0
- Bienvenue aux contributors
- Templates issues / PR
- Code of Conduct (Contributor Covenant)
- Mentoring des nouveaux contributors
- Reconnaissance (contributors list, mentions release notes)

### Maintainership progressif
- Contributors actifs → committers (review/merge)
- Committers actifs → maintainers (vote stratégique)
- Évite la dépendance à 1 personne

## Bus factor

### Risque "1 personne disparait"
Critère sain : tout le code doit pouvoir être maintenu par au moins 2 personnes.

### Mitigation
- **Documentation interne** des décisions
- **Pair-programming** régulier
- **Reviews croisées** obligatoires
- **Onboarding doc** maintenue

## Compensation (si structure professionnelle)

### Founders / early
- Equity + salaire selon stage
- Pas de stock-options bidon

### Employés
- Salaire de marché Rust ($90-180K selon location et seniority)
- Equity selon ancienneté
- Stock-options vesting standard

### Contributors externes
- Pas de salaire (sauf retainer maintainers)
- Sponsorisations possibles (GitHub Sponsors)
- Reconnaissance (mentions, lettres de recommandation)

## Modèle de financement (hypothèses)

### Option A — Bootstrapped
- Founders auto-financés
- Croissance organique
- Service / support paid quand l'usage le justifie

### Option B — Subventions
- NLnet (EU)
- Open Tech Fund
- Mozilla Open Source Support
- Sovereign Tech Fund (Germany)

### Option C — VC (déconseillé)
- Open-source + VC = tension permanente
- Sauf si stratégie SaaS BSE Cloud clarifiée

### Option D — Foundation (long terme)
- Type Cloud Native Computing Foundation
- Donations multiples entreprises

## Métriques de santé équipe

- Time-to-merge PR : <48 h en moyenne
- Vélocité stable (pas de boom/bust)
- Burnout signals : auto-rapportés en hebdo sync
- Satisfaction interne : sondage trimestriel

## Mentors externes

Profils à recruter en advisory :
- Expert CRDT (Martin Kleppmann, Joseph Gentle, ou similaires)
- Expert UX produit (qqun de Figma alumni)
- Expert open-source business (Sentry, Plausible, Sourcegraph)
- Expert Rust ecosystem (cargo team alumni)

## Liens

- MVP scope → [01-mvp.md](./01-mvp.md)
- Roadmap → [02-jalons-v0-v1.md](./02-jalons-v0-v1.md)
- Risques → [03-risques.md](./03-risques.md)
