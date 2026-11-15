# Nucleus UI â€” Refactor, Modernization & Meta-Framework Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Systematically refactor `crates/nucleus-ui` from a monolithic 20k-line codebase into a clean, component-driven meta-framework â€” replacing raw `div()` chains with gpui-kit and domain components, decomposing the 4,947-line `window.rs` God-class, and introducing new visual and UX polish throughout.

**Architecture:** Each task produces an independently verifiable delta. Tasks within the same phase are designed to be executed in parallel by separate subagents where possible. No task requires another task's output unless explicitly noted in its `Interfaces: Consumes` block.

**Tech Stack:** Rust Â· GPUI / gpui-kit (`gpui_kit::component::*`) Â· `gpui_component` (Longbridge) Â· `crate::components::*` meta-framework Â· `crate::theme::*` design tokens

**Spec:** This plan is self-contained. Implementers must also read:
- `crates/nucleus-ui/src/components/mod.rs` (component catalogue)
- `crates/nucleus-ui/src/theme.rs` (design tokens)
- `.agents/skills/gpui-ui-safety/SKILL.md` (all 17 safety rules â€” mandatory pre-read)
- `.agents/skills/gpui-kit/SKILL.md` (component catalog â€” mandatory pre-read)
- `AGENTS.md` Â§3 (Handler Compatibility) and Â§C (GPUI type invariants)

---

## Global Constraints

- `recursion_limit` in `lib.rs` MUST remain `â‰¤ 1024`; flatten with `.into_any_element()` at every component boundary instead of bumping it.
- All new render helpers MUST return `AnyElement`, never a concrete GPUI builder type.
- `tokio::time::sleep` is FORBIDDEN in `cx.spawn` tasks â€” use `cx.background_executor().timer(dur).await`.
- Never call `cx.entity().read(cx)` inside `Render::render` â€” pass borrowed `&self` fields down.
- All new `div()` with `.on_click` MUST have `.id(...)` first.
- All new `div()` with `.tooltip(...)` MUST have `.id(...)` first.
- `CouchButton`, `CouchTag`, `CouchModal`, `CouchOptionRow`, `CouchSlotCard`, `GameCover`, `CouchSearchBar`, `CouchPicker<T>`, `PartitionPreview` are the canonical building blocks â€” use them in ALL new and refactored UI code, never raw `div()` equivalents.
- Verification sequence after every task: `cargo check -p nucleus-ui -j4` â†’ `cargo check -p nucleus-app -j4` â†’ `cargo test -p nucleus-ui -j4` â†’ `cargo build -j4 --bin nucleus-coop` (dev, NOT `--release`).
- Deploy artefact: `C:\Games\NucleusCoop242\nucleus-coop.exe`.

## Review Focus

1. **Double-lease panics in `window.rs`** â€” Any helper that calls `cx.entity().read(cx)` while inside `<MainWindow as Render>::render` will crash immediately. Verify every extracted helper receives `&self` or slices, not a re-borrowed entity.
2. **Generic type explosion** â€” Extracted helpers that return concrete GPUI builder types (not `AnyElement`) cause compiler stack overflows. Every helper boundary MUST end in `.into_any_element()`.
3. **Focus handle lifecycle** â€” `FocusHandle` values stored in `Option<FocusHandle>` on `MainWindow` must be initialised in `Render::render` only if `None` (via `cx.focus_handle()`), never re-created on every frame.
4. **Tokio reactor in GPUI tasks** â€” Any `tokio::sync::mpsc` channel receives inside `cx.spawn` using `.await` are fine; calling `tokio::time::sleep` will panic. Grep for `tokio::time` after each task.
5. **Callback Arc clone overhead** â€” The 25+ `Arc<dyn Fn...>` fields that are cloned on every render frame cause heap allocations. The `MainWindowCallbacks` struct (Task 1) must be introduced before any other task that touches callbacks.

---

## Phase 1 â€” Foundation Decomposition (Parallelizable)

### Task 1: Extract `MainWindowCallbacks` Builder Struct

**Files:**
- Create: `crates/nucleus-ui/src/callbacks.rs`
- Modify: `crates/nucleus-ui/src/window.rs` (fields 136â€“228, `with_*` builder methods 1172â€“1424, all call sites)
- Modify: `crates/nucleus-ui/src/lib.rs` (add `pub mod callbacks; pub use callbacks::MainWindowCallbacks;`)

**Interfaces:**
- Produces: `pub struct MainWindowCallbacks { ... }` with all 20 `Arc<dyn Fn...>` callback fields moved out of `MainWindow`; `MainWindow.callbacks: MainWindowCallbacks`; all `with_*` builder methods moved to `MainWindowCallbacks` builder.
- Produces: `impl MainWindow { pub fn with_callbacks(mut self, cbs: MainWindowCallbacks) -> Self }`.

- [x] **Step 1: Create `callbacks.rs` with `MainWindowCallbacks`**

  Move all 20 `pub on_*: Option<Arc<dyn Fn(...) + Send + Sync>>` fields from `MainWindow` (lines 134â€“228 of `window.rs`) into a new `pub struct MainWindowCallbacks`. Derive nothing â€” use `Default` impl with all fields `None`. Include all `with_*` builder methods currently on `MainWindow` as methods on `MainWindowCallbacks`.

  ```rust
  // crates/nucleus-ui/src/callbacks.rs
  use std::sync::Arc;
  use crate::models::HandlerItem;

  #[derive(Default)]
  pub struct MainWindowCallbacks {
      pub on_toggle_favorite: Option<Arc<dyn Fn(&str, bool) + Send + Sync>>,
      pub on_save_profile: Option<Arc<dyn Fn(&HandlerItem) + Send + Sync>>,
      pub on_delete_save_path: Option<Arc<dyn Fn(&str) + Send + Sync>>,
      pub on_remove_game: Option<Arc<dyn Fn(&str) + Send + Sync>>,
      #[allow(clippy::type_complexity)]
      pub on_launch: Option<Arc<dyn Fn(&str, usize, &str, &std::collections::HashMap<String,String>, &[String]) + Send + Sync>>,
      #[allow(clippy::type_complexity)]
      pub on_add_game: Option<Arc<dyn Fn(&str) -> Option<HandlerItem> + Send + Sync>>,
      #[allow(clippy::type_complexity)]
      pub on_search_hub: Option<Arc<dyn Fn(&str) -> Vec<crate::models::HubPackageItem> + Send + Sync>>,
      #[allow(clippy::type_complexity)]
      pub on_install_handler: Option<Arc<dyn Fn(&str, Box<dyn Fn(f32) + Send + Sync>) -> Option<HandlerItem> + Send + Sync>>,
      pub on_handlers_dir_changed: Option<Arc<dyn Fn(&str) -> Vec<HandlerItem> + Send + Sync>>,
      pub on_theme_changed: Option<Arc<dyn Fn(&str) + Send + Sync>>,
      pub on_auto_scan_changed: Option<Arc<dyn Fn(bool) + Send + Sync>>,
      #[allow(clippy::type_complexity)]
      pub on_list_profiles: Option<Arc<dyn Fn(&str) -> Vec<(usize, String)> + Send + Sync>>,
      #[allow(clippy::type_complexity)]
      pub on_load_numbered_profile: Option<Arc<dyn Fn(&str, usize) -> Option<nucleus_profile::GameProfile> + Send + Sync>>,
      #[allow(clippy::type_complexity)]
      pub on_save_numbered_profile: Option<Arc<dyn Fn(&str, Option<usize>, &nucleus_profile::GameProfile) -> Option<usize> + Send + Sync>>,
      pub on_delete_numbered_profile: Option<Arc<dyn Fn(&str, usize) -> bool + Send + Sync>>,
      pub on_unload_profile: Option<Arc<dyn Fn() + Send + Sync>>,
      pub on_toggle_freeze: Option<Arc<dyn Fn() -> Option<bool> + Send + Sync>>,
      pub on_reassign_controller: Option<Arc<dyn Fn(usize, usize) + Send + Sync>>,
      pub on_re_enforce_windows: Option<Arc<dyn Fn() + Send + Sync>>,
      pub on_toggle_input_lock: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
      pub on_stop_session: Option<Arc<dyn Fn() + Send + Sync>>,
      pub on_check_session_active: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
      pub on_toggle_osd_topmost: Option<Arc<dyn Fn(bool) + Send + Sync>>,
  }

  impl MainWindowCallbacks {
      pub fn with_launcher<F>(mut self, f: F) -> Self where F: Fn(&str, usize, &str, &std::collections::HashMap<String,String>, &[String]) + Send + Sync + 'static { self.on_launch = Some(Arc::new(f)); self }
      // ... (one with_* builder per field, same pattern)
  }
  ```

- [x] **Step 2: Replace `MainWindow` callback fields with `pub callbacks: MainWindowCallbacks`**

  In `window.rs`:
  - Remove all 20 `pub on_*` fields.
  - Add `pub callbacks: MainWindowCallbacks`.
  - In `MainWindow::new`, set `callbacks: MainWindowCallbacks::default()`.
  - Add `pub fn with_callbacks(mut self, cbs: MainWindowCallbacks) -> Self { self.callbacks = cbs; self }`.
  - Remove all 20 `with_*` methods from `MainWindow`.
  - Update every call site in `window.rs` from `self.on_*` â†’ `self.callbacks.on_*`.

- [x] **Step 3: Verify**

  Run: `cargo check -p nucleus-ui -j4`
  Expected: PASS, 0 errors.

  Run: `cargo check -p nucleus-app -j4`
  Expected: PASS (nucleus-app creates MainWindow then chains `.with_callbacks(MainWindowCallbacks::default().with_launcher(...).with_...)`).

- [x] **Step 4: Commit**

  ```
  git add crates/nucleus-ui/src/callbacks.rs crates/nucleus-ui/src/window.rs crates/nucleus-ui/src/lib.rs
  git commit -m "refactor(ui): extract MainWindowCallbacks builder struct from MainWindow"
  ```

---

### Task 2: Extract `render_title_bar` â†’ `CouchTitleBar` Meta-Component

> **Prerequisite:** None. Can run in parallel with Task 3, 4, 5.

**Files:**
- Create: `crates/nucleus-ui/src/components/title_bar.rs`
- Modify: `crates/nucleus-ui/src/components/mod.rs` (add `pub mod title_bar; pub use title_bar::CouchTitleBar;`)
- Modify: `crates/nucleus-ui/src/window.rs` (`render_title_bar` â†’ delegates to `CouchTitleBar`)

**Interfaces:**
- Produces: `pub struct CouchTitleBar` with builder API: `.selected_game(Option<&str>)`, `.is_maximized(bool)`, `.on_toggle_big_picture(handler)`, `.on_minimize(handler)`, `.on_maximize(handler)`, `.on_close(handler)`. Implements `IntoElement` â†’ `AnyElement`.

- [x] **Step 1: Create `components/title_bar.rs`**

  Extract the `render_title_bar` body (lines 1854â€“2064 of `window.rs`) into `CouchTitleBar::into_any_element()`. The CSD title bar contains: drag region, branding badge ("NC"), title text, Big Picture toggle button, minimize/maximize/close window controls.

  Rules:
  - Window control buttons (`â€”`, `â–¡`, `âœ•`) MUST remain raw `div()` elements (they call GPUI Window methods, not CouchButton which accepts App callbacks).
  - The Big Picture toggle button MUST become `CouchButton::ghost("btn-toggle-big-picture", "ðŸŽ® Big Picture").on_click(self.on_toggle_big_picture)`.
  - Preserve the `#[cfg(target_os = "windows")]` unsafe `PostMessageW`/`IsZoomed` blocks verbatim â€” do not simplify or remove them.
  - Return type is `AnyElement`.

- [x] **Step 2: Replace `render_title_bar` in `window.rs`**

  ```rust
  fn render_title_bar(&self, _window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
      let is_maximized = /* existing cfg block */;
      CouchTitleBar::new()
          .selected_game(self.selected_game.as_deref())
          .is_maximized(is_maximized)
          .on_toggle_big_picture(cx.listener(|this, _, _, cx| this.toggle_big_picture(cx)))
          .on_minimize(cx.listener(|_, _, window, _| window.minimize_window()))
          .on_maximize(cx.listener(|_, _, window, cx| { /* existing PostMessageW block */ }))
          .on_close(cx.listener(|_, _, window, _| window.remove_window()))
          .into_any_element()
  }
  ```

- [x] **Step 3: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4 && cargo build -j4 --bin nucleus-coop
  git add crates/nucleus-ui/src/components/title_bar.rs crates/nucleus-ui/src/components/mod.rs crates/nucleus-ui/src/window.rs
  git commit -m "refactor(ui): extract CouchTitleBar meta-component from window.rs render_title_bar"
  ```

---

### Task 3: Extract `render_sidebar` â†’ `CouchSidebarNav` Meta-Component

> **Prerequisite:** None. Can run in parallel with Task 2, 4, 5.

**Files:**
- Create: `crates/nucleus-ui/src/components/sidebar_nav.rs`
- Modify: `crates/nucleus-ui/src/components/mod.rs`
- Modify: `crates/nucleus-ui/src/window.rs` (`render_sidebar`)

**Interfaces:**
- Produces: `pub struct CouchSidebarNav` with: `.active_tab(ActiveTab)`, `.detected_devices(usize)`, `.handler_count(usize)`, `.is_collapsed(bool)`, `.on_tab_click(handler)`, `.on_toggle_collapse(handler)`. Returns `AnyElement`.

- [x] **Step 1: Create `components/sidebar_nav.rs`**

  Move the body of `render_sidebar` (lines 2067â€“2246 of `window.rs`) into `CouchSidebarNav::into_any_element()`.

  The sidebar contains:
  - Branding badge + app name + version text (collapsed/expanded variants).
  - Collapse toggle button (uses `Â«`/`Â»` glyphs, raw `div()` is fine).
  - 4 navigation tab items â€” use `CouchButton::ghost` for each tab entry, with `.selected(is_active)` state.
  - Bottom stats strip: gamepad dot indicator + handler count.

  Width breakpoints: collapsed = `px(60.0)`, expanded = `px(200.0)`.

- [x] **Step 2: Replace `render_sidebar` in `window.rs`**

  ```rust
  fn render_sidebar(&self, window_width: Pixels, cx: &mut Context<Self>) -> AnyElement {
      let is_compact = window_width < px(960.0);
      let is_collapsed = self.sidebar_collapsed.unwrap_or(is_compact);
      CouchSidebarNav::new()
          .active_tab(self.active_tab)
          .detected_devices(self.detected_devices)
          .handler_count(self.handlers.len())
          .is_collapsed(is_collapsed)
          .on_tab_click(cx.listener(|this, tab, _, cx| {
              this.active_tab = tab;
              if tab == ActiveTab::Setup { this.refresh_saved_profiles(); }
              cx.notify();
          }))
          .on_toggle_collapse(cx.listener(move |this, _, _, cx| {
              let cur = this.sidebar_collapsed.unwrap_or(is_compact);
              this.sidebar_collapsed = Some(!cur);
              cx.notify();
          }))
          .into_any_element()
  }
  ```

- [x] **Step 3: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4 && cargo build -j4 --bin nucleus-coop
  git add crates/nucleus-ui/src/components/sidebar_nav.rs crates/nucleus-ui/src/components/mod.rs crates/nucleus-ui/src/window.rs
  git commit -m "refactor(ui): extract CouchSidebarNav meta-component from window.rs render_sidebar"
  ```

---

### Task 4: Extract `render_status_bar` â†’ `CouchStatusBar` Meta-Component

> **Prerequisite:** None. Can run in parallel with Tasks 2, 3, 5.

**Files:**
- Create: `crates/nucleus-ui/src/components/status_bar.rs`
- Modify: `crates/nucleus-ui/src/components/mod.rs`
- Modify: `crates/nucleus-ui/src/window.rs`

**Interfaces:**
- Produces: `pub struct CouchStatusBar` with: `.message(&str)`, `.detected_devices(usize)`, `.session_active(bool)`. Returns `AnyElement`.

- [x] **Step 1: Create `components/status_bar.rs`**

  Extract lines 2249â€“2298 of `window.rs`. The status bar shows: status dot + message (truncated, `flex_1().min_w_0()`), session state, gamepad count, platform label. The `"Session: Idle"` string must become `if session_active { "Session: Active" } else { "Session: Idle" }`.

- [x] **Step 2: Replace `render_status_bar` in `window.rs`**

  ```rust
  fn render_status_bar(&self) -> AnyElement {
      CouchStatusBar::new()
          .message(&self.status_message)
          .detected_devices(self.detected_devices)
          .session_active(self.is_session_active())
          .into_any_element()
  }
  ```

- [x] **Step 3: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4
  git add crates/nucleus-ui/src/components/status_bar.rs crates/nucleus-ui/src/components/mod.rs crates/nucleus-ui/src/window.rs
  git commit -m "refactor(ui): extract CouchStatusBar, fix session_active string"
  ```

---

### Task 5: Extract `render_osd_overlay` and `render_hotkey_recording_modal` into Component Functions

> **Prerequisite:** None. Can run in parallel with Tasks 2, 3, 4.

**Files:**
- Create: `crates/nucleus-ui/src/osd.rs`
- Modify: `crates/nucleus-ui/src/lib.rs`
- Modify: `crates/nucleus-ui/src/window.rs`

**Interfaces:**
- Produces: `pub fn render_osd_overlay(text: &str, color: Hsla) -> AnyElement` (stateless).
- Produces: `pub fn render_hotkey_recording_modal(target_name: &str, focus_handle: &FocusHandle, cx: &mut Context<MainWindow>) -> AnyElement`.

- [x] **Step 1: Create `osd.rs`**

  Move `render_osd_overlay` (lines 1615â€“1650) and `render_hotkey_recording_modal` (lines 1709â€“1851) out of `window.rs` into free functions in `osd.rs`. These are purely presentational â€” they accept only primitive data, not `&self`.

  For `render_hotkey_recording_modal`: the `cancel` on_click listener must be passed in as a `Box<dyn Fn(&mut Window, &mut App)>` parameter (do not capture `cx`).

- [x] **Step 2: Update `window.rs` call sites**

  ```rust
  fn render_osd_overlay(&self) -> Option<AnyElement> {
      if let Some((ref text, start, dur)) = self.osd_message {
          if start.elapsed() < dur {
              let color = parse_osd_color(&self.app_settings.osd_color);
              return Some(crate::osd::render_osd_overlay(text, color));
          }
      }
      None
  }
  ```

- [x] **Step 3: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4
  git add crates/nucleus-ui/src/osd.rs crates/nucleus-ui/src/window.rs crates/nucleus-ui/src/lib.rs
  git commit -m "refactor(ui): extract OSD overlay and hotkey recording modal to osd.rs"
  ```

---

## Phase 2 â€” Context Menu, Confirm Dialog & Details Modal Extraction

> Phase 2 tasks may begin as soon as Phase 1 Tasks 1â€“5 are merged.

### Task 6: Extract Context Menu into `CouchContextMenu` Component

**Files:**
- Create: `crates/nucleus-ui/src/components/context_menu.rs`
- Modify: `crates/nucleus-ui/src/components/mod.rs`
- Modify: `crates/nucleus-ui/src/window.rs` (the ~350-line context menu render block inside `Render::render`)

**Interfaces:**
- Consumes: `MainWindowCallbacks` (Task 1).
- Produces: `pub struct CouchContextMenu` with: `.handler(HandlerItem)`, `.on_launch(handler)`, `.on_favorite(handler)`, `.on_details(handler)`, `.on_remove(handler)`, `.on_close(handler)`. Returns `AnyElement` via `IntoElement`.

- [x] **Step 1: Locate context menu render block in `window.rs`**

  Search for `context_menu_handler` in `window.rs`. The context menu block is an `if let Some(ref h) = self.context_menu_handler { ... }` block inside `Render::render`. It includes: backdrop, card, action rows.

- [x] **Step 2: Create `components/context_menu.rs`**

  Move the entire block into `CouchContextMenu::into_any_element()`. All action buttons in the menu MUST use `CouchButton::ghost(...)`. The backdrop `div` must stop propagation on both `MouseButton::Left` and `MouseButton::Right` (per safety rule 6).

- [x] **Step 3: Replace in `window.rs`**

  ```rust
  // Inside Render::render, where context_menu_handler renders:
  if let Some(ref h) = self.context_menu_handler {
      Some(CouchContextMenu::new(h.clone())
          .on_close(cx.listener(|this, _, _, cx| {
              this.context_menu_handler = None;
              cx.notify();
          }))
          .on_launch(cx.listener(...))
          .on_favorite(cx.listener(...))
          .on_details(cx.listener(...))
          .on_remove(cx.listener(...))
          .into_any_element())
  } else { None }
  ```

- [x] **Step 4: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4 && cargo build -j4 --bin nucleus-coop
  git add crates/nucleus-ui/src/components/context_menu.rs crates/nucleus-ui/src/components/mod.rs crates/nucleus-ui/src/window.rs
  git commit -m "refactor(ui): extract CouchContextMenu component from window.rs Render::render"
  ```

---

### Task 7: Extract Confirm Dialog into `CouchConfirmDialog` Component

**Files:**
- Create: `crates/nucleus-ui/src/components/confirm_dialog.rs`
- Modify: `crates/nucleus-ui/src/components/mod.rs`
- Modify: `crates/nucleus-ui/src/window.rs`

**Interfaces:**
- Produces: `pub struct CouchConfirmDialog` wrapping `CouchModal` with: `.state(&ConfirmDialogState)`, `.on_confirm(handler)`, `.on_cancel(handler)`. Returns `AnyElement`.
- `ConfirmDialogState` already defined in `models.rs` â€” no change needed there.

- [x] **Step 1: Create `components/confirm_dialog.rs`**

  The confirm dialog render block appears inside `Render::render` in `window.rs`. Extract to a single `CouchConfirmDialog` that uses `CouchModal` internally with `ModalWidth::Sm`. Footer actions must be `CouchButton::danger` (confirm) and `CouchButton::secondary` (cancel).

- [x] **Step 2: Replace all `confirm_dialog` render sites in `window.rs`**

  ```rust
  if let Some(ref state) = self.confirm_dialog {
      Some(CouchConfirmDialog::new(state)
          .on_confirm(cx.listener(|this, _, _, cx| {
              if let Some(action) = this.confirm_dialog.take().map(|d| d.action) {
                  this.execute_confirm_action(action, cx);
              }
          }))
          .on_cancel(cx.listener(|this, _, _, cx| {
              this.confirm_dialog = None;
              cx.notify();
          }))
          .into_any_element())
  } else { None }
  ```

- [x] **Step 3: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4
  git add crates/nucleus-ui/src/components/confirm_dialog.rs crates/nucleus-ui/src/components/mod.rs crates/nucleus-ui/src/window.rs
  git commit -m "refactor(ui): extract CouchConfirmDialog, deduplicate confirm_dialog render sites"
  ```

---

### Task 8: Extract Handler Details Modal from `window.rs`

**Files:**
- Modify: `crates/nucleus-ui/src/window.rs` (`render_details_modal` for the local handler branch)
- Create: `crates/nucleus-ui/src/handler_details.rs`
- Modify: `crates/nucleus-ui/src/lib.rs`

**Interfaces:**
- Produces: `pub fn render_handler_details_modal(h: &HandlerItem, cx: &mut Context<MainWindow>) -> AnyElement` in `handler_details.rs`.

- [x] **Step 1: Create `handler_details.rs`**

  Move the `else if let Some(ref h) = self.inspecting_handler` branch of `render_details_modal` (~lines 2319â€“2600 of `window.rs`) into `render_handler_details_modal`. It must use `CouchModal::new(...).body(...).action(...).on_close(...)`. Cover image fallback must use `GameCover`.

- [x] **Step 2: Update `render_details_modal` in `window.rs`**

  ```rust
  fn render_details_modal(&self, cx: &mut Context<Self>) -> Option<AnyElement> {
      if let Some(ref pkg) = self.inspecting_hub_package {
          Some(crate::hub_details::render_hub_details_screen(...))
      } else if let Some(ref h) = self.inspecting_handler {
          Some(crate::handler_details::render_handler_details_modal(h, cx))
      } else {
          None
      }
  }
  ```

- [x] **Step 3: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4
  git add crates/nucleus-ui/src/handler_details.rs crates/nucleus-ui/src/lib.rs crates/nucleus-ui/src/window.rs
  git commit -m "refactor(ui): extract handler details modal to handler_details.rs"
  ```

---

## Phase 3 â€” Settings View Component-ification

> Can start in parallel with Phase 2.

### Task 9: Replace Raw `div()` Buttons in `settings_view.rs` with `CouchButton`

**Files:**
- Modify: `crates/nucleus-ui/src/settings_view.rs` (all free-function render helpers)

**Scope:** `settings_view.rs` has 3,196 lines. The goal is NOT to restructure the file â€” only to swap every raw `div().id(...).cursor_pointer()...on_click(...)` button pattern with the equivalent `CouchButton::*` call.

**Interfaces:**
- Consumes: `CouchButton`, `CouchTag`, `CouchOptionRow`, `CouchModal` from `crate::components::*`.
- Produces: Same rendered output with consistent styling.

- [x] **Step 1: Identify and catalogue all raw button patterns**

  Grep `settings_view.rs` for `cursor_pointer()`. Each `div()` with `.cursor_pointer()` and `.on_click(...)` that resembles a button is a candidate. Catalogue as:
  - Save / apply buttons â†’ `CouchButton::primary`
  - Reset / cancel â†’ `CouchButton::secondary`
  - Danger (delete, reset-to-default) â†’ `CouchButton::danger`
  - Hotkey record buttons â†’ `CouchButton::secondary` with `.icon(IconName::Keyboard)`

- [x] **Step 2: Replace all candidate raw button divs**

  For each identified button, replace the raw `div()` chain with the appropriate `CouchButton::*` constructor. Tooltip calls on `div()` must move to `.tooltip(...)` on `CouchButton`.

  Example â€” hotkey "Record" button:
  ```rust
  // Before:
  div().id("btn-record-lock").cursor_pointer().px_3().py_1p5()...on_click(...).child("âŒ¨ï¸ Record")
  // After:
  CouchButton::secondary("btn-record-lock", "âŒ¨ï¸ Record")
      .icon(IconName::Keyboard)
      .size(CouchButtonSize::Sm)
      .on_click(cx.listener(|this, _, _, cx| { this.settings_view.recording_hotkey = Some("lock".into()); cx.notify(); }))
  ```

- [x] **Step 3: Replace category tab raw divs with `CouchButton::ghost(...).selected(is_active)`**

  The `render_category_bar` function renders 6 category buttons as raw `div()` chains. Replace each with:
  ```rust
  CouchButton::ghost(("settings-cat", idx), cat.label())
      .selected(active_cat == cat)
      .on_click(cx.listener(move |this, _, _, cx| { this.settings_view.active_category = cat; cx.notify(); }))
  ```

- [x] **Step 4: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4 && cargo build -j4 --bin nucleus-coop
  git add crates/nucleus-ui/src/settings_view.rs
  git commit -m "refactor(ui/settings): replace raw div buttons with CouchButton throughout settings_view.rs"
  ```

---

### Task 10: Replace Raw `div()` Setting Rows in `settings_view.rs` with `CouchOptionRow`

**Files:**
- Modify: `crates/nucleus-ui/src/settings_view.rs`

**Scope:** Any two-column row in settings that has a text label on the left and a toggling control (prev/next value, cycle button, or switch) on the right is a candidate for `CouchOptionRow`.

- [x] **Step 1: Identify toggle/cycle rows**

  Look for patterns in `settings_view.rs` matching: left label `div()` + right cycling button(s). These appear in Hotkeys tab (shortcut value display), Players tab (Steam ID/nickname cycling), and Layout tab (merger settings).

- [x] **Step 2: Replace with `CouchOptionRow`**

  ```rust
  // Example: OSD color cycle row
  CouchOptionRow::new(
      "row-osd-color",
      "OSD Color",
      "Color of the on-screen display banner text",
      &current_osd_color,
  )
  .on_prev(cx.listener(|this, _, _, cx| { /* cycle prev */ cx.notify(); }))
  .on_next(cx.listener(|this, _, _, cx| { /* cycle next */ cx.notify(); }))
  ```

- [x] **Step 3: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4
  git add crates/nucleus-ui/src/settings_view.rs
  git commit -m "refactor(ui/settings): replace raw two-column option rows with CouchOptionRow"
  ```

---

## Phase 4 â€” Library View & Game Card Modernization

### Task 11: Promote `GameCardComponent` to a Full Render Component

**Files:**
- Modify: `crates/nucleus-ui/src/game_card.rs` (add `render_game_card` function)
- Modify: `crates/nucleus-ui/src/game_list.rs` (replace `render_game_card_item` with delegation to `game_card.rs`)

**Interfaces:**
- Produces: `pub fn render_game_card(h: &HandlerItem, is_selected: bool, cx: &mut Context<MainWindow>) -> AnyElement` in `game_card.rs`.
- Consumes: `CouchTag`, `CouchButton`, `GameCover` from `crate::components::*`.

- [x] **Step 1: Move `render_game_card_item` body from `game_list.rs` to `game_card.rs`**

  The function `render_game_card_item` (lines 529â€“end of `game_list.rs`) should live in `game_card.rs` as `pub fn render_game_card`. It already uses `CouchTag` and `GameCover` â€” keep those. Replace the action button `div()` chains at the bottom with `CouchButton::*`.

  Action buttons on each card:
  - Installed â†’ `CouchButton::primary("btn-launch-{guid}", "â–¶ Launch")` + `CouchButton::ghost("btn-configure-{guid}", "âš™ Configure")`
  - Uninstalled â†’ `CouchButton::secondary("btn-locate-{guid}", "ðŸ“‚ Locate .exe")`

- [x] **Step 2: Update `game_list.rs` to call `game_card::render_game_card`**

  Replace the inline `render_game_card_item` body with:
  ```rust
  fn render_game_card_item(idx: usize, h: &HandlerItem, selected_game: Option<&str>, cx: &mut Context<MainWindow>) -> AnyElement {
      let is_selected = selected_game == Some(h.game_name.as_str()) || selected_game == Some(h.guid.as_str());
      crate::game_card::render_game_card(h, is_selected, cx)
  }
  ```

- [x] **Step 3: Replace filter chip raw divs with `CouchTag`**

  In `render_library_view`, the filter chips loop uses raw `div()` with manual active/inactive styling. Replace each chip with:
  ```rust
  div()
      .id(("lib-filter", idx))
      .cursor_pointer()
      .on_click(cx.listener(move |this, _, _, cx| { /* toggle filter */ }))
      .child(CouchTag::new(filter_chip).tone(if is_active { TagTone::Accent } else { TagTone::Neutral }))
  ```

- [x] **Step 4: Replace sort tab raw divs with `CouchButton::ghost(...).selected()`**

  ```rust
  CouchButton::ghost(("lib-sort-opt", idx), label)
      .selected(is_active)
      .on_click(cx.listener(move |this, _, _, cx| { this.library_sort = sort_opt; cx.notify(); }))
  ```

- [x] **Step 5: Replace custom scrollbar with `gpui_component::scroll::Scrollbar`**

  The hand-rolled scrollbar (`div().id("library-custom-scrollbar")...`) lines 492â€“521 should be replaced with the gpui-kit `Scrollbar` component. Verify import path at `https://gpui-kit.com/component/scroll.md` before writing.

- [x] **Step 6: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4 && cargo build -j4 --bin nucleus-coop
  git add crates/nucleus-ui/src/game_card.rs crates/nucleus-ui/src/game_list.rs
  git commit -m "refactor(ui/library): promote GameCard, componentify filter chips, sort tabs, scrollbar"
  ```

---

## Phase 5 â€” Setup Screen & Profile Management Componentification

### Task 12: Replace Raw `div()` Profile Buttons in `setup_screen.rs`

**Files:**
- Modify: `crates/nucleus-ui/src/setup_screen.rs`

- [x] **Step 1: Replace profile management button divs**

  - `âž• Save New Profile` â†’ `CouchButton::primary("btn-save-new-profile", "âž• Save New Profile")`
  - `ðŸ’¾ Update Profile N` â†’ `CouchButton::secondary(("btn-update-profile", cur_id), format!("ðŸ’¾ Update Profile {}", cur_id))`
  - `Unload Profile` â†’ `CouchButton::danger("btn-unload-profile", "Unload Profile")`

- [x] **Step 2: Replace profile chip raw divs with `CouchTag` wrappers**

  Profile chips (lines 237â€“300) are raw styled `div()` containers. Replace each chip container with a `div().id(("profile-chip", prof_id)).cursor_pointer().on_click(...).child(CouchTag::new(...).tone(...))`.

- [x] **Step 3: Replace player count stepper buttons**

  Locate `+`/`-` player count buttons. Replace with `CouchButton::ghost("btn-player-count-dec", "âˆ’").size(CouchButtonSize::Sm)` and `CouchButton::ghost("btn-player-count-inc", "+").size(CouchButtonSize::Sm)`.

- [x] **Step 4: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4 && cargo build -j4 --bin nucleus-coop
  git add crates/nucleus-ui/src/setup_screen.rs
  git commit -m "refactor(ui/setup): replace raw div buttons and profile chips with CouchButton/CouchTag"
  ```

---

## Phase 6 â€” Big Picture & Hub Componentification

### Task 13: Componentify Big Picture Carousel Action Buttons

**Files:**
- Modify: `crates/nucleus-ui/src/big_picture/carousel.rs`

- [x] **Step 1: Replace carousel navigation arrows and prompt buttons**

  - Navigation arrows â†’ `CouchButton::ghost("btn-carousel-left", "â—€")`, `CouchButton::ghost("btn-carousel-right", "â–¶")`.
  - Gamepad prompt row items â†’ `CouchButton::ghost(...)` with `CouchButtonSize::Lg`.
  - Filter mode chips â†’ `CouchTag::new(filter.label()).id(("bp-filter", idx)).tone(...)`.

- [x] **Step 2: Replace carousel card uninstalled overlay with `CouchTag`**

  ```rust
  CouchTag::new("Uninstalled").tone(TagTone::Warning)
  ```

- [x] **Step 3: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4 && cargo build -j4 --bin nucleus-coop
  git add crates/nucleus-ui/src/big_picture/carousel.rs
  git commit -m "refactor(ui/carousel): replace raw div buttons and chips with CouchButton/CouchTag"
  ```

---

### Task 14: Componentify Couch Hub & Setup Screens

**Files:**
- Modify: `crates/nucleus-ui/src/big_picture/hub.rs`
- Modify: `crates/nucleus-ui/src/big_picture/setup.rs`

- [x] **Step 1: Replace hub action buttons in `hub.rs`**

  - `"ðŸŽ® Play"` â†’ `CouchButton::primary("btn-hub-play-{id}", "ðŸŽ® Play").size(CouchButtonSize::Lg)`
  - `"â¬‡ Install"` â†’ `CouchButton::secondary("btn-hub-install-{id}", "â¬‡ Install").size(CouchButtonSize::Lg)`
  - `"âœ“ Installed"` â†’ `CouchButton::success("btn-hub-installed-{id}", "âœ“ Installed").disabled(true).size(CouchButtonSize::Lg)`
  - Category filter chips â†’ `CouchTag::new(cat.label()).id(("hub-cat", idx)).tone(...)`

- [x] **Step 2: Replace setup screen action buttons in `setup.rs`**

  `Launch` and `Back` buttons, layout cycle button â†’ `CouchButton::*` of `CouchButtonSize::Lg` for couch mode.

- [x] **Step 3: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4 && cargo build -j4 --bin nucleus-coop
  git add crates/nucleus-ui/src/big_picture/hub.rs crates/nucleus-ui/src/big_picture/setup.rs
  git commit -m "refactor(ui/bigpicture): replace raw div buttons in hub.rs and setup.rs"
  ```

---

## Phase 7 â€” Visual Enhancements & UX Improvements

### Task 15: Animated Progress Bar for Hub Install Cards

**Files:**
- Modify: `crates/nucleus-ui/src/big_picture/hub.rs`
- Modify: `crates/nucleus-ui/src/hub_browser.rs`

- [x] **Step 1: Import and use smooth progress indicator**

  Verify import path at `https://gpui-kit.com/component/progress.md`. Wherever `install_progress: Some(p)`:
  ```rust
  div().flex_col().gap_1().w_full()
      .child(div().text_xs().text_color(theme::text_muted()).child(format!("Downloading... {:.0}%", p * 100.0)))
      .child(Progress::new().value(p).into_any_element())
  ```

- [x] **Step 2: Apply same pattern to desktop `hub_browser.rs`**

- [x] **Step 3: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4 && cargo build -j4 --bin nucleus-coop
  git add crates/nucleus-ui/src/big_picture/hub.rs crates/nucleus-ui/src/hub_browser.rs
  git commit -m "feat(ui): animated Progress bar for hub handler downloads"
  ```

---

### Task 16: Skeleton Loading States for Library and Hub

**Files:**
- Modify: `crates/nucleus-ui/src/game_list.rs`
- Modify: `crates/nucleus-ui/src/hub_browser.rs`

- [x] **Step 1: Setup skeleton loading state structure**

  Verify import at `https://gpui-kit.com/component/skeleton.md`.

- [x] **Step 2: Show 5 skeleton rows in library when empty and not filtered**

  ```rust
  // When handlers.is_empty() && search_query.is_empty() && selected_filters.is_empty():
  div().flex_col().gap_3()
      .children((0..5).map(|i| {
          div().w_full().h(px(108.0)).pb_3()
              .child(Skeleton::new().id(("lib-skeleton", i)).w_full().h(px(108.0)).rounded_lg())
      }))
      .into_any_element()
  ```

- [x] **Step 3: Show 6 skeleton grid in hub when `hub_loading == true`**

- [x] **Step 4: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4
  git add crates/nucleus-ui/src/game_list.rs crates/nucleus-ui/src/hub_browser.rs
  git commit -m "feat(ui): Skeleton loading states for library and hub browser"
  ```

---

### Task 17: Tooltip Coverage for Icon-Only and Ambiguous Controls

**Files:**
- Modify: `crates/nucleus-ui/src/components/title_bar.rs`
- Modify: `crates/nucleus-ui/src/components/sidebar_nav.rs`
- Modify: `crates/nucleus-ui/src/game_card.rs`
- Modify: `crates/nucleus-ui/src/big_picture/carousel.rs`

- [x] **Step 1: Sidebar nav (collapsed mode) — add tab label tooltips**

  When `is_collapsed`, each `CouchButton::ghost` tab entry must have `.tooltip(label)`.

- [x] **Step 2: Title bar window controls — add tooltips**

  Minimize â†’ `.tooltip("Minimize")`, Maximize â†’ `.tooltip("Maximize / Restore")`, Close â†’ `.tooltip("Close")`.

- [x] **Step 3: Game card action buttons — add tooltips**

  Launch â†’ `.tooltip("Launch splitscreen session")`, Configure â†’ `.tooltip("Open splitscreen setup")`.

- [x] **Step 4: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4
  git add crates/nucleus-ui/src/components/title_bar.rs crates/nucleus-ui/src/components/sidebar_nav.rs crates/nucleus-ui/src/game_card.rs crates/nucleus-ui/src/big_picture/carousel.rs
  git commit -m "feat(ui): comprehensive tooltip coverage for icon-only and ambiguous controls"
  ```

---

### Task 18: Toast Notifications for Key Session Events

**Files:**
- Modify: `crates/nucleus-ui/src/window.rs`

- [ ] **Step 1: Verify `WindowExt` and `Notification` import path**

  Fetch `https://gpui-kit.com/component/notification.md`.

- [ ] **Step 2: Add `push_notification` for handler install, session launch, session stop**

  Only call `window.push_notification` from methods that have `&mut Window` available â€” route through `cx.spawn(async move |this, cx| { this.update(..., |win, cx| { /* push here */ }).ok(); })` if needed.

- [x] **Step 3: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4 && cargo build -j4 --bin nucleus-coop
  git add crates/nucleus-ui/src/window.rs
  git commit -m "feat(ui): push_notification toast for handler install, session start/stop events"
  ```

---

### Task 19: Fix `CouchModal` Scroll and Add Backdrop Dismiss

**Files:**
- Modify: `crates/nucleus-ui/src/components/modal.rs`

- [ ] **Step 1: Fix `overflow_y_scrollbar()` â†’ `overflow_y_scroll()`**

  Line 189 of `modal.rs`: change `.overflow_y_scrollbar()` to `.overflow_y_scroll()`.

- [ ] **Step 2: Wire backdrop `on_mouse_down` to `on_close` callback**

  The backdrop's `on_mouse_down` for `MouseButton::Left` must call `cx.stop_propagation()` AND invoke `self.on_close` if set. The inner card's `on_mouse_down` must ONLY call `cx.stop_propagation()`.

- [x] **Step 3: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4
  git add crates/nucleus-ui/src/components/modal.rs
  git commit -m "fix(ui/modal): fix overflow_y_scrollbar height expansion, add backdrop dismiss support"
  ```

---

### Task 20: Centralize Hardcoded `rgb()` Values into Theme Tokens

**Files:**
- Modify: `crates/nucleus-ui/src/theme.rs`
- Modify: `crates/nucleus-ui/src/components/badge.rs`
- Modify: `crates/nucleus-ui/src/components/button.rs`

- [ ] **Step 1: Add semantic well/text token functions to `theme.rs`**

  ```rust
  pub fn success_bg() -> Hsla { rgb(0x064e3b).into() }
  pub fn success_text() -> Hsla { rgb(0x34d399).into() }
  pub fn warning_bg() -> Hsla { rgb(0x451a03).into() }
  pub fn warning_text() -> Hsla { rgb(0xfbbf24).into() }
  pub fn danger_bg() -> Hsla { rgb(0x4c0519).into() }
  pub fn danger_text() -> Hsla { rgb(0xf43f5e).into() }
  ```

- [ ] **Step 2: Replace all hardcoded `rgb()` in `badge.rs` and `button.rs`**

- [ ] **Step 3: Run theme tests**

  ```
  cargo test -p nucleus-ui -j4
  ```

- [x] **Step 4: Commit**

  ```
  git add crates/nucleus-ui/src/theme.rs crates/nucleus-ui/src/components/badge.rs crates/nucleus-ui/src/components/button.rs
  git commit -m "refactor(ui/theme): centralize hardcoded rgb() values into theme token functions"
  ```

---

## Phase 8 â€” `window.rs` God-Class Final Decomposition

> Phase 8 must start AFTER Phase 1 (Tasks 1â€“5) are complete and merged.

### Task 21: Reduce `Render::render` to â‰¤ 80 Lines

**Files:**
- Modify: `crates/nucleus-ui/src/window.rs`

- [ ] **Step 1: Audit remaining inline blocks in `Render::render`**

  After Phases 1â€“7, any inline block > 20 lines not yet a named function call must be extracted.

- [ ] **Step 2: Extract remaining large inline blocks**

  Likely remaining: Hub tab delegation, focus state management block. Extract focus management to `fn update_focus_state(&mut self, window: &mut Window, cx: &mut Context<Self>)` called from `render`.

- [ ] **Step 3: Final `Render::render` target structure**

  ```rust
  impl Render for MainWindow {
      fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
          if self.is_big_picture {
              return crate::big_picture::render_big_picture(self, window, cx).into_any_element();
          }
          let window_width = window.viewport_size().width;
          div()
              .flex().flex_col().size_full().bg(theme::bg_main())
              .child(self.render_title_bar(window, cx))
              .child(div().flex().flex_row().flex_1().min_h_0()
                  .child(self.render_sidebar(window_width, cx))
                  .child(self.render_tab_content(cx)))
              .child(self.render_status_bar())
              .children(self.render_osd_overlay())
              .children(self.render_hotkey_recording_modal(cx))
              .children(self.render_details_modal(cx))
              .children(self.render_context_menu(cx))
              .children(self.render_confirm_dialog(cx))
              .into_any_element()
      }
  }
  ```

- [ ] **Step 4: Verify line count â‰¤ 80 and commit**

  ```
  cargo check -p nucleus-ui -j4 && cargo test -p nucleus-ui -j4 && cargo build -j4 --bin nucleus-coop
  git add crates/nucleus-ui/src/window.rs
  git commit -m "refactor(ui): final Render::render decomposition â€” â‰¤80 lines, full delegation"
  ```

---

### Task 22: Split `window.rs` into Sub-Modules

**Files:**
- Create: `crates/nucleus-ui/src/window/mod.rs` (rename from `window.rs`)
- Create: `crates/nucleus-ui/src/window/actions.rs`
- Create: `crates/nucleus-ui/src/window/session.rs`
- Create: `crates/nucleus-ui/src/window/devices.rs`
- Create: `crates/nucleus-ui/src/window/hub.rs`

**Interfaces:** All public items on `MainWindow` remain accessible from `crate::window::MainWindow` â€” no public API change.

- [ ] **Step 1: Create `window/` directory, move `window.rs` â†’ `window/mod.rs`**

- [ ] **Step 2: Move `dispatch_couch_action_by_name` to `window/actions.rs`**

- [ ] **Step 3: Move session methods to `window/session.rs`**

  `launch_couch_session`, `stop_session`, `toggle_freeze_session`, `toggle_in_game_osd`, `set_osd_visible`, `handle_osd_confirm`, `re_enforce_windows`, `toggle_input_lock`, `is_session_active`, `show_osd`.

- [ ] **Step 4: Move device methods to `window/devices.rs`**

  `sync_devices`, `sync_player_assignments`, `cycle_player_device`, `add_virtual_gamepad`, `clear_virtual_gamepads`, `restore_ignored_controller`, `tick_disconnect_resilience`, `tick_disconnect_resilience_state`.

- [ ] **Step 5: Move hub methods to `window/hub.rs`**

  `trigger_hub_search_async`, `load_hub_package_details`, `mark_package_installed`, `hub_next_page`, `hub_prev_page`, `download_and_play_couch_package`.

- [ ] **Step 6: Verify line count targets**

  `window/mod.rs` â‰¤ 1,200 lines. Each sub-module â‰¤ 400 lines.

- [ ] **Step 7: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4 && cargo test -p nucleus-ui -j4 && cargo build -j4 --bin nucleus-coop
  git add crates/nucleus-ui/src/window/
  git commit -m "refactor(ui): split window.rs into window/ sub-modules (actions, session, devices, hub)"
  ```

---

## Phase 9 â€” `CouchToggle` â†’ gpui-kit `Switch` Migration

### Task 23: Delegate `CouchToggle` to gpui-kit `Switch`

**Files:**
- Modify: `crates/nucleus-ui/src/components/toggle.rs`

- [ ] **Step 1: Verify `Switch` API** at `https://gpui-kit.com/component/switch.md`

- [ ] **Step 2: Rewrite `CouchToggle` as a thin wrapper over `Switch`**

  Preserve the same builder API (`new`, `checked`, `disabled`, `on_change`, `into_any_element`).

- [ ] **Step 3: Verify all `CouchToggle` call sites still compile**

- [x] **Step 4: Verify and commit**

  ```
  cargo check -p nucleus-ui -j4 && cargo build -j4 --bin nucleus-coop
  git add crates/nucleus-ui/src/components/toggle.rs
  git commit -m "refactor(ui/toggle): delegate CouchToggle to gpui-kit Switch component"
  ```

---

## Phase 10 â€” Verification, Docs, and Cleanup

### Task 24: Update `docs/ui/components_guide.md`

**Files:**
- Create/Modify: `nucleus-coop/docs/ui/components_guide.md`

- [ ] **Step 1: Document all new components** (CouchTitleBar, CouchSidebarNav, CouchStatusBar, CouchContextMenu, CouchConfirmDialog) with builder API, usage examples, and invariants.

- [ ] **Step 2: Document `MainWindowCallbacks` builder pattern** with idiomatic wiring example.

- [ ] **Step 3: Commit**

  ```
  git add docs/ui/components_guide.md
  git commit -m "docs(ui): update components_guide.md with all Phase 1-9 components"
  ```

---

### Task 25: Final Verification Run

- [ ] **Step 1: Full check sequence**

  ```
  cargo check -p nucleus-ui -j4
  cargo check -p nucleus-app -j4
  cargo test -p nucleus-ui -j4
  cargo build -j4 --bin nucleus-coop
  ```

- [ ] **Step 2: Deploy and smoke test**

  Deploy to `C:\Games\NucleusCoop242\nucleus-coop.exe`. Verify:
  - Library tab: handler list, filter chips, sort tabs, scrollbar.
  - Setup tab: partition preview, player slots, profile buttons.
  - Hub tab: package grid, progress bars, Skeleton when loading.
  - Settings tab: all 6 categories with CouchButton controls.
  - Big Picture mode: carousel, couch setup, couch hub.
  - Right-click context menu, confirm dialog, handler details modal.

- [ ] **Step 3: Confirm `recursion_limit` unchanged**

  ```
  grep recursion_limit crates/nucleus-ui/src/lib.rs
  ```
  Expected: `#![recursion_limit = "4096"]`.

- [ ] **Step 4: Final commit**

  ```
  git commit --allow-empty -m "chore(ui): Phase 1-10 refactor complete â€” all checks passing"
  ```

---

## Execution Notes for Subagent Workers

Each task should be executed by a fresh subagent with these pre-reads:
1. `.agents/skills/gpui-ui-safety/SKILL.md` (all 17 rules â€” mandatory)
2. `crates/nucleus-ui/src/components/mod.rs` (component catalogue)
3. The specific files listed in the task's **Files** section
4. This plan document (for interface contracts and global constraints)

**Parallel execution is safe within the same phase.** Tasks in different phases with explicit prerequisite dependencies (noted in `Interfaces: Consumes`) must wait.

**When a subagent discovers an issue not covered by this plan:** report it via `send_message` to the orchestrating agent rather than making ad-hoc changes to files outside the task's **Files** list.

**Model recommendation:** Flash 3.8 (High) subagents are suitable for all tasks. Tasks 1 and 22 (interface/module design) may benefit from a stronger model if the subagent encounters ambiguity.
