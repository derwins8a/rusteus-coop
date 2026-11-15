//! Fullscreen In-Game Couch OSD overlay modal and action execution.
//!
//! Provides a 10-foot gamepad-navigable overlay displayed across the physical screen
//! on top of active split-screen game sessions. Includes process freeze/unfreeze
//! (NtSuspendProcess / NtResumeProcess), controller reassignment for hot-swapping dead batteries,
//! window bounds re-enforcing, and raw input lock toggling.
//!
//! Built on the Nefos UI design system with zero emojis, pure vector icons, and dynamic
//! summoner-tinted gamepad focus rings.

use gpui::prelude::*;
use gpui::*;

use crate::theme;
use crate::window::MainWindow;
use crate::{Icon, IconName};

/// Actions available within the In-Game Couch OSD menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsdAction {
    ResumeGame,
    ToggleFreeze,
    ReassignControllers,
    DropInPlayer,
    DropOutPlayer,
    ReEnforceWindows,
    ToggleInputLock,
    StopSession,
}

impl OsdAction {
    /// Ordered list of all OSD menu actions.
    pub const ALL: [OsdAction; 8] = [
        OsdAction::ResumeGame,
        OsdAction::ToggleFreeze,
        OsdAction::ReassignControllers,
        OsdAction::DropInPlayer,
        OsdAction::DropOutPlayer,
        OsdAction::ReEnforceWindows,
        OsdAction::ToggleInputLock,
        OsdAction::StopSession,
    ];

    /// Total number of OSD menu items.
    pub const fn count() -> usize {
        Self::ALL.len()
    }
}

/// Canonical split layouts supported for a given active player count,
/// matching `big_picture::setup::AVAILABLE_LAYOUTS`.
pub fn valid_layouts_for_player_count(player_count: usize) -> Vec<&'static str> {
    match player_count {
        0 | 1 => vec!["Fullscreen (1 Player)"],
        2 => vec!["2x1 (Side-by-Side)", "1x2 (Stacked)", "Multi-Monitor"],
        3 => vec![
            "Top 1 / Bottom 2",
            "Top 2 / Bottom 1",
            "2x2 (Grid)",
            "Multi-Monitor",
        ],
        4 => vec![
            "2x2 (Grid)",
            "2x1 (Side-by-Side)",
            "1x2 (Stacked)",
            "Multi-Monitor",
        ],
        5..=6 => vec!["3x2 (Grid 6)", "4x2 (Grid 8)", "Multi-Monitor"],
        7..=8 => vec!["4x2 (Grid 8)", "4x4 (Grid 16)", "Multi-Monitor"],
        _ => vec!["4x4 (Grid 16)", "Multi-Monitor"],
    }
}

/// Active player slot displayed in the In-Game OSD overlay and Drop-Out picker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsdActivePlayer {
    pub slot_idx: usize,
    pub nickname: String,
    pub device_label: String,
}

/// 2-step Drop-In Player flow inside the In-Game OSD overlay.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DropInStep {
    #[default]
    None,
    PickController {
        focused_idx: usize,
    },
    SelectLayout {
        chosen_device_id: Option<usize>,
        chosen_device_label: String,
        chosen_assignment: String,
        focused_layout_idx: usize,
    },
    Spawning {
        message: String,
    },
}

/// 2-step Drop-Out Player flow inside the In-Game OSD overlay.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DropOutStep {
    #[default]
    None,
    PickSlot {
        focused_idx: usize,
    },
    SelectLayout {
        target_slot_idx: usize,
        focused_layout_idx: usize,
    },
    Applying {
        message: String,
    },
}

/// Active sub-view displayed within the Couch OSD right-hand panel (Image 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OsdSubView {
    #[default]
    Main,
    PlayerDevices,
    GameGuide,
}

/// Formats runtime seconds into `HH:MM:SS` (e.g. `00:12:48`).
pub fn format_duration_hhmmss(seconds: u64) -> String {
    let hours = seconds / 3600;
    let mins = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

/// State tracking the In-Game Couch OSD overlay mid-session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InGameOsdState {
    /// Whether the OSD overlay modal is currently displayed.
    pub is_visible: bool,
    /// Currently focused menu item index (0..OsdAction::count()).
    pub selected_index: usize,
    /// Whether all instance processes are currently frozen/suspended.
    pub is_frozen: bool,
    /// Whether raw input lock is currently active.
    pub input_locked: bool,
    /// Whether the controller reassignment sub-modal is currently open.
    pub reassign_modal_open: bool,
    /// Player slot index targeted for controller reassignment.
    pub reassign_slot_idx: usize,
    /// Replacement device ID chosen for reassignment.
    pub reassign_new_device_id: Option<usize>,
    /// Player slot index of the player who summoned the overlay (if known).
    pub summoner_slot: Option<usize>,
    /// Snapshot of currently active player slots.
    pub active_players: Vec<OsdActivePlayer>,
    /// Maximum supported players for the running handler.
    pub max_players: usize,
    /// Currently active screen split layout name.
    pub current_layout: String,
    /// Multi-step Drop-In submodal state.
    pub drop_in_step: DropInStep,
    /// Multi-step Drop-Out submodal state.
    pub drop_out_step: DropOutStep,
    /// Active sub-view within the Couch OSD menu.
    pub sub_view: OsdSubView,
    /// Real-time session elapsed duration in seconds.
    pub elapsed_seconds: u64,
    /// Error or guard banner message displayed inside the OSD.
    pub action_error: Option<String>,
}

impl Default for InGameOsdState {
    fn default() -> Self {
        Self::new()
    }
}

impl InGameOsdState {
    /// Creates a new inactive OSD state.
    pub fn new() -> Self {
        Self {
            is_visible: false,
            selected_index: 0,
            is_frozen: false,
            input_locked: false,
            reassign_modal_open: false,
            reassign_slot_idx: 0,
            reassign_new_device_id: None,
            summoner_slot: None,
            active_players: Vec::new(),
            max_players: 4,
            current_layout: String::new(),
            drop_in_step: DropInStep::None,
            drop_out_step: DropOutStep::None,
            sub_view: OsdSubView::Main,
            elapsed_seconds: 0,
            action_error: None,
        }
    }

    /// Opens and displays the in-game OSD menu.
    pub fn open(&mut self) {
        self.is_visible = true;
    }

    /// Closes and hides the in-game OSD menu and any sub-modals.
    pub fn close(&mut self) {
        self.is_visible = false;
        self.reassign_modal_open = false;
        self.reassign_new_device_id = None;
        self.summoner_slot = None;
        self.drop_in_step = DropInStep::None;
        self.drop_out_step = DropOutStep::None;
        self.action_error = None;
        self.sub_view = OsdSubView::Main;
    }

    /// Toggles the visibility of the in-game OSD menu.
    pub fn toggle(&mut self) {
        self.is_visible = !self.is_visible;
        if !self.is_visible {
            self.reassign_modal_open = false;
            self.reassign_new_device_id = None;
            self.drop_in_step = DropInStep::None;
            self.drop_out_step = DropOutStep::None;
            self.action_error = None;
            self.sub_view = OsdSubView::Main;
        }
    }

    /// Moves selection up by one item, wrapping around from 0 to `OsdAction::count() - 1`.
    pub fn move_up(&mut self) {
        let last = OsdAction::count().saturating_sub(1);
        if self.selected_index == 0 {
            self.selected_index = last;
        } else {
            self.selected_index -= 1;
        }
    }

    /// Moves selection down by one item, wrapping around from `OsdAction::count() - 1` to 0.
    pub fn move_down(&mut self) {
        let last = OsdAction::count().saturating_sub(1);
        if self.selected_index >= last {
            self.selected_index = 0;
        } else {
            self.selected_index += 1;
        }
    }

    /// Sets the focused menu index, clamping to valid indices (`0..OsdAction::count()`).
    pub fn select(&mut self, idx: usize) {
        let last = OsdAction::count().saturating_sub(1);
        self.selected_index = idx.min(last);
    }

    /// Returns the currently selected `OsdAction`.
    pub fn selected_action(&self) -> OsdAction {
        OsdAction::ALL
            .get(self.selected_index)
            .copied()
            .unwrap_or(OsdAction::ResumeGame)
    }

    /// Synchronizes active slots, capacity, layout, and freeze state from a live engine snapshot.
    pub fn sync_from_snapshot(&mut self, snap: &nucleus_engine::session::ActiveSessionSnapshot) {
        self.active_players = snap
            .slots
            .iter()
            .map(|s| OsdActivePlayer {
                slot_idx: s.slot_idx,
                nickname: s.nickname.clone(),
                device_label: s.device_assignment.clone(),
            })
            .collect();
        self.max_players = snap.max_players.max(snap.slots.len()).max(1);
        self.current_layout = snap.layout.clone();
        self.is_frozen = snap.is_frozen;
        self.elapsed_seconds = snap.elapsed_seconds;
    }

    /// Navigates to the "Player devices" sub-view within the Couch OSD (Image 2).
    pub fn open_player_devices(&mut self) {
        self.sub_view = OsdSubView::PlayerDevices;
        self.action_error = None;
    }

    /// Navigates to the "Read first / Game guide" sub-view within the Couch OSD (Image 2).
    pub fn open_game_guide(&mut self) {
        self.sub_view = OsdSubView::GameGuide;
        self.action_error = None;
    }

    /// Returns to the primary Couch OSD actions menu.
    pub fn back_to_main(&mut self) {
        self.sub_view = OsdSubView::Main;
        self.action_error = None;
    }

    /// Starts the 2-step Drop-In Player flow if capacity allows.
    pub fn open_drop_in(&mut self, active_count: usize, max_players: usize) -> bool {
        if active_count >= max_players {
            self.action_error = Some(format!(
                "Session already at maximum {} player capacity.",
                max_players
            ));
            return false;
        }
        self.action_error = None;
        self.reassign_modal_open = false;
        self.reassign_new_device_id = None;
        self.drop_out_step = DropOutStep::None;
        self.drop_in_step = DropInStep::PickController { focused_idx: 0 };
        true
    }

    /// Records chosen Drop-In input device and advances to Step 2 layout selection.
    pub fn select_drop_in_device(
        &mut self,
        dev_id: Option<usize>,
        label: String,
        assignment: String,
    ) {
        self.action_error = None;
        self.drop_in_step = DropInStep::SelectLayout {
            chosen_device_id: dev_id,
            chosen_device_label: label,
            chosen_assignment: assignment,
            focused_layout_idx: 0,
        };
    }

    /// Starts the 2-step Drop-Out Player flow if at least 2 players are active.
    pub fn open_drop_out(&mut self, summoner_slot: Option<usize>) -> bool {
        if self.active_players.len() <= 1 {
            self.action_error =
                Some("Only 1 active player — use End Game Session to exit.".to_string());
            return false;
        }
        let focused_idx = summoner_slot
            .and_then(|s| self.active_players.iter().position(|p| p.slot_idx == s))
            .unwrap_or(0);
        self.action_error = None;
        self.reassign_modal_open = false;
        self.reassign_new_device_id = None;
        self.drop_in_step = DropInStep::None;
        self.drop_out_step = DropOutStep::PickSlot { focused_idx };
        true
    }

    /// Records chosen Drop-Out player slot and advances to Step 2 survivor layout selection.
    pub fn select_drop_out_slot(&mut self, slot_idx: usize) {
        self.action_error = None;
        self.drop_out_step = DropOutStep::SelectLayout {
            target_slot_idx: slot_idx,
            focused_layout_idx: 0,
        };
    }

    /// Steps back one level or dismisses an open sub-modal.
    /// Returns `true` if a sub-modal state changed, `false` if already at top-level OSD menu.
    pub fn cancel_submodal(&mut self) -> bool {
        self.action_error = None;
        if self.reassign_modal_open {
            self.close_reassign();
            return true;
        }
        match &self.drop_in_step {
            DropInStep::SelectLayout { .. } => {
                self.drop_in_step = DropInStep::PickController { focused_idx: 0 };
                return true;
            }
            DropInStep::PickController { .. } | DropInStep::Spawning { .. } => {
                self.drop_in_step = DropInStep::None;
                return true;
            }
            DropInStep::None => {}
        }
        match &self.drop_out_step {
            DropOutStep::SelectLayout {
                target_slot_idx, ..
            } => {
                let target = *target_slot_idx;
                let focused_idx = self
                    .active_players
                    .iter()
                    .position(|p| p.slot_idx == target)
                    .unwrap_or(0);
                self.drop_out_step = DropOutStep::PickSlot { focused_idx };
                return true;
            }
            DropOutStep::PickSlot { .. } | DropOutStep::Applying { .. } => {
                self.drop_out_step = DropOutStep::None;
                return true;
            }
            DropOutStep::None => {}
        }
        if self.sub_view != OsdSubView::Main {
            self.sub_view = OsdSubView::Main;
            return true;
        }
        false
    }

    /// Opens the controller reassignment sub-modal for the specified player slot.
    pub fn open_reassign(&mut self, slot_idx: usize) {
        self.reassign_modal_open = true;
        self.reassign_slot_idx = slot_idx;
        self.reassign_new_device_id = None;
    }

    /// Closes the controller reassignment sub-modal.
    pub fn close_reassign(&mut self) {
        self.reassign_modal_open = false;
        self.reassign_new_device_id = None;
    }

    /// Confirms and consumes the controller reassignment, returning `(slot_idx, new_device_id)`.
    pub fn confirm_reassign(&mut self) -> Option<(usize, usize)> {
        if self.reassign_modal_open {
            let slot = self.reassign_slot_idx;
            let dev = self.reassign_new_device_id;
            self.reassign_modal_open = false;
            self.reassign_new_device_id = None;
            dev.map(|d| (slot, d))
        } else {
            None
        }
    }
}

/// Returns the dynamic focus accent color associated with the player who summoned the overlay.
pub fn summoner_focus_color(summoner_slot: Option<usize>) -> Hsla {
    match summoner_slot {
        Some(1) => hsla(210.0 / 360.0, 0.9, 0.6, 1.0), // Blue (Player 2)
        Some(2) => hsla(38.0 / 360.0, 0.95, 0.55, 1.0), // Amber (Player 3)
        Some(3) => hsla(270.0 / 360.0, 0.85, 0.65, 1.0), // Purple (Player 4)
        _ => theme::accent(),                          // Emerald / Host Default (Player 1)
    }
}

/// Returns the slot tint color for player index.
pub fn player_slot_tint(slot_idx: usize) -> Hsla {
    match slot_idx % 4 {
        0 => theme::accent(),
        1 => hsla(210.0 / 360.0, 0.9, 0.6, 1.0),
        2 => hsla(38.0 / 360.0, 0.95, 0.55, 1.0),
        _ => hsla(270.0 / 360.0, 0.85, 0.65, 1.0),
    }
}

/// Synchronizes `win.in_game_osd` active player list from `on_query_active_session`
/// or seeds fallback slots from `win.player_count` and `win.player_device_assignments`.
pub fn sync_osd_session_snapshot(win: &mut MainWindow) {
    if let Some(ref querier) = win.callbacks.on_query_active_session {
        if let Some(snap) = querier() {
            win.in_game_osd.sync_from_snapshot(&snap);
            return;
        }
    }
    if win.in_game_osd.active_players.is_empty() && win.player_count > 0 {
        win.in_game_osd.active_players = (0..win.player_count)
            .map(|slot_idx| OsdActivePlayer {
                slot_idx,
                nickname: format!("Player {}", slot_idx + 1),
                device_label: win
                    .player_device_assignments
                    .get(slot_idx)
                    .cloned()
                    .unwrap_or_else(|| format!("Slot {}", slot_idx + 1)),
            })
            .collect();
        win.in_game_osd.current_layout = win.selected_layout.clone();
    }
}

/// Returns assignable controller/input candidates `(device_id, display_label, assignment_string, is_keyboard)`
/// for Drop-In Step 1, excluding devices already bound to active player slots.
pub fn drop_in_candidate_devices(win: &MainWindow) -> Vec<(Option<usize>, String, String, bool)> {
    let active_labels: Vec<String> = if win.in_game_osd.active_players.is_empty() {
        win.player_device_assignments.clone()
    } else {
        win.in_game_osd
            .active_players
            .iter()
            .map(|p| p.device_label.clone())
            .collect()
    };

    let mut candidates = Vec::new();
    for dev in &win.detected_devices_list {
        let already_used = active_labels.iter().any(|lbl| {
            lbl.contains(&format!("#{}", dev.id))
                || lbl.contains(&format!("Controller {}", dev.id))
                || lbl.contains(&format!("Gamepad {}", dev.id))
                || (!dev.name.is_empty() && lbl.contains(&dev.name))
        });
        if !already_used {
            candidates.push((
                Some(dev.id),
                format!("Device #{}: {}", dev.id, dev.name),
                format!("Gamepad #{} ({})", dev.id, dev.name),
                false,
            ));
        }
    }

    candidates.push((
        None,
        "Virtual Gamepad (Mock / Test)".to_string(),
        "Virtual Gamepad".to_string(),
        false,
    ));

    let kb_used = active_labels
        .iter()
        .any(|lbl| lbl.contains("Keyboard") || lbl.contains("ProtoInput"));
    if !kb_used {
        candidates.push((
            None,
            "Keyboard & Mouse".to_string(),
            "Keyboard & Mouse".to_string(),
            true,
        ));
    }

    candidates
}

/// Executes Step 2 confirmation for Drop-In Player, invoking `on_drop_in_player`
/// and returning focus to the game instance upon success.
pub fn confirm_drop_in_layout(
    win: &mut MainWindow,
    assignment: &str,
    layout_name: &str,
) -> Result<usize, String> {
    let result = if let Some(ref cb) = win.callbacks.on_drop_in_player.clone() {
        cb(assignment, layout_name)
    } else {
        let used: std::collections::HashSet<usize> = win
            .in_game_osd
            .active_players
            .iter()
            .map(|p| p.slot_idx)
            .collect();
        let next_slot = (0..win.in_game_osd.max_players.max(16))
            .find(|idx| !used.contains(idx))
            .unwrap_or(win.in_game_osd.active_players.len());
        win.in_game_osd.active_players.push(OsdActivePlayer {
            slot_idx: next_slot,
            nickname: format!("Player {}", next_slot + 1),
            device_label: assignment.to_string(),
        });
        win.in_game_osd.active_players.sort_by_key(|p| p.slot_idx);
        Ok(next_slot)
    };

    match result {
        Ok(slot_idx) => {
            if let Some(ref querier) = win.callbacks.on_query_active_session {
                if let Some(snap) = querier() {
                    win.in_game_osd.sync_from_snapshot(&snap);
                }
            }
            win.selected_layout = layout_name.to_string();
            win.in_game_osd.current_layout = layout_name.to_string();
            win.player_count = win.in_game_osd.active_players.len().max(1);
            win.in_game_osd.drop_in_step = DropInStep::None;
            win.in_game_osd.action_error = None;
            win.status_message = format!(
                "Player {} dropped in with {} ({})",
                slot_idx + 1,
                assignment,
                layout_name
            );
            win.set_osd_visible(false);
            Ok(slot_idx)
        }
        Err(err) => {
            win.in_game_osd.action_error = Some(err.clone());
            win.in_game_osd.drop_in_step = DropInStep::PickController { focused_idx: 0 };
            Err(err)
        }
    }
}

/// Executes Step 2 confirmation for Drop-Out Player, invoking `on_drop_out_player`,
/// preserving survivors, re-tiling to `layout_name`, and closing OSD.
pub fn confirm_drop_out_layout(
    win: &mut MainWindow,
    slot_idx: usize,
    layout_name: &str,
) -> Result<usize, String> {
    let result = if let Some(ref cb) = win.callbacks.on_drop_out_player.clone() {
        cb(slot_idx, layout_name)
    } else {
        win.in_game_osd
            .active_players
            .retain(|p| p.slot_idx != slot_idx);
        Ok(win.in_game_osd.active_players.len())
    };

    match result {
        Ok(remaining) => {
            if let Some(ref querier) = win.callbacks.on_query_active_session {
                if let Some(snap) = querier() {
                    win.in_game_osd.sync_from_snapshot(&snap);
                }
            }
            win.selected_layout = layout_name.to_string();
            win.in_game_osd.current_layout = layout_name.to_string();
            win.player_count = remaining.max(1);
            win.in_game_osd.drop_out_step = DropOutStep::None;
            win.in_game_osd.action_error = None;
            win.status_message = format!(
                "Player {} dropped out cleanly; {} active ({})",
                slot_idx + 1,
                remaining,
                layout_name
            );
            win.set_osd_visible(false);
            Ok(remaining)
        }
        Err(err) => {
            win.in_game_osd.action_error = Some(err.clone());
            win.in_game_osd.drop_out_step = DropOutStep::PickSlot { focused_idx: 0 };
            Err(err)
        }
    }
}

/// Renders the fullscreen 10-foot Couch In-Game OSD overlay on top of active split-screen viewports.
pub fn render_in_game_osd(win: &MainWindow, cx: &mut Context<MainWindow>) -> AnyElement {
    let game_title = win
        .selected_game
        .as_deref()
        .unwrap_or("Active Game Session");
    let is_frozen = win.in_game_osd.is_frozen;
    let input_locked = win.in_game_osd.input_locked;
    let selected_idx = win.in_game_osd.selected_index;
    let reassign_open = win.in_game_osd.reassign_modal_open;
    let drop_in_open = !matches!(win.in_game_osd.drop_in_step, DropInStep::None);
    let drop_out_open = !matches!(win.in_game_osd.drop_out_step, DropOutStep::None);
    let summoner_slot = win.in_game_osd.summoner_slot;
    let summoner_color = summoner_focus_color(summoner_slot);
    let action_error = win.in_game_osd.action_error.clone();

    div()
        .id("in-game-couch-osd-scrim")
        .absolute()
        .inset_0()
        // Fullscreen frosted dark acrylic canvas over live gameplay viewports
        .bg(theme::bg_dark().opacity(0.85))
        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
        .flex()
        .flex_col()
        .items_center()
        .justify_between()
        .p_8()
        // Top Navigation & Session Status Bar (Image 2)
        .child(render_overlay_top_bar(
            game_title,
            is_frozen,
            input_locked,
            summoner_slot,
            summoner_color,
            cx,
        ))
        // Title Banner (Image 2)
        .child(
            div()
                .flex()
                .flex_col()
                .items_start()
                .w_full()
                .max_w(px(1120.0))
                .pt_1()
                .pb_2()
                .child(
                    div()
                        .text_2xl()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .child("Session controls"),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::text_muted())
                        .child("Your game keeps running while this menu is open."),
                ),
        )
        // Center Console Content (Two-Column Layout matching Image 2)
        .child(
            div()
                .id("in-game-couch-osd-container")
                .w_full()
                .max_w(px(1120.0))
                .flex()
                .flex_row()
                .items_start()
                .gap_6()
                // Left Column: Hero Card with Cover Artwork, Duration, and Connected Player Dots
                .child(render_osd_hero_card(win, summoner_color, cx))
                // Right Column: Actions Menu or Submodal
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w(px(540.0))
                        .gap_4()
                        .when_some(action_error, |this, err| {
                            this.child(
                                div()
                                    .id("osd-action-error-banner")
                                    .flex()
                                    .items_center()
                                    .gap_2p5()
                                    .px_4()
                                    .py_2p5()
                                    .rounded_lg()
                                    .bg(theme::danger().opacity(0.16))
                                    .border_1()
                                    .border_color(theme::danger())
                                    .child(
                                        Icon::new(IconName::TriangleAlert)
                                            .size_4()
                                            .text_color(theme::danger()),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(theme::text_primary())
                                            .child(err),
                                    ),
                            )
                        })
                        .child(if drop_in_open {
                            render_drop_in_submodal(win, summoner_color, cx)
                        } else if drop_out_open {
                            render_drop_out_submodal(win, summoner_color, cx)
                        } else if reassign_open {
                            render_reassign_panel(win, summoner_color, cx)
                        } else if win.in_game_osd.sub_view == OsdSubView::PlayerDevices {
                            render_player_devices_view(win, summoner_color, cx)
                        } else if win.in_game_osd.sub_view == OsdSubView::GameGuide {
                            render_game_guide_view(win, summoner_color, cx)
                        } else {
                            render_image2_actions_menu(
                                win,
                                is_frozen,
                                selected_idx,
                                summoner_color,
                                cx,
                            )
                        }),
                ),
        )
        // Bottom Couch Prompt Navigation Bar
        .child(render_overlay_bottom_bar(summoner_color))
        .into_any_element()
}

/// Renders the sleek top bar displaying game title, summoner indicator, and live session status.
fn render_overlay_top_bar(
    _game_title: &str,
    is_frozen: bool,
    input_locked: bool,
    summoner_slot: Option<usize>,
    summoner_color: Hsla,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    div()
        .id("overlay-top-bar")
        .w_full()
        .max_w(px(1120.0))
        .flex()
        .items_center()
        .justify_between()
        .pb_3()
        .border_b_1()
        .border_color(theme::border_subtle())
        // Left: Nucleus Co-op brand + navigation breadcrumbs (Library, Add games, Sessions, Settings)
        .child(
            div()
                .flex()
                .items_center()
                .gap_4()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_2p5()
                        .py_1()
                        .rounded_md()
                        .bg(theme::accent().opacity(0.16))
                        .border_1()
                        .border_color(theme::accent())
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::accent())
                                .child("NUCLEUS CO-OP"),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::text_muted())
                                .child("Library"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::text_muted())
                                .child("Add games"),
                        )
                        .child(
                            div()
                                .px_2p5()
                                .py_0p5()
                                .rounded_full()
                                .bg(theme::bg_panel())
                                .border_1()
                                .border_color(theme::accent())
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::accent())
                                .child("Sessions"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::text_muted())
                                .child("Settings"),
                        ),
                ),
        )
        // Center / Right: Summoner pill, Live execution state pill, and ESC / B Resume
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                // Summoner indicator pill
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py_1()
                        .rounded_full()
                        .bg(summoner_color.opacity(0.14))
                        .border_1()
                        .border_color(summoner_color)
                        .child(div().w_2().h_2().rounded_full().bg(summoner_color))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(summoner_color)
                                .child(match summoner_slot {
                                    Some(slot) => format!("Player {} Summoned", slot + 1),
                                    None => "Host Summoned (Keyboard)".to_string(),
                                }),
                        ),
                )
                // Execution state pill
                .child(if is_frozen {
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_3()
                        .py_1()
                        .rounded_full()
                        .bg(theme::warning().opacity(0.14))
                        .border_1()
                        .border_color(theme::warning())
                        .child(
                            Icon::new(IconName::Pause)
                                .size_3p5()
                                .text_color(theme::warning()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::warning())
                                .child("INSTANCES FROZEN"),
                        )
                } else if input_locked {
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_3()
                        .py_1()
                        .rounded_full()
                        .bg(theme::success().opacity(0.14))
                        .border_1()
                        .border_color(theme::success())
                        .child(
                            Icon::new(IconName::Lock)
                                .size_3p5()
                                .text_color(theme::success()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::success())
                                .child("INPUT LOCKED"),
                        )
                } else {
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_3()
                        .py_1()
                        .rounded_full()
                        .bg(theme::accent().opacity(0.14))
                        .border_1()
                        .border_color(theme::accent())
                        .child(
                            Icon::new(IconName::Play)
                                .size_3p5()
                                .text_color(theme::accent()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::accent())
                                .child("SESSION LIVE"),
                        )
                })
                // Resume Button
                .child(
                    div()
                        .id("btn-close-osd")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py_1p5()
                        .rounded_lg()
                        .bg(theme::bg_panel())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .hover(|s| s.bg(theme::bg_hover()))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.set_osd_visible(false);
                            cx.notify();
                        }))
                        .child(
                            Icon::new(IconName::X)
                                .size_3p5()
                                .text_color(theme::text_muted()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child("ESC / B Resume"),
                        ),
                ),
        )
        .into_any_element()
}

/// Renders the Left Hero Card for Image 2 containing artwork, duration counter, and connected player status dots.
fn render_osd_hero_card(
    win: &MainWindow,
    _summoner_color: Hsla,
    _cx: &mut Context<MainWindow>,
) -> AnyElement {
    let game_title = win
        .selected_game
        .as_deref()
        .unwrap_or("Active Game Session");
    let elapsed = win.in_game_osd.elapsed_seconds;
    let time_str = format_duration_hhmmss(elapsed);
    let player_count = win
        .in_game_osd
        .active_players
        .len()
        .max(win.player_count)
        .max(1);
    let layout_name = if !win.in_game_osd.current_layout.is_empty() {
        &win.in_game_osd.current_layout
    } else {
        &win.selected_layout
    };

    div()
        .id("osd-hero-card")
        .flex()
        .flex_col()
        .w(px(380.0))
        .flex_shrink_0()
        .p_5()
        .rounded_2xl()
        .bg(theme::bg_panel().opacity(0.95))
        .border_1()
        .border_color(theme::border_subtle())
        .shadow_2xl()
        .gap_4()
        // Artwork cover with overlay badge
        .child(
            div()
                .relative()
                .w_full()
                .h(px(180.0))
                .rounded_xl()
                .overflow_hidden()
                .bg(theme::bg_subtle())
                .child(
                    div()
                        .absolute()
                        .inset_0()
                        .bg(theme::bg_dark())
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            Icon::new(IconName::Gamepad2)
                                .size_10()
                                .text_color(theme::text_muted().opacity(0.35)),
                        ),
                )
                // "● SESSION RUNNING" Badge
                .child(
                    div()
                        .absolute()
                        .top_3()
                        .left_3()
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_3()
                        .py_1p5()
                        .rounded_full()
                        .bg(theme::bg_dark().opacity(0.85))
                        .border_1()
                        .border_color(theme::accent().opacity(0.6))
                        .child(div().w_2p5().h_2p5().rounded_full().bg(theme::accent()))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::accent())
                                .child("SESSION RUNNING"),
                        ),
                ),
        )
        // Game Title & Metadata
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .text_lg()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .child(game_title.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::text_muted())
                        .child(format!(
                            "{} Player{} · {} · Display 1",
                            player_count,
                            if player_count == 1 { "" } else { "s" },
                            layout_name
                        )),
                ),
        )
        // Duration Counter: 00:12:48 Session time
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .px_4()
                .py_2p5()
                .rounded_xl()
                .bg(theme::bg_subtle())
                .border_1()
                .border_color(theme::border_subtle())
                .child(
                    Icon::new(IconName::Clock)
                        .size_4()
                        .text_color(theme::accent()),
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .child(format!("{} Session time", time_str)),
                ),
        )
        // Divider
        .child(div().w_full().h_px().bg(theme::border_subtle()))
        // Connected Players Status Dots
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .children(win.in_game_osd.active_players.iter().map(|p| {
                    let dot_color = player_slot_tint(p.slot_idx);
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_3()
                        .py_2()
                        .rounded_lg()
                        .bg(theme::bg_subtle().opacity(0.6))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2p5()
                                .child(div().w_2p5().h_2p5().rounded_full().bg(dot_color))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child(format!("{} connected", p.nickname)),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child(p.device_label.clone()),
                        )
                })),
        )
        .into_any_element()
}

/// Renders the 5 primary actions from Image 2 with quick utilities.
fn render_image2_actions_menu(
    win: &MainWindow,
    is_frozen: bool,
    selected_idx: usize,
    summoner_color: Hsla,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let (_freeze_title, _freeze_desc, freeze_icon) = if is_frozen {
        (
            "Unfreeze (NtResumeProcess)",
            "Unfreeze suspended game threads and resume multi-player gameplay",
            IconName::Play,
        )
    } else {
        (
            "Freeze (NtSuspendProcess)",
            "Suspend child instance threads for bathroom breaks or mid-game pauses",
            IconName::Pause,
        )
    };

    let active_player_count = win
        .in_game_osd
        .active_players
        .len()
        .max(win.player_count)
        .max(1);

    div()
        .id("osd-actions-container")
        .flex()
        .flex_col()
        .gap_4()
        // 1. Primary Action: Resume Game (Large prominent emerald card)
        .child({
            let is_selected = selected_idx == 0;
            div()
                .id("osd-action-resume")
                .flex()
                .items_center()
                .justify_between()
                .px_5()
                .py_4()
                .rounded_xl()
                .cursor_pointer()
                .bg(theme::accent().opacity(0.18))
                .border_2()
                .border_color(if is_selected { theme::accent() } else { theme::accent().opacity(0.5) })
                .hover(|s| s.bg(theme::accent().opacity(0.28)))
                .shadow_lg()
                .on_click(cx.listener(|this, _, _, cx| {
                    this.set_osd_visible(false);
                    cx.notify();
                }))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(
                            div()
                                .p_2p5()
                                .rounded_lg()
                                .bg(theme::accent())
                                .child(
                                    Icon::new(IconName::Play)
                                        .size_5()
                                        .text_color(theme::bg_dark()),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_0p5()
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child("Resume game"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child("Return immediately to active split-screen viewports"),
                                ),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_3()
                        .py_1p5()
                        .rounded_md()
                        .bg(theme::accent().opacity(0.25))
                        .border_1()
                        .border_color(theme::accent())
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::accent())
                                .child("START / A"),
                        ),
                )
        })
        // 2. Secondary Main Actions matching Image 2
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2p5()
                // Action: Player Devices
                .child(
                    div()
                        .id("osd-action-player-devices")
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_4()
                        .py_3()
                        .rounded_xl()
                        .cursor_pointer()
                        .bg(theme::bg_panel())
                        .border_1()
                        .border_color(if selected_idx == 1 { summoner_color } else { theme::border_subtle() })
                        .hover(|s| s.bg(theme::bg_hover()))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.in_game_osd.open_player_devices();
                            cx.notify();
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3p5()
                                .child(
                                    div()
                                        .p_2()
                                        .rounded_lg()
                                        .bg(theme::bg_subtle())
                                        .child(
                                            Icon::new(IconName::Gamepad2)
                                                .size_4()
                                                .text_color(summoner_color),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_0p5()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme::text_primary())
                                                .child("Player devices"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme::text_muted())
                                                .child("Drop in new players, drop out, or swap controllers"),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::accent())
                                .child(format!("{} Connected >", active_player_count)),
                        ),
                )
                // Action: Read first / Game guide
                .child(
                    div()
                        .id("osd-action-game-guide")
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_4()
                        .py_3()
                        .rounded_xl()
                        .cursor_pointer()
                        .bg(theme::bg_panel())
                        .border_1()
                        .border_color(if selected_idx == 2 { summoner_color } else { theme::border_subtle() })
                        .hover(|s| s.bg(theme::bg_hover()))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.in_game_osd.open_game_guide();
                            cx.notify();
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3p5()
                                .child(
                                    div()
                                        .p_2()
                                        .rounded_lg()
                                        .bg(theme::bg_subtle())
                                        .child(
                                            Icon::new(IconName::BookOpen)
                                                .size_4()
                                                .text_color(theme::text_muted()),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_0p5()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme::text_primary())
                                                .child("Read first / Game guide"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme::text_muted())
                                                .child("Review game handler setup notes and splitscreen instructions"),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::text_muted())
                                .child("Guide >"),
                        ),
                )
                // Action: Save session profile
                .child(
                    div()
                        .id("osd-action-save-profile")
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_4()
                        .py_3()
                        .rounded_xl()
                        .cursor_pointer()
                        .bg(theme::bg_panel())
                        .border_1()
                        .border_color(if selected_idx == 3 { summoner_color } else { theme::border_subtle() })
                        .hover(|s| s.bg(theme::bg_hover()))
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some(ref g_name) = this.selected_game {
                                if let Some(item) = this.handlers.iter().find(|h| &h.game_name == g_name) {
                                    if let Some(ref cb) = this.callbacks.on_save_profile {
                                        cb(item);
                                    }
                                }
                            }
                            this.status_message = "Session profile saved successfully.".to_string();
                            cx.notify();
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3p5()
                                .child(
                                    div()
                                        .p_2()
                                        .rounded_lg()
                                        .bg(theme::bg_subtle())
                                        .child(
                                            Icon::new(IconName::Save)
                                                .size_4()
                                                .text_color(theme::text_muted()),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_0p5()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme::text_primary())
                                                .child("Save session profile"),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme::text_muted())
                                                .child("Save current player and screen partition setup for quick launch"),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::text_muted())
                                .child("Save"),
                        ),
                )
                // Action: End session... (subdued red text button)
                .child(
                    div()
                        .id("osd-action-end-session")
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_4()
                        .py_3()
                        .rounded_xl()
                        .cursor_pointer()
                        .bg(theme::bg_panel())
                        .border_1()
                        .border_color(if selected_idx == 7 { theme::danger() } else { theme::border_subtle() })
                        .hover(|s| s.bg(theme::danger().opacity(0.12)))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.stop_session(cx);
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3p5()
                                .child(
                                    div()
                                        .p_2()
                                        .rounded_lg()
                                        .bg(theme::danger().opacity(0.15))
                                        .child(
                                            Icon::new(IconName::X)
                                                .size_4()
                                                .text_color(theme::danger()),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_0p5()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme::danger())
                                                .child("End session..."),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme::text_muted())
                                                .child("Terminate all instance processes and restore desktop environment"),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::danger())
                                .child("Exit"),
                        ),
                ),
        )
        // 3. Quick Utilities Row (Freeze, Snap Coordinates, Raw Input Lock)
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .pt_1()
                .child(
                    div()
                        .id("btn-quick-freeze")
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .px_3()
                        .py_2()
                        .rounded_lg()
                        .cursor_pointer()
                        .bg(theme::bg_panel())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .hover(|s| s.bg(theme::bg_hover()))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.toggle_freeze_session(cx);
                        }))
                        .child(Icon::new(freeze_icon).size_3p5().text_color(if is_frozen { theme::warning() } else { theme::accent() }))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::text_primary())
                                .child(if is_frozen { "Unfreeze" } else { "Freeze (Pause)" }),
                        ),
                )
                .child(
                    div()
                        .id("btn-quick-reposition")
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .px_3()
                        .py_2()
                        .rounded_lg()
                        .cursor_pointer()
                        .bg(theme::bg_panel())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .hover(|s| s.bg(theme::bg_hover()))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.re_enforce_windows(cx);
                            this.set_osd_visible(false);
                            cx.notify();
                        }))
                        .child(Icon::new(IconName::Maximize).size_3p5().text_color(theme::text_muted()))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::text_primary())
                                .child("Snap Windows"),
                        ),
                )
                .child(
                    div()
                        .id("btn-quick-input-lock")
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .px_3()
                        .py_2()
                        .rounded_lg()
                        .cursor_pointer()
                        .bg(theme::bg_panel())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .hover(|s| s.bg(theme::bg_hover()))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.toggle_input_lock(cx);
                        }))
                        .child(Icon::new(IconName::Lock).size_3p5().text_color(if win.in_game_osd.input_locked { theme::success() } else { theme::text_muted() }))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(theme::text_primary())
                                .child(if win.in_game_osd.input_locked { "Input Locked" } else { "Lock Input" }),
                        ),
                ),
        )
        .into_any_element()
}

/// Renders the Player Devices management panel (Image 2 sub-view).
fn render_player_devices_view(
    win: &MainWindow,
    summoner_color: Hsla,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let active_slots = win.in_game_osd.active_players.clone();
    let max_players = win.in_game_osd.max_players;
    let can_drop_in = active_slots.len() < max_players;
    let can_drop_out = active_slots.len() > 1;

    div()
        .id("osd-player-devices-view")
        .flex()
        .flex_col()
        .gap_4()
        .p_5()
        .rounded_xl()
        .bg(theme::bg_panel())
        .border_1()
        .border_color(theme::border_subtle())
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2p5()
                        .child(
                            Icon::new(IconName::Gamepad2)
                                .size_5()
                                .text_color(summoner_color),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_base()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child("Player Devices & Assignments"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child("Hot-swap controllers, add players mid-session, or drop out cleanly."),
                                ),
                        ),
                )
                .child(
                    div()
                        .px_2p5()
                        .py_1()
                        .rounded_full()
                        .bg(theme::accent().opacity(0.14))
                        .border_1()
                        .border_color(theme::accent())
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::accent())
                        .child(format!("{}/{} Players Active", active_slots.len(), max_players)),
                ),
        )
        // Connected Players List
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2p5()
                .children(active_slots.iter().map(|p| {
                    let slot_idx = p.slot_idx;
                    let slot_tint = player_slot_tint(slot_idx);
                    div()
                        .id(SharedString::from(format!("osd-player-device-card-{}", slot_idx)))
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_4()
                        .py_3()
                        .rounded_lg()
                        .bg(theme::bg_subtle())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .px_2p5()
                                        .py_1()
                                        .rounded_md()
                                        .bg(slot_tint.opacity(0.18))
                                        .border_1()
                                        .border_color(slot_tint)
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(slot_tint)
                                        .child(format!("P{}", slot_idx + 1)),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(theme::text_primary())
                                                .child(p.nickname.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(theme::text_muted())
                                                .child(p.device_label.clone()),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(
                                    div()
                                        .id(SharedString::from(format!("btn-swap-dev-{}", slot_idx)))
                                        .cursor_pointer()
                                        .px_3()
                                        .py_1p5()
                                        .rounded_md()
                                        .bg(theme::bg_panel())
                                        .border_1()
                                        .border_color(theme::border_subtle())
                                        .hover(|s| s.bg(theme::bg_hover()).border_color(theme::accent()))
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            this.in_game_osd.open_reassign(slot_idx);
                                            cx.notify();
                                        }))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(theme::text_primary())
                                                .child("Swap Device"),
                                        ),
                                )
                                .child({
                                    let mut drop_out_btn = div()
                                        .id(SharedString::from(format!("btn-drop-out-{}", slot_idx)))
                                        .px_3()
                                        .py_1p5()
                                        .rounded_md()
                                        .border_1();
                                    if can_drop_out {
                                        drop_out_btn = drop_out_btn
                                            .cursor_pointer()
                                            .bg(theme::danger().opacity(0.12))
                                            .border_color(theme::danger().opacity(0.4))
                                            .hover(|s| s.bg(theme::danger().opacity(0.24)).border_color(theme::danger()))
                                            .on_click(cx.listener(move |this, _, _, cx| {
                                                this.in_game_osd.open_drop_out(Some(slot_idx));
                                                cx.notify();
                                            }))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(theme::danger())
                                                    .child("Drop Out"),
                                            );
                                    } else {
                                        drop_out_btn = drop_out_btn
                                            .bg(theme::bg_subtle())
                                            .border_color(theme::border_subtle())
                                            .opacity(0.4)
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::MEDIUM)
                                                    .text_color(theme::text_muted())
                                                    .child("Last Player"),
                                            );
                                    }
                                    drop_out_btn
                                }),
                        )
                })),
        )
        // Bottom Action Bar: + Drop-In New Player & ← Back to Controls
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .pt_2()
                .border_t_1()
                .border_color(theme::border_subtle())
                .child(
                    div()
                        .id("btn-devices-back")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_4()
                        .py_2()
                        .rounded_lg()
                        .bg(theme::bg_hover())
                        .hover(|s| s.bg(theme::bg_subtle()))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.in_game_osd.back_to_main();
                            cx.notify();
                        }))
                        .child(Icon::new(IconName::ChevronLeft).size_3p5().text_color(theme::text_primary()))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child("Back to Session Controls"),
                        ),
                )
                .child({
                    let mut drop_in_btn = div()
                        .id("btn-player-devices-drop-in")
                        .flex()
                        .items_center()
                        .gap_2()
                        .px_4()
                        .py_2()
                        .rounded_lg()
                        .border_1();
                    if can_drop_in {
                        drop_in_btn = drop_in_btn
                            .cursor_pointer()
                            .bg(theme::accent().opacity(0.18))
                            .border_color(theme::accent())
                            .hover(|s| s.bg(theme::accent().opacity(0.28)))
                            .on_click(cx.listener(move |this, _, _, cx| {
                                let count = this.in_game_osd.active_players.len().max(1);
                                let max_p = this.in_game_osd.max_players.max(count).max(4);
                                this.in_game_osd.open_drop_in(count, max_p);
                                cx.notify();
                            }))
                            .child(Icon::new(IconName::Plus).size_4().text_color(theme::accent()))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme::accent())
                                    .child("+ Drop-In New Player"),
                            );
                    } else {
                        drop_in_btn = drop_in_btn
                            .bg(theme::bg_subtle())
                            .border_color(theme::border_subtle())
                            .opacity(0.5)
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme::text_muted())
                                    .child("Session at Max Capacity"),
                            );
                    }
                    drop_in_btn
                }),
        )
        .into_any_element()
}

/// Renders the Game Guide panel (Image 2 sub-view).
fn render_game_guide_view(
    win: &MainWindow,
    summoner_color: Hsla,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let game_title = win.selected_game.as_deref().unwrap_or("Active Game");

    div()
        .id("osd-game-guide-view")
        .flex()
        .flex_col()
        .gap_4()
        .p_5()
        .rounded_xl()
        .bg(theme::bg_panel())
        .border_1()
        .border_color(theme::border_subtle())
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .child(
                    Icon::new(IconName::BookOpen)
                        .size_5()
                        .text_color(summoner_color),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_base()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child(format!("Read First / Game Guide — {}", game_title)),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("Handler setup notes, recommended configurations, and tips."),
                        ),
                ),
        )
        // Guide Bullet Points
        .child(
            div()
                .flex()
                .flex_col()
                .gap_3()
                .p_4()
                .rounded_lg()
                .bg(theme::bg_subtle())
                .border_1()
                .border_color(theme::border_subtle())
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap_2p5()
                        .child(Icon::new(IconName::Lock).size_4().text_color(theme::accent()))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("Input Lock: Press VK_END (or toggle in quick utilities) to lock your cursor inside splitscreen instance windows."),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap_2p5()
                        .child(Icon::new(IconName::Maximize).size_4().text_color(theme::accent()))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("Window Snapping: If in-game video settings change your resolution, use 'Snap Windows' to re-enforce configured borders."),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap_2p5()
                        .child(Icon::new(IconName::Gamepad2).size_4().text_color(theme::accent()))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("Hot-Swapping: Dead gamepad batteries can be reassigned without closing instances under 'Player Devices'."),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap_2p5()
                        .child(Icon::new(IconName::Users).size_4().text_color(theme::accent()))
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("Drop-In / Drop-Out: Invite friends or let players leave cleanly mid-game; surviving instances continue without restart."),
                        ),
                ),
        )
        // Bottom Action Bar: Back to controls
        .child(
            div()
                .flex()
                .items_center()
                .justify_start()
                .pt_2()
                .border_t_1()
                .border_color(theme::border_subtle())
                .child(
                    div()
                        .id("btn-guide-back")
                        .cursor_pointer()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_4()
                        .py_2()
                        .rounded_lg()
                        .bg(theme::bg_hover())
                        .hover(|s| s.bg(theme::bg_subtle()))
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.in_game_osd.back_to_main();
                            cx.notify();
                        }))
                        .child(Icon::new(IconName::ChevronLeft).size_3p5().text_color(theme::text_primary()))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child("Back to Session Controls"),
                        ),
                ),
        )
        .into_any_element()
}

#[allow(dead_code)]
/// Renders the 8 primary couch actions with summoner-tinted gamepad focus rings.
fn render_actions_menu(
    is_frozen: bool,
    selected_idx: usize,
    summoner_color: Hsla,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let (freeze_title, freeze_desc, freeze_icon) = if is_frozen {
        (
            "Resume All Instances (NtResumeProcess)",
            "Unfreeze suspended game threads and resume multi-player gameplay",
            IconName::Play,
        )
    } else {
        (
            "Freeze All Instances (NtSuspendProcess)",
            "Suspend child instance threads for bathroom breaks or mid-game pauses",
            IconName::Pause,
        )
    };

    let items = [
        (
            "Resume Session (B / Start)",
            "Return immediately to active split-screen viewports",
            IconName::Play,
        ),
        (freeze_title, freeze_desc, freeze_icon),
        (
            "Reassign Controllers (Dead Battery Hot-Swap)",
            "Reassign player slot controller without restarting instances",
            IconName::Gamepad2,
        ),
        (
            "Drop-In Player",
            "Join a new local player mid-session and choose an updated screen layout",
            IconName::Plus,
        ),
        (
            "Drop-Out Player",
            "Let an active player leave cleanly while remaining players continue playing",
            IconName::LogOut,
        ),
        (
            "Re-enforce Window Positions (Snap Coordinates)",
            "Force shifted direct3d viewports back into configured screen tiles",
            IconName::Maximize,
        ),
        (
            "Toggle Input Lock (VK_END Parity)",
            "Lock or unlock raw keyboard/mouse input across instances",
            IconName::Lock,
        ),
        (
            "End Game Session (Clean Exit)",
            "Terminate all instance processes and restore desktop environment",
            IconName::X,
        ),
    ];

    div()
        .id("osd-actions-list")
        .flex()
        .flex_col()
        .gap_2()
        .children(items.iter().enumerate().map(|(idx, (title, desc, icon))| {
            let is_selected = selected_idx == idx;
            let mut card = div()
                .id(format!("osd-action-{}", idx))
                .flex()
                .items_center()
                .justify_between()
                .px_4()
                .py_3()
                .rounded_xl()
                .cursor_pointer()
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.in_game_osd.select(idx);
                    this.handle_osd_confirm(cx);
                }));

            if is_selected {
                // Focused state: summoner-tinted dynamic focus ring and active background
                card = card
                    .border_2()
                    .border_color(summoner_color)
                    .bg(theme::bg_active())
                    .shadow_lg();
            } else {
                card = card
                    .border_1()
                    .border_color(theme::border_subtle())
                    .bg(theme::bg_panel())
                    .hover(|s| s.bg(theme::bg_hover()));
            }

            card.child(
                div()
                    .flex()
                    .items_center()
                    .gap_3p5()
                    .child(
                        div()
                            .p_2()
                            .rounded_lg()
                            .bg(if is_selected {
                                summoner_color.opacity(0.18)
                            } else {
                                theme::bg_subtle()
                            })
                            .child(Icon::new(*icon).size_4().text_color(if is_selected {
                                summoner_color
                            } else {
                                theme::text_muted()
                            })),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(if is_selected {
                                        summoner_color
                                    } else {
                                        theme::text_primary()
                                    })
                                    .child(title.to_string()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme::text_muted())
                                    .child(desc.to_string()),
                            ),
                    ),
            )
            .child(if is_selected {
                div()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .px_2p5()
                    .py_1()
                    .rounded_md()
                    .bg(summoner_color.opacity(0.18))
                    .border_1()
                    .border_color(summoner_color)
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(summoner_color)
                            .child("PRESS A"),
                    )
            } else {
                div().child("")
            })
            .into_any_element()
        }))
        .into_any_element()
}

/// Renders a horizontal strip of active player slots and assigned controllers.
#[allow(dead_code)]
fn render_player_slots_strip(win: &MainWindow, cx: &mut Context<MainWindow>) -> AnyElement {
    let slots: Vec<OsdActivePlayer> = if !win.in_game_osd.active_players.is_empty() {
        win.in_game_osd.active_players.clone()
    } else {
        let player_count = std::cmp::max(win.player_count, 1);
        (0..player_count)
            .map(|slot_idx| OsdActivePlayer {
                slot_idx,
                nickname: format!("Player {}", slot_idx + 1),
                device_label: win
                    .player_device_assignments
                    .get(slot_idx)
                    .cloned()
                    .unwrap_or_else(|| format!("Slot {}", slot_idx + 1)),
            })
            .collect()
    };

    div()
        .id("osd-players-strip")
        .flex()
        .items_center()
        .gap_3()
        .pt_2()
        .children(slots.into_iter().map(|player| {
            let slot_idx = player.slot_idx;
            let slot_color = player_slot_tint(slot_idx);
            let current_dev = player.device_label;

            div()
                .id(format!("osd-player-slot-{}", slot_idx))
                .flex_1()
                .flex()
                .items_center()
                .justify_between()
                .px_3()
                .py_2()
                .rounded_lg()
                .bg(theme::bg_panel())
                .border_1()
                .border_color(slot_color.opacity(0.4))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().w_2p5().h_2p5().rounded_full().bg(slot_color))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child(format!("P{}", slot_idx + 1)),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child(current_dev),
                                ),
                        ),
                )
                .child(
                    div()
                        .id(format!("btn-swap-slot-{}", slot_idx))
                        .cursor_pointer()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(theme::bg_subtle())
                        .hover(|s| s.bg(theme::bg_hover()))
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::text_primary())
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.in_game_osd.open_reassign(slot_idx);
                            cx.notify();
                        }))
                        .child("Swap"),
                )
                .into_any_element()
        }))
        .into_any_element()
}

fn layout_preview_icon(layout_name: &str) -> IconName {
    if layout_name.contains("Multi-Monitor") {
        IconName::Monitor
    } else if layout_name.contains("2x1") {
        IconName::Columns2
    } else if layout_name.contains("1x2") {
        IconName::Rows2
    } else if layout_name.contains("Fullscreen") {
        IconName::Maximize
    } else if layout_name.contains("3x2") {
        IconName::Grid3x3
    } else if layout_name.contains("2x2") {
        IconName::Grid2x2
    } else {
        IconName::LayoutGrid
    }
}

/// Renders a shared 2-column screen layout picker panel for Drop-In / Drop-Out transitions.
fn render_osd_layout_selector<F, B>(
    id_prefix: &str,
    heading: String,
    subtitle: String,
    layouts: Vec<&'static str>,
    focused_idx: usize,
    summoner_color: Hsla,
    on_select: F,
    on_back: B,
    cx: &mut Context<MainWindow>,
) -> AnyElement
where
    F: Fn(&mut MainWindow, &str, &mut Context<MainWindow>) + Clone + 'static,
    B: Fn(&mut MainWindow, &mut Context<MainWindow>) + 'static,
{
    div()
        .id(SharedString::from(format!("{}-layout-dialog", id_prefix)))
        .flex()
        .flex_col()
        .gap_4()
        .p_5()
        .rounded_xl()
        .bg(theme::bg_panel())
        .border_1()
        .border_color(theme::border_subtle())
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2p5()
                        .child(
                            Icon::new(IconName::LayoutGrid)
                                .size_4()
                                .text_color(summoner_color),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(summoner_color)
                                .child(heading),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::text_muted())
                        .child(subtitle),
                ),
        )
        .child(
            div()
                .id(SharedString::from(format!("{}-layouts-grid", id_prefix)))
                .grid()
                .grid_cols(2)
                .gap_2p5()
                .children(layouts.into_iter().enumerate().map(|(idx, layout)| {
                    let is_focused = idx == focused_idx;
                    let on_select = on_select.clone();
                    let layout_owned = layout.to_string();
                    div()
                        .id(SharedString::from(format!("{}-layout-{}", id_prefix, idx)))
                        .flex()
                        .items_center()
                        .justify_between()
                        .px_3p5()
                        .py_3()
                        .rounded_lg()
                        .cursor_pointer()
                        .border_2()
                        .border_color(if is_focused {
                            summoner_color
                        } else {
                            theme::border_subtle()
                        })
                        .bg(if is_focused {
                            theme::bg_active()
                        } else {
                            theme::bg_subtle()
                        })
                        .hover(|s| s.bg(theme::bg_hover()))
                        .on_click(cx.listener(move |this, _, _, cx| {
                            on_select(this, &layout_owned, cx);
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2p5()
                                .child(Icon::new(layout_preview_icon(layout)).size_4().text_color(
                                    if is_focused {
                                        summoner_color
                                    } else {
                                        theme::text_muted()
                                    },
                                ))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child(layout),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(if is_focused {
                                    summoner_color
                                } else {
                                    theme::text_muted()
                                })
                                .child(if is_focused { "APPLY" } else { "Select" }),
                        )
                        .into_any_element()
                })),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .gap_3()
                .pt_2()
                .child(
                    div()
                        .id(SharedString::from(format!("{}-btn-back", id_prefix)))
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .cursor_pointer()
                        .bg(theme::bg_hover())
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .on_click(cx.listener(move |this, _, _, cx| {
                            on_back(this, cx);
                        }))
                        .child(
                            Icon::new(IconName::ChevronLeft)
                                .size_3p5()
                                .text_color(theme::text_primary()),
                        )
                        .child("Back"),
                ),
        )
        .into_any_element()
}

/// Renders the 2-step Drop-In Player submodal (Step 1: Controller Picker -> Step 2: Layout Selector).
fn render_drop_in_submodal(
    win: &MainWindow,
    summoner_color: Hsla,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    match &win.in_game_osd.drop_in_step {
        DropInStep::None => div().into_any_element(),
        DropInStep::PickController { focused_idx } => {
            let candidates = drop_in_candidate_devices(win);
            let focused_idx = *focused_idx;

            div()
                .id("osd-drop-in-picker")
                .flex()
                .flex_col()
                .gap_4()
                .p_5()
                .rounded_xl()
                .bg(theme::bg_panel())
                .border_1()
                .border_color(theme::border_subtle())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2p5()
                                .child(
                                    Icon::new(IconName::Plus)
                                        .size_4()
                                        .text_color(summoner_color),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(summoner_color)
                                        .child("Drop-In Player — Step 1: Select Input Device"),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .px_2p5()
                                .py_1()
                                .rounded_md()
                                .bg(summoner_color.opacity(0.14))
                                .child(
                                    Icon::new(IconName::Gamepad2)
                                        .size_3p5()
                                        .text_color(summoner_color),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(summoner_color)
                                        .child(
                                            "Press any button on an unassigned controller to join",
                                        ),
                                ),
                        ),
                )
                .child(
                    div()
                        .id("osd-drop-in-devices-list")
                        .flex()
                        .flex_col()
                        .gap_2()
                        .max_h(px(220.0))
                        .overflow_y_scroll()
                        .children(candidates.into_iter().enumerate().map(
                            |(idx, (dev_id, label, assignment, is_kb))| {
                                let is_focused = idx == focused_idx;
                                let icon = if is_kb {
                                    IconName::Keyboard
                                } else {
                                    IconName::Gamepad2
                                };
                                let label_click = label.clone();
                                div()
                                    .id(SharedString::from(format!("osd-dropin-dev-{}", idx)))
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .px_3p5()
                                    .py_2p5()
                                    .rounded_lg()
                                    .cursor_pointer()
                                    .border_1()
                                    .border_color(if is_focused {
                                        summoner_color
                                    } else {
                                        theme::border_subtle()
                                    })
                                    .bg(if is_focused {
                                        theme::bg_active()
                                    } else {
                                        theme::bg_subtle()
                                    })
                                    .hover(|s| s.bg(theme::bg_hover()))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.in_game_osd.select_drop_in_device(
                                            dev_id,
                                            label_click.clone(),
                                            assignment.clone(),
                                        );
                                        cx.notify();
                                    }))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2p5()
                                            .child(Icon::new(icon).size_4().text_color(
                                                if is_focused {
                                                    summoner_color
                                                } else {
                                                    theme::text_muted()
                                                },
                                            ))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(theme::text_primary())
                                                    .child(label),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(if is_focused {
                                                summoner_color
                                            } else {
                                                theme::text_muted()
                                            })
                                            .child("Next: Layout"),
                                    )
                                    .into_any_element()
                            },
                        )),
                )
                .child(
                    div().flex().items_center().justify_end().pt_1().child(
                        div()
                            .id("btn-cancel-dropin")
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .cursor_pointer()
                            .bg(theme::bg_hover())
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.in_game_osd.cancel_submodal();
                                cx.notify();
                            }))
                            .child("Cancel"),
                    ),
                )
                .into_any_element()
        }
        DropInStep::SelectLayout {
            chosen_device_label,
            chosen_assignment,
            focused_layout_idx,
            ..
        } => {
            let active_count = if win.in_game_osd.active_players.is_empty() {
                win.player_count.max(1)
            } else {
                win.in_game_osd.active_players.len()
            };
            let target_count = active_count + 1;
            let layouts = valid_layouts_for_player_count(target_count);
            let chosen_assignment = chosen_assignment.clone();
            render_osd_layout_selector(
                "dropin",
                format!(
                    "Drop-In Player — Step 2: Layout for {} Players",
                    target_count
                ),
                format!("Input: {}", chosen_device_label),
                layouts,
                *focused_layout_idx,
                summoner_color,
                move |this, layout, cx| {
                    let _ = confirm_drop_in_layout(this, &chosen_assignment, layout);
                    cx.notify();
                },
                |this, cx| {
                    this.in_game_osd.cancel_submodal();
                    cx.notify();
                },
                cx,
            )
        }
        DropInStep::Spawning { message } => div()
            .id("osd-drop-in-spawning")
            .flex()
            .items_center()
            .justify_center()
            .gap_3()
            .p_8()
            .rounded_xl()
            .bg(theme::bg_panel())
            .border_1()
            .border_color(summoner_color)
            .child(
                Icon::new(IconName::Loader)
                    .size_5()
                    .text_color(summoner_color),
            )
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme::text_primary())
                    .child(message.clone()),
            )
            .into_any_element(),
    }
}

/// Renders the 2-step Drop-Out Player submodal (Step 1: Slot Picker -> Step 2: Survivor Layout Selector).
fn render_drop_out_submodal(
    win: &MainWindow,
    summoner_color: Hsla,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    match &win.in_game_osd.drop_out_step {
        DropOutStep::None => div().into_any_element(),
        DropOutStep::PickSlot { focused_idx } => {
            let players = win.in_game_osd.active_players.clone();
            let summoner_slot = win.in_game_osd.summoner_slot;
            let focused_idx = *focused_idx;

            div()
                .id("osd-drop-out-picker")
                .flex()
                .flex_col()
                .gap_4()
                .p_5()
                .rounded_xl()
                .bg(theme::bg_panel())
                .border_1()
                .border_color(theme::border_subtle())
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2p5()
                                .child(
                                    Icon::new(IconName::LogOut)
                                        .size_4()
                                        .text_color(summoner_color),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(summoner_color)
                                        .child("Drop-Out Player — Step 1: Select Leaving Player"),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child("Other instances stay running and saves are kept"),
                        ),
                )
                .child(
                    div()
                        .id("osd-drop-out-slots-list")
                        .flex()
                        .flex_col()
                        .gap_2()
                        .children(players.into_iter().enumerate().map(|(idx, player)| {
                            let is_focused = idx == focused_idx;
                            let is_summoner = summoner_slot == Some(player.slot_idx);
                            let tint = player_slot_tint(player.slot_idx);
                            let slot_idx = player.slot_idx;

                            div()
                                .id(SharedString::from(format!("osd-dropout-slot-{}", slot_idx)))
                                .flex()
                                .items_center()
                                .justify_between()
                                .px_3p5()
                                .py_2p5()
                                .rounded_lg()
                                .cursor_pointer()
                                .border_1()
                                .border_color(if is_focused {
                                    summoner_color
                                } else {
                                    theme::border_subtle()
                                })
                                .bg(if is_focused {
                                    theme::bg_active()
                                } else {
                                    theme::bg_subtle()
                                })
                                .hover(|s| s.bg(theme::bg_hover()))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.in_game_osd.select_drop_out_slot(slot_idx);
                                    cx.notify();
                                }))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2p5()
                                        .child(div().w_2p5().h_2p5().rounded_full().bg(tint))
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(theme::text_primary())
                                                        .child(format!(
                                                            "Player {} — {}",
                                                            slot_idx + 1,
                                                            player.nickname
                                                        )),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme::text_muted())
                                                        .child(player.device_label),
                                                ),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .when(is_summoner, |this| {
                                            this.child(
                                                div()
                                                    .px_2()
                                                    .py_0p5()
                                                    .rounded_md()
                                                    .bg(summoner_color.opacity(0.16))
                                                    .text_xs()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(summoner_color)
                                                    .child("Summoner"),
                                            )
                                        })
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(if is_focused {
                                                    summoner_color
                                                } else {
                                                    theme::text_muted()
                                                })
                                                .child("Next: Re-Tile"),
                                        ),
                                )
                                .into_any_element()
                        })),
                )
                .child(
                    div().flex().items_center().justify_end().pt_1().child(
                        div()
                            .id("btn-cancel-dropout")
                            .px_4()
                            .py_2()
                            .rounded_md()
                            .cursor_pointer()
                            .bg(theme::bg_hover())
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme::text_primary())
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.in_game_osd.cancel_submodal();
                                cx.notify();
                            }))
                            .child("Cancel"),
                    ),
                )
                .into_any_element()
        }
        DropOutStep::SelectLayout {
            target_slot_idx,
            focused_layout_idx,
        } => {
            let remaining = win
                .in_game_osd
                .active_players
                .len()
                .saturating_sub(1)
                .max(1);
            let layouts = valid_layouts_for_player_count(remaining);
            let slot_idx = *target_slot_idx;
            render_osd_layout_selector(
                "dropout",
                format!(
                    "Drop-Out Player {} — Step 2: Re-Tile {} Remaining Player(s)",
                    slot_idx + 1,
                    remaining
                ),
                format!("Surviving players keep running cleanly"),
                layouts,
                *focused_layout_idx,
                summoner_color,
                move |this, layout, cx| {
                    let _ = confirm_drop_out_layout(this, slot_idx, layout);
                    cx.notify();
                },
                |this, cx| {
                    this.in_game_osd.cancel_submodal();
                    cx.notify();
                },
                cx,
            )
        }
        DropOutStep::Applying { message } => div()
            .id("osd-drop-out-applying")
            .flex()
            .items_center()
            .justify_center()
            .gap_3()
            .p_8()
            .rounded_xl()
            .bg(theme::bg_panel())
            .border_1()
            .border_color(summoner_color)
            .child(
                Icon::new(IconName::Loader)
                    .size_5()
                    .text_color(summoner_color),
            )
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme::text_primary())
                    .child(message.clone()),
            )
            .into_any_element(),
    }
}

/// Renders the controller hot-swap reassignment panel.
fn render_reassign_panel(
    win: &MainWindow,
    summoner_color: Hsla,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let targeted_slot = win.in_game_osd.reassign_slot_idx;
    let chosen_device = win.in_game_osd.reassign_new_device_id;
    let slots: Vec<OsdActivePlayer> = if !win.in_game_osd.active_players.is_empty() {
        win.in_game_osd.active_players.clone()
    } else {
        let player_count = std::cmp::max(win.player_count, 1);
        (0..player_count)
            .map(|slot_idx| OsdActivePlayer {
                slot_idx,
                nickname: format!("Player {}", slot_idx + 1),
                device_label: win
                    .player_device_assignments
                    .get(slot_idx)
                    .cloned()
                    .unwrap_or_else(|| format!("Slot {}", slot_idx + 1)),
            })
            .collect()
    };

    div()
        .id("reassign-controllers-dialog")
        .flex()
        .flex_col()
        .gap_4()
        .p_5()
        .rounded_xl()
        .bg(theme::bg_panel())
        .border_1()
        .border_color(theme::border_subtle())
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(summoner_color)
                .child("Step 1: Select Player Slot to Reassign"),
        )
        // Player Slot selection row
        .child(
            div()
                .flex()
                .items_center()
                .gap_2p5()
                .children(slots.into_iter().map(|player| {
                    let slot_idx = player.slot_idx;
                    let is_active_slot = slot_idx == targeted_slot;
                    let slot_tint = player_slot_tint(slot_idx);
                    let current_name = player.device_label;

                    div()
                        .id(format!("reassign-slot-{}", slot_idx))
                        .flex_1()
                        .flex()
                        .flex_col()
                        .p_2p5()
                        .rounded_lg()
                        .cursor_pointer()
                        .border_1()
                        .border_color(if is_active_slot {
                            summoner_color
                        } else {
                            theme::border_subtle()
                        })
                        .bg(if is_active_slot {
                            theme::bg_active()
                        } else {
                            theme::bg_panel()
                        })
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.in_game_osd.reassign_slot_idx = slot_idx;
                            cx.notify();
                        }))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_1p5()
                                .child(div().w_2().h_2().rounded_full().bg(slot_tint))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(if is_active_slot {
                                            summoner_color
                                        } else {
                                            theme::text_primary()
                                        })
                                        .child(format!("Player {}", slot_idx + 1)),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(theme::text_muted())
                                .child(current_name),
                        )
                        .into_any_element()
                })),
        )
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(summoner_color)
                .child("Step 2: Choose Detected Replacement Controller"),
        )
        // Detected replacement devices list
        .child(
            div()
                .id("reassign-devices-scroll")
                .flex()
                .flex_col()
                .gap_2()
                .max_h(px(160.0))
                .overflow_y_scroll()
                .children(if win.detected_devices_list.is_empty() {
                    vec![
                        div()
                            .p_3()
                            .rounded_md()
                            .bg(theme::bg_subtle())
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child("No gamepads detected. Plug in replacement controller.")
                            .into_any_element(),
                    ]
                } else {
                    win.detected_devices_list
                        .iter()
                        .map(|dev| {
                            let is_chosen = chosen_device == Some(dev.id);
                            let dev_id = dev.id;
                            div()
                                .id(format!("reassign-dev-{}", dev_id))
                                .flex()
                                .items_center()
                                .justify_between()
                                .px_3()
                                .py_2()
                                .rounded_md()
                                .cursor_pointer()
                                .border_1()
                                .border_color(if is_chosen {
                                    summoner_color
                                } else {
                                    theme::border_subtle()
                                })
                                .bg(if is_chosen {
                                    theme::bg_active()
                                } else {
                                    theme::bg_panel()
                                })
                                .hover(|s| s.bg(theme::bg_hover()))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.in_game_osd.reassign_new_device_id = Some(dev_id);
                                    cx.notify();
                                }))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(Icon::new(IconName::Gamepad2).size_3p5().text_color(
                                            if is_chosen {
                                                summoner_color
                                            } else {
                                                theme::text_muted()
                                            },
                                        ))
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(if is_chosen {
                                                    summoner_color
                                                } else {
                                                    theme::text_primary()
                                                })
                                                .child(format!("Device #{}: {}", dev.id, dev.name)),
                                        ),
                                )
                                .child(if is_chosen {
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .child(
                                            Icon::new(IconName::Check)
                                                .size_3()
                                                .text_color(summoner_color),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(summoner_color)
                                                .child("SELECTED"),
                                        )
                                } else {
                                    div().child("")
                                })
                                .into_any_element()
                        })
                        .collect()
                }),
        )
        // Dialog Action buttons
        .child(
            div()
                .flex()
                .items_center()
                .justify_end()
                .gap_3()
                .pt_2()
                .child(
                    div()
                        .id("btn-cancel-reassign")
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .cursor_pointer()
                        .bg(theme::bg_hover())
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.in_game_osd.close_reassign();
                            cx.notify();
                        }))
                        .child("Cancel"),
                )
                .child(
                    div()
                        .id("btn-confirm-reassign")
                        .px_4()
                        .py_2()
                        .rounded_md()
                        .cursor_pointer()
                        .bg(summoner_color)
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_on_accent())
                        .on_click(cx.listener(|this, _, _, cx| {
                            if let Some((slot, dev_id)) = this.in_game_osd.confirm_reassign() {
                                this.reassign_controller(slot, dev_id, cx);
                            }
                        }))
                        .child("Confirm Reassignment"),
                ),
        )
        .into_any_element()
}

/// Renders the bottom couch navigation shortcuts bar.
fn render_overlay_bottom_bar(summoner_color: Hsla) -> AnyElement {
    div()
        .id("overlay-bottom-bar")
        .w_full()
        .max_w(px(840.0))
        .flex()
        .items_center()
        .justify_center()
        .gap_8()
        .pt_4()
        .border_t_1()
        .border_color(theme::border_subtle())
        .child(render_couch_prompt("A", "Confirm Action", summoner_color))
        .child(render_couch_prompt(
            "B / Start",
            "Resume Session",
            summoner_color,
        ))
        .child(render_couch_prompt(
            "D-Pad / Arrows",
            "Navigate Menu",
            summoner_color,
        ))
        .into_any_element()
}

/// Helper that renders an accessible couch keycap button prompt.
fn render_couch_prompt(btn: &str, action: &str, accent: Hsla) -> AnyElement {
    div()
        .flex()
        .items_center()
        .gap_2()
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .px_2()
                .py_0p5()
                .rounded_md()
                .bg(theme::bg_panel())
                .border_1()
                .border_color(theme::border_subtle())
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(accent)
                .child(btn.to_string()),
        )
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme::text_muted())
                .child(action.to_string()),
        )
        .into_any_element()
}
