# 01.03 — Outils existants : panorama

> Vue d'ensemble des outils que les équipes utilisent aujourd'hui. Pour chaque outil : positionnement, forces, faiblesses, modèle économique. Comparaison technique détaillée dans [../02-ETAT-DE-LART/](../02-ETAT-DE-LART/).

## Catégorie 1 — Whiteboarding pur

### Miro
- **Positionnement** : leader enterprise du whiteboarding collaboratif.
- **Force** : écosystème de templates massif (centaines), intégrations (Jira, Slack, Confluence…), entreprise-ready (SSO, audit logs).
- **Faiblesse** : web-only, propriétaire, cher en équipe, données chez Miro.
- **Modèle** : freemium (3 boards gratuits) → SaaS par siège.
- **Note BSE** : référence en termes de richesse de templates et facilitation.

### Mural
- **Positionnement** : concurrent direct de Miro, axé *facilitation*.
- **Force** : *Facilitation Superpowers* — timers, mode privé, « summon participants » (rappeler tout le monde au même endroit). Très utilisé en consulting.
- **Faiblesse** : équivalent Miro sur les autres axes ; performance perçue parfois inférieure.
- **Modèle** : SaaS par siège.
- **Note BSE** : étudier le mode facilitation comme inspiration.

### FigJam (Figma)
- **Positionnement** : whiteboarding ludique intégré à l'écosystème Figma.
- **Force** : intégration Figma native, design soigné, plus rapide / léger que Miro.
- **Faiblesse** : moins de templates pro, peu autonome sans Figma.
- **Modèle** : freemium → SaaS, lié au plan Figma.
- **Note BSE** : référence UX *fun & playful*.

## Catégorie 2 — Whiteboarding open-source

### Excalidraw
- **Positionnement** : whiteboard minimaliste, esthétique « croquis main levée ».
- **Force** : open-source, simple, E2E encryption, embeddable.
- **Faiblesse** : pas de multi-projet natif, features limitées (pas de mindmap structuré), collab via serveur centralisé.
- **Modèle** : OSS gratuit + Excalidraw+ (SaaS premium).
- **Note BSE** : inspiration sur le style visuel et la simplicité.

### tldraw
- **Positionnement** : SDK whiteboard infinite-canvas pour développeurs.
- **Force** : engine très performant, multiplayer pro (tldraw sync, Cloudflare Durable Objects), API riche.
- **Faiblesse** : SDK plus que produit fini ; licence du SDK pro (non MIT) ; web only.
- **Modèle** : freemium → licence commerciale du SDK.
- **Note BSE** : inspiration technique majeure pour le sync engine.

### Drawpile
- **Positionnement** : dessin pixel collaboratif (raster).
- **Force** : open-source, multi-user, dessin pression réel.
- **Faiblesse** : raster only (pas de zoom infini propre), UX vieillissante.
- **Note BSE** : peu pertinent (raster, pas vectoriel).

### Excalidraw vs tldraw vs BSE

| Critère | Excalidraw | tldraw | BSE (cible) |
|---|---|---|---|
| Open-source code app | ✅ | ⚠️ (SDK partiel) | ✅ |
| Desktop natif | ❌ | ❌ | ✅ |
| Multi-projet natif | ❌ | ⚠️ | ✅ |
| Performance GPU native | ❌ | ⚠️ | ✅ |
| Auto-hébergeable simple | ✅ | ✅ | ✅ |
| Features riches (mindmap, …) | ❌ | ⚠️ | ✅ |
| Licence permissive | ✅ MIT | ⚠️ mixte | ✅ Apache-2 |

## Catégorie 3 — Plateformes hybrides

### Microsoft Whiteboard
- Intégré Teams/Microsoft 365. Adoption forcée en entreprise MS.
- Faible en features avancées, dépendance écosystème.

### Google Jamboard
- **Discontinué fin 2024**. À noter pour les équipes qui cherchent une alternative.

### Apple Freeform
- App native macOS/iOS, jolie, collaborative via iCloud.
- Verrouillé écosystème Apple, pas multi-plateforme.

## Catégorie 4 — Mindmap dédié

### XMind, MindMeister, MindNode
- Outils spécialisés mindmap.
- Force : édition de mindmap structurée, exports propres.
- Faiblesse : pas de canvas libre, pas vraiment collaboratif temps réel (ou en mode dégradé).

### Coggle
- Mindmap collab simple, web only.

## Catégorie 5 — Outils périphériques

### Notion, Coda
- Documents collaboratifs. Pas vraiment canvas — plutôt « page » structurée.
- Forte adoption mais paradigme différent.

### Whimsical
- Mid-fidelity (wireframes + mindmap + flowchart).
- Belle UX, ciblée producteurs.

### Conceptboard, Stormboard, Lucidspark
- Concurrents Miro de second rang.

## Synthèse : les segments de marché

```
┌─────────────────────┬──────────────────────────────┐
│   Enterprise SaaS   │  Miro, Mural, Lucidspark     │
├─────────────────────┼──────────────────────────────┤
│   Design-led        │  FigJam, Figma               │
├─────────────────────┼──────────────────────────────┤
│   Open-source       │  Excalidraw, tldraw          │
├─────────────────────┼──────────────────────────────┤
│   Desktop natif     │  ❌ (créneau de BSE)         │
├─────────────────────┼──────────────────────────────┤
│   Mindmap dédié     │  XMind, MindMeister          │
└─────────────────────┴──────────────────────────────┘
```

## L'opportunité BSE

Le segment **« desktop natif open-source riche »** est **vide**. C'est la thèse de marché de BSE.

Voir [../02-ETAT-DE-LART/06-positionnement-bse.md](../02-ETAT-DE-LART/06-positionnement-bse.md) pour le détail.
