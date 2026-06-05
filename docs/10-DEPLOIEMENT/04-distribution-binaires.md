# 10.04 — Distribution des binaires

> Comment l'utilisateur final récupère et installe BSE.

## Cibles de plateformes

| OS | Architecture | Format |
|---|---|---|
| Windows 10+ | x86_64 | `.msi` installer |
| Windows 11+ | aarch64 | `.msi` (v1.x) |
| macOS 12+ | Apple Silicon (arm64) | `.dmg` + `.app` |
| macOS 12+ | Intel (x86_64) | `.dmg` |
| Linux | x86_64 | `.deb`, `.rpm`, AppImage, Flatpak |
| Linux | arm64 | AppImage (v1.x) |
| Web | WASM | hébergé sur bse.app/web (v2) |

## Distribution

### v1.0
- **GitHub Releases** : binaires `.msi`, `.dmg`, `.deb`, `.rpm`, AppImage
- **Homebrew** (macOS) : `brew install bse`
- **winget** (Windows) : `winget install bse`
- **Flathub** (Linux) : application Flatpak
- **Snap Store** (Linux) : `snap install bse` (optionnel)

### v1.x
- **Microsoft Store** : si la conversion MSIX est faisable
- **Mac App Store** : si la sandbox permet (probable contraintes)

## Build pipeline

### Outils
- **cargo-dist** : orchestre les builds cross-platform via GitHub Actions
- Génère automatiquement les release artifacts

### Workflow GitHub Actions
```yaml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        target:
          - x86_64-pc-windows-msvc
          - aarch64-apple-darwin
          - x86_64-apple-darwin
          - x86_64-unknown-linux-gnu
        include:
          - target: x86_64-pc-windows-msvc
            os: windows-latest
          - target: aarch64-apple-darwin
            os: macos-latest
          # ...
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release --target ${{ matrix.target }}
      - run: ./scripts/package.sh ${{ matrix.target }}
      - uses: actions/upload-artifact@v4
        with:
          path: dist/
```

## Code signing

### Windows
- Authenticode signing avec EV cert
- ~300-500 €/an pour le cert
- Sinon SmartScreen warning au lancement (UX horrible)

### macOS
- Apple Developer certificate (99 $/an)
- Notarization required pour pas être bloqué par Gatekeeper
- `codesign` + `xcrun notarytool submit`

### Linux
- Pas de signing imposé
- GPG signature des artifacts (cosign ou GPG classique)

## Auto-update (opt-in)

Lib : `self_update` ou solution maison.

### Flow
1. App vérifie périodiquement (1×/jour)
2. Notifie l'utilisateur si update dispo : « v1.2.0 disponible. Mettre à jour ? »
3. User accepte → download du nouveau binaire
4. Restart automatique

### Politique
- **Patch versions (x.y.Z)** : auto-update encouragé
- **Minor versions (x.Y.0)** : notification + manuel
- **Major versions (X.0.0)** : info bandeau, choix utilisateur

### Sécurité auto-update
- Vérification de la signature
- HTTPS pour le download
- Public key pinning dans le binaire

## Taille des binaires

Cible :
- Windows : <30 MB compressé MSI
- macOS : <30 MB compressé DMG
- Linux : <25 MB AppImage

### Stratégies de réduction
- LTO activé en release
- Strip symbols
- `opt-level = "z"` (size optim, mais perf -10%)
- UPX compression (pas recommandé sur macOS)
- Polices bundled : 5 MB (essentielles)

## Versioning

### SemVer
- `MAJOR.MINOR.PATCH`
- Major : breaking changes
- Minor : nouvelles features compatibles
- Patch : bugfixes

### Tags Git
- `v1.0.0`, `v1.1.0`, `v1.1.1`...
- Annotated tags signés GPG

### Channels (v1.x)
- **Stable** : releases officielles
- **Beta** : releases candidates
- **Nightly** : builds master

## Update notification

Au démarrage, l'app peut afficher :
- "Une mise à jour est disponible : v1.2.0"
- Bouton « Voir les changements » → release notes
- Bouton « Mettre à jour maintenant »
- Bouton « Plus tard »
- Setting : « Mises à jour automatiques »

## Téléchargements anonymes

- Compteur de downloads via GitHub API
- Pas de telemetrie phone-home (sauf opt-in)
- Pas de tracking d'installation

## Telemetrie (opt-in seulement)

À la première ouverture, on demande :
> *« Voulez-vous partager des statistiques d'usage anonymes pour aider à améliorer BSE ? »*

Si oui :
- Métriques anonymes : OS, version BSE, fréquence d'usage des features
- Crash reports (avec `sentry-rust` ou maison)
- Aucune donnée projet
- Désactivable à tout moment

Si non : aucune donnée transmise.

## Reproducible builds (v1.x)

Objectif : que n'importe qui puisse rebuilder le même binaire :
- Toolchain pinné
- Deps reproductible via Cargo.lock
- Dates fixes via env var
- Comparable byte-à-byte

## Distribution non-officielle

Anyone peut distribuer (licence Apache-2 permet) :
- Un fork avec modifs custom
- Doit changer le nom (anti-confusion)

Notre rôle : maintenir le repo officiel et les builds officiels.

## CI/CD complète

```
Push commit ────► CI (test, lint, fmt)
                       │
                       ▼
Push tag v* ────► Release CI :
                  ├── Build Windows
                  ├── Build macOS arm64
                  ├── Build macOS x86_64
                  ├── Build Linux x86_64
                  ├── Sign + notarize
                  ├── Generate checksums
                  ├── Generate SBOM
                  ├── Create GitHub release
                  └── Update brew, winget, flathub
```

## Tests post-release

- Installer le binaire sur OS clean dans VM
- Lancer, vérifier qu'il démarre
- Aucune erreur dans les logs
- Self-update fonctionne sur version N-1

## Liens

- Stack technique → [../04-STACK-TECHNIQUE/](../04-STACK-TECHNIQUE/)
- Roadmap → [../11-ROADMAP-EXECUTION/](../11-ROADMAP-EXECUTION/)
