# 05.04 — Présence et curseurs distants

> L'awareness est **séparée du CRDT**. C'est de l'état éphémère, jamais persisté.

## Qu'est-ce que l'awareness ?

L'awareness = tout ce qui décrit **l'état actuel d'un peer**, qui doit être partagé aux autres en temps réel mais qui **disparaît à la déconnexion** :
- Position du curseur
- Outil actif
- Sélection en cours
- Statut « en train de taper »
- Couleur assignée
- Nom et avatar
- Position de la caméra (si « follow user »)

**Aucun de ces éléments n'a vocation à survivre au peer**. Donc on ne les met **pas** dans le CRDT.

## Pourquoi pas dans le CRDT ?

1. **Volume** : un curseur à 30 Hz × 10 peers × 24h = 26M ops par jour
2. **Persistence inutile** : on s'en fout 5 minutes après
3. **Performance** : un CRDT s'alourdit avec l'historique

→ L'awareness vit dans un **second canal** parallèle.

## yrs-awareness

yrs fournit un module `yrs::sync::Awareness` qui implémente ce canal :

```rust
use yrs::sync::Awareness;

let mut awareness = Awareness::new(doc.clone());

// Set local state
awareness.set_local_state(json!({
    "cursor": { "x": 100.0, "y": 200.0 },
    "user": { "name": "Alice", "color": "#FF0000" },
    "tool": "pen",
    "selection": ["el_123", "el_456"]
}));

// Subscribe to remote changes
awareness.on_update(|added, updated, removed| {
    // un autre peer a modifié son state
});

// Sérialiser pour envoyer aux peers
let update = awareness.update();  // binaire compact
```

## Schéma BSE de l'awareness

```rust
pub struct AwarenessState {
    pub user: UserInfo,                    // immutable durant session
    pub color: Color,                       // assignée par serveur
    pub cursor: Option<CursorState>,        // updated 30 Hz max
    pub camera: Option<CameraState>,        // updated 5 Hz max
    pub selection: Vec<ElementId>,          // updated à chaque sélection
    pub active_tool: ToolKind,
    pub is_typing: Option<TypingState>,
    pub last_active_ms: u64,
}

pub struct CursorState {
    pub world_x: f32,
    pub world_y: f32,
    pub pressing: bool,
}

pub struct CameraState {
    pub center_x: f32,
    pub center_y: f32,
    pub zoom: f32,
}

pub struct TypingState {
    pub element_id: ElementId,
    pub last_keystroke_ms: u64,
}
```

## Diffusion via WebSocket

Awareness updates ont leur propre type de message :

```rust
// Côté client envoi
fn on_cursor_move(&mut self, pos: WorldPos) {
    if self.last_cursor_send.elapsed() < CURSOR_THROTTLE {
        return;
    }
    self.send(ClientMessage::Cursor(CursorPayload {
        x: pos.x,
        y: pos.y,
    }));
    self.last_cursor_send = Instant::now();
}

// Côté serveur broadcast
fn on_peer_cursor(&mut self, peer_id: PeerId, payload: CursorPayload) {
    self.awareness.update_cursor(peer_id, payload);
    self.broadcast_except(peer_id, ServerMessage::Cursor { peer_id, payload });
}
```

## Throttling

| Awareness type | Fréquence max | Raison |
|---|---|---|
| Cursor | 30 Hz | Fluidité visuelle |
| Camera | 5 Hz | Suffisant pour follow |
| Selection | au changement | Évent-driven |
| Tool active | au changement | Évent-driven |
| Typing indicator | 1 Hz pendant typing | Économie |

Au-delà : drop des envois (pas de buffering — on perd les events intermédiaires de toute façon).

## Rendu UI

### Curseurs distants
Chaque peer connecté affiche un curseur fantôme :

```
   ↗ Bob
   (en bleu)
   
   ↗ Charlie  
   (en vert)
```

- Animation de fluidité : interpolation entre les positions reçues (sinon = saccadé)
- Couleur du curseur = couleur du peer
- Label avec nom (s'efface après 2 s d'inactivité)
- Icône d'outil (crayon, sélection, etc.)
- Petit cercle remplissant si le peer dessine (pressing)

### Liste des peers connectés
Panel discret en haut à droite :

```
┌─────────────────┐
│ 👤 Alice (vous) │
│ 👤 Bob          │
│ 👤 Charlie     ●│  ← rond vert = en train de taper
└─────────────────┘
```

### Sélections distantes
Quand Bob sélectionne un élément, Alice voit une **bordure colorée** autour (couleur de Bob) + son nom au coin.

```
┌──────────────┐
│  Élément X   │
│ ──────────── │ ← bordure bleue (couleur Bob)
└──────────────┘
   ↳ Bob
```

### Follow user
Option « suivre la caméra de Bob » : Alice's caméra reprend celle de Bob en continu. Utile pour mode présentation.

## Couleurs assignées

Le serveur assigne une couleur à chaque peer à l'arrivée :
- Palette de 8-12 couleurs distinctes WCAG AA contrast
- Première dispo
- Sur saturation : on recommence le cycle (2 peers peuvent avoir la même)
- L'utilisateur peut personnaliser sa couleur dans les settings (override)

Palette de base :
```
#E63946  rouge
#F77F00  orange
#FCBF49  jaune
#06A77D  vert clair
#118AB2  bleu cyan
#073B4C  bleu foncé
#7400B8  violet
#FF006E  magenta
```

## Anonymat

Si le projet est en **mode anonyme** :
- Le nom affiché est « Anonyme #N » (N stable par session)
- Couleur quand même assignée
- Le serveur connaît le mapping mais ne le diffuse pas
- À utiliser pour les sessions de brainstorm sensibles

## Disconnect / cleanup

Quand un peer se déconnecte :
- Le serveur retire son awareness de la room
- Broadcast `PeerLeave` aux autres peers
- Les clients enlèvent son curseur, sélection, label
- Timeout grace : 5 s pour les déconnexions transitoires (reconnexion rapide ne supprime pas)

## Performance / scalabilité

### Au-delà de N peers
À 50+ peers actifs, les curseurs deviennent un bruit visuel ET du traffic.

Stratégies de mitigation v1.x :
- **Heatmap mode** : au-delà de 20 peers, on n'affiche plus les curseurs individuels mais une heatmap d'activité
- **Spatial filtering** : on n'envoie le curseur d'un peer qu'aux autres peers dont le viewport intersecte
- **Awareness compression** : updates dedupliquées au niveau serveur

## Persistance de la dernière position ?

**Non.** Au reload du client, la caméra repart à zoom 1.0 sur (0,0) ou un point de welcome (centre de masse des éléments).

## Tests

- Test : 2 clients, A dessine, B voit le curseur de A en quasi temps réel (<100 ms LAN)
- Test : 10 peers en parallèle, throughput d'awareness, pas de saturation
- Test : peer disparait brutalement (kill), son curseur disparaît côté autres en <5 s

## Liens

- Protocole → [../03-ARCHITECTURE/04-protocole-reseau.md](../03-ARCHITECTURE/04-protocole-reseau.md)
- UI multi-curseurs → [../08-UX-UI/03-multi-curseurs-presence.md](../08-UX-UI/03-multi-curseurs-presence.md)
