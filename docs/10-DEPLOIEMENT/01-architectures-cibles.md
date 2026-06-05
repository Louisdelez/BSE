# 10.01 — Architectures de déploiement cibles

> Les 4 topologies que BSE doit supporter.

## A. Standalone (zéro serveur)

```
   Desktop ──► SQLite local
              + filesystem images
```

**Use case** : utilisateur unique, pas de collaboration.

**Setup** : install + run. Aucune config.

**Performance** : maximale, pas de latence réseau.

**Limites** : pas de multi-user.

## B. Self-host LAN (PME, équipe)

```
   Client 1 ─┐
   Client 2 ─┼──► Serveur BSE ──► Postgres + MinIO
   Client 3 ─┘     (1 machine)
```

**Use case** : équipe de 5-50 personnes, infra contrôlée.

**Setup** : `docker compose up`. ~10 min.

**Composants** :
- Serveur BSE Rust
- Postgres
- MinIO (ou filesystem direct)
- Reverse proxy (Caddy avec TLS auto)

**Performance** : excellente, LAN latency <10 ms.

## C. Self-host cloud (entreprise)

```
   Clients ──► Reverse proxy ──► N × Serveur BSE ─► PG HA + S3
                                                 ─► Redis (futur)
```

**Use case** : entreprise avec besoins de HA, multi-région.

**Setup** : K8s ou bare VPS + ops modérés.

**Composants** :
- 2+ instances serveur BSE (load balanced)
- Postgres en mode HA (primary + replica)
- S3 (AWS / R2 / B2)
- Reverse proxy (Caddy, Traefik, Nginx)
- Monitoring (Grafana, Prometheus)
- Backups automatisés

**Performance** : très bonne. <100 ms p95 WAN.

## D. SaaS BSE Cloud (futur)

```
              ┌──────────────┐
   Clients ──►│ Multi-region │──► Region EU
              │   gateway    │──► Region US
              └──────────────┘──► Region APAC
```

**Use case** : utilisateurs qui ne veulent pas self-host.

**Setup** : signup, c'est tout.

**Composants** : équivalent à C mais multi-région + managed.

**Performance** : variable selon distance. Latence routée à la région du projet.

**Pricing futur** : à définir (modèle SaaS standard).

## Choix de topologie selon scale

| Users | Projets actifs simultanés | Topologie recommandée |
|---|---|---|
| 1 | 1 | A (standalone) |
| 2-10 | 1-3 | B (self-host LAN) |
| 10-50 | 5-10 | B ou C |
| 50-500 | 10-50 | C (self-host cloud) |
| 500+ | 50+ | C avec sharding |
| Variable | Variable | D (SaaS, futur) |

## Critères architecturaux par topologie

| Critère | A | B | C | D |
|---|---|---|---|---|
| Setup complexity | ★ | ★★ | ★★★★ | ★ (no-op) |
| Performance | ★★★★★ | ★★★★ | ★★★★ | ★★★ |
| Souveraineté | ★★★★★ | ★★★★★ | ★★★★★ | ★★★ |
| Cost | gratuit | low | medium | subscription |
| Scaling | 1 user | 50 users | 1000+ | unlimited |

## Migration entre topologies

L'utilisateur peut **passer de A à B** facilement :
- Export JSON du projet local
- Import sur le serveur BSE

Et de B/C à D :
- Export JSON
- Import sur le SaaS

Pas de lock-in.

## Hybrides

### Standalone avec sync optionnelle
Un user qui veut le standalone + une sauvegarde cloud :
- Configurer une "remote backup" → sync vers S3
- Pas de collaboration temps réel, juste backup

### Self-host avec users guests via SaaS cloud (v2)
Un projet self-host peut inviter un guest externe via le SaaS BSE Cloud. À voir.

## Liens

- Self-host détaillé → [02-self-hosted.md](./02-self-hosted.md)
- Cloud → [03-cloud.md](./03-cloud.md)
- Distribution → [04-distribution-binaires.md](./04-distribution-binaires.md)
- Architecture technique → [../03-ARCHITECTURE/](../03-ARCHITECTURE/)
