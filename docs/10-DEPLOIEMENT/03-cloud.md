# 10.03 — BSE Cloud (futur)

> Le SaaS BSE managé. v2.0+ — pas une priorité v1.

## Vision

> *« Tu veux BSE sans rien gérer ? Signup en 30 secondes, c'est prêt. »*

## Positionnement

- Pour les **utilisateurs individuels** et **PME** qui veulent BSE sans ops
- Pour **gros teams** qui acceptent SaaS et veulent vendor-managed
- **Pas** une alternative au self-host (qui reste premier-class)

## Modèle économique

À définir, hypothèses :

### Free tier
- 3 projets actifs
- 5 collaborateurs par projet
- 100 MB storage
- Watermark export discret

### Plan « Team » (~10 €/user/mois)
- Projets illimités
- 50 collaborateurs par projet
- 5 GB storage par user
- Pas de watermark
- Support email

### Plan « Enterprise »
- Sur devis
- SSO SAML/OIDC
- SLA, support prioritaire
- Audit logs étendus
- Régions dédiées

## Architecture

```
                      ┌──────────────────┐
                      │ Cloudflare /     │ ← Edge, DDoS protection
                      │ Front load bal.  │
                      └────────┬─────────┘
                               │
                ┌──────────────┼──────────────┐
                ▼              ▼              ▼
          Region EU       Region US     Region APAC
                │              │              │
                ▼              ▼              ▼
        ┌──────────────┐ ┌──────────┐ ┌──────────┐
        │ BSE servers  │ │ BSE      │ │ BSE      │
        │  (replicas)  │ │ servers  │ │ servers  │
        └──────┬───────┘ └────┬─────┘ └────┬─────┘
               │              │            │
               ▼              ▼            ▼
        ┌──────────────┐ ┌──────────┐ ┌──────────┐
        │ Postgres HA  │ │ Postgres │ │ Postgres │
        │ + R2 storage │ │ + R2     │ │ + R2     │
        └──────────────┘ └──────────┘ └──────────┘
```

## Choix d'infra (cible)

| Composant | Choix | Raison |
|---|---|---|
| Compute | VPS bare metal (Hetzner, OVH) ou K8s managed | Performance/cost ratio |
| Postgres | Managed Postgres (Neon, Supabase, RDS) | Délégation des ops |
| Object storage | Cloudflare R2 | Pas d'egress fees |
| CDN | Cloudflare | Référence |
| TLS | Let's Encrypt ou Cloudflare | Standard |
| Monitoring | Grafana Cloud free tier | Bon TCO |
| Logs | Better Stack ou Loki self-host | Coût raisonnable |
| Email | Postmark, Mailgun | Délivrabilité |
| Payments | Stripe | Standard |
| Auth | Self-hosted (OIDC vers Google/etc.) ou Clerk | Trade-off control / speed |

## Régions

- **EU** : Frankfurt ou Paris (RGPD-friendly)
- **US** : Virginia (low latency East coast)
- **APAC** : Singapour (v2+)

Chaque utilisateur a un projet "homed" sur une région.

## Pricing infrastructure

Pour 1000 utilisateurs actifs, ~100 projets actifs simultanés :

| Service | Coût mensuel |
|---|---|
| Compute (Hetzner 8 vCPU, 16 GB × 2) | 40 € |
| Postgres managé (Neon) | 50 € |
| R2 storage 500 GB | 8 € |
| R2 egress | 0 € (gratuit !) |
| Cloudflare front | 0 € (free tier) |
| Postmark email | 15 € |
| Stripe fees | ~3% du revenu |
| Monitoring | 0-30 € |
| **Total infra** | **~120-150 €/mois** |

Soit ~0.15 €/MAU. Très scalable.

## Compliance

- **RGPD** : data EU pour users EU
- **SOC 2 Type II** : visée v2.5
- **HIPAA** : potentiellement v2+ pour US healthcare

## Migration self-host → Cloud (et inverse)

L'utilisateur peut migrer dans les deux sens :
- Export complet d'un projet en JSON
- Import dans l'autre instance
- Documenté dans l'aide

## Quand lancer BSE Cloud ?

Pas avant **v1.0 stable self-host** :
- 12-18 mois post-MVP
- Quand le self-host est polished, documenté, validé par early adopters
- Pas de focus split prématuré

## Tâches préparatoires

Avant de lancer :
1. Audit sécurité externe
2. Penetration test
3. Politique de confidentialité claire
4. TOS rédigés (avocat)
5. Billing & subscription management
6. Support tier 1 (email)
7. Status page (statuspage.io ou self-host)

## Risques cloud spécifiques

| Risque | Mitigation |
|---|---|
| Outage régional | Multi-AZ deploy |
| Outage Cloudflare | Fallback DNS direct |
| Data loss | Backups multi-region |
| Cost explosion | Alertes budget |
| Abuse | Rate limit, audit, ban |

## Pour l'instant

> BSE v1.0 = self-host only. BSE Cloud est une **option future**, pas une obligation. La doc actuelle se concentre sur faire un excellent produit self-host.

## Liens

- Architectures cibles → [01-architectures-cibles.md](./01-architectures-cibles.md)
- Self-host → [02-self-hosted.md](./02-self-hosted.md)
- Roadmap → [../11-ROADMAP-EXECUTION/](../11-ROADMAP-EXECUTION/)
