# 01.06 — IA comme partenaire de brainstorming

> En 2026, l'IA est devenue un **acteur d'idéation**. BSE peut nativement intégrer un assistant IA — ou rester agnostique. Ce document explore le pourquoi/comment.

## Le constat 2025-2026

Statistiques significatives (sources : Jenova, McKinsey, marketing surveys) :
- **45 % des marketeurs** utilisent une IA pour brainstormer du contenu en 2026.
- Les équipes utilisant des outils d'IA pour l'idéation rapportent **+27 % de vitesse de livraison projet**.
- Les modèles spécialisés brainstorming (Claude, GPT, Gemini avec prompts dédiés) produisent rapidement des dizaines d'idées de qualité moyenne.

## Comment l'IA s'intègre dans le processus

### Mode 1 — Amorce de session
> *« Donne-moi 30 idées pour résoudre [problème] »*

L'IA pose les premières 20-30 idées qui servent de point de départ. L'équipe rebondit, écrème, complète.

- ✅ Bat la page blanche
- ✅ Donne immédiatement de la matière à critiquer
- ❌ Risque d'ancrer la réflexion sur les patterns IA

### Mode 2 — Diversification active
> *« Donne-moi 10 idées qu'aucun marketeur ne proposerait »*  
> *« Imagine ce qu'un enfant de 7 ans répondrait »*  
> *« Donne-moi des idées si on était en 1950 »*

L'IA force des angles inattendus. Cassage de biais.

### Mode 3 — Critique constructive
> *« Joue l'avocat du diable pour cette idée »*  
> *« Quels seraient les 5 obstacles principaux ? »*

L'IA challenge les idées, fait émerger les hypothèses cachées.

### Mode 4 — Synthèse et regroupement
> *« Voici 80 post-its. Regroupe-les en clusters thématiques. »*

L'IA fait l'affinity mapping en quelques secondes. L'humain valide.

### Mode 5 — Expansion d'idée
> *« Cette idée : "marketplace inter-équipes". Décline-la en 5 variantes plus précises. »*

L'IA approfondit une idée vague, propose des variantes.

## Le bon pattern : humain → IA → humain

> **L'IA est un *amplificateur* d'équipe, pas un remplaçant.** Le bon flow est *humain → IA → humain*. L'IA seule produit du « moyen statistique » utile pour la quantité mais l'originalité reste humaine.

```
[Génération humaine] → [Diversification IA] → [Tri humain]
                        ↑
                Évite le « centre statistique »
```

## Risques et anti-patterns

### Anti-pattern 1 — « Demander l'idée à l'IA »
Si on attend que l'IA produise *la* bonne idée, on aura toujours une idée moyenne qui ressemble à ce qui existe déjà.

### Anti-pattern 2 — Ancrage IA
La première sortie de l'IA biaise tous les humains. Solution : générer en humain *avant* de consulter l'IA.

### Anti-pattern 3 — Dépendance qualité
Sans regard humain critique, on accepte des idées superficiellement plausibles mais creuses.

### Anti-pattern 4 — Perte de propriété intellectuelle
Selon où tournent les modèles, les idées peuvent fuir. Critique pour BSE qui se positionne « souveraineté ».

## Comment BSE peut intégrer l'IA

### Option A : agnostique total
- Pas d'IA intégrée
- L'utilisateur copie-colle dans son propre Claude / GPT à côté
- ✅ Souveraineté max, pas de dépendance externe
- ❌ Friction UX, pas de différenciation

### Option B : intégration optionnelle, locale ou cloud
- Une feature *AI co-pilot* que l'utilisateur peut activer
- Configuration : endpoint LLM au choix (Claude API, OpenAI, Ollama local, llama.cpp local…)
- L'utilisateur choisit son provider
- ✅ Souveraineté préservée (Ollama local)
- ✅ Confort (cloud par défaut si veut)
- ⚠️ Complexité d'implémentation

### Option C : Intégration native d'un LLM local
- Embarquer un small model (Llama 3 8B, Phi-3) via `llama.cpp` Rust binding
- Tout en local
- ✅ Vraie souveraineté
- ❌ Modèle moins performant que cloud
- ❌ Poids du binaire

**Recommandation BSE** : **Option B**. Configurable, local par défaut (Ollama), cloud optionnel.

## Features IA candidates pour BSE

### Pour le mode session de brainstorm
- **« Sème la toile »** : génère 20 idées initiales sur un sujet.
- **« Renverse »** : pour chaque idée sélectionnée, propose son inverse / variante créative.
- **« Avocat du diable »** : critique 3 idées sélectionnées.
- **« Regroupe »** : affinity mapping automatique des post-its.
- **« Vote IA »** : l'IA donne son avis ; à comparer au vote humain.

### Pour l'éditeur de toile (autres moments)
- **« Dessine-moi un schéma de [X] »** : génère un croquis vectoriel basique.
- **« Résume cette zone »** : résumé textuel d'un cluster d'éléments.
- **« Auto-tag »** : suggère des tags pour chaque idée.

## Considérations éthiques et UX

- **Toujours indiquer ce qui vient de l'IA** : un post-it généré par IA a une icône claire.
- **Consentement explicite** : les contributions des utilisateurs ne sont jamais envoyées à un service tiers sans accord par projet.
- **Mode « 100 % humain »** : un projet peut bloquer toute utilisation d'IA pour assurer un brainstorm « pur ».
- **Transparence du provider** : l'UI indique quel modèle est utilisé.

## Synthèse

> BSE en v1.0 reste **agnostique** à l'IA. À partir de v1.x, on introduit une **intégration optionnelle** configurable (Option B). Toujours préserver le mode « 100 % humain ». L'IA est un outil, pas une fonctionnalité par défaut. Cette approche colle aux valeurs de souveraineté et de simplicité de BSE.

## Liens

- [03-outils-existants.md](./03-outils-existants.md) — outils comme Jenova qui agrègent plusieurs LLMs
- [../12-REFERENCES/02-projets-open-source.md](../12-REFERENCES/02-projets-open-source.md) — `llama.cpp` Rust bindings
