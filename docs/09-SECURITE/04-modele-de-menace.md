# 09.04 — Modèle de menace

> Threat model BSE. Qui attaque quoi, comment, et comment on s'en défend.

## Approche STRIDE

On évalue les menaces selon le framework STRIDE :
- **S**poofing (usurpation)
- **T**ampering (altération)
- **R**epudiation (déni)
- **I**nformation disclosure (divulgation)
- **D**enial of Service
- **E**levation of privilege

## Acteurs

| Acteur | Description |
|---|---|
| **Attaquant externe** | Internet, pas d'accès initial |
| **Attaquant en LAN** | Sur le même réseau qu'un user (café, bureau) |
| **User authentifié malveillant** | Compte légitime mais hostile |
| **Admin serveur curieux** | A accès au serveur mais pas censé lire les données |
| **Insider corrompu** | Dev BSE / mainteneur compromis |
| **État (sub poena)** | Demande légale d'accès |

## Assets à protéger

| Asset | Sensibilité |
|---|---|
| Contenu des projets | ⭐⭐⭐⭐⭐ (peut être IP critique) |
| Credentials user | ⭐⭐⭐⭐⭐ |
| Métadonnées (qui a accès à quoi) | ⭐⭐⭐ |
| Identité des collaborateurs | ⭐⭐⭐ |
| Logs serveur | ⭐⭐ |

## Menaces et mitigations

### M1 — Eavesdropping réseau
**Threat** : un attaquant capte le trafic.
**Vector** : MITM sur Wi-Fi public, BGP hijack, mauvais cert.
**Impact** : lecture contenu projets, vol JWT.

**Mitigation** :
- TLS 1.3 obligatoire
- HSTS
- Cert pinning (v1.x)
- Pour confidentialité absolue : mode E2EE

### M2 — Brute force credentials
**Threat** : attaque dictionnaire sur `/login`.
**Vector** : Internet.
**Impact** : prise de compte.

**Mitigation** :
- Argon2id (hash lent)
- Rate limit sur l'IP + sur l'utilisateur
- Limite : 5 essais / 10 min puis backoff
- Notification email après échec multiples (v1.x)

### M3 — Vol de JWT
**Threat** : attaquant obtient un JWT (via XSS, malware, etc.).
**Impact** : usurpation de session.

**Mitigation** :
- Access token court (15 min)
- Refresh rotation (détecte vol)
- Logout révoque tous les refresh
- Pas de cookies (donc XSS-immune pour BSE app native)
- Storage du refresh en OS keyring

### M4 — Account takeover via OIDC
**Threat** : si l'IdP est compromis, l'attaquant prend tous les comptes.
**Impact** : massif.

**Mitigation** :
- Pas notre problème direct (responsabilité IdP)
- MFA recommandée
- Possibilité de désactiver OIDC en faveur de password+MFA

### M5 — User malveillant
**Threat** : un utilisateur authentifié essaie d'accéder à un projet où il n'a pas droit.
**Impact** : confidentialité.

**Mitigation** :
- RBAC strict côté serveur (cf [02-permissions-rbac.md](./02-permissions-rbac.md))
- Toutes les vérifications dans des transactions
- Audit logs
- Tests automatisés des permissions

### M6 — Spam / abuse
**Threat** : un user crée 10 000 projets vides ou upload du contenu malveillant.
**Impact** : disponibilité, coûts.

**Mitigation** :
- Quotas par user (projets, storage)
- Rate limit sur les créations
- Reporting + bannissement
- Modération communautaire (v1.x)

### M7 — Sub-resource integrity
**Threat** : un asset uploaded est en réalité un malware en JPG.
**Impact** : si BSE rend ces assets, potentiel exploit.

**Mitigation** :
- Validation magic bytes
- BSE rend les images via libs Rust mémoire-sûres (`image`)
- Pas d'exécution
- SVG sanitization (anti-XSS pour browsers — pas notre cas mais bonne pratique)
- Antivirus scan optionnel (ClamAV)

### M8 — Denial of Service serveur
**Threat** : floods de connexions WS, ops massives.
**Impact** : indisponibilité.

**Mitigation** :
- Rate limit par IP, par user, par room
- Max connexions par room (50)
- Max ops/s par peer (100)
- Reverse proxy avec protection DDoS (Cloudflare optionnel)

### M9 — CRDT bomb
**Threat** : un attaquant envoie une op CRDT crafted pour faire crasher / consommer beaucoup.
**Impact** : DoS, peut-être RCE selon implem.

**Mitigation** :
- Validation taille des ops
- Sandbox de l'application des ops (panic catching)
- Fuzz testing du parser
- Update régulier des libs CRDT

### M10 — SQL injection
**Threat** : injection dans les inputs.
**Impact** : data leak, RCE potentielle.

**Mitigation** :
- `sqlx` avec paramètres typés (compile-time check)
- Jamais de format string SQL
- ORM-style queries

### M11 — Server compromise
**Threat** : un attaquant prend le serveur.
**Impact** : massif — accès à tout.

**Mitigation** :
- Hardening OS standard (cf cybersecurity guides)
- Updates auto de l'OS
- Pas de service exposé inutile
- Backups encrypted offsite
- En mode E2EE : confidentialité contenus préservée même si serveur compromis

### M12 — Insider threat (admin curieux)
**Threat** : un admin du serveur lit les projets.
**Impact** : confidentialité.

**Mitigation** :
- Mode E2EE : admin ne peut pas lire
- Audit logs des accès admin
- Principe du least privilege
- Logging des connexions admin

### M13 — Supply chain attack
**Threat** : une dépendance Rust compromise (typosquatting, etc.).
**Impact** : code malveillant dans BSE.

**Mitigation** :
- `cargo-deny` pour audit licences et vulnérabilités
- `cargo-audit` régulier
- `Cargo.lock` committé
- Reproducible builds (v1.x)
- Signed releases (cosign, v1.x)

### M14 — Phishing
**Threat** : faux site BSE qui vole les credentials.
**Impact** : ATO.

**Mitigation** :
- Pas notre problème principal (app native, OIDC chez l'IdP officiel)
- Education users
- Domain officiel clair

### M15 — Stolen device
**Threat** : laptop volé avec session active BSE.
**Impact** : accès aux projets.

**Mitigation** :
- Pas notre problème direct (OS doit chiffrer le disque)
- Logout possible par email (v1.x)
- Sessions listées dans Settings → révocable

## Menaces hors scope explicitement

- 🚫 Quantum-resistant crypto : v2+
- 🚫 Side-channel attacks : non
- 🚫 Hardware exploits : non
- 🚫 Anti-rev (anti-reverse engineering) : on est open-source

## Modèle de menace par déploiement

### Self-host en LAN (PME)
- Threats principaux : M1 (eavesdropping LAN), M5, M11
- Mitigation minimum : TLS + auth standard

### Self-host derrière VPN (entreprise sensible)
- Threats principaux : M11, M12
- Mitigation : mode E2EE recommandé

### Public Cloud BSE (SaaS futur)
- Threats principaux : M1, M3, M8, M11, M13
- Mitigation : tout + monitoring + bug bounty

## Réponse à incident

Si une vuln est trouvée :
1. Email `security@bse.app` (private disclosure)
2. Patch en priorité
3. CVE attribuée
4. Release dans 1-7 jours selon criticité
5. Communication transparente

## Bug bounty (futur)

Quand BSE atteint une certaine traction :
- Programme via HackerOne / Intigriti
- Rewards selon criticité
- Hall of fame

## Audit externe

Visé pour v1.0 :
- Pentest par cabinet spécialisé
- Audit crypto si E2EE production-ready
- Audit code des composants critiques (auth, CRDT)

## Liens

- Auth → [01-authentification.md](./01-authentification.md)
- Permissions → [02-permissions-rbac.md](./02-permissions-rbac.md)
- Chiffrement → [03-chiffrement.md](./03-chiffrement.md)
