# 08.06 — Implémentation UI moderne sur egui

> Stratégie technique pour atteindre un look Miro/Linear-grade dans BSE
> en restant sur egui + eframe + wgpu. Aucune migration de framework.

## Contexte

Le projet utilise **egui 0.30+** comme framework UI immédiat-mode, choisi pour :
- Rust pur, intégration native avec wgpu (où vit le canvas)
- Cohérence avec le reste du stack (15 crates)
- Coût mémoire et performance prévisibles

Le look par défaut d'egui est "engineer dashboard" — gris, plat, sans
animations. Pour atteindre le look documenté dans
[`/DESIGN.md`](../../DESIGN.md) (Miro-inspired pastels + pill buttons
+ Inter), on ajoute une **couche de présentation** au-dessus d'egui
sans toucher au reste du codebase.

## Audit des contraintes egui

### Ce qu'egui sait faire nativement
- Custom fonts via `Context::set_fonts` + `FontDefinitions`
- Rounded corners (`CornerRadius`) — pill buttons OK
- Drop shadows (`Shadow` avec offset/color/blur/spread)
- Theming via `Visuals` (toutes les couleurs des composants)
- Animations linéaires via `Context::animate_bool` / `animate_value_with_time`
- Layouts immédiats : horizontal/vertical/Grid (single-pass)

### Ce qu'egui ne sait PAS faire (et nos contournements)
- ❌ **Backdrop blur** ([#1299](https://github.com/emilk/egui/discussions/1299))
  → Contournement : surface dimmed semi-transparente
- ❌ **Gradients fluides** ([#3822](https://github.com/emilk/egui/issues/3822))
  → Contournement : aplats Miro (déjà 99% du DESIGN.md)
- ❌ **Per-side borders CSS-like** ([#4019](https://github.com/emilk/egui/issues/4019))
  → Contournement : `Frame` avec borders symétriques + custom painting si vraiment besoin
- ❌ **Spring physics natif**
  → Contournement : `egui_animation` crate (par l'équipe Rerun) + cubic-bezier eased
- ❌ **Layout multi-pass** (mesure-avant-position)
  → Contournement : pré-calculs manuels, layouts contraints

**Verdict** : on atteint 80-85% du look Miro/Linear. Les 15% manquants
ne sont pas bloquants pour la valeur produit.

## Stack technique adoptée

### Crates ajoutées au workspace (Tier 1)

| Crate | Version | Rôle |
|---|---|---|
| `egui-phosphor` | 0.12+ | 7000+ icônes Phosphor, 6 weights, intégration native |
| `egui-notify` | 0.22+ | Toasts animés avec timers visuels |
| `egui_animation` | 0.11+ | Easings + animations collapse (maintenu par Rerun) |
| `egui-theme-lerp` | latest | Interpolation fluide light ↔ dark |
| `egui_extras` | 0.34+ | `install_image_loaders()` (SVG via resvg, raster, HTTP) |

### Crates en réserve (Tier 2 — à activer au besoin)

| Crate | Rôle |
|---|---|
| `hello_egui` | Méta-crate : `egui_flex`, `egui_dnd`, `egui_virtual_list`, `egui_router` |
| `egui_taffy` | Vrai flexbox/grid CSS via taffy (si layouts complexes) |
| `egui_commonmark` | Markdown rendering pour help text |
| `egui_colors` | Palettes 12-step OkHsl avec contraste APCA |

## Code porté depuis Rerun (`re_ui`, MIT/Apache-2.0)

Rerun publie `re_ui` sur crates.io mais ses assets sont tightly-coupled.
Stratégie : **vendoriser/porter** les modules utiles plutôt que dépendre.

Fichiers ciblés pour port :

1. **`design_tokens.rs` + `color_table.ron`** — système de tokens RON
   hot-reloadable. Mappé sur les valeurs de notre `DESIGN.md`.
2. **`hot_reload_design_tokens.rs`** — watcher `notify` cfg-gated qui
   recharge les tokens à la sauvegarde du fichier RON. Itération
   design × 10 sans recompile.
3. **`list_item/` (dossier entier)** — composant "row" ultra-réutilisable
   (`label_content`, `property_content`, `button_content`). Utilisé
   pour room picker, peer list, properties panel.
4. **`modal.rs`, `alert.rs`, `notifications.rs`, `loading_indicator.rs`** —
   widgets autoporteurs, scale-in animations.
5. **`command_palette.rs`** — Cmd+K Figma-style (modal + TextEdit + liste
   filtrée).
6. **`ui_ext.rs`, `context_ext.rs`** — extension traits ergonomiques.

## Assets embarqués

### Polices
- **Inter 4.1 Variable** (`InterVariable.ttf`, ~800 KB, OFL)
  → Source : [github.com/rsms/inter/releases/tag/v4.1](https://github.com/rsms/inter/releases/tag/v4.1)
  → Axes : `wght 100-900`, `opsz 14-32`
  → Embarqué dans `crates/bse-ui/assets/fonts/`

### Icônes
- **Phosphor** via `egui-phosphor` crate — le crate bundle la font, rien
  à télécharger. 1488 icônes × 6 weights (Thin/Light/Regular/Bold/Fill/Duotone).

### Couleurs
- Tokens Miro directement portés depuis `/DESIGN.md` en constantes Rust
  (`crates/bse-ui/src/theme/colors.rs`).
- Pour les neutres et états : valeurs Radix Colors 12-step (MIT) en
  référence.

## Architecture du module `bse-ui` après refonte

```
crates/bse-ui/
  Cargo.toml
  assets/
    fonts/
      InterVariable.ttf
    themes/
      light.ron        # tokens hot-reloadables
      dark.ron
  src/
    lib.rs
    theme/
      mod.rs           # apply_bse_theme(ctx, ThemeMode)
      colors.rs        # tokens Miro depuis DESIGN.md
      typography.rs    # Inter setup + tailles
      motion.rs        # duration/easing/spring tokens
      tokens.rs        # struct DesignTokens (port re_ui)
      hot_reload.rs    # cfg-gated, port re_ui
    components/
      mod.rs
      pill_button.rs   # Miro pill button (radius:full)
      card.rs          # feature card (radius:28, shadow)
      modal.rs         # scale-in modal (port re_ui)
      list_item.rs     # row composable (port re_ui)
      command_palette.rs  # Cmd+K (port re_ui)
      avatar.rs        # presence avatar (cercle + initiale)
      status_pill.rs   # connection/role badges
    info.rs            # existing
    status_bar.rs      # refactor avec status_pill
    toolbar.rs         # refactor floating bottom + Phosphor
```

## Tokens de motion (cohérence cross-composants)

```rust
// crates/bse-ui/src/theme/motion.rs
pub mod duration {
    use std::time::Duration;
    pub const MICRO: Duration    = Duration::from_millis(120);
    pub const STANDARD: Duration = Duration::from_millis(200);
    pub const MACRO: Duration    = Duration::from_millis(350);
}

pub mod easing {
    // Cubic bezier "Vercel curve" — system-driven (modals, menus)
    pub const STANDARD: [f32; 4] = [0.16, 1.0, 0.3, 1.0];
    // Cubic bezier "back-out" — user-driven (drops, stamps)
    pub const BACK_OUT: [f32; 4] = [0.34, 1.56, 0.64, 1.0];
}

pub mod spring {
    pub struct SpringConfig { pub stiffness: f32, pub damping: f32 }
    pub const DEFAULT: SpringConfig = SpringConfig { stiffness: 300.0, damping: 20.0 };
    pub const FAST:    SpringConfig = SpringConfig { stiffness: 500.0, damping: 30.0 };
}
```

## Patterns d'animation à appliquer

Tirés de Figma, FigJam, tldraw, Linear :

| Pattern | Détails |
|---|---|
| **Tool select** | Scale `1.0 → 1.08 → 1.0`, spring (s=300, d=20) |
| **Modal open** | Fade + scale `0.96 → 1.0` + `translateY(-4 → 0)`, 150ms, cubic-bezier STANDARD |
| **Hover row** | Background tint en 80-100ms ease-out, pas de mouvement |
| **Status change** | Color crossfade 180ms + scale bump `1.0 → 1.04 → 1.0` |
| **Remote cursor** | Interpolation 60fps avec 100-150ms ease-out smoothing |
| **Notification toast** | Slide-in droite 220ms + timer ring qui se vide |

## Roadmap d'implémentation

Voir [11-ROADMAP-EXECUTION/02-jalons-v0-v1.md](../11-ROADMAP-EXECUTION/02-jalons-v0-v1.md)
pour le détail des milestones v026-v035.

Résumé :

| Tag | Contenu |
|---|---|
| v026 | Theme foundation : crates + Inter + apply_bse_theme + tokens DESIGN.md |
| v027 | Composants core : PillButton, Card, Modal (animations comprises) |
| v028 | Refactor login modal avec nouveau theme + composants |
| v029 | Refactor room picker en cards + list_item style |
| v030 | Floating toolbar (style tldraw) + icônes Phosphor + animations |
| v031 | Status bar polish + StatusPill + Avatar (présence) |
| v032 | Notifications via egui-notify (sign-in, invites, sync errors) |
| v033 | Command palette (Cmd+K) |
| v034 | Hot-reload des tokens RON (cfg-gated, dev only) |
| v035 | Pass final : prefers-reduced-motion + accessibilité + tests visuels |

## Performance et accessibilité

- **prefers-reduced-motion** : detect via egui input → bypass animations,
  garder seulement les fades opacity ≤ 100ms
- **AccessKit** : déjà intégré dans eframe, on garde
- **WCAG AA** : tous les contrastes texte/fond ≥ 4.5:1 — validé via
  un test unit sur le module `theme::colors`
- **60 FPS** : aucune animation ne doit faire chuter le frame rate
  (mesure via `last_frame` déjà en place dans BseApp)

## Liens

- [DESIGN.md](../../DESIGN.md) — tokens source de vérité
- [01-principes-design.md](./01-principes-design.md) — principes UX
- [05-design-system.md](./05-design-system.md) — adoption Miro
- Sources externes :
  - [egui ecosystem wiki](https://github.com/emilk/egui/wiki/3rd-party-egui-crates)
  - [Rerun re_ui code](https://github.com/rerun-io/rerun/tree/main/crates/viewer/re_ui)
  - [hello_egui](https://github.com/lucasmerlin/hello_egui)
  - [Inter font releases](https://github.com/rsms/inter/releases)
  - [Phosphor icons](https://phosphoricons.com/)
