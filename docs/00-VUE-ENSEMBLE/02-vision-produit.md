# 00.02 — Vision produit

## Vision à 3 ans

> *« BSE est l'outil de référence open-source pour la créativité collaborative en équipe, choisi par les organisations qui veulent un canvas infini puissant, performant et qu'elles contrôlent. »*

## Valeurs produit (les arbitrages)

Quand un choix est ambigu, ces valeurs tranchent — par ordre de priorité.

### 1. La performance avant l'exhaustivité
Mieux vaut 10 features qui tournent à 144 FPS qu'une centaine qui rament. La fluidité est une *feature*. Tout dépassement du budget de frame (16 ms à 60 FPS) est un bug.

### 2. La collaboration avant le mono-utilisateur
BSE est conçu pour N utilisateurs. Les choix d'architecture (CRDT, transactions, présence) s'optimisent pour N≥2, même si N=1 fonctionne forcément.

### 3. La souveraineté avant la simplicité de déploiement cloud
Self-host doit toujours rester un chemin de premier ordre, pas un *afterthought*. Aucun feature ne dépend d'un service propriétaire externe non substituable.

### 4. L'open-source avant la monétisation
Le code core est sous licence permissive (Apache-2 ou MIT). Un éventuel modèle commercial vient *au-dessus* (hébergement, support), pas *à l'intérieur* (features propriétaires).

### 5. La simplicité avant la flexibilité
Pas de plugins en V1. Pas de scripting. Pas de DSL. Un outil simple et opinionné > un outil flexible mais flou.

## Les 3 « pari » du produit

### Pari 1 : Desktop natif gagne contre Electron
Tous les concurrents sont des web apps. On parie qu'une app Rust native + GPU sera *visiblement* plus fluide (zoom, pan, dessin pression) et que ça crée un *wow effect* différenciant.

### Pari 2 : Le canvas infini + multijoueur est plus utile que pixel-perfect
On parie qu'une équipe préfère « brainstormer » que « designer ». Le marché du whiteboarding collaboratif (Miro = $17.5 mds en 2022) le prouve.

### Pari 3 : Self-host devient un critère majeur
RGPD, Cloud Act, souveraineté tech UE, sensibilité défense / pharma / banque → on parie sur une demande croissante en outils auto-hébergeables.

## Anti-vision (ce qu'on refuse de devenir)

- ❌ Un outil bloated avec 200 features qu'on n'utilise jamais
- ❌ Un clone exact d'un concurrent
- ❌ Une plateforme web déguisée en app native (≠ Tauri + React si la perf en pâtit)
- ❌ Une app où la version « gratuite » est volontairement bridée
- ❌ Un projet qui requiert un compte cloud pour fonctionner localement

## Cycle de release visé

| Phase | Durée cible | Livrable |
|---|---|---|
| **MVP interne** | 3 mois | Canvas + dessin + multi-user en LAN, 1 projet |
| **v0.1 publique** | +2 mois | Multi-projets, auth, persistance serveur |
| **v0.5** | +3 mois | Images, mindmaps, templates |
| **v1.0** | +4 mois | Production-ready, packages OS, doc utilisateur |
| **v1.x** | +continu | Mobile companion, IA, plugins ? |

## Métriques de succès (12 mois post-v1.0)

- ⭐ 5 000+ stars GitHub
- 👥 100+ organisations en self-host
- 🐛 <50 issues critiques ouvertes
- 🎬 60 FPS soutenus sur GPU intégré moyen
- 🚀 Démarrage <500 ms cold
- 🌐 Latence p95 <150 ms en LAN, <500 ms WAN

## Décisions / changements

| Date | Décision | Raison |
|---|---|---|
| 2026-06-05 | Création du document | Vision initiale |
