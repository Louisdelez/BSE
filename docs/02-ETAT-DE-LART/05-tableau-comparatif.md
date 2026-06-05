# 02.05 — Tableau comparatif

## Vue synthétique

| Critère | Figma/FigJam | Miro | Mural | Excalidraw | tldraw | **BSE** |
|---|---|---|---|---|---|---|
| **Type** | SaaS | SaaS | SaaS | OSS + SaaS | SDK + SaaS | **OSS + self-host** |
| **Licence app** | Propriétaire | Propriétaire | Propriétaire | MIT | Mixte | **Apache-2 / MIT** |
| **Front-end** | Web (PWA) | Web | Web | Web | Web (React) | **Desktop natif** |
| **Tech backend** | Rust + TS | Java/Kotlin | Node | Node (relay) | Cloudflare DO | **Rust + tokio** |
| **Sync** | LWW custom | Custom hybride | Custom | LWW + relay E2E | Custom records | **CRDT (yrs/Loro)** |
| **Persistance** | S3+DynamoDB+PG | MySQL+Cassandra | NDA | Firebase | KV+R2 | **Postgres + S3/MinIO** |
| **Self-host** | ❌ | ❌ | ❌ | ✅ | ⚠️ | **✅** |
| **E2E encryption** | ❌ | ❌ | ❌ | ✅ | ❌ | **✅ optionnel** |

## Features produit

| Feature | Figma | Miro | Mural | Excalidraw | tldraw | **BSE v1.0** |
|---|---|---|---|---|---|---|
| Canvas infini | ✅ | ✅ | ✅ | ✅ | ✅ | **✅** |
| Multi-projet | ✅ | ✅ | ✅ | ⚠️ | ⚠️ | **✅** |
| Dessin libre | ✅ FigJam | ✅ | ✅ | ✅ | ✅ | **✅** |
| Formes | ✅ | ✅ | ✅ | ✅ | ✅ | **✅** |
| Texte | ✅ | ✅ | ✅ | ✅ | ✅ | **✅** |
| Images | ✅ | ✅ | ✅ | ✅ | ✅ | **✅** |
| Post-its colorés | ✅ | ✅ | ✅ | ⚠️ | ⚠️ | **✅** |
| Mindmap | ⚠️ | ✅ | ⚠️ | ❌ | ⚠️ | **✅** |
| Connecteurs auto | ✅ | ✅ | ✅ | ✅ | ✅ | **✅** |
| Templates | ✅ | ✅✅✅ | ✅✅ | ❌ | ⚠️ | **✅ (10+)** |
| Voting | ✅ | ✅ | ✅✅ | ❌ | ❌ | **✅** |
| Timer facilitateur | ⚠️ | ✅ | ✅ | ❌ | ❌ | **✅** |
| Mode privé | ⚠️ | ⚠️ | ✅ | ❌ | ❌ | **✅** |
| Présence (curseurs) | ✅ | ✅ | ✅ | ✅ | ✅ | **✅** |
| Mode anonyme | ⚠️ | ⚠️ | ⚠️ | ❌ | ❌ | **✅** |
| IA co-pilot | ✅ | ✅ | ⚠️ | ❌ | ⚠️ | **post-v1** |
| Comments | ✅ | ✅ | ✅ | ⚠️ | ⚠️ | **v0.5** |
| Versioning / history | ✅ | ✅ | ✅ | ⚠️ | ⚠️ | **v1.0** |
| Mode présentation | ✅ | ✅ | ✅ | ❌ | ⚠️ | **v0.5** |
| Export PNG/PDF | ✅ | ✅ | ✅ | ✅ | ✅ | **✅** |

## Performance

| Métrique | Figma | Miro | Excalidraw | tldraw | **BSE cible** |
|---|---|---|---|---|---|
| FPS standard | 60 | 30-60 | 60 | 60 | **60-144** |
| Demarrage cold | ~2 s | ~3 s | ~1 s | ~1.5 s | **<500 ms** |
| Memory | 300+ MB | 400+ MB | 150 MB | 250 MB | **<100 MB** |
| Elements limit fluid | ~10K | ~2-3K | ~500 | ~5K | **>10K** |
| Latence sync LAN p95 | ~80 ms | ~150 ms | ~100 ms | ~100 ms | **<50 ms** |

## Modèle économique

| Outil | Modèle | Free tier | Entrée payante |
|---|---|---|---|
| Figma/FigJam | Freemium SaaS | 3 fichiers | $12/u/mois |
| Miro | Freemium SaaS | 3 boards | $8/u/mois |
| Mural | Freemium SaaS | 3 murals | $9/u/mois |
| Excalidraw | OSS + SaaS | OSS gratuit | $7/u/mois pour Excalidraw+ |
| tldraw | OSS SDK + SaaS | OSS personnel | Licence sur devis |
| **BSE** | OSS + cloud opt. | **Tout gratuit self-host** | TBD : hébergement managé |

## Sécurité / souveraineté

| Critère | Figma | Miro | Mural | Excalidraw | tldraw | **BSE** |
|---|---|---|---|---|---|---|
| SOC2 | ✅ | ✅ | ✅ | N/A | ⚠️ | **Visée v1.5** |
| HIPAA | ✅ enterprise | ⚠️ | ⚠️ | ❌ | ❌ | **Self-host = oui** |
| EU sovereignty | ❌ (US) | ⚠️ (EU plan) | ❌ | Si self-host | ❌ | **✅ par design** |
| RGPD | ✅ | ✅ | ✅ | ✅ | ✅ | **✅** |
| Données chez l'user | ❌ | ❌ | ❌ | ✅ self-host | ⚠️ | **✅ par design** |
| Audit code | ❌ | ❌ | ❌ | ✅ | partial | **✅** |

## Communauté / écosystème

| Outil | Stars GitHub | NPM (web) | Discord/forum |
|---|---|---|---|
| Figma | privé | N/A | community.figma.com (huge) |
| Miro | privé | N/A | community.miro.com |
| Mural | privé | N/A | mural.co/community |
| Excalidraw | ~80K | excalidraw | discord actif |
| tldraw | ~36K | tldraw | discord très actif |
| **BSE** | **0 → cible 5K** | N/A | **discord + GH discussions** |

## Verdict synthétique pour BSE

> BSE peut **gagner sur 4 axes simultanés** que personne ne couvre :
> 1. **Performance native** (60-144 FPS, démarrage rapide, peu de mémoire)
> 2. **Souveraineté** (self-host facile, OSS, E2E optionnel)
> 3. **Facilitation poussée** (à la Mural)
> 4. **Multi-projet riche** (à la Miro)

Aucun concurrent ne couvre 3 axes sur 4. C'est notre opportunité de marché.

## Référence détaillée par produit

- Figma → [01-figma-figjam.md](./01-figma-figjam.md)
- Miro & Mural → [02-miro-mural.md](./02-miro-mural.md)
- Excalidraw → [03-excalidraw.md](./03-excalidraw.md)
- tldraw → [04-tldraw.md](./04-tldraw.md)
- Positionnement BSE → [06-positionnement-bse.md](./06-positionnement-bse.md)
