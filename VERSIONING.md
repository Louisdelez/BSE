# Système de versionning de BSE

> Comment ce repo est structuré en termes de versions, branches et tags.

## 🎯 Principe général

BSE utilise un système de **paliers numérotés** (`v001`, `v002`, `v003`…) alignés sur la roadmap projet. Chaque palier représente **un livrable cohérent et démontrable**.

Quand le projet atteindra la maturité, les paliers internes seront complétés par des **releases publiques SemVer** (`v0.1.0`, `v0.5.0`, `v1.0.0`).

---

## 🌳 Structure des branches

### `main`
- **Toujours stable**
- Reflète le dernier palier terminé
- Protégée : pas de push direct, uniquement via PR (à activer post-v005)
- Chaque commit sur `main` correspond à un palier `v0XX` taggué

### `feat/0XX-nom-court`
- Branches de travail pour un palier en cours
- Format : `feat/<numero>-<nom-kebab-case>`
- Exemples :
  - `feat/002-cargo-workspace`
  - `feat/003-canvas-pan-zoom`
  - `feat/004-first-tool-rectangle`
- Mergées dans `main` quand le palier est complet (squash merge ou rebase)
- Supprimées après merge

### `fix/0XX-description`
- Branches de correctif urgent
- Mergées dans `main` rapidement
- Génèrent un tag `v0XX-fix` ou bumpent un patch

### `docs/0XX-sujet`
- Branches dédiées documentation seule (pas de code)
- Mergées comme les autres

---

## 🏷️ Tags

### Tags de palier (`v0XX`)
- Format : `v` + numéro à **3 chiffres**, padded
- Exemples : `v001`, `v002`, `v015`, `v100`
- **Annotated tags** (pas lightweight) avec :
  - Message décrivant le contenu du palier
  - Auteur, date
- 1 tag = 1 palier de la roadmap

### Tags de release publique (futurs)
- Format SemVer : `v0.1.0`, `v0.5.0`, `v1.0.0`
- Apparaîtront quand le projet sera distribuable
- Co-existent avec les tags `v0XX`
- Exemple : le tag `v0.1.0` pourrait correspondre au palier `v015` (release beta privée)

---

## 📋 Plan des paliers (extrait roadmap)

| Palier | Contenu | Statut |
|---|---|---|
| **v001** | Documentation initiale complète (70 fichiers, 13 sections) | ✅ |
| **v002** | Cargo workspace + crates squelettes + CI Windows/macOS/Linux | À venir |
| **v003** | Fenêtre desktop + canvas vide pan/zoom 60 FPS | À venir |
| **v004** | Premier outil : Rectangle (SDF rendering) | À venir |
| **v005** | Outils : Ellipse + Select + suppression | À venir |
| **v006** | Outil Pen (perfect-freehand) | À venir |
| **v007** | Spatial index (Quadtree) + culling | À venir |
| **v008** | Serveur Axum + tokio + route WS basique | À venir |
| **v009** | Intégration CRDT (yrs) + sync 2 clients | À venir |
| **v010** | Awareness + curseurs distants | À venir |
| **v011** | Persistance locale SQLite + reload | À venir |
| **v012** | Polish + démo MVP fermé | À venir |
| **v013-v020** | Auth, multi-projet, texte, images → **v0.1.0 beta privée** | À venir |
| **v021-v035** | Mindmap, post-its, templates, facilitation → **v0.5.0 beta publique** | À venir |
| **v036-v050** | Polish, packaging, doc → **v1.0.0 stable** | À venir |

Cf. roadmap détaillée : [docs/11-ROADMAP-EXECUTION/](./docs/11-ROADMAP-EXECUTION/).

---

## ✍️ Convention de messages de commit

BSE utilise **Conventional Commits** combiné avec le numéro de palier.

### Format

```
<type>(<scope>): <description courte>

<corps optionnel détaillé>

<footer optionnel : refs, breaking>
```

### Types autorisés

| Type | Description |
|---|---|
| `feat` | Nouvelle fonctionnalité |
| `fix` | Correction de bug |
| `docs` | Documentation seule |
| `style` | Formatage, pas de change comportement |
| `refactor` | Refactor sans bug ni feature |
| `perf` | Amélioration de perf |
| `test` | Ajout/modification de tests |
| `build` | Système de build, deps |
| `ci` | Pipelines CI/CD |
| `chore` | Tâches de maintenance |
| `revert` | Annulation d'un commit |

### Scope (optionnel)

Indique le module/domaine touché :
- `client`, `server`, `canvas`, `crdt`, `render`, `sync`, `auth`, `docs`, `ci`, `deps`

### Exemples

```
docs(v001): initial complete documentation (70 files, 13 sections)
feat(canvas): implement pan and zoom with camera transformation
feat(server): add WebSocket upgrade endpoint with axum
fix(crdt): resolve interleaving anomaly on concurrent text edit
perf(render): batch shapes draw calls for 5x throughput
chore(deps): bump yrs to 0.21
```

### Breaking changes

Si breaking change, ajouter `!` après le type :
```
feat(crdt)!: migrate to Loro from yrs

BREAKING CHANGE: existing v002-v008 snapshots are not compatible.
Migration guide in docs/MIGRATION-v015.md
```

---

## 🔄 Workflow type d'un palier

```bash
# 1. Créer la branche de travail
git checkout main
git pull
git checkout -b feat/002-cargo-workspace

# 2. Coder le palier, commits réguliers
git add .
git commit -m "feat(build): init Cargo workspace with 10 crates"
git commit -m "feat(client): hello world wgpu + winit + egui"
git commit -m "ci: matrix Windows/macOS/Linux"

# 3. Push de la branche
git push -u origin feat/002-cargo-workspace

# 4. Pull Request vers main (review obligatoire post-v005)

# 5. Merge dans main (squash recommandé pour grouper les commits du palier)

# 6. Sur main : tag annoté
git checkout main
git pull
git tag -a v002 -m "v002 — MVP setup

- Cargo workspace structuré en 10 crates
- Hello world wgpu fenêtre
- CI Windows / macOS / Linux opérationnelle

Cf. CHANGELOG.md pour détail."
git push origin v002
```

---

## 📝 CHANGELOG

Le [CHANGELOG.md](./CHANGELOG.md) est mis à jour à chaque palier — pas à chaque commit.

Format inspiré de [Keep a Changelog](https://keepachangelog.com).

---

## 🔒 Politique de tag

- **Tags signés** (GPG ou SSH) à partir de `v005`
- **Tags annotés** dès `v001`
- **Pas de force push** sur les tags
- **Pas de réécriture** des tags publiés

---

## 🚀 Releases GitHub

À partir de `v002`, chaque tag `v0XX` génère une **GitHub Release** avec :
- Notes de version (extrait CHANGELOG)
- Lien vers la doc à jour
- Binaires (à partir du palier où ils sont produits, ~v013+)

---

## ❓ FAQ

### Pourquoi numérotation 3 chiffres ?
Pour permettre **999 paliers** avant collision. Pad à 3 facilite le tri lexicographique (`v002` < `v015` < `v100`).

### Quand passer à SemVer ?
Quand le projet aura une **API stable** (interne et CLI). Probablement à partir du palier ~v013 (v0.1.0 = première release publique).

### Que faire si je veux skipper un palier ?
Possible mais documenté dans le CHANGELOG. Ex : `v007` non utilisé → `v008` directement, avec note explicative.

### Comment référencer un palier dans une issue/PR ?
Utiliser le tag : `Closes #42` + mention `[v002]` dans le titre.

---

> *Cette convention est éditable. Si un palier impose une exception, la documenter en haut du PR avec rationale.*
