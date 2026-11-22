//! Couch Splitscreen Setup screen for 10-foot television interface.
//!
//! Provides instant controller auto-assignment, gamepad slot swap motion,
//! controller removal to an unassigned tray, hardware blacklisting (ignore device),
//! split layout cycling, and 1-click launch.

use std::time::Duration;

use gpui::prelude::*;
use gpui::*;
use nucleus_common::types::DeviceType;
use nucleus_input::gamepad::{DeviceFingerprint, GamepadDevice};

use crate::components::{CouchButton, CouchButtonSize, CouchTag, TagTone, *};
use crate::models::DeviceItem;
use crate::theme;
use crate::window::MainWindow;
use crate::{Icon, IconName};

/// Connection resilience status for an assigned couch player slot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisconnectStatus {
    /// Device is connected and active.
    Connected,
    /// Device disconnected; 10s toast banner is active waiting for replug.
    ReplugPending { remaining_ms: u64, total_ms: u64 },
    /// 10s countdown expired; toast is dismissed, but slot remains preserved.
    DisconnectedSilent,
}

impl Default for DisconnectStatus {
    fn default() -> Self {
        Self::Connected
    }
}

/// A player slot in the Couch Setup screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CouchPlayerSlot {
    /// 0-based player index (0 for Player 1, 1 for Player 2...).
    pub player_index: usize,
    /// Assigned runtime device identifier.
    pub device_id: Option<usize>,
    /// Device friendly name.
    pub device_name: String,
    /// Hardware GUID string.
    pub device_guid: String,
    /// Whether the device is active/connected.
    pub is_connected: bool,
    /// Whether this slot is actively grabbed for swap motion.
    pub is_grabbed: bool,
    /// Connection resilience status.
    pub status: DisconnectStatus,
    /// Device persistent hardware fingerprint.
    pub fingerprint: Option<DeviceFingerprint>,
    /// Optional ProtoInput individual keyboard GUID.
    pub raw_keyboard_guid: Option<String>,
    /// Optional ProtoInput individual keyboard name.
    pub raw_keyboard_name: Option<String>,
    /// Optional ProtoInput individual mouse GUID.
    pub raw_mouse_guid: Option<String>,
    /// Optional ProtoInput individual mouse name.
    pub raw_mouse_name: Option<String>,
}

impl CouchPlayerSlot {
    /// Returns true if this slot is assigned standard paired Keyboard & Mouse.
    pub fn is_paired_kbm(&self) -> bool {
        self.device_guid == "keyboard-mouse-0"
            || self.device_guid == "keyboard-mouse-paired"
            || self.device_name.contains("Keyboard & Mouse")
    }

    /// Returns true if this slot has a ProtoInput separate keyboard assigned.
    pub fn has_raw_keyboard(&self) -> bool {
        self.raw_keyboard_guid.is_some() || self.device_guid.starts_with("raw-keyboard")
    }

    /// Returns true if this slot has a ProtoInput separate mouse assigned.
    pub fn has_raw_mouse(&self) -> bool {
        self.raw_mouse_guid.is_some() || self.device_guid.starts_with("raw-mouse")
    }

    /// Returns true if this slot has both a ProtoInput keyboard and mouse assigned.
    pub fn is_complete_raw_kbm(&self) -> bool {
        self.has_raw_keyboard() && self.has_raw_mouse()
    }

    /// Returns true if this slot is a pure gamepad controller.
    pub fn is_pure_gamepad(&self) -> bool {
        !self.is_paired_kbm()
            && !self.has_raw_keyboard()
            && !self.has_raw_mouse()
            && !is_keyboard_device(&self.device_name, &self.device_guid)
    }

    /// Assigns or updates the ProtoInput keyboard on this slot.
    pub fn assign_raw_keyboard(&mut self, guid: &str, name: &str) {
        self.raw_keyboard_guid = Some(guid.to_string());
        self.raw_keyboard_name = Some(name.to_string());
        self.update_composite_name();
    }

    /// Assigns or updates the ProtoInput mouse on this slot.
    pub fn assign_raw_mouse(&mut self, guid: &str, name: &str) {
        self.raw_mouse_guid = Some(guid.to_string());
        self.raw_mouse_name = Some(name.to_string());
        self.update_composite_name();
    }

    /// Synchronizes friendly composite display name for the slot.
    pub fn update_composite_name(&mut self) {
        if let (Some(kb), Some(mouse)) = (&self.raw_keyboard_name, &self.raw_mouse_name) {
            self.device_name = format!("{} + {}", kb, mouse);
            self.device_guid = format!(
                "{}+{}",
                self.raw_keyboard_guid.as_deref().unwrap_or("kb"),
                self.raw_mouse_guid.as_deref().unwrap_or("m")
            );
        } else if let Some(ref kb) = self.raw_keyboard_name {
            self.device_name = format!("{} (Needs Mouse)", kb);
            self.device_guid = self.raw_keyboard_guid.clone().unwrap_or_default();
        } else if let Some(ref mouse) = self.raw_mouse_name {
            self.device_name = format!("{} (Needs Keyboard)", mouse);
            self.device_guid = self.raw_mouse_guid.clone().unwrap_or_default();
        }
    }
}

/// Available splitscreen layouts for cycling.
pub const AVAILABLE_LAYOUTS: &[&str] = &[
    "Fullscreen (1 Player)",
    "2x1 (Side-by-Side)",
    "1x2 (Stacked)",
    "2x2 (Grid)",
    "Top 1 / Bottom 2",
    "Top 2 / Bottom 1",
    "3x2 (Grid 6)",
    "4x2 (Grid 8)",
    "4x4 (Grid 16)",
    "Multi-Monitor",
];

/// The 4 distinct steps in the Couch Splitscreen Setup wizard (Images 9 & 10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CouchSetupStep {
    #[default]
    Players,
    ScreenLayout,
    GameOptions,
    Launch,
}

impl CouchSetupStep {
    pub const ALL: [CouchSetupStep; 4] = [
        CouchSetupStep::Players,
        CouchSetupStep::ScreenLayout,
        CouchSetupStep::GameOptions,
        CouchSetupStep::Launch,
    ];

    pub const fn count() -> usize {
        Self::ALL.len()
    }

    pub fn index(self) -> usize {
        match self {
            Self::Players => 0,
            Self::ScreenLayout => 1,
            Self::GameOptions => 2,
            Self::Launch => 3,
        }
    }

    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::Players,
            1 => Self::ScreenLayout,
            2 => Self::GameOptions,
            _ => Self::Launch,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Players => "01 Players",
            Self::ScreenLayout => "02 Screen layout",
            Self::GameOptions => "03 Game options",
            Self::Launch => "04 Launch",
        }
    }

    pub fn short_name(self) -> &'static str {
        match self {
            Self::Players => "Players",
            Self::ScreenLayout => "Screen layout",
            Self::GameOptions => "Game options",
            Self::Launch => "Launch",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Players => "Who's playing?",
            Self::ScreenLayout => "A screen for everyone.",
            Self::GameOptions => "Game options",
            Self::Launch => "Launch splitscreen",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Players => Self::ScreenLayout,
            Self::ScreenLayout => Self::GameOptions,
            Self::GameOptions => Self::Launch,
            Self::Launch => Self::Launch,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Players => Self::Players,
            Self::ScreenLayout => Self::Players,
            Self::GameOptions => Self::ScreenLayout,
            Self::Launch => Self::GameOptions,
        }
    }
}

/// State tracking couch player slots, unassigned controllers, and layout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CouchSetupState {
    /// Ordered list of active player slots.
    pub slots: Vec<CouchPlayerSlot>,
    /// Tray of gamepads removed from slots, available for re-assignment.
    pub unassigned_tray: Vec<GamepadDevice>,
    /// Index of currently focused slot card.
    pub focused_slot: usize,
    /// Index of slot currently grabbed for swap motion.
    pub grabbed_slot: Option<usize>,
    /// Currently chosen splitscreen layout name.
    pub selected_layout: String,
    /// Currently active setup wizard step (01 Players -> 02 Screen layout -> 03 Game options -> 04 Launch).
    pub current_step: CouchSetupStep,
}

impl Default for CouchSetupState {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            unassigned_tray: Vec::new(),
            focused_slot: 0,
            grabbed_slot: None,
            selected_layout: "Fullscreen (1 Player)".to_string(),
            current_step: CouchSetupStep::Players,
        }
    }
}

impl CouchSetupState {
    /// Creates a new default `CouchSetupState`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Advances to the next setup step, capped at Step 4 (Launch).
    pub fn next_step(&mut self) {
        self.current_step = self.current_step.next();
    }

    /// Regresses to the previous setup step, capped at Step 1 (Players).
    pub fn prev_step(&mut self) {
        self.current_step = self.current_step.prev();
    }

    /// Sets the active setup step.
    pub fn set_step(&mut self, step: CouchSetupStep) {
        self.current_step = step;
    }

    /// Returns true if the specified step is considered completed.
    pub fn is_step_completed(&self, step: CouchSetupStep) -> bool {
        match step {
            CouchSetupStep::Players => !self.slots.is_empty(),
            CouchSetupStep::ScreenLayout => !self.selected_layout.is_empty(),
            CouchSetupStep::GameOptions => true,
            CouchSetupStep::Launch => false,
        }
    }

    /// Automatically assigns connected devices not in `ignored_guids` to player slots,
    /// optionally including virtual Keyboard & Mouse if `supports_keyboard` is true.
    pub fn auto_assign_with_options(
        &mut self,
        devices: &[GamepadDevice],
        ignored_guids: &[String],
        supports_keyboard: bool,
    ) {
        let mut all_devices: Vec<GamepadDevice> = devices
            .iter()
            .filter(|d| d.is_connected && !ignored_guids.iter().any(|g| g == &d.guid))
            .cloned()
            .collect();

        if supports_keyboard && !ignored_guids.iter().any(|g| g == "keyboard-mouse-0") {
            if !all_devices.iter().any(|d| d.guid == "keyboard-mouse-0") {
                all_devices.push(create_keyboard_mouse_device());
            }
        }

        self.slots = all_devices
            .into_iter()
            .enumerate()
            .map(|(i, d)| CouchPlayerSlot {
                player_index: i,
                device_id: Some(d.id),
                device_name: d.name.clone(),
                device_guid: d.guid.clone(),
                is_connected: d.is_connected,
                is_grabbed: false,
                status: DisconnectStatus::Connected,
                fingerprint: Some(d.device_fingerprint()),
                raw_keyboard_guid: None,
                raw_keyboard_name: None,
                raw_mouse_guid: None,
                raw_mouse_name: None,
            })
            .collect();

        self.grabbed_slot = None;
        self.focused_slot = 0;
        self.selected_layout = match self.slots.len() {
            0 | 1 => "Fullscreen (1 Player)".to_string(),
            2 => "2x1 (Side-by-Side)".to_string(),
            3 => "Top 1 / Bottom 2".to_string(),
            4 => "2x2 (Grid)".to_string(),
            5 | 6 => "3x2 (Grid 6)".to_string(),
            7 | 8 => "4x2 (Grid 8)".to_string(),
            _ => "4x4 (Grid 16)".to_string(),
        };
    }

    /// Automatically assigns all connected devices not in `ignored_guids` to player slots.
    pub fn auto_assign(&mut self, devices: &[GamepadDevice], ignored_guids: &[String]) {
        self.auto_assign_with_options(devices, ignored_guids, false);
    }

    /// Converts UI `DeviceItem` instances to `GamepadDevice` and invokes `auto_assign_with_options`.
    pub fn auto_assign_from_items_with_options(
        &mut self,
        devices: &[DeviceItem],
        ignored_guids: &[String],
        supports_keyboard: bool,
    ) {
        let gamepad_devices: Vec<GamepadDevice> = devices
            .iter()
            .map(|item| GamepadDevice {
                id: item.id,
                name: item.name.clone(),
                guid: item.guid.clone(),
                device_type: if is_keyboard_device(&item.name, &item.guid) {
                    DeviceType::Keyboard
                } else {
                    DeviceType::XInputGamepad
                },
                is_connected: item.is_connected,
            })
            .collect();
        self.auto_assign_with_options(&gamepad_devices, ignored_guids, supports_keyboard);
    }

    /// Converts UI `DeviceItem` instances to `GamepadDevice` and invokes `auto_assign`.
    pub fn auto_assign_from_items(&mut self, devices: &[DeviceItem], ignored_guids: &[String]) {
        self.auto_assign_from_items_with_options(devices, ignored_guids, false);
    }

    /// Grabs a slot card to initiate swap motion.
    pub fn grab_slot(&mut self, slot_idx: usize) {
        if slot_idx < self.slots.len() {
            for (i, slot) in self.slots.iter_mut().enumerate() {
                slot.is_grabbed = i == slot_idx;
            }
            self.grabbed_slot = Some(slot_idx);
            self.focused_slot = slot_idx;
        }
    }

    /// Drops the grabbed slot onto `target_idx`, interchanging device assignments,
    /// or merging a raw mouse and raw keyboard into a single composite slot.
    pub fn drop_swap_target(&mut self, target_idx: usize) {
        if let Some(grabbed) = self.grabbed_slot {
            if grabbed < self.slots.len() && target_idx < self.slots.len() && grabbed != target_idx
            {
                let is_mouse_onto_kb = self.slots[grabbed].has_raw_mouse()
                    && !self.slots[grabbed].has_raw_keyboard()
                    && self.slots[target_idx].has_raw_keyboard()
                    && !self.slots[target_idx].has_raw_mouse();
                let is_kb_onto_mouse = self.slots[grabbed].has_raw_keyboard()
                    && !self.slots[grabbed].has_raw_mouse()
                    && self.slots[target_idx].has_raw_mouse()
                    && !self.slots[target_idx].has_raw_keyboard();

                if is_mouse_onto_kb {
                    let mouse_guid = self.slots[grabbed]
                        .raw_mouse_guid
                        .clone()
                        .unwrap_or_else(|| self.slots[grabbed].device_guid.clone());
                    let mouse_name = self.slots[grabbed]
                        .raw_mouse_name
                        .clone()
                        .unwrap_or_else(|| self.slots[grabbed].device_name.clone());
                    self.slots[target_idx].assign_raw_mouse(&mouse_guid, &mouse_name);
                    self.slots.remove(grabbed);
                    self.cancel_grab();
                    for (i, s) in self.slots.iter_mut().enumerate() {
                        s.player_index = i;
                    }
                    self.update_layout_for_slot_count();
                    self.focused_slot = if target_idx >= self.slots.len() {
                        self.slots.len().saturating_sub(1)
                    } else {
                        target_idx
                    };
                    return;
                } else if is_kb_onto_mouse {
                    let kb_guid = self.slots[grabbed]
                        .raw_keyboard_guid
                        .clone()
                        .unwrap_or_else(|| self.slots[grabbed].device_guid.clone());
                    let kb_name = self.slots[grabbed]
                        .raw_keyboard_name
                        .clone()
                        .unwrap_or_else(|| self.slots[grabbed].device_name.clone());
                    self.slots[target_idx].assign_raw_keyboard(&kb_guid, &kb_name);
                    self.slots.remove(grabbed);
                    self.cancel_grab();
                    for (i, s) in self.slots.iter_mut().enumerate() {
                        s.player_index = i;
                    }
                    self.update_layout_for_slot_count();
                    self.focused_slot = if target_idx >= self.slots.len() {
                        self.slots.len().saturating_sub(1)
                    } else {
                        target_idx
                    };
                    return;
                }

                let g_id = self.slots[grabbed].device_id;
                let g_name = self.slots[grabbed].device_name.clone();
                let g_guid = self.slots[grabbed].device_guid.clone();
                let g_conn = self.slots[grabbed].is_connected;
                let g_status = self.slots[grabbed].status.clone();
                let g_fp = self.slots[grabbed].fingerprint.clone();
                let g_rkb_g = self.slots[grabbed].raw_keyboard_guid.clone();
                let g_rkb_n = self.slots[grabbed].raw_keyboard_name.clone();
                let g_rm_g = self.slots[grabbed].raw_mouse_guid.clone();
                let g_rm_n = self.slots[grabbed].raw_mouse_name.clone();

                self.slots[grabbed].device_id = self.slots[target_idx].device_id;
                self.slots[grabbed].device_name = self.slots[target_idx].device_name.clone();
                self.slots[grabbed].device_guid = self.slots[target_idx].device_guid.clone();
                self.slots[grabbed].is_connected = self.slots[target_idx].is_connected;
                self.slots[grabbed].status = self.slots[target_idx].status.clone();
                self.slots[grabbed].fingerprint = self.slots[target_idx].fingerprint.clone();
                self.slots[grabbed].raw_keyboard_guid =
                    self.slots[target_idx].raw_keyboard_guid.clone();
                self.slots[grabbed].raw_keyboard_name =
                    self.slots[target_idx].raw_keyboard_name.clone();
                self.slots[grabbed].raw_mouse_guid = self.slots[target_idx].raw_mouse_guid.clone();
                self.slots[grabbed].raw_mouse_name = self.slots[target_idx].raw_mouse_name.clone();

                self.slots[target_idx].device_id = g_id;
                self.slots[target_idx].device_name = g_name;
                self.slots[target_idx].device_guid = g_guid;
                self.slots[target_idx].is_connected = g_conn;
                self.slots[target_idx].status = g_status;
                self.slots[target_idx].fingerprint = g_fp;
                self.slots[target_idx].raw_keyboard_guid = g_rkb_g;
                self.slots[target_idx].raw_keyboard_name = g_rkb_n;
                self.slots[target_idx].raw_mouse_guid = g_rm_g;
                self.slots[target_idx].raw_mouse_name = g_rm_n;
            }
            self.cancel_grab();
            if target_idx < self.slots.len() {
                self.focused_slot = target_idx;
            }
        }
    }

    /// Splits a merged ProtoInput keyboard and mouse slot into two separate player slots.
    pub fn split_raw_kbm(&mut self, slot_idx: usize) {
        if slot_idx < self.slots.len() && self.slots[slot_idx].is_complete_raw_kbm() {
            let mouse_guid = self.slots[slot_idx].raw_mouse_guid.take();
            let mouse_name = self.slots[slot_idx].raw_mouse_name.take();
            self.slots[slot_idx].update_composite_name();

            if let (Some(m_guid), Some(m_name)) = (mouse_guid, mouse_name) {
                let new_idx = self.slots.len();
                let mut new_slot = CouchPlayerSlot {
                    player_index: new_idx,
                    device_id: Some(8100 + new_idx),
                    device_name: m_name.clone(),
                    device_guid: m_guid.clone(),
                    is_connected: true,
                    is_grabbed: false,
                    status: DisconnectStatus::Connected,
                    fingerprint: None,
                    raw_keyboard_guid: None,
                    raw_keyboard_name: None,
                    raw_mouse_guid: Some(m_guid),
                    raw_mouse_name: Some(m_name),
                };
                new_slot.update_composite_name();
                self.slots.push(new_slot);
                self.update_layout_for_slot_count();
            }
        }
    }

    /// Explicitly assigns or adds a paired Keyboard & Mouse device to player slots.
    pub fn add_paired_kbm(&mut self) {
        if !self.slots.iter().any(|s| s.is_paired_kbm()) {
            let new_idx = self.slots.len();
            self.slots.push(CouchPlayerSlot {
                player_index: new_idx,
                device_id: Some(9999),
                device_name: "Keyboard & Mouse (Paired)".to_string(),
                device_guid: "keyboard-mouse-0".to_string(),
                is_connected: true,
                is_grabbed: false,
                status: DisconnectStatus::Connected,
                fingerprint: None,
                raw_keyboard_guid: None,
                raw_keyboard_name: None,
                raw_mouse_guid: None,
                raw_mouse_name: None,
            });
            self.update_layout_for_slot_count();
        }
    }

    /// Explicitly adds or assigns an individual ProtoInput keyboard to the focused or next slot.
    pub fn add_raw_keyboard(&mut self, kb_idx: usize) {
        let guid = format!("raw-keyboard-{}", kb_idx);
        let name = format!("Keyboard #{}", kb_idx + 1);
        if !self.slots.is_empty() && self.focused_slot < self.slots.len() {
            self.slots[self.focused_slot].assign_raw_keyboard(&guid, &name);
        } else {
            let new_idx = self.slots.len();
            let mut slot = CouchPlayerSlot {
                player_index: new_idx,
                device_id: Some(8000 + kb_idx),
                device_name: name.clone(),
                device_guid: guid.clone(),
                is_connected: true,
                is_grabbed: false,
                status: DisconnectStatus::Connected,
                fingerprint: None,
                raw_keyboard_guid: Some(guid),
                raw_keyboard_name: Some(name),
                raw_mouse_guid: None,
                raw_mouse_name: None,
            };
            slot.update_composite_name();
            self.slots.push(slot);
            self.update_layout_for_slot_count();
        }
    }

    /// Explicitly adds or assigns an individual ProtoInput mouse to the focused or next slot.
    pub fn add_raw_mouse(&mut self, m_idx: usize) {
        let guid = format!("raw-mouse-{}", m_idx);
        let name = format!("Mouse #{}", m_idx + 1);
        if !self.slots.is_empty() && self.focused_slot < self.slots.len() {
            self.slots[self.focused_slot].assign_raw_mouse(&guid, &name);
        } else {
            let new_idx = self.slots.len();
            let mut slot = CouchPlayerSlot {
                player_index: new_idx,
                device_id: Some(8100 + m_idx),
                device_name: name.clone(),
                device_guid: guid.clone(),
                is_connected: true,
                is_grabbed: false,
                status: DisconnectStatus::Connected,
                fingerprint: None,
                raw_keyboard_guid: None,
                raw_keyboard_name: None,
                raw_mouse_guid: Some(guid),
                raw_mouse_name: Some(name),
            };
            slot.update_composite_name();
            self.slots.push(slot);
            self.update_layout_for_slot_count();
        }
    }

    /// Automatically updates layout when slots are added or removed.
    pub fn update_layout_for_slot_count(&mut self) {
        self.selected_layout = match self.slots.len() {
            0 | 1 => "Fullscreen (1 Player)".to_string(),
            2 => "2x1 (Side-by-Side)".to_string(),
            3 => "Top 1 / Bottom 2".to_string(),
            4 => "2x2 (Grid)".to_string(),
            5 | 6 => "3x2 (Grid 6)".to_string(),
            7 | 8 => "4x2 (Grid 8)".to_string(),
            _ => "4x4 (Grid 16)".to_string(),
        };
    }

    /// Cancels active grab motion without swapping.
    pub fn cancel_grab(&mut self) {
        self.grabbed_slot = None;
        for slot in &mut self.slots {
            slot.is_grabbed = false;
        }
    }

    /// Removes a player slot and moves the device to `unassigned_tray`.
    pub fn remove_slot(&mut self, slot_idx: usize) {
        if slot_idx < self.slots.len() {
            let removed = self.slots.remove(slot_idx);
            let dev_type = if is_keyboard_device(&removed.device_name, &removed.device_guid) {
                DeviceType::Keyboard
            } else {
                DeviceType::XInputGamepad
            };
            self.unassigned_tray.push(GamepadDevice {
                id: removed.device_id.unwrap_or(0),
                name: removed.device_name,
                guid: removed.device_guid,
                device_type: dev_type,
                is_connected: removed.is_connected,
            });
            self.cancel_grab();
            for (i, slot) in self.slots.iter_mut().enumerate() {
                slot.player_index = i;
            }
            if self.slots.is_empty() {
                self.focused_slot = 0;
            } else {
                self.focused_slot = self.focused_slot.min(self.slots.len() - 1);
            }
        }
    }

    /// Reclaims an unassigned device from `unassigned_tray` into a new slot.
    pub fn reclaim_unassigned(&mut self, tray_idx: usize) {
        if tray_idx < self.unassigned_tray.len() {
            let dev = self.unassigned_tray.remove(tray_idx);
            let new_idx = self.slots.len();
            let fp = dev.device_fingerprint();
            self.slots.push(CouchPlayerSlot {
                player_index: new_idx,
                device_id: Some(dev.id),
                device_name: dev.name,
                device_guid: dev.guid,
                is_connected: dev.is_connected,
                is_grabbed: false,
                status: DisconnectStatus::Connected,
                fingerprint: Some(fp),
                raw_keyboard_guid: None,
                raw_keyboard_name: None,
                raw_mouse_guid: None,
                raw_mouse_name: None,
            });
        }
    }

    /// Permanently ignores a hardware device GUID and purges matching slots/tray items.
    pub fn ignore_device(&mut self, guid: &str, ignored_guids: &mut Vec<String>) {
        if !ignored_guids.iter().any(|g| g == guid) {
            ignored_guids.push(guid.to_string());
        }
        self.slots.retain(|s| s.device_guid != guid);
        self.unassigned_tray.retain(|d| d.guid != guid);
        self.cancel_grab();
        for (i, slot) in self.slots.iter_mut().enumerate() {
            slot.player_index = i;
        }
        if self.slots.is_empty() {
            self.focused_slot = 0;
        } else {
            self.focused_slot = self.focused_slot.min(self.slots.len() - 1);
        }
    }

    /// Restores a previously ignored device, removing its GUID from `ignored_guids`
    /// and re-adding it to an active player slot if not already present.
    pub fn restore_device(&mut self, device: &GamepadDevice, ignored_guids: &mut Vec<String>) {
        ignored_guids.retain(|g| g != &device.guid);
        self.unassigned_tray.retain(|d| d.guid != device.guid);
        if !self.slots.iter().any(|s| s.device_guid == device.guid) {
            let new_idx = self.slots.len();
            let fp = device.device_fingerprint();
            self.slots.push(CouchPlayerSlot {
                player_index: new_idx,
                device_id: Some(device.id),
                device_name: device.name.clone(),
                device_guid: device.guid.clone(),
                is_connected: device.is_connected,
                is_grabbed: false,
                status: DisconnectStatus::Connected,
                fingerprint: Some(fp),
                raw_keyboard_guid: None,
                raw_keyboard_name: None,
                raw_mouse_guid: None,
                raw_mouse_name: None,
            });
        }
    }

    /// Converts UI `DeviceItem` to `GamepadDevice` and restores it via `restore_device`.
    pub fn restore_device_from_item(&mut self, item: &DeviceItem, ignored_guids: &mut Vec<String>) {
        let dev_type = if is_keyboard_device(&item.name, &item.guid) {
            DeviceType::Keyboard
        } else {
            DeviceType::XInputGamepad
        };
        let dev = GamepadDevice {
            id: item.id,
            name: item.name.clone(),
            guid: item.guid.clone(),
            device_type: dev_type,
            is_connected: item.is_connected,
        };
        self.restore_device(&dev, ignored_guids);
    }

    /// Cycles through supported splitscreen layouts.
    pub fn cycle_layout(&mut self, forward: bool) {
        let cur_idx = AVAILABLE_LAYOUTS
            .iter()
            .position(|&l| l == self.selected_layout)
            .unwrap_or(0);
        let next_idx = if forward {
            (cur_idx + 1) % AVAILABLE_LAYOUTS.len()
        } else if cur_idx == 0 {
            AVAILABLE_LAYOUTS.len() - 1
        } else {
            cur_idx - 1
        };
        self.selected_layout = AVAILABLE_LAYOUTS[next_idx].to_string();
    }

    /// Navigates focused slot to the left with wrapping.
    pub fn navigate_slot_left(&mut self) {
        if self.slots.is_empty() {
            self.focused_slot = 0;
            return;
        }
        if self.focused_slot == 0 {
            self.focused_slot = self.slots.len() - 1;
        } else {
            self.focused_slot -= 1;
        }
    }

    /// Navigates focused slot to the right with wrapping.
    pub fn navigate_slot_right(&mut self) {
        if self.slots.is_empty() {
            self.focused_slot = 0;
            return;
        }
        if self.focused_slot + 1 >= self.slots.len() {
            self.focused_slot = 0;
        } else {
            self.focused_slot += 1;
        }
    }

    /// Marks the slot at `slot_idx` as disconnected and begins 10-second replug countdown.
    pub fn handle_device_disconnect(&mut self, slot_idx: usize) {
        if slot_idx < self.slots.len() {
            self.slots[slot_idx].is_connected = false;
            self.slots[slot_idx].status = DisconnectStatus::ReplugPending {
                remaining_ms: 10_000,
                total_ms: 10_000,
            };
        }
    }

    /// Finds slot assigned to `dev_id` and marks it as disconnected.
    pub fn handle_device_disconnect_by_id(&mut self, dev_id: usize) {
        if let Some(idx) = self.slots.iter().position(|s| s.device_id == Some(dev_id)) {
            self.handle_device_disconnect(idx);
        }
    }

    /// Attempts to restore a disconnected player slot when a device is re-detected.
    /// Matches by GUID or persistent hardware fingerprint.
    pub fn handle_device_reconnect(
        &mut self,
        dev_id: usize,
        guid: &str,
        name: &str,
        fingerprint: Option<&nucleus_input::gamepad::DeviceFingerprint>,
    ) -> bool {
        let target_idx = self.slots.iter().position(|slot| {
            let is_disconnected = !slot.is_connected || slot.status != DisconnectStatus::Connected;
            if !is_disconnected {
                return false;
            }
            if let Some(fp) = fingerprint {
                if let Some(ref slot_fp) = slot.fingerprint {
                    if slot_fp == fp {
                        return true;
                    }
                }
            }
            slot.device_guid == guid
        });

        if let Some(idx) = target_idx {
            let slot = &mut self.slots[idx];
            slot.device_id = Some(dev_id);
            slot.device_name = name.to_string();
            slot.is_connected = true;
            slot.status = DisconnectStatus::Connected;
            if let Some(fp) = fingerprint {
                slot.fingerprint = Some(fp.clone());
            } else if slot.fingerprint.is_none() {
                slot.fingerprint = Some(DeviceFingerprint::new(
                    guid,
                    name,
                    &DeviceType::XInputGamepad,
                ));
            }
            true
        } else {
            false
        }
    }

    /// Ticks disconnect resilience countdowns for all pending slots.
    pub fn tick_disconnect(&mut self, delta: Duration) {
        for slot in &mut self.slots {
            if let DisconnectStatus::ReplugPending {
                ref mut remaining_ms,
                ..
            } = slot.status
            {
                let delta_ms = delta.as_millis() as u64;
                if *remaining_ms <= delta_ms {
                    slot.status = DisconnectStatus::DisconnectedSilent;
                } else {
                    *remaining_ms -= delta_ms;
                }
            }
        }
    }

    /// Returns the first slot actively waiting in `ReplugPending` status, along with countdown progress.
    pub fn active_disconnect_banner(&self) -> Option<(usize, &CouchPlayerSlot, u64, u64)> {
        self.slots
            .iter()
            .enumerate()
            .find_map(|(idx, slot)| match slot.status {
                DisconnectStatus::ReplugPending {
                    remaining_ms,
                    total_ms,
                } => Some((idx, slot, remaining_ms, total_ms)),
                _ => None,
            })
    }

    /// Returns the active partition layout kind based on `selected_layout`.
    pub fn partition_layout_kind(&self) -> PartitionLayoutKind {
        PartitionLayoutKind::from_layout_name(&self.selected_layout)
    }

    /// Returns the player slot assigned to a given partition slot index (0..N), if assigned.
    pub fn partition_slot(&self, partition_idx: usize) -> Option<&CouchPlayerSlot> {
        self.slots.get(partition_idx)
    }
}

/// Helper to construct a virtual "Keyboard & Mouse" device for hybrid couch play.
pub fn create_keyboard_mouse_device() -> GamepadDevice {
    GamepadDevice {
        id: 9999,
        name: "Keyboard & Mouse".to_string(),
        guid: "keyboard-mouse-0".to_string(),
        device_type: DeviceType::Keyboard,
        is_connected: true,
    }
}

/// Helper to determine if a device name or GUID represents a keyboard/mouse.
pub fn is_keyboard_device(device_name: &str, device_guid: &str) -> bool {
    let lower_name = device_name.to_lowercase();
    let lower_guid = device_guid.to_lowercase();
    lower_name.contains("keyboard") || lower_guid.contains("keyboard")
}

/// Identifies the partition geometry variant from the layout string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PartitionLayoutKind {
    Fullscreen,
    SideBySide,
    Stacked,
    Grid,
    TopOneBottomTwo,
    TopTwoBottomOne,
    Grid6,
    Grid8,
    Grid16,
    MultiMonitor,
}

impl PartitionLayoutKind {
    /// Resolves the partition layout variant from a layout string.
    pub fn from_layout_name(name: &str) -> Self {
        let lower = name.to_lowercase();
        if lower.contains("fullscreen") || lower.contains("1x1") || lower.contains("single") {
            Self::Fullscreen
        } else if lower.contains("stacked") || lower.contains("1x2") {
            Self::Stacked
        } else if lower.contains("2x3")
            || lower.contains("3x2")
            || lower.contains("grid 6")
            || lower.contains("grid6")
        {
            Self::Grid6
        } else if lower.contains("2x4")
            || lower.contains("4x2")
            || lower.contains("grid 8")
            || lower.contains("grid8")
        {
            Self::Grid8
        } else if lower.contains("4x4") || lower.contains("grid 16") || lower.contains("grid16") {
            Self::Grid16
        } else if lower.contains("multi-monitor")
            || lower.contains("multimonitor")
            || lower.contains("multi monitor")
        {
            Self::MultiMonitor
        } else if lower.contains("grid") || lower.contains("2x2") {
            Self::Grid
        } else if lower.contains("top 1") || lower.contains("top 1 / bottom 2") {
            Self::TopOneBottomTwo
        } else if lower.contains("top 2") || lower.contains("top 2 / bottom 1") {
            Self::TopTwoBottomOne
        } else {
            Self::SideBySide
        }
    }

    /// Number of partition viewport regions required for this layout.
    pub fn slot_count(&self) -> usize {
        match self {
            Self::Fullscreen => 1,
            Self::SideBySide => 2,
            Self::Stacked => 2,
            Self::TopOneBottomTwo => 3,
            Self::TopTwoBottomOne => 3,
            Self::Grid => 4,
            Self::Grid6 => 6,
            Self::Grid8 => 8,
            Self::Grid16 => 16,
            Self::MultiMonitor => 2,
        }
    }
}

/// Renders floating on-screen disconnect toast banner with animated countdown bar.
pub fn render_disconnect_toast_banner(
    slot_idx: usize,
    _slot: &CouchPlayerSlot,
    remaining_ms: u64,
    total_ms: u64,
) -> AnyElement {
    let ratio = if total_ms > 0 {
        (remaining_ms as f32 / total_ms as f32).clamp(0.0, 1.0)
    } else {
        0.0
    };

    div()
        .id("disconnect-toast-banner")
        .absolute()
        .top_6()
        .left_0()
        .right_0()
        .flex()
        .justify_center()
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .w(px(540.0))
                .p_4()
                .rounded_xl()
                .bg(theme::bg_panel())
                .border_2()
                .border_color(theme::warning())
                .shadow_lg()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .w_full()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_3()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .size(px(32.0))
                                        .rounded_full()
                                        .bg(theme::warning_subtle())
                                        .child(
                                            Icon::new(IconName::TriangleAlert)
                                                .size_5()
                                                .text_color(theme::warning()),
                                        ),
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
                                                .child(format!(
                                                    "Player {} controller disconnected!",
                                                    slot_idx + 1
                                                )),
                                        )
                                        .child(
                                            div().text_xs().text_color(theme::text_muted()).child(
                                                "Replug USB / reconnect Bluetooth to resume.",
                                            ),
                                        ),
                                ),
                        )
                        .child(
                            div()
                                .px_2p5()
                                .py_1()
                                .rounded_md()
                                .bg(theme::bg_subtle())
                                .border_1()
                                .border_color(theme::border_subtle())
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::warning())
                                .child(format!("{:.1}s", remaining_ms as f32 / 1000.0)),
                        ),
                )
                // Animated countdown progress bar track
                .child(
                    div()
                        .w_full()
                        .h(px(4.0))
                        .rounded_full()
                        .bg(theme::bg_subtle())
                        .overflow_hidden()
                        .child(
                            div()
                                .h_full()
                                .w(relative(ratio))
                                .bg(theme::warning())
                                .rounded_full(),
                        ),
                ),
        )
        .into_any_element()
}

/// Renders the complete Couch Splitscreen Setup view with 4-step wizard stepper.
pub fn render_couch_setup_view(win: &MainWindow, cx: &mut Context<MainWindow>) -> AnyElement {
    let state = &win.couch_setup_state;

    div()
        .id("couch-setup-view")
        .flex()
        .flex_col()
        .size_full()
        .p_8()
        .pb_16()
        .gap_6()
        .overflow_y_scroll()
        .child(render_couch_setup_header(win, state, cx))
        .child(match state.current_step {
            CouchSetupStep::Players => render_step_1_players(win, state, cx),
            CouchSetupStep::ScreenLayout => render_step_2_layout(win, state, cx),
            CouchSetupStep::GameOptions => render_step_3_options(win, state, cx),
            CouchSetupStep::Launch => render_step_4_launch(win, state, cx),
        })
        .into_any_element()
}

/// Step 1: "Who's playing?" (Image 9)
fn render_step_1_players(
    win: &MainWindow,
    state: &CouchSetupState,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    div()
        .id("couch-step-1-players")
        .flex()
        .flex_row()
        .items_start()
        .gap_6()
        .w_full()
        // Left Column: Player Slots & Trays (~60%)
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap_6()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child("Who's playing?"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::text_muted())
                                .child("Assign controllers to player slots or drag to swap. Disconnected gamepads show replug countdown."),
                        ),
                )
                .child(render_km_assignment_bar(win, state, cx))
                .child(render_slots_grid(win, state, cx))
                .children(render_unassigned_tray(state, cx))
                .children(render_ignored_controllers_tray(win, state, cx))
                .child({
                    let is_next_focused = state.focused_slot == state.slots.len();
                    div()
                        .flex()
                        .items_center()
                        .justify_end()
                        .pt_4()
                        .child(
                            CouchButton::primary("btn-next-step-layout", "Next: Screen layout")
                                .icon(IconName::ArrowRight)
                                .size(CouchButtonSize::Lg)
                                .focused(is_next_focused)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.couch_setup_state.next_step();
                                    this.couch_setup_state.focused_slot = 0;
                                    cx.notify();
                                })),
                        )
                })
                .into_any_element(),
        )
        // Right Column: Your Screen Preview & Let each player join card (~40% / 440px)
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(440.0))
                .flex_shrink_0()
                .min_w_0()
                .gap_5()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .p_4()
                        .rounded_xl()
                        .bg(theme::bg_panel())
                        .border_1()
                        .border_color(theme::border_color())
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child(format!("Your screen ({})", state.selected_layout)),
                        )
                        .child(render_virtual_monitor_partition_box(win, state, cx)),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .p_4()
                        .rounded_xl()
                        .bg(theme::bg_panel())
                        .border_1()
                        .border_color(theme::border_color())
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(Icon::new(IconName::Users).size_4().text_color(theme::accent()))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child("Let each player join."),
                                ),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::text_muted())
                                .child("Press any button on a controller to identify it. Press A to grab and swap slots. Press X to remove a player. Press Y to exclude an unwanted hardware device."),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_wrap()
                                .gap_2()
                                .child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded_md()
                                        .bg(theme::bg_subtle())
                                        .border_1()
                                        .border_color(theme::border_subtle())
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child("A: Swap Slots"),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded_md()
                                        .bg(theme::bg_subtle())
                                        .border_1()
                                        .border_color(theme::border_subtle())
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::danger())
                                        .child("X: Remove Slot"),
                                )
                                .child(
                                    div()
                                        .px_2()
                                        .py_1()
                                        .rounded_md()
                                        .bg(theme::bg_subtle())
                                        .border_1()
                                        .border_color(theme::border_subtle())
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::warning())
                                        .child("Y: Ignore Controller"),
                                ),
                        ),
                )
                .into_any_element(),
        )
        .into_any_element()
}

/// Step 2: "A screen for everyone." (Image 10)
fn render_step_2_layout(
    win: &MainWindow,
    state: &CouchSetupState,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let player_count = state.slots.len().max(1);

    div()
        .id("couch-step-2-layout")
        .flex()
        .flex_row()
        .items_start()
        .gap_6()
        .w_full()
        // Left Column: Selectable Layout Cards
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap_6()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child("A screen for everyone."),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::text_muted())
                                .child("Choose how your display is split across players. Layouts adapt automatically to connected players."),
                        ),
                )
                .child(
                    div()
                        .id("layout-cards-grid")
                        .grid()
                        .grid_cols(2)
                        .gap_3()
                        .max_h(px(320.0))
                        .overflow_y_scroll()
                        .children(AVAILABLE_LAYOUTS.iter().enumerate().map(|(idx, &layout)| {
                            let is_selected = state.selected_layout == layout;
                            let is_card_focused = state.focused_slot == idx;
                            let icon = layout_icon_for_name(layout);
                            let min_players = match layout {
                                "Fullscreen (1 Player)" => 1,
                                "2x1 (Side-by-Side)" | "1x2 (Stacked)" | "Multi-Monitor" => 2,
                                "Top 1 / Bottom 2" | "Top 2 / Bottom 1" => 3,
                                "2x2 (Grid)" => 4,
                                "3x2 (Grid 6)" => 6,
                                "4x2 (Grid 8)" => 8,
                                _ => 16,
                            };
                            let layout_str = layout.to_string();

                            div()
                                .id(SharedString::from(format!("layout-card-{}", idx)))
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .justify_between()
                                .p_3p5()
                                .rounded_xl()
                                .border_2()
                                .border_color(if is_card_focused {
                                    rgb(0x22c55e).into()
                                } else if is_selected {
                                    theme::accent()
                                } else {
                                    theme::border_subtle()
                                })
                                .bg(if is_card_focused || is_selected {
                                    theme::accent().opacity(0.12)
                                } else {
                                    theme::bg_panel()
                                })
                                .hover(|s| s.bg(theme::bg_hover()))
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.couch_setup_state.selected_layout = layout_str.clone();
                                    this.selected_layout = layout_str.clone();
                                    cx.notify();
                                }))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_3()
                                        .child(
                                            div()
                                                .p_2()
                                                .rounded_lg()
                                                .bg(if is_selected { theme::accent().opacity(0.2) } else { theme::bg_subtle() })
                                                .child(Icon::new(icon).size_5().text_color(if is_selected { theme::accent() } else { theme::text_muted() })),
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
                                                        .text_color(if is_selected { theme::accent() } else { theme::text_primary() })
                                                        .child(layout),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(theme::text_muted())
                                                        .child(format!("Min {} player{}", min_players, if min_players == 1 { "" } else { "s" })),
                                                ),
                                        ),
                                )
                                .child(if is_selected {
                                    div()
                                        .p_1()
                                        .rounded_full()
                                        .bg(theme::accent())
                                        .child(Icon::new(IconName::Check).size_3p5().text_color(theme::text_on_accent()))
                                } else {
                                    div()
                                })
                                .into_any_element()
                        })),
                )
                // Navigation buttons
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .pt_4()
                        .child(
                            CouchButton::secondary("btn-back-to-players", "Back to Players")
                                .icon(IconName::ArrowLeft)
                                .size(CouchButtonSize::Lg)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.couch_setup_state.prev_step();
                                    cx.notify();
                                })),
                        )
                        .child(
                            CouchButton::primary("btn-next-to-options", "Next: Game options")
                                .icon(IconName::ArrowRight)
                                .size(CouchButtonSize::Lg)
                                .focused(state.focused_slot >= AVAILABLE_LAYOUTS.len())
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.couch_setup_state.next_step();
                                    this.couch_setup_state.focused_slot = 0;
                                    cx.notify();
                                })),
                        ),
                )
                .into_any_element(),
        )
        // Right Column: Virtual Monitor Partition Preview Card
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(440.0))
                .flex_shrink_0()
                .min_w_0()
                .gap_5()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_3()
                        .p_4()
                        .rounded_xl()
                        .bg(theme::bg_panel())
                        .border_1()
                        .border_color(theme::border_color())
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child({
                                    let active_disp_idx = win.player_monitor_assignments.first().copied().unwrap_or(0);
                                    let (disp_name, disp_res, is_primary) = if let Some(d) = win.connected_displays.get(active_disp_idx) {
                                        (d.name.clone(), format!("{} × {}", d.bounds.width, d.bounds.height), d.is_primary)
                                    } else {
                                        ("Display 1".to_string(), "1920 × 1080".to_string(), true)
                                    };
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child(format!("{} — {} · {}", disp_name, disp_res, if is_primary { "Primary display" } else { "Display" }))
                                })
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme::text_muted())
                                        .child(format!("{} players", player_count)),
                                ),
                        )
                        .child(render_virtual_monitor_partition_box(win, state, cx))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .pt_2()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme::text_muted())
                                        .child(format!("1 display · {} player screen{}", player_count, if player_count == 1 { "" } else { "s" })),
                                )
                                .child(
                                    CouchButton::secondary("btn-change-display", "Change display")
                                        .size(CouchButtonSize::Sm)
                                        .icon(IconName::Monitor)
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            if !this.connected_displays.is_empty() {
                                                let current_disp = this.player_monitor_assignments.first().copied().unwrap_or(0);
                                                let next_disp = (current_disp + 1) % this.connected_displays.len();
                                                let num_players = this.couch_setup_state.slots.len().max(1);
                                                this.player_monitor_assignments.resize(num_players, next_disp);
                                                for slot in 0..this.player_monitor_assignments.len() {
                                                    this.player_monitor_assignments[slot] = next_disp;
                                                }
                                                let d = &this.connected_displays[next_disp];
                                                this.status_message = format!("Switched to {}: {} ({}x{})", if d.is_primary { "Primary Display" } else { "Display" }, d.name, d.bounds.width, d.bounds.height);
                                            } else {
                                                this.status_message = "No additional displays detected.".to_string();
                                            }
                                            cx.notify();
                                        })),
                                ),
                        ),
                )
                .into_any_element(),
        )
        .into_any_element()
}

/// Step 3: "Game options"
fn render_step_3_options(
    win: &MainWindow,
    state: &CouchSetupState,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let game_name = win.selected_game.as_deref().unwrap_or("No Game Selected");
    let handler = win.handlers.iter().find(|h| {
        h.game_name.eq_ignore_ascii_case(game_name) || h.guid.eq_ignore_ascii_case(game_name)
    });
    let has_options = handler.map(|h| !h.options.is_empty()).unwrap_or(false);

    div()
        .id("couch-step-3-options")
        .flex()
        .flex_row()
        .items_start()
        .gap_6()
        .w_full()
        // Left Column: Custom Handler Options & Language
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap_6()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child("Game options"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::text_muted())
                                .child("Configure handler options, game language, and execution parameters."),
                        ),
                )
                .child(
                    div()
                        .id("game-options-scroll-list")
                        .flex()
                        .flex_col()
                        .gap_3()
                        .max_h(px(340.0))
                        .overflow_y_scroll()
                        .children(if has_options {
                            let opts = handler.unwrap().options.clone();
                            let entity = cx.entity().clone();
                            opts.into_iter().enumerate().map(|(idx, opt)| {
                                let values = opt.values.clone();
                                let opt_count = values.len();
                                let current_val = win.selected_handler_options.get(&opt.key).cloned().unwrap_or_else(|| values.first().cloned().unwrap_or_default());

                                if opt_count > 4 {
                                    let opt_key_for_select = opt.key.clone();
                                    let opt_key_for_toggle = opt.key.clone();
                                    let is_open = win.setup_screen.combobox_open_key.as_deref() == Some(opt.key.as_str());
                                    let draft = if is_open { win.setup_screen.combobox_search_draft.clone() } else { String::new() };
                                    let validation_error = if is_open { win.setup_screen.combobox_validation_error.clone() } else { None };
                                    let entity_select = entity.clone();

                                    CouchOptionCombobox::new(
                                        format!("opt-combo-{}", idx),
                                        opt.name.clone(),
                                        opt.description.clone(),
                                        current_val,
                                        values,
                                    )
                                    .focused(state.focused_slot == idx)
                                    .is_open(is_open)
                                    .search_draft(draft)
                                    .validation_error(validation_error)
                                    .on_toggle(cx.listener(move |this, _, _, cx| {
                                        this.setup_screen.toggle_combobox(&opt_key_for_toggle);
                                        cx.notify();
                                    }))
                                    .on_select(move |val, _window, app| {
                                        let _ = entity_select.update(app, |this, cx| {
                                            this.selected_handler_options.insert(opt_key_for_select.clone(), val);
                                            this.setup_screen.close_combobox();
                                            cx.notify();
                                        });
                                    })
                                    .into_any_element()
                                } else {
                                    let opt_key_prev = opt.key.clone();
                                    let opt_key_next = opt.key.clone();
                                    let vals_prev = values.clone();
                                    let vals_next = values.clone();
                                    let cur_prev = current_val.clone();
                                    let cur_next = current_val.clone();

                                    CouchOptionRow::new(
                                        format!("opt-row-{}", idx),
                                        opt.name.clone(),
                                        opt.description.clone(),
                                        current_val,
                                    )
                                    .focused(state.focused_slot == idx)
                                    .on_prev(cx.listener(move |this, _, _, cx| {
                                        if let Some(pos) = vals_prev.iter().position(|v| v == &cur_prev) {
                                            let prev_idx = (pos + vals_prev.len() - 1) % vals_prev.len();
                                            if let Some(new_val) = vals_prev.get(prev_idx) {
                                                this.selected_handler_options.insert(opt_key_prev.clone(), new_val.clone());
                                            }
                                        } else if let Some(first) = vals_prev.first() {
                                            this.selected_handler_options.insert(opt_key_prev.clone(), first.clone());
                                        }
                                        cx.notify();
                                    }))
                                    .on_next(cx.listener(move |this, _, _, cx| {
                                        if let Some(pos) = vals_next.iter().position(|v| v == &cur_next) {
                                            let next_idx = (pos + 1) % vals_next.len();
                                            if let Some(new_val) = vals_next.get(next_idx) {
                                                this.selected_handler_options.insert(opt_key_next.clone(), new_val.clone());
                                            }
                                        } else if let Some(first) = vals_next.first() {
                                            this.selected_handler_options.insert(opt_key_next.clone(), first.clone());
                                        }
                                        cx.notify();
                                    }))
                                    .into_any_element()
                                }
                            }).collect()
                        } else {
                            vec![
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .p_4()
                                    .rounded_xl()
                                    .bg(theme::bg_panel())
                                    .border_1()
                                    .border_color(theme::border_subtle())
                                    .child(Icon::new(IconName::SlidersHorizontal).size_5().text_color(theme::accent()))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_color(theme::text_primary())
                                                    .child("Standard Handler Settings"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(theme::text_muted())
                                                    .child("This game handler runs with optimized standard defaults without extra custom options."),
                                            ),
                                    )
                                    .into_any_element(),
                            ]
                        })
                )
                // Language Selector Row (Standard 56px row)
                .child(
                    CouchOptionRow::new(
                        "opt-game-language",
                        "Game Language",
                        "Interface and subtitle language configured for all instances.",
                        "English",
                    )
                )
                // Navigation buttons
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .pt_4()
                        .child(
                            CouchButton::secondary("btn-back-to-layout", "Back to Screen layout")
                                .icon(IconName::ArrowLeft)
                                .size(CouchButtonSize::Lg)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.couch_setup_state.prev_step();
                                    cx.notify();
                                })),
                        )
                        .child({
                            let opts_count = if has_options { handler.as_ref().map(|h| h.options.len()).unwrap_or(0) } else { 0 };
                            let is_next_focused = state.focused_slot >= opts_count;
                            CouchButton::primary("btn-next-to-launch", "Next: Launch")
                                .icon(IconName::ArrowRight)
                                .size(CouchButtonSize::Lg)
                                .focused(is_next_focused)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.couch_setup_state.next_step();
                                    this.couch_setup_state.focused_slot = 0;
                                    cx.notify();
                                }))
                        }),
                )
                .into_any_element(),
        )
        // Right Column: Handler Setup Instructions / Good to know
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(440.0))
                .flex_shrink_0()
                .min_w_0()
                .gap_5()
                .child(render_couch_handler_notes_panel(win, cx))
                .into_any_element(),
        )
        .into_any_element()
}

/// Step 4: "Launch"
fn render_step_4_launch(
    win: &MainWindow,
    state: &CouchSetupState,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let can_launch = !state.slots.is_empty();
    let player_count = state.slots.len();

    div()
        .id("couch-step-4-launch")
        .flex()
        .flex_row()
        .items_start()
        .gap_6()
        .w_full()
        // Left Column: Artwork, Summary, Profiles
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .gap_6()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .text_xl()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child("Launch splitscreen"),
                        )
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::text_muted())
                                .child("Review your configuration and start the multi-instance splitscreen session."),
                        ),
                )
                .child(render_couch_game_details_card(win, cx))
                // Session summary card
                .child(
                    div()
                        .id("session-summary-card")
                        .flex()
                        .flex_col()
                        .gap_3()
                        .p_4()
                        .rounded_xl()
                        .bg(theme::bg_panel())
                        .border_1()
                        .border_color(theme::border_color())
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child("Configuration Summary:"),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_2()
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child(Icon::new(IconName::Users).size_4().text_color(theme::accent()))
                                        .child(format!("{} Player{} Assigned", player_count, if player_count == 1 { "" } else { "s" })),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child(Icon::new(IconName::LayoutGrid).size_4().text_color(theme::accent()))
                                        .child(format!("Screen Partition: {}", state.selected_layout)),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child(Icon::new(IconName::Monitor).size_4().text_color(theme::accent()))
                                        .child("Active Display: Display 1 (1920 × 1080)"),
                                ),
                        ),
                )
                .child(render_couch_profiles_section(win, cx))
                .into_any_element(),
        )
        // Right Column: Launch Action Card (~40% / 440px)
        .child(
            div()
                .flex()
                .flex_col()
                .w(px(440.0))
                .flex_shrink_0()
                .min_w_0()
                .gap_5()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_4()
                        .p_6()
                        .rounded_xl()
                        .bg(theme::bg_panel())
                        .border_2()
                        .border_color(if can_launch { theme::accent() } else { theme::border_subtle() })
                        .shadow_lg()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap_1p5()
                                .child(
                                    div()
                                        .text_lg()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child("Ready to play?"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme::text_muted())
                                        .child("All instances will be staged into isolated folders, Goldberg SteamIDs assigned, and windows tiled seamlessly."),
                                ),
                        )
                        .child({
                            let mut btn = CouchButton::primary("btn-start-splitscreen-session", "Start splitscreen session")
                                .icon(IconName::Play)
                                .size(CouchButtonSize::Lg)
                                .disabled(!can_launch);
                            if can_launch {
                                btn = btn.on_click(cx.listener(|this, _, _, cx| {
                                    this.launch_couch_session(cx);
                                }));
                            }
                            btn
                        })
                        .child(
                            CouchButton::secondary("btn-back-to-options", "Back to Game options")
                                .icon(IconName::ArrowLeft)
                                .size(CouchButtonSize::Lg)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.couch_setup_state.prev_step();
                                    cx.notify();
                                })),
                        ),
                )
                .into_any_element(),
        )
        .into_any_element()
}

fn render_km_assignment_bar(
    _win: &MainWindow,
    state: &CouchSetupState,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let _kb_count = state.slots.iter().filter(|s| s.has_raw_keyboard()).count();
    let _mouse_count = state.slots.iter().filter(|s| s.has_raw_mouse()).count();

    div()
        .id("couch-km-assignment-bar")
        .flex()
        .flex_wrap()
        .items_center()
        .gap_3()
        .p_3()
        .rounded_lg()
        .bg(theme::bg_panel())
        .border_1()
        .border_color(theme::border_subtle())
        .child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .child(
                    Icon::new(IconName::Keyboard)
                        .size_4()
                        .text_color(theme::text_muted()),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_muted())
                        .child("K&M Quick Assignment:"),
                ),
        )
        // 1. Paired Keyboard & Mouse button
        .child(
            CouchButton::secondary("btn-add-paired-kbm", "Add Paired KBM")
                .size(CouchButtonSize::Lg)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.couch_setup_state.add_paired_kbm();
                    cx.notify();
                })),
        )
        // 2. Individual ProtoInput Keyboard button
        .child(
            CouchButton::secondary("btn-add-raw-keyboard", "Add Raw Keyboard")
                .size(CouchButtonSize::Lg)
                .on_click(cx.listener(move |this, _, _, cx| {
                    let count = this
                        .couch_setup_state
                        .slots
                        .iter()
                        .filter(|s| s.has_raw_keyboard())
                        .count();
                    this.couch_setup_state.add_raw_keyboard(count);
                    cx.notify();
                })),
        )
        // 3. Individual ProtoInput Mouse button
        .child(
            CouchButton::secondary("btn-add-raw-mouse", "Add Raw Mouse")
                .size(CouchButtonSize::Lg)
                .on_click(cx.listener(move |this, _, _, cx| {
                    let count = this
                        .couch_setup_state
                        .slots
                        .iter()
                        .filter(|s| s.has_raw_mouse())
                        .count();
                    this.couch_setup_state.add_raw_mouse(count);
                    cx.notify();
                })),
        )
        .into_any_element()
}

fn render_couch_game_details_card(win: &MainWindow, _cx: &mut Context<MainWindow>) -> AnyElement {
    let game_name = win.selected_game.as_deref().unwrap_or("No Game Selected");
    let handler = win.handlers.iter().find(|h| {
        h.game_name.eq_ignore_ascii_case(game_name) || h.guid.eq_ignore_ascii_case(game_name)
    });

    let cover_url = handler.and_then(|h| h.cover_image_url.as_deref());
    let max_players = handler.map(|h| h.max_players).unwrap_or(4);
    let exe_name = handler
        .map(|h| h.executable_name.as_str())
        .unwrap_or("game.exe");
    let is_installed = handler.map(|h| h.is_installed).unwrap_or(false);

    let cover_elem: AnyElement = if let Some(url) = cover_url {
        img(crate::make_image_source(url))
            .size_full()
            .object_fit(gpui::ObjectFit::Cover)
            .rounded_lg()
            .into_any_element()
    } else {
        div()
            .flex()
            .items_center()
            .justify_center()
            .size_full()
            .bg(theme::bg_subtle())
            .rounded_lg()
            .child(
                Icon::new(IconName::Gamepad2)
                    .size_10()
                    .text_color(theme::text_muted()),
            )
            .into_any_element()
    };

    div()
        .id("couch-details-card")
        .flex()
        .flex_col()
        .gap_3()
        .p_4()
        .rounded_xl()
        .bg(theme::bg_panel())
        .border_1()
        .border_color(theme::border_color())
        .child(
            div()
                .w_full()
                .h(px(180.0))
                .rounded_lg()
                .overflow_hidden()
                .child(cover_elem),
        )
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
                        .child(game_name.to_string()),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::text_muted())
                        .child(format!("Executable: {}", exe_name)),
                ),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_2p5()
                        .py_1()
                        .rounded_md()
                        .bg(theme::bg_subtle())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::accent())
                        .child(
                            Icon::new(IconName::Users)
                                .size_3p5()
                                .text_color(theme::accent()),
                        )
                        .child(format!("Max: {} Players", max_players)),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1p5()
                        .px_2p5()
                        .py_1()
                        .rounded_md()
                        .bg(if is_installed {
                            theme::success_subtle()
                        } else {
                            theme::warning_subtle()
                        })
                        .border_1()
                        .border_color(if is_installed {
                            theme::success().opacity(0.5)
                        } else {
                            theme::warning().opacity(0.5)
                        })
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(if is_installed {
                            theme::success()
                        } else {
                            theme::warning()
                        })
                        .child(
                            Icon::new(if is_installed {
                                IconName::Check
                            } else {
                                IconName::TriangleAlert
                            })
                            .size_3p5(),
                        )
                        .child(if is_installed {
                            "Installed"
                        } else {
                            "Missing Exe"
                        }),
                ),
        )
        .into_any_element()
}

fn render_couch_handler_notes_panel(win: &MainWindow, _cx: &mut Context<MainWindow>) -> AnyElement {
    let game_name = win.selected_game.as_deref().unwrap_or("No Game Selected");
    let handler = win.handlers.iter().find(|h| {
        h.game_name.eq_ignore_ascii_case(game_name) || h.guid.eq_ignore_ascii_case(game_name)
    });
    let desc = handler
        .map(|h| h.description.as_str())
        .unwrap_or("No handler instructions provided.");

    div()
        .id("couch-handler-notes-panel")
        .flex()
        .flex_col()
        .gap_2p5()
        .p_4()
        .rounded_xl()
        .bg(theme::bg_panel())
        .border_1()
        .border_color(theme::border_color())
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(
                    Icon::new(IconName::BookOpen)
                        .size_4()
                        .text_color(theme::accent()),
                )
                .child(
                    div()
                        .text_sm()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .child("Handler Setup Instructions:"),
                ),
        )
        .child(
            div()
                .id("couch-handler-notes-scroll")
                .w_full()
                .max_h(px(240.0))
                .overflow_y_scroll()
                .p_3()
                .rounded_lg()
                .bg(theme::bg_subtle())
                .border_1()
                .border_color(theme::border_subtle())
                .child(crate::markdown::render_markdown_with_id(
                    "couch-notes-md",
                    desc,
                )),
        )
        .into_any_element()
}

fn render_couch_profiles_section(win: &MainWindow, cx: &mut Context<MainWindow>) -> AnyElement {
    let profiles = &win.saved_profiles;
    let active_id = win.active_profile_id;
    let game_name = win.selected_game.clone().unwrap_or_default();
    let guid = win
        .handlers
        .iter()
        .find(|h| h.game_name == game_name)
        .map(|h| h.guid.clone())
        .unwrap_or_else(|| game_name.clone());

    div()
        .id("couch-profiles-section")
        .flex()
        .flex_col()
        .gap_3()
        .p_4()
        .rounded_xl()
        .bg(theme::bg_panel())
        .border_1()
        .border_color(theme::border_color())
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(
                            Icon::new(IconName::Save)
                                .size_4()
                                .text_color(theme::accent()),
                        )
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::text_primary())
                                .child("Game Profiles"),
                        ),
                )
                .child(
                    CouchButton::primary("btn-couch-save-new-profile", "Save New Profile")
                        .icon(IconName::Plus)
                        .size(CouchButtonSize::Lg)
                        .on_click(cx.listener({
                            let guid_copy = guid.clone();
                            move |this, _, _, cx| {
                                let mut prof = nucleus_profile::GameProfile::default();
                                prof.player_count = this.couch_setup_state.slots.len();
                                if let Some(ref saver) = this.callbacks.on_save_numbered_profile {
                                    if let Some(new_id) = saver(&guid_copy, None, &prof) {
                                        this.active_profile_id = Some(new_id);
                                        this.refresh_saved_profiles();
                                        this.status_message =
                                            format!("Saved new profile (Profile {})!", new_id);
                                    }
                                }
                                cx.notify();
                            }
                        })),
                ),
        )
        .child(
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .children(if profiles.is_empty() {
                    vec![
                        div()
                            .text_xs()
                            .text_color(theme::text_muted())
                            .child(
                                "No saved profiles yet. Click 'Save Setup' to record slot mapping.",
                            )
                            .into_any_element(),
                    ]
                } else {
                    profiles
                        .iter()
                        .map(|&(prof_id, ref _title)| {
                            let is_active = active_id == Some(prof_id);
                            let guid_c = guid.clone();
                            div()
                                .id(SharedString::from(format!(
                                    "couch-profile-chip-{}",
                                    prof_id
                                )))
                                .cursor_pointer()
                                .flex()
                                .items_center()
                                .gap_2()
                                .px_3()
                                .py_1p5()
                                .rounded_md()
                                .bg(if is_active {
                                    theme::accent()
                                } else {
                                    theme::bg_subtle()
                                })
                                .text_color(if is_active {
                                    theme::text_on_accent()
                                } else {
                                    theme::text_primary()
                                })
                                .border_1()
                                .border_color(if is_active {
                                    theme::accent()
                                } else {
                                    theme::border_subtle()
                                })
                                .hover(|s| s.bg(theme::bg_hover()))
                                .on_click(cx.listener({
                                    let g = guid_c.clone();
                                    move |this, _, _, cx| {
                                        if let Some(ref loader) =
                                            this.callbacks.on_load_numbered_profile
                                        {
                                            if let Some(_prof) = loader(&g, prof_id) {
                                                this.active_profile_id = Some(prof_id);
                                                this.status_message =
                                                    format!("Loaded profile #{}!", prof_id);
                                            }
                                        }
                                        cx.notify();
                                    }
                                }))
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .child(format!("Profile {}", prof_id)),
                                )
                                .into_any_element()
                        })
                        .collect()
                }),
        )
        .into_any_element()
}

fn render_bumper_badge(badge: &'static str) -> AnyElement {
    div()
        .px_2()
        .py_1()
        .rounded_md()
        .bg(theme::bg_subtle())
        .border_1()
        .border_color(theme::border_subtle())
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(theme::text_muted())
        .child(badge)
        .into_any_element()
}

fn layout_icon_for_name(layout: &str) -> IconName {
    if layout.contains("Multi-Monitor") {
        IconName::Monitor
    } else if layout.contains("2x1") || layout.contains("Side-by-Side") {
        IconName::Columns2
    } else if layout.contains("1x2") || layout.contains("Stacked") {
        IconName::Rows2
    } else if layout.contains("Fullscreen") {
        IconName::Maximize
    } else if layout.contains("3x2") {
        IconName::Grid3x3
    } else if layout.contains("2x2") || layout.contains("Grid") {
        IconName::Grid2x2
    } else {
        IconName::LayoutGrid
    }
}

fn render_couch_setup_header(
    win: &MainWindow,
    state: &CouchSetupState,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let game_name = win.selected_game.as_deref().unwrap_or("No Game Selected");
    let current_step = state.current_step;

    div()
        .id("couch-setup-header")
        .flex()
        .flex_col()
        .gap_3()
        .w_full()
        // Top row: Game Title + Step Subtitle
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2p5()
                        .child(
                            Icon::new(IconName::Gamepad2)
                                .size_6()
                                .text_color(theme::accent()),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_2xl()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child(format!("Splitscreen Setup — {}", game_name)),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme::text_muted())
                                        .child(format!(
                                            "Step {} of 4: {}",
                                            current_step.index() + 1,
                                            current_step.title()
                                        )),
                                ),
                        ),
                )
                // Right: Bumper hints
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(render_bumper_badge("LB / Q"))
                        .child(
                            div()
                                .text_xs()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(theme::text_muted())
                                .child("Switch steps"),
                        )
                        .child(render_bumper_badge("RB / E")),
                ),
        )
        // 4-Step Stepper Bar
        .child(
            div()
                .id("couch-stepper-bar")
                .flex()
                .items_center()
                .gap_3()
                .p_1p5()
                .rounded_xl()
                .bg(theme::bg_panel())
                .border_1()
                .border_color(theme::border_subtle())
                .children(CouchSetupStep::ALL.iter().enumerate().map(|(idx, &step)| {
                    let is_active = step == current_step;
                    let is_done =
                        state.is_step_completed(step) && step.index() < current_step.index();

                    let mut pill = div()
                        .id(SharedString::from(format!("stepper-tab-{}", idx)))
                        .cursor_pointer()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .gap_2()
                        .px_4()
                        .py_2()
                        .rounded_lg()
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.couch_setup_state.set_step(step);
                            cx.notify();
                        }));

                    if is_active {
                        pill = pill
                            .bg(theme::accent().opacity(0.18))
                            .border_2()
                            .border_color(theme::accent())
                            .shadow_md()
                            .child(
                                Icon::new(IconName::Sparkles)
                                    .size_4()
                                    .text_color(theme::accent()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme::accent())
                                    .child(step.label()),
                            );
                    } else if is_done {
                        pill = pill
                            .bg(theme::bg_subtle())
                            .border_1()
                            .border_color(theme::border_subtle())
                            .hover(|s| s.bg(theme::bg_hover()))
                            .child(
                                Icon::new(IconName::Check)
                                    .size_3p5()
                                    .text_color(theme::success()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme::text_primary())
                                    .child(step.short_name()),
                            );
                    } else {
                        pill = pill
                            .bg(theme::bg_panel())
                            .border_1()
                            .border_color(theme::border_subtle())
                            .hover(|s| s.bg(theme::bg_hover()))
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(theme::text_muted())
                                    .child(step.label()),
                            );
                    }

                    pill.into_any_element()
                })),
        )
        .into_any_element()
}

#[allow(dead_code)]
fn render_layout_selector_strip(
    state: &CouchSetupState,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    div()
        .id("layout-selector-bar")
        .flex()
        .items_center()
        .gap_4()
        .px_4()
        .py_2()
        .rounded_lg()
        .bg(theme::bg_panel())
        .border_1()
        .border_color(theme::border_color())
        .child(
            div()
                .text_sm()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(theme::text_muted())
                .child("Screen Split:"),
        )
        .child(
            CouchButton::ghost("btn-layout-prev", "◀")
                .size(CouchButtonSize::Lg)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.couch_setup_state.cycle_layout(false);
                    cx.notify();
                })),
        )
        .child(
            div()
                .id("layout-current-name")
                .flex()
                .items_center()
                .px_4()
                .py_1p5()
                .rounded_md()
                .bg(theme::accent_subtle())
                .border_1()
                .border_color(theme::accent())
                .text_sm()
                .font_weight(FontWeight::BOLD)
                .text_color(theme::accent())
                .child(state.selected_layout.clone()),
        )
        .child(
            CouchButton::ghost("btn-layout-next", "▶")
                .size(CouchButtonSize::Lg)
                .on_click(cx.listener(|this, _, _, cx| {
                    this.couch_setup_state.cycle_layout(true);
                    cx.notify();
                })),
        )
        .into_any_element()
}

/// Renders a stylized 16:9 virtual monitor preview displaying where each player
/// viewport will be positioned on the primary display.
pub fn render_virtual_monitor_partition_box(
    _win: &MainWindow,
    state: &CouchSetupState,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    div()
        .id("virtual-monitor-partition-preview")
        .flex()
        .flex_col()
        .items_center()
        .w_full()
        .gap_1()
        .child(
            // Header tag above monitor
            div()
                .flex()
                .items_center()
                .justify_between()
                .w(px(540.0))
                .px_1()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(theme::text_primary())
                        .child(
                            Icon::new(IconName::Monitor)
                                .size_4()
                                .text_color(theme::accent()),
                        )
                        .child("Primary Display Layout Preview"),
                )
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme::text_muted())
                        .child(format!("Layout: {}", state.selected_layout)),
                ),
        )
        // Outer stylized monitor bezel / chassis
        .child(
            div()
                .id("virtual-monitor-chassis")
                .flex()
                .flex_col()
                .w(px(540.0))
                .h(px(220.0))
                .p_2()
                .rounded_xl()
                .bg(theme::bg_panel())
                .border_2()
                .border_color(theme::border_color())
                .shadow_lg()
                // Inner 16:9 screen glass viewport
                .child(
                    div()
                        .id("virtual-monitor-screen")
                        .flex_1()
                        .w_full()
                        .min_w_0()
                        .rounded_md()
                        .overflow_hidden()
                        .p_2()
                        .bg(theme::bg_dark())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .child(render_partition_content(state, cx)),
                ),
        )
        // Monitor stand neck & base
        .child(
            div()
                .id("virtual-monitor-stand")
                .flex()
                .flex_col()
                .items_center()
                .child(
                    div()
                        .w(px(36.0))
                        .h(px(8.0))
                        .bg(theme::bg_subtle())
                        .border_l_1()
                        .border_r_1()
                        .border_color(theme::border_subtle()),
                )
                .child(
                    div()
                        .w(px(110.0))
                        .h(px(4.0))
                        .rounded_t_sm()
                        .bg(theme::border_color()),
                ),
        )
        .into_any_element()
}

/// Returns the exact pixel dimensions (width, height) for a given partition cell slot,
/// assuming a 1920x1080 display baseline.
pub fn partition_cell_dimensions(
    layout: &str,
    slot_idx: usize,
    total_w: u32,
    total_h: u32,
) -> (u32, u32) {
    if layout.contains("1x2") || layout.contains("Stacked") {
        (total_w, total_h / 2)
    } else if layout.contains("2x1") || layout.contains("Side-by-Side") {
        (total_w / 2, total_h)
    } else if layout.contains("Top 1 / Bottom 2") {
        if slot_idx == 0 {
            (total_w, total_h / 2)
        } else {
            (total_w / 2, total_h / 2)
        }
    } else if layout.contains("Top 2 / Bottom 1") {
        if slot_idx < 2 {
            (total_w / 2, total_h / 2)
        } else {
            (total_w, total_h / 2)
        }
    } else if layout.contains("3x2") {
        (total_w / 3, total_h / 2)
    } else if layout.contains("4x2") {
        (total_w / 4, total_h / 2)
    } else if layout.contains("4x4") {
        (total_w / 4, total_h / 4)
    } else if layout.contains("2x2") || layout.contains("Grid") {
        (total_w / 2, total_h / 2)
    } else {
        (total_w, total_h)
    }
}

fn render_partition_cell(
    state: &CouchSetupState,
    slot_idx: usize,
    cx: &mut Context<MainWindow>,
) -> Stateful<Div> {
    if let Some(slot) = state.slots.get(slot_idx) {
        let is_focused = slot_idx == state.focused_slot;
        let is_grabbed = slot.is_grabbed;
        let p_idx = slot.player_index;
        let p_color = theme::player_color(p_idx);
        let p_color_subtle = theme::player_color_subtle(p_idx);
        let p_color_text = theme::player_color_text(p_idx);
        let (cell_w, cell_h) =
            partition_cell_dimensions(&state.selected_layout, slot_idx, 1920, 1080);

        let border_color = if is_grabbed || is_focused {
            theme::accent()
        } else {
            p_color
        };

        let is_keyboard = is_keyboard_device(&slot.device_name, &slot.device_guid);
        let icon_name = if is_keyboard {
            IconName::Keyboard
        } else {
            IconName::Gamepad2
        };

        let mut cell = div()
            .id(SharedString::from(format!("partition-slot-{}", slot_idx)))
            .cursor_pointer()
            .relative()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .min_w_0()
            .overflow_hidden()
            .p_2()
            .rounded_md()
            .border_2()
            .border_color(border_color)
            .bg(p_color_subtle)
            .gap_1()
            .on_click(cx.listener(move |this, _, _, cx| {
                if this.couch_setup_state.grabbed_slot.is_some() {
                    this.couch_setup_state.drop_swap_target(slot_idx);
                } else {
                    this.couch_setup_state.grab_slot(slot_idx);
                }
                cx.notify();
            }));

        if is_grabbed || is_focused {
            cell = cell.shadow_md();
        }

        cell.child(
            div()
                .flex()
                .items_center()
                .gap_1p5()
                .px_2()
                .py_0p5()
                .rounded_sm()
                .bg(theme::bg_panel())
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(if is_grabbed {
                            theme::accent()
                        } else {
                            p_color_text
                        })
                        .child(format!("P{} · {} × {}", p_idx + 1, cell_w, cell_h)),
                )
                .children(if is_grabbed {
                    Some(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(theme::accent())
                            .child("GRABBED"),
                    )
                } else {
                    None
                }),
        )
        .child(
            div()
                .flex()
                .items_center()
                .justify_center()
                .gap_1p5()
                .w_full()
                .min_w_0()
                .child(Icon::new(icon_name).size_4().text_color(p_color))
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(theme::text_primary())
                        .truncate()
                        .child(slot.device_name.clone()),
                ),
        )
    } else {
        div()
            .id(SharedString::from(format!(
                "partition-unassigned-{}",
                slot_idx
            )))
            .relative()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .min_w_0()
            .overflow_hidden()
            .p_2()
            .rounded_md()
            .border_2()
            .border_dashed()
            .border_color(theme::border_subtle())
            .bg(theme::bg_subtle())
            .gap_1()
            .child(
                Icon::new(IconName::Gamepad2)
                    .size_4()
                    .text_color(theme::text_subtle()),
            )
            .child(
                div()
                    .text_xs()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme::text_muted())
                    .truncate()
                    .child(format!("Unassigned (Player {})", slot_idx + 1)),
            )
    }
}

fn render_partition_content(state: &CouchSetupState, cx: &mut Context<MainWindow>) -> AnyElement {
    let kind = PartitionLayoutKind::from_layout_name(&state.selected_layout);
    match kind {
        PartitionLayoutKind::Fullscreen => div()
            .flex()
            .size_full()
            .child(
                render_partition_cell(state, 0, cx)
                    .flex_1()
                    .h_full()
                    .w_full()
                    .min_w_0(),
            )
            .into_any_element(),
        PartitionLayoutKind::SideBySide => div()
            .flex()
            .flex_row()
            .size_full()
            .gap_2()
            .child(
                render_partition_cell(state, 0, cx)
                    .flex_1()
                    .h_full()
                    .min_w_0(),
            )
            .child(
                render_partition_cell(state, 1, cx)
                    .flex_1()
                    .h_full()
                    .min_w_0(),
            )
            .into_any_element(),
        PartitionLayoutKind::Stacked => div()
            .flex()
            .flex_col()
            .size_full()
            .gap_2()
            .child(
                render_partition_cell(state, 0, cx)
                    .flex_1()
                    .w_full()
                    .min_w_0(),
            )
            .child(
                render_partition_cell(state, 1, cx)
                    .flex_1()
                    .w_full()
                    .min_w_0(),
            )
            .into_any_element(),
        PartitionLayoutKind::Grid => div()
            .flex()
            .flex_col()
            .size_full()
            .gap_2()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .w_full()
                    .min_w_0()
                    .gap_2()
                    .child(
                        render_partition_cell(state, 0, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    )
                    .child(
                        render_partition_cell(state, 1, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .w_full()
                    .min_w_0()
                    .gap_2()
                    .child(
                        render_partition_cell(state, 2, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    )
                    .child(
                        render_partition_cell(state, 3, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    ),
            )
            .into_any_element(),
        PartitionLayoutKind::TopOneBottomTwo => div()
            .flex()
            .flex_col()
            .size_full()
            .gap_2()
            .child(
                render_partition_cell(state, 0, cx)
                    .flex_1()
                    .w_full()
                    .min_w_0(),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .w_full()
                    .min_w_0()
                    .gap_2()
                    .child(
                        render_partition_cell(state, 1, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    )
                    .child(
                        render_partition_cell(state, 2, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    ),
            )
            .into_any_element(),
        PartitionLayoutKind::TopTwoBottomOne => div()
            .flex()
            .flex_col()
            .size_full()
            .gap_2()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .w_full()
                    .min_w_0()
                    .gap_2()
                    .child(
                        render_partition_cell(state, 0, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    )
                    .child(
                        render_partition_cell(state, 1, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    ),
            )
            .child(
                render_partition_cell(state, 2, cx)
                    .flex_1()
                    .w_full()
                    .min_w_0(),
            )
            .into_any_element(),
        PartitionLayoutKind::Grid6 => div()
            .flex()
            .flex_col()
            .size_full()
            .gap_2()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .w_full()
                    .min_w_0()
                    .gap_2()
                    .child(
                        render_partition_cell(state, 0, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    )
                    .child(
                        render_partition_cell(state, 1, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    )
                    .child(
                        render_partition_cell(state, 2, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .w_full()
                    .min_w_0()
                    .gap_2()
                    .child(
                        render_partition_cell(state, 3, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    )
                    .child(
                        render_partition_cell(state, 4, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    )
                    .child(
                        render_partition_cell(state, 5, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    ),
            )
            .into_any_element(),
        PartitionLayoutKind::Grid8 => div()
            .flex()
            .flex_col()
            .size_full()
            .gap_2()
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .w_full()
                    .min_w_0()
                    .gap_2()
                    .child(
                        render_partition_cell(state, 0, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    )
                    .child(
                        render_partition_cell(state, 1, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    )
                    .child(
                        render_partition_cell(state, 2, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    )
                    .child(
                        render_partition_cell(state, 3, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .w_full()
                    .min_w_0()
                    .gap_2()
                    .child(
                        render_partition_cell(state, 4, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    )
                    .child(
                        render_partition_cell(state, 5, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    )
                    .child(
                        render_partition_cell(state, 6, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    )
                    .child(
                        render_partition_cell(state, 7, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    ),
            )
            .into_any_element(),
        PartitionLayoutKind::Grid16 => {
            let mut col = div().flex().flex_col().size_full().gap_1();
            for r in 0..4 {
                let mut row = div().flex().flex_row().flex_1().w_full().min_w_0().gap_1();
                for c in 0..4 {
                    let idx = r * 4 + c;
                    row = row.child(
                        render_partition_cell(state, idx, cx)
                            .flex_1()
                            .h_full()
                            .min_w_0(),
                    );
                }
                col = col.child(row);
            }
            col.into_any_element()
        }
        PartitionLayoutKind::MultiMonitor => div()
            .flex()
            .flex_row()
            .size_full()
            .gap_2()
            .child(
                render_partition_cell(state, 0, cx)
                    .flex_1()
                    .h_full()
                    .min_w_0(),
            )
            .child(
                render_partition_cell(state, 1, cx)
                    .flex_1()
                    .h_full()
                    .min_w_0(),
            )
            .into_any_element(),
    }
}

fn render_slots_grid(
    win: &MainWindow,
    state: &CouchSetupState,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    if state.slots.is_empty() {
        return div()
            .id("slots-grid-empty")
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .h(px(240.0))
            .w_full()
            .rounded_xl()
            .bg(theme::bg_panel())
            .border_1()
            .border_dashed()
            .border_color(theme::border_color())
            .gap_3()
            .child(Icon::new(IconName::Gamepad2).size_12().text_color(theme::text_muted()))
            .child(
                div()
                    .text_base()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme::text_muted())
                    .child("No controllers assigned. Plug in a controller or reclaim one from the tray below."),
            )
            .into_any_element();
    }

    div()
        .id("slots-grid")
        .flex()
        .flex_wrap()
        .gap_6()
        .w_full()
        .children(
            state
                .slots
                .iter()
                .enumerate()
                .map(|(i, slot)| render_slot_card(win, state, slot, i, cx)),
        )
        .into_any_element()
}

fn render_slot_card(
    _win: &MainWindow,
    state: &CouchSetupState,
    slot: &CouchPlayerSlot,
    idx: usize,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let is_focused = idx == state.focused_slot;
    let is_grabbed = slot.is_grabbed;

    let status_badge = match slot.status {
        DisconnectStatus::ReplugPending { remaining_ms, .. } => {
            CouchTag::new(format!("Replug ({:.1}s)", remaining_ms as f32 / 1000.0))
                .tone(TagTone::Warning)
                .into_any_element()
        }
        DisconnectStatus::DisconnectedSilent => CouchTag::new("Disconnected")
            .tone(TagTone::Neutral)
            .into_any_element(),
        DisconnectStatus::Connected => {
            if slot.is_connected {
                CouchTag::new("Connected")
                    .tone(TagTone::Success)
                    .with_dot()
                    .into_any_element()
            } else {
                CouchTag::new("Disconnected")
                    .tone(TagTone::Neutral)
                    .into_any_element()
            }
        }
    };

    let card_id_copy = idx;
    let guid_copy = slot.device_guid.clone();
    let is_kbm = slot.is_paired_kbm() || is_keyboard_device(&slot.device_name, &slot.device_guid);

    let mut sc = CouchSlotCard::new(format!("slot-card-{}", idx), slot.player_index)
        .device_label(Some(slot.device_name.clone()))
        .device_guid(Some(slot.device_guid.clone()))
        .kbm(is_kbm)
        .merged_kbm(slot.is_complete_raw_kbm())
        .raw_device(slot.has_raw_mouse() || slot.has_raw_keyboard())
        .focused(is_focused)
        .grabbed(is_grabbed)
        .status_badge(status_badge)
        .on_click(cx.listener(move |this, _, _, cx| {
            if this.couch_setup_state.grabbed_slot.is_some() {
                this.couch_setup_state.drop_swap_target(card_id_copy);
            } else {
                this.couch_setup_state.grab_slot(card_id_copy);
            }
            cx.notify();
        }))
        .on_remove(cx.listener(move |this, _, _, cx| {
            this.couch_setup_state.remove_slot(card_id_copy);
            cx.notify();
        }))
        .on_ignore(cx.listener(move |this, _, _, cx| {
            this.couch_setup_state
                .ignore_device(&guid_copy, &mut this.app_settings.ignored_device_guids);
            let _ = this.app_settings.save();
            cx.notify();
        }));

    if slot.is_complete_raw_kbm() {
        sc = sc.on_split(cx.listener(move |this, _, _, cx| {
            this.couch_setup_state.split_raw_kbm(card_id_copy);
            cx.notify();
        }));
    }

    sc.into_any_element()
}

fn render_unassigned_tray(
    state: &CouchSetupState,
    cx: &mut Context<MainWindow>,
) -> Option<AnyElement> {
    if state.unassigned_tray.is_empty() {
        return None;
    }

    Some(
        div()
            .id("unassigned-tray")
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .rounded_xl()
            .bg(theme::bg_panel())
            .border_1()
            .border_color(theme::border_color())
            .child(
                div()
                    .text_sm()
                    .font_weight(FontWeight::BOLD)
                    .text_color(theme::text_primary())
                    .child("Unassigned Controllers Tray:"),
            )
            .child(div().flex().flex_wrap().gap_3().children(
                state.unassigned_tray.iter().enumerate().map(|(i, dev)| {
                    let tray_idx = i;
                    let is_kbd = is_keyboard_device(&dev.name, &dev.guid);
                    let tray_icon = if is_kbd {
                        IconName::Keyboard
                    } else {
                        IconName::Gamepad2
                    };

                    div()
                        .id(SharedString::from(format!("tray-item-{}", i)))
                        .flex()
                        .items_center()
                        .gap_3()
                        .px_3()
                        .py_2()
                        .rounded_lg()
                        .bg(theme::bg_subtle())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .child(
                            Icon::new(tray_icon)
                                .size_5()
                                .text_color(theme::text_muted()),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child(dev.name.clone()),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(theme::text_muted())
                                        .child(dev.guid.clone()),
                                ),
                        )
                        .child(
                            div()
                                .id(SharedString::from(format!("btn-reclaim-{}", i)))
                                .cursor_pointer()
                                .px_2p5()
                                .py_1()
                                .rounded_md()
                                .bg(theme::accent_subtle())
                                .border_1()
                                .border_color(theme::accent())
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::accent())
                                .child("+ Re-assign")
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.couch_setup_state.reclaim_unassigned(tray_idx);
                                    cx.notify();
                                })),
                        )
                }),
            ))
            .into_any_element(),
    )
}

/// Renders the Ignored Controllers (Blacklist) Tray in Big Picture Couch Setup,
/// displaying ignored gamepads with a button to restore/re-enable each controller.
pub fn render_ignored_controllers_tray(
    win: &MainWindow,
    _state: &CouchSetupState,
    cx: &mut Context<MainWindow>,
) -> Option<AnyElement> {
    let valid_ignored_guids: Vec<String> = win
        .app_settings
        .ignored_device_guids
        .iter()
        .filter(|g| {
            let lower = g.to_lowercase();
            !lower.starts_with("mock")
                && !lower.starts_with("virtual-pad")
                && !lower.starts_with("dev-guid-")
                && lower != "xinput-0"
                && lower != "xinput-1"
        })
        .cloned()
        .collect();

    if valid_ignored_guids.is_empty() {
        return None;
    }

    Some(
        div()
            .id("ignored-controllers-tray")
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .rounded_xl()
            .bg(theme::bg_panel())
            .border_1()
            .border_color(theme::border_color())
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1p5()
                            .child(
                                Icon::new(IconName::Ban)
                                    .size_4()
                                    .text_color(theme::danger()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(theme::text_primary())
                                    .child("Ignored Controllers (Blacklist):"),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(theme::text_muted())
                            .child("[Click Re-Enable to restore controller to active setup]"),
                    ),
            )
            .child(div().flex().flex_wrap().gap_3().children(
                valid_ignored_guids.iter().enumerate().map(|(i, guid)| {
                    let guid_str = guid.clone();
                    let dev_name = if guid == "keyboard-mouse-0" {
                        "Keyboard & Mouse".to_string()
                    } else {
                        win.detected_devices_list
                            .iter()
                            .find(|d| &d.guid == guid)
                            .map(|d| d.name.clone())
                            .unwrap_or_else(|| "Hardware Controller".to_string())
                    };
                    let is_kbd = is_keyboard_device(&dev_name, guid);
                    let ignored_icon = if is_kbd {
                        IconName::Keyboard
                    } else {
                        IconName::Gamepad2
                    };

                    div()
                        .id(SharedString::from(format!("ignored-chip-{}", i)))
                        .flex()
                        .items_center()
                        .gap_3()
                        .px_3()
                        .py_2()
                        .rounded_lg()
                        .bg(theme::bg_subtle())
                        .border_1()
                        .border_color(theme::border_subtle())
                        .child(
                            Icon::new(ignored_icon)
                                .size_5()
                                .text_color(theme::text_muted()),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .min_w_0()
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(theme::text_primary())
                                        .child(dev_name),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .font_weight(FontWeight::MEDIUM)
                                        .text_color(theme::text_muted())
                                        .child(guid.clone()),
                                ),
                        )
                        .child(
                            div()
                                .id(SharedString::from(format!("btn-restore-ignored-{}", i)))
                                .cursor_pointer()
                                .flex_shrink_0()
                                .px_2p5()
                                .py_1()
                                .rounded_md()
                                .bg(theme::success_subtle())
                                .hover(|s| s.bg(theme::bg_hover()))
                                .border_1()
                                .border_color(theme::success())
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .text_color(theme::success())
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1()
                                        .child(
                                            Icon::new(IconName::RotateCcw)
                                                .size_3p5()
                                                .text_color(theme::success()),
                                        )
                                        .child("Re-Enable"),
                                )
                                .on_click(cx.listener(move |this, _, _, cx| {
                                    this.restore_ignored_controller(&guid_str, cx);
                                })),
                        )
                }),
            ))
            .into_any_element(),
    )
}

#[allow(dead_code)]
fn render_launch_button_section(
    _win: &MainWindow,
    state: &CouchSetupState,
    cx: &mut Context<MainWindow>,
) -> AnyElement {
    let can_launch = !state.slots.is_empty();

    let mut btn = CouchButton::primary(
        "btn-couch-launch-session",
        "Start Splitscreen Session (Start / A)",
    )
    .icon(IconName::Play)
    .size(CouchButtonSize::Lg)
    .disabled(!can_launch);

    if can_launch {
        btn = btn.on_click(cx.listener(|this, _, _, cx| {
            this.launch_couch_session(cx);
        }));
    }

    div().flex().w_full().pt_4().child(btn).into_any_element()
}
