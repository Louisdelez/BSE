# 04.02 — Choix du framework GUI

> Comparaison des options Rust et choix final pour BSE.

## TL;DR

> **Recommandation : `egui`** comme framework principal, avec `winit` pour la fenêtre, `wgpu` pour le rendu, et un **canvas custom** au-dessus pour la toile infinie.

## Les options en présence (2026)

| Framework | Style | Maturité | Cible BSE ? |
|---|---|---|---|
| **egui** | Immediate-mode | ⭐⭐⭐⭐⭐ | ✅ **Choix #1** |
| **iced** | Elm-like retained | ⭐⭐⭐⭐ | ✅ Choix #2 |
| **slint** | DSL déclaratif | ⭐⭐⭐⭐ | ⚠️ Licence GPL/commercial |
| **dioxus** | React-like | ⭐⭐⭐⭐ | ❌ pensé pour web/SSR |
| **gpui** | Zed editor framework | ⭐⭐⭐ | ⚠️ jeune, Zed-coupled |
| **floem** | Reactive | ⭐⭐⭐ | ⚠️ jeune |
| **xilem** | Linebender framework | ⭐⭐ | ❌ trop jeune |
| **fltk** | Wrappers FLTK | ⭐⭐⭐ | ❌ pas natif Rust |
| **gtk-rs / relm4** | Bindings GTK | ⭐⭐⭐⭐ | ❌ GTK on Windows/Mac mauvais |
| **Tauri + web UI** | WebView | ⭐⭐⭐⭐⭐ | ❌ perf canvas dégradée |

## Analyse détaillée

### egui (le choix)

**Type** : immediate-mode (comme Dear ImGui).

**Architecture** :
- L'UI est redessiné à chaque frame
- Pas d'arbre de widgets persistent
- État simple, pas de databinding complexe

**Forces** :
- ✅ **Très simple et productif** (pas de DSL, pas de macros)
- ✅ **Rendu via wgpu** → compatible avec notre stack
- ✅ **Performance excellente** pour des UIs interactives
- ✅ **Cross-platform**, fonctionne sur Windows/macOS/Linux et WASM
- ✅ **Communauté active**, releases fréquentes
- ✅ **Customisable** : on peut dessiner directement sur le contexte wgpu
- ✅ **Cohabite avec wgpu sans friction** : on peut avoir un widget custom qui dessine via wgpu et le reste en egui

**Faiblesses** :
- ⚠️ Pas natif look-and-feel (mais on s'en fout, on veut un look BSE)
- ⚠️ IME (langues asiatiques) historiquement faible — s'est amélioré
- ⚠️ Layouts plus rigides qu'un framework retained

**Pourquoi pour BSE** :
- L'**UI chrome** (toolbar, panels) est mineure par rapport au canvas custom
- Le canvas custom sera de toutes manières du rendu wgpu direct
- egui rend la partie UI triviale à coder, peu d'investissement frontal

### iced (alternative principale)

**Type** : retained, Elm-like (Model-Update-View).

**Forces** :
- ✅ Architecture **propre** (TEA — The Elm Architecture)
- ✅ Bonnes perf
- ✅ Custom widgets propres

**Faiblesses** :
- ⚠️ Plus verbeux qu'egui
- ⚠️ Pattern Elm peut être lourd pour les manipulations directes du canvas

**Quand le préférer ?** : si BSE devient massivement piloté par des messages, peut-être migration future. Mais pour la v1, egui est meilleur fit.

### slint

**Forces** :
- ✅ DSL déclaratif élégant (`.slint` files)
- ✅ Look natif sur chaque plateforme
- ✅ Très productif

**Bloquant** :
- ❌ **Licence** : GPL pour le free tier, payante pour commercial use sans royalty
- BSE étant Apache-2, incompatible avec GPL côté framework

### gpui (Zed)

**Forces** :
- ✅ Très performant (utilisé par Zed editor)
- ✅ Stack moderne

**Bloquant en 2026** :
- ❌ Encore jeune comme lib séparée
- ❌ Couplé aux usages de Zed
- ❌ Doc et écosystème limités hors Zed

À reconsidérer en v2.0 si gpui devient stable et bien documenté.

### Tauri (web UI)

**Forces** :
- ✅ Très productif si on connaît React/Vue/Svelte
- ✅ Bundle léger (vs Electron)

**Bloquant pour BSE** :
- ❌ Le canvas tournerait dans une WebView (WebKit/WebView2)
- ❌ Perte du contrôle GPU bas niveau
- ❌ Latence input plus élevée (1 frame de plus)
- ❌ Antithétique à notre pari « performance native »

## Le canvas custom (le cœur de BSE)

**egui ne sera utilisé que pour le chrome** (toolbar, panels, dialogs, sidebar). La **toile elle-même** est un widget custom qui :

1. Capture les inputs (pointer, clavier) via la zone réservée par egui
2. Maintient son propre état (caméra, sélection, outil actif)
3. Rend directement via **wgpu** dans son propre frame
4. egui compose ensuite ses overlays par-dessus

### Pattern d'intégration

```rust
// pseudo-code simplifié
fn update(ctx: &egui::Context, frame: &mut eframe::Frame) {
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        draw_toolbar(ui, state);
    });
    
    egui::SidePanel::right("properties").show(ctx, |ui| {
        draw_properties_panel(ui, state);
    });
    
    egui::CentralPanel::default().show(ctx, |ui| {
        let canvas_response = ui.allocate_response(
            ui.available_size(),
            egui::Sense::click_and_drag(),
        );
        
        // 1. handle inputs
        state.canvas.handle_input(&canvas_response, ui);
        
        // 2. render the canvas via wgpu (paint callback)
        let paint_callback = egui_wgpu::CallbackFn::new()
            .prepare(|device, queue, encoder, resources| {
                state.canvas.render(device, queue, encoder, ...);
            });
        ui.painter().add(paint_callback);
    });
}
```

## Empilement réel des dépendances

```
   ┌──────────────────┐
   │   eframe         │  (boilerplate desktop autour de egui)
   └────────┬─────────┘
            │
   ┌────────▼─────────┐
   │     egui         │  (immediate-mode UI)
   └────────┬─────────┘
            │
   ┌────────▼─────────┐    ┌────────────────┐
   │   egui-wgpu      │◄──►│  wgpu (rendu)  │
   └──────────────────┘    └───────┬────────┘
                                   │
                          ┌────────▼────────┐
                          │  winit          │  (fenêtre, events OS)
                          └─────────────────┘
```

## Considérations spécifiques

### Hautement DPI / Retina
- egui gère le facteur d'échelle natif
- Important pour : police, taille des icônes, tracé des stylets

### Tablette graphique / stylet
- winit expose les events stylet (pression, tilt) via `WindowEvent::Touch`
- Sur Windows : pression via WinTab/WinPointer
- Sur macOS : via NSEvent
- Sur Linux : via libinput
- On exposera ces données dans notre couche de capture d'input

### Accessibilité
- egui a un support a11y via AccessKit (en croissance en 2026)
- Cible v1 : labels lisibles screen reader pour les contrôles principaux

## Plan B si egui montre ses limites

Si en v0.5 on rencontre des limites bloquantes :
1. Migration vers `iced` (retained, plus structuré)
2. Évaluation de `gpui` si stabilisé
3. **Le canvas custom resterait inchangé** (c'est notre code, pas du framework)

Cette modularité est précieuse.

## Décision finale

> **`egui` + `wgpu` + `winit` (via `eframe`)** pour la v1.
>
> Le **canvas custom** est notre code, indépendant du framework UI. C'est notre invariant.

## Sources

- Comparison perf 2023 : lukaskalbertodt.github.io
- *State of Rust GUI* — Rust Bytes Substack
- Survey 2025 : *Which GUI framework should I choose* — Rust users forum
