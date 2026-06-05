# 11.02 — Jalons v0.1 → v1.0

> Le détail des releases entre MVP et v1.0 stable.

## v0.1 — Beta privée (M3 → M5, 2 mois post-MVP)

### Périmètre additionnel
- ✅ **Auth basique** : email + password local OU OIDC (1 provider testé : Google)
- ✅ **Multi-projet** : création, sélection, switch
- ✅ **Texte** sur canvas (yrs::Text édition collaborative)
- ✅ **Import image** drag & drop
- ✅ **Awareness complète** : curseur + sélection + nom + couleur
- ✅ **Undo/redo local**
- ✅ **Export PNG** du viewport
- ✅ **Serveur Docker** déployable

### Cibles de stabilité
- Aucun crash sur 1 h d'usage
- Resync robuste après partition réseau
- 10 peers simultanés sans dégradation

### Tests
- 5 beta-testers externes invités
- Bug tracker GitHub Issues actif
- Discord créé pour feedback

## v0.5 — Beta publique (M5 → M8, 3 mois)

### Périmètre additionnel
- ✅ **Mindmap** (nœuds + connecteurs ancrés)
- ✅ **Post-its colorés** avec voting basique
- ✅ **Templates** intégrés (10 templates v1.0)
- ✅ **Permissions** par projet (owner, editor, viewer)
- ✅ **Recherche** dans un projet (Ctrl+F)
- ✅ **Mode présentation** (follow camera)
- ✅ **Tablette graphique** (pression stylet)
- ✅ **Polices custom** (système ou import)
- ✅ **Export SVG** et PDF
- ✅ **Mini-map**
- ✅ **Comments** ancrés
- ✅ **Mode facilitateur** : timer, mode privé, summon

### Cibles
- 60 FPS sur 1000 éléments
- 60 FPS sur 5000 éléments avec LOD
- Latence sync LAN p95 <50 ms
- 50 peers max par room

### Évaluation Loro vs yrs
- Benchmarks réels sur cas BSE
- Décision finale : continuer yrs OU migrer Loro

### Tests
- 50+ utilisateurs en beta publique
- Premier audit pentest (light)

## v0.8 — Release candidate (M8 → M10, 2 mois)

### Périmètre
Pas de nouvelle feature majeure. **Stabilité et polish** :
- Performance profilée et optimisée
- Bugfixes en masse
- Documentation utilisateur complète
- Packagings testés (MSI, DMG, AppImage, .deb)
- Setup OIDC documenté pour 4+ providers
- Localisation FR + EN complètes

### Critères blocants
- Zero P0/P1 bug ouvert
- Performance cibles atteintes
- Doc complète
- Tests automatisés couverture >70%

## v1.0 — Stable (M10 → M12, 2 mois)

### Critères de release v1.0

- ✅ Pas de crash en session 4 h
- ✅ Performance 60 FPS sur GPU intégré moyen
- ✅ Démarrage <500 ms
- ✅ Memory <100 MB au repos, <300 MB en session
- ✅ Audit sécurité externe passé
- ✅ Auto-update fonctionnel
- ✅ Site web bse.app live
- ✅ Vidéos demo professionnelles
- ✅ Doc utilisateur complète
- ✅ Pricing/legal préparé (si Cloud futur)
- ✅ Packaging signed
  - Windows : signed MSI
  - macOS : signed + notarized DMG
  - Linux : AppImage + .deb + .rpm
- ✅ Disponibilité :
  - GitHub Releases
  - Homebrew
  - winget
  - Flathub

### Marketing v1.0
- HN post
- ProductHunt launch
- Vidéo YouTube technique
- Article blog explicatif
- Tweet thread

## v1.x — Croissance (M12 → M18)

### Features cibles
- IA assistant intégré (provider configurable)
- Mobile companion (iPad / Android tablet) — lecture + édition basique
- Rich text dans les blocs texte (bold, italic, color)
- Pixel eraser (vs object eraser)
- E2EE production-ready
- Plugins légers (v2 plus probable)
- Bug bounty officiel

### Métriques cibles 12 mois post-v1.0
- ⭐ 5 000+ stars GitHub
- 👥 100+ organisations en self-host actif
- 🌐 1 000+ MAU sur les self-hosts qui partagent les stats
- 🐛 <50 issues critiques ouvertes
- 🎓 Adoption universitaire (3-5 universités l'utilisent)

## v2.0 — Plateformisation (M18 → M24)

### Possibles
- **BSE Cloud** SaaS managé
- **Marketplace de templates** communautaire
- **API publique** pour intégrations
- **Plugins/extensions**
- **Live presentation share** (URL publique)
- **Mobile native** (iOS / Android complets)
- **Web client** (WASM compilé du moteur Rust)

## Gestion des incidents

### Patch release rapide
Pour un bug critique post-v1.0 :
- Hotfix branch à partir du tag
- Cherry-pick fix
- Tag v1.0.1
- Release dans 24-48 h
- Notification users via auto-update

### LTS (v2+ peut-être)
- Branches LTS supportées 18 mois
- Updates sécurité only

## Communication

### Release notes
Format standardisé :
```markdown
# BSE v1.2.0 — 2026-09-15

## ✨ Nouveautés
- ...

## 🐛 Bugfixes
- ...

## ⚡ Performance
- ...

## ⚠️ Breaking changes
- ...

## 🔒 Sécurité
- ...
```

### Cadence
- Patch (.X.Y.Z) : ~tous les 1-2 semaines
- Minor (.X.Y.0) : ~tous les 2 mois
- Major (X.0.0) : 1-2 par an max

## Liens

- MVP → [01-mvp.md](./01-mvp.md)
- Risques → [03-risques.md](./03-risques.md)
- Équipe → [04-equipe.md](./04-equipe.md)
