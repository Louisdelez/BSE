# 00.03 — Glossaire

Vocabulaire utilisé dans toute la documentation BSE. Quand un mot est en *italique* dans le reste de la doc, il est défini ici.

## Produit

- **Projet** — Conteneur racine. Un projet possède une toile, des collaborateurs, des permissions, un historique. Un utilisateur peut avoir N projets.
- **Toile** (*canvas* / *board*) — La surface infinie où tout est dessiné. Une toile appartient à un projet.
- **Élément** (*element* / *shape*) — N'importe quel objet posé sur la toile : un trait, une forme, un texte, une image, un post-it.
- **Scène** — L'ensemble des éléments actuellement chargés en mémoire pour rendu. Subset de la toile complète.
- **Viewport** — La fenêtre de la toile actuellement visible à l'écran. Définie par caméra + zoom.
- **Caméra** — État de navigation : `(x, y, zoom)` du centre de viewport en coordonnées monde.
- **Outil** (*tool*) — Mode actif du curseur : sélection, dessin, ellipse, texte, etc.
- **Post-it** — Carte colorée carrée avec texte court, métaphore éclair Miro/FigJam.
- **Mindmap** — Arbre d'idées avec nœuds reliés par des connecteurs.
- **Template** — Snapshot prêt-à-l'emploi pour démarrer une session (rétrospective, brainstorm SCAMPER, etc.).

## Collaboration

- **Room** — Espace de collaboration côté serveur. 1 projet actif = 1 room. Tous les clients connectés au même projet partagent la même room.
- **Peer** — Un client connecté à une room.
- **Awareness** — Métadonnées éphémères partagées entre peers : position du curseur, sélection, nom, couleur. Non persistées.
- **Présence** — Synonyme d'awareness côté UI.
- **CRDT** (*Conflict-free Replicated Data Type*) — Structure de données qui permet à plusieurs réplicas de converger vers le même état sans coordination centrale.
- **OT** (*Operational Transformation*) — Alternative aux CRDT, basée sur la transformation d'opérations concurrentes.
- **LWW** (*Last-Writer-Wins*) — Stratégie de résolution de conflits : la modification la plus récente gagne, identifiée par timestamp + ID de peer.
- **Snapshot** — État complet et sérialisé d'une toile à un instant T.
- **Delta** / **Update** — Modification incrémentale d'une toile, diffusable aux autres peers.

## Réseau

- **WebSocket** — Protocole bidirectionnel full-duplex sur TCP. Choix par défaut pour la sync BSE.
- **QUIC** — Protocole de transport plus moderne (UDP, multiplexage natif, 0-RTT). Alternatif via iroh.
- **NAT traversal** — Capacité à établir des connexions P2P malgré les routeurs NAT (hole-punching).
- **Relais** — Serveur intermédiaire qui transmet les messages quand le P2P direct échoue.

## Rendu

- **wgpu** — Bibliothèque Rust qui abstrait Vulkan/Metal/DX12/WebGPU.
- **Vello** — Renderer 2D GPU-compute, candidat pour le rendu vectoriel de BSE.
- **Frame budget** — Temps maximum pour produire un frame. 16.6 ms à 60 FPS, 6.9 ms à 144 FPS.
- **Culling** — Élimination des éléments hors viewport avant rendu.
- **LOD** (*Level of Detail*) — Rendu simplifié pour les éléments très petits ou très éloignés.
- **Spatial index** — Structure (quadtree, R-tree) qui permet de répondre vite à « quels éléments sont dans cette région ? ».
- **Quadtree** — Arbre où chaque nœud a 4 enfants (subdivisions en quadrants), utilisé pour indexer un espace 2D.

## Architecture

- **Client** — L'application desktop installée chez l'utilisateur.
- **Serveur** — Le backend qui héberge les rooms et persiste les projets.
- **Daemon** — Service serveur tournant en arrière-plan (sur la machine de l'utilisateur en self-host, ou un VPS).
- **Bus d'événements** — Mécanisme de pub/sub interne au client (entrées utilisateur → état → rendu).
- **Hot path** — Code chemin critique pour la performance (rendu de frame, traitement d'input).

## Sécurité

- **OAuth 2.1** — Protocole d'autorisation déléguée (Google, GitHub, etc.).
- **OIDC** (*OpenID Connect*) — Couche d'identité au-dessus d'OAuth 2.
- **JWT** (*JSON Web Token*) — Format de token signé, lisible côté client.
- **RBAC** (*Role-Based Access Control*) — Permissions assignées via rôles.
- **E2EE** (*End-to-End Encryption*) — Chiffrement où seuls les peers déchiffrent ; le serveur relaie sans pouvoir lire.

## Process / produit

- **Divergent** — Phase de brainstorm qui *ouvre* (génération d'idées).
- **Convergent** — Phase qui *referme* (sélection, priorisation).
- **Production blocking** — Perte d'idées due à l'attente de son tour de parole dans un brainstorm verbal.
- **Affinity mapping** — Regroupement de post-its par thèmes après divergence.
- **Dot voting** — Vote par points distribués sur les meilleures idées.

## Repères de mesures

- **p50 / p95 / p99** — Percentiles d'une distribution (50 % / 95 % / 99 % des cas en dessous).
- **RTT** — Round-trip time, temps aller-retour réseau.
- **Throughput** — Nombre d'opérations par seconde.
