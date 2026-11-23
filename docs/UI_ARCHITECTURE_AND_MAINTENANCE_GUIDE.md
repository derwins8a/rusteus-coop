# NucleusCoop UI Architecture & Maintenance Guide

This document is the normative architectural and developer guide for maintaining, extending, and refactoring the graphical user interface (`nucleus-ui` and `nucleus-app`) of NucleusCoop Rust Rewrite.

---

## 1. Architectural Philosophy & Mode Separation

NucleusCoop provides two distinct user interfaces tailored to fundamentally different user scenarios:

```
                      ┌─────────────────────────┐
                      │    MainWindow Model     │
                      │  (crates/nucleus-ui)    │
                      └────────────┬────────────┘
                                   │
             ┌─────────────────────┴─────────────────────┐
             ▼                                           ▼
┌─────────────────────────┐                 ┌─────────────────────────┐
│      Desktop Mode       │                 │    Big Picture Mode     │
│   (Mouse & Keyboard)    │                 │   (10-Foot Couch Pad)   │
├─────────────────────────┤                 ├─────────────────────────┤
│ • High-density layout   │                 │ • Spatial 2D navigation │
│ • Native file dialogs   │                 │ • Large typography      │
│ • Sidebar navigation    │                 │ • Gamepad button glyphs │
│ • Multi-column lists    │                 │ • Radial/carousel flow  │
│ • Deep debugging/tools  │                 │ • In-game OSD overlay   │
└─────────────────────────┘                 └─────────────────────────┘
```

> [!IMPORTANT]
> **Preserve the Dual Paradigm**:
> Desktop screens and Big Picture screens are **purposely intended to look and feel different**.
> Desktop mode caters to power users configuring handlers, inspecting scripts, managing files, and scanning directories with precise mouse clicks and keyboard shortcuts.
> Big Picture mode caters to living-room couch players with controllers navigating via D-pad/analog sticks with large focus indicators and zero mouse dependency.
> **Never convert Desktop screens into Big Picture screens or vice-versa.**

---

## 2. Crate & Directory Organization

The UI codebase is centered in `crates/nucleus-ui` with binary entry points in `crates/nucleus-app`:

```
crates/nucleus-ui/src/
├── big_picture/            # 10-foot gamepad-driven screens
│   ├── carousel.rs         # Game library carousel with cover art & filter ribbons
│   ├── details.rs          # Game detail view (quick launch, options, stats)
│   ├── hub.rs              # Community handler search & download browser
│   ├── management.rs       # Controller manager, raw input binding, and pairing
│   ├── options.rs          # Gamepad-navigable handler options modal
│   ├── overlay.rs          # In-game HUD/OSD menu for mid-game reconfiguration
│   ├── sessions.rs         # Active session monitor and saved profile loader
│   ├── settings.rs         # Spatial preferences (display, audio, scale)
│   └── setup.rs            # Splitscreen setup, player slots, partition picker
├── components/             # Unified custom component meta-framework
│   ├── badge.rs            # CouchTag status pills and labels
│   ├── banner.rs           # NefosBanner alerts and preflight notices
│   ├── button.rs           # CouchButton with primary/secondary/ghost/danger variants
│   ├── card.rs             # NefosCard container surfaces with elevation/borders
│   ├── confirm_dialog.rs   # Modal confirmation prompts
│   ├── context_menu.rs     # Dropdown context menus with gamepad remapping
│   ├── cover.rs            # GameCover with aspect ratio enforcement & caching
│   ├── launch_prompt.rs    # Interactive multi-instance staging prompts
│   ├── modal.rs            # CouchModal base wrapper with backdrop click dismissal
│   ├── option_combobox.rs  # Searchable dropdown option selector
│   ├── option_row.rs       # Key-value handler option rows with inline controls
│   ├── partition.rs        # Interactive monitor split layout preview
│   ├── picker.rs           # Multi-option discrete value selector
│   ├── search_bar.rs       # Debounced search bar with clear button & focus routing
│   ├── segmented_control.rs# Pill-style segmented tabs
│   ├── sidebar_nav.rs      # Desktop collapsible left navigation sidebar
│   ├── slot_card.rs        # Player slot card with gamepad/KBM status
│   ├── status_bar.rs       # Bottom status bar with device counts and notifications
│   ├── title_bar.rs        # Client-Side Decorated (CSD) draggable title bar
│   └── toggle.rs           # Boolean switch toggle
├── nefos/                  # Backward-compatibility alias layer
│   └── mod.rs              # Re-exports all components under legacy Nefos* types
├── window/                 # MainWindow implementation modularized by domain
│   ├── mod.rs              # Struct definition, constructors, builders & Render loop
│   ├── couch_actions.rs    # Couch spatial navigation & action dispatchers
│   ├── desktop_actions.rs  # Desktop launchers, filesystem dialogs & scanning
│   ├── modals.rs           # Popovers, dialogs, shortcuts, and button glyphs
│   ├── session_osd.rs      # Session heartbeat, OSD visibility, and lock hotkeys
│   └── tests.rs            # Unit test suite
├── callbacks.rs            # EngineToGui & MainWindowCallbacks MPSC bridge
├── dialogs.rs              # Native Windows Explorer dialogs via STA PowerShell
├── http_client.rs          # Async cover art image loader with disk caching
├── models.rs               # UI view models (HandlerItem, DeviceItem, etc.)
└── theme.rs                # Central color tokens, borders, and typography
```

---

## 3. Core Architectural Invariants

### A. Type Boundary Flattening (`into_any_element()`)
GPUI uses deeply nested recursive generic types to describe the UI tree. If conditional branches (`if ... else ...`) return different concrete generic wrappers, or if views nest tens of builder calls, compile times degrade exponentially and can cause rustc recursion limits.

**Rule**: Every custom component implements `.into_any_element()` returning `gpui::AnyElement`. Always call `.into_any_element()` at component boundaries.

```rust
// CORRECT:
let content: AnyElement = if is_grid {
    render_grid(cx).into_any_element()
} else {
    render_list(cx).into_any_element()
};
```

### B. Asynchronous Non-Blocking UI
**Rule**: Never execute synchronous I/O, file copies, disk scanning, PowerShell dialogs, or network HTTP requests directly inside GPUI event handlers or `render()` methods. Doing so freezes the desktop window and drops frame rendering.

- **For Quick Background Tasks**: Use `cx.spawn(...)` with `tokio::sync::mpsc::channel`:
  ```rust
  let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
  std::thread::spawn(move || {
      let result = perform_heavy_computation();
      let _ = tx.send(result);
  });
  cx.spawn(async move |this, cx| {
      if let Some(res) = rx.recv().await {
          this.update(&mut *cx, |win, cx| {
              win.apply_result(res);
              cx.notify();
          }).ok();
      }
  }).detach();
  ```
- **For Native File Pickers**: Use `crate::dialogs::pick_file_async` or `crate::dialogs::pick_folder_async` which run STA PowerShell in a worker pool and `await` the selection asynchronously.

### C. Thread Safety (`!Send` GPUI Context)
`gpui::Context<T>` and `gpui::App` are `!Send`. They cannot be sent across thread boundaries or captured in `std::thread::spawn` closures.
Always communicate between worker threads and GPUI using channels (`std::sync::mpsc` or `tokio::sync::mpsc`) or update through `cx.spawn` tasks.

### D. Focus & Event Propagation Safety
When displaying modals, context menus, or overlay dialogs, ensure that mouse clicks inside the modal do not fall through to the underlying card or search bar:
- `CouchModal` automatically calls `.on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())` on its dialog container.
- When opening an input modal (e.g. search bar or hotkey recorder), focus the corresponding `FocusHandle` so keyboard events route directly to the active control.

---

## 4. How-To Guides for Human Developers

### Adding a New Custom Component
1. Create `crates/nucleus-ui/src/components/your_component.rs`.
2. Follow the standard builder pattern:
   - Hold properties in a private struct.
   - Implement `new(...) -> Self`.
   - Implement chainable configuration methods (`pub fn variant(mut self, ...) -> Self`).
   - Implement `pub fn into_any_element(self) -> AnyElement`.
3. Export the component in `crates/nucleus-ui/src/components/mod.rs`.
4. If needed for backward compatibility, add an alias in `crates/nucleus-ui/src/nefos/mod.rs`.

### Adding a New Desktop Navigation Tab
1. Add a variant to `pub enum ActiveTab` in `crates/nucleus-ui/src/window/mod.rs`.
2. Update `CouchSidebarNav::new()` in `crates/nucleus-ui/src/components/sidebar_nav.rs` to include the icon and title for the tab.
3. In `MainWindow::render()` (`crates/nucleus-ui/src/window/mod.rs`), add the branch matching `ActiveTab::YourTab` to render the view.

### Adding a New Big Picture Screen
1. Add a variant to `pub enum BigPictureScreen` in `crates/nucleus-ui/src/big_picture/mod.rs`.
2. Create the screen view in `crates/nucleus-ui/src/big_picture/your_screen.rs`.
3. In `crates/nucleus-ui/src/window/couch_actions.rs`, handle the directional navigation (`Up`, `Down`, `Left`, `Right`, `Confirm`, `Cancel`) for your screen in `dispatch_couch_action_by_name`.
4. In `MainWindow::render()`, render the screen when `self.big_picture_screen == BigPictureScreen::YourScreen`.

---

## 5. Maintenance Checklist

Before submitting PRs or commits modifying the UI:
- [ ] Run `cargo check -p nucleus-ui` to ensure zero compilation errors and warnings.
- [ ] Run `cargo check -p nucleus-app --bin nucleus-coop` to verify application wiring.
- [ ] Verify that Desktop screens retain their high-density, mouse-driven controls.
- [ ] Verify that Big Picture screens retain full spatial navigation via gamepad actions.
- [ ] Ensure any newly added blocking I/O is wrapped in async background tasks.
