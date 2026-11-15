//! Runtime context associated with a specific player instance.

use serde::{Deserialize, Serialize};

pub use crate::enums::DPIHandling;

fn default_true() -> bool {
    true
}

/// Per-player runtime context passed to handler script functions and lifecycle events.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct HandlerContext {
    /// Zero-based player slot index.
    pub player_index: usize,
    /// Unique numeric ID assigned to the player (e.g. Steam ID or Goldberg user ID).
    pub player_id: u64,
    /// Display nickname for the player.
    pub nickname: String,
    /// Absolute filesystem path to the instanced game directory.
    pub instance_folder: String,
    /// Root game installation folder.
    pub root_install_folder: String,
    /// Root working folder for the instance (symlink or copy destination).
    pub root_folder: String,
    /// Original root folder of the game before instancing.
    pub orig_root_folder: String,
    /// Path to executable being run.
    pub exe_path: String,
    /// Custom isolated save directory for this instance.
    pub save_path: Option<String>,
    /// Command line arguments for starting the instance.
    pub start_arguments: String,
    /// Assigned window width in pixels.
    pub width: i32,
    /// Assigned window height in pixels.
    pub height: i32,
    /// Assigned window X coordinate on desktop.
    pub pos_x: i32,
    /// Assigned window Y coordinate on desktop.
    pub pos_y: i32,
    /// Assigned monitor width.
    pub monitor_width: i32,
    /// Assigned monitor height.
    pub monitor_height: i32,
    /// Whether this instance is controlled via keyboard/mouse.
    pub is_keyboard_player: bool,
    /// Whether this instance is using an individual ProtoInput raw keyboard.
    #[serde(default)]
    pub is_raw_keyboard: bool,
    /// Device GUID or handle string of assigned raw keyboard.
    #[serde(default)]
    pub raw_keyboard_guid: Option<String>,
    /// Whether this instance is using an individual ProtoInput raw mouse.
    #[serde(default)]
    pub is_raw_mouse: bool,
    /// Device GUID or handle string of assigned raw mouse.
    #[serde(default)]
    pub raw_mouse_guid: Option<String>,
    /// Assigned gamepad ID (1-based for x360ce / dinput).
    pub gamepad_id: i32,
    /// GUID of assigned gamepad if gamepad input is used.
    pub gamepad_guid: Option<String>,
    /// Monitor display index assigned to this instance.
    pub monitor_index: i32,
    /// DPI handling method.
    pub dpi_handling: DPIHandling,
    /// Total number of players participating in this session.
    pub number_of_players: usize,
    /// Handler unique identifier.
    pub handler_guid: String,
    /// OS process identifier of the running instance once launched.
    pub process_id: Option<u32>,
    /// OS native window handle (HWND on Windows) for the main game window.
    pub window_handle: Option<usize>,
    /// Selected options key-value pairs (e.g. "Native" -> "No", "Scale" -> "NoScaled").
    #[serde(default)]
    pub options: std::collections::HashMap<String, String>,
    /// Selected monitor/split layout name (e.g. "1x2 (Stacked)", "2x1 (Side-by-Side)").
    #[serde(default)]
    pub layout: String,
    /// Virtual or local IP address assigned to this instance.
    #[serde(default)]
    pub local_ip: String,
    /// Original window width when launched or initialized.
    #[serde(default)]
    pub orig_width: Option<i32>,
    /// Original window height when launched or initialized.
    #[serde(default)]
    pub orig_height: Option<i32>,
    /// Whether to keep aspect ratio for this context.
    #[serde(default)]
    pub keep_aspect_ratio: bool,
    /// Whether this player slot is currently active in the running session.
    #[serde(default = "default_true")]
    pub is_active: bool,
    /// AutoHotkey script execution directives requested by the handler.
    #[serde(default)]
    pub ahk_directives: Vec<AhkScriptDirective>,
}

/// AutoHotkey script execution directive (imported file or inline code).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AhkScriptDirective {
    pub id: String,
    pub is_inline: bool,
    pub content: String,
    pub version: Option<String>,
    pub timing: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
}

impl HandlerContext {
    /// Records an AHK file to be executed for this instance.
    pub fn import_ahk_file(&mut self, rel_path: impl Into<String>, version: Option<String>, timing: Option<String>, args: Vec<String>) {
        let p = rel_path.into();
        let id = format!("file_{}_{}", self.player_index, self.ahk_directives.len());
        self.ahk_directives.push(AhkScriptDirective {
            id,
            is_inline: false,
            content: p,
            version,
            timing,
            args,
        });
    }

    /// Records inline AHK code to be executed for this instance.
    pub fn run_ahk_code(&mut self, code: impl Into<String>, version: Option<String>, timing: Option<String>, args: Vec<String>) {
        let c = code.into();
        let id = format!("inline_{}_{}", self.player_index, self.ahk_directives.len());
        self.ahk_directives.push(AhkScriptDirective {
            id,
            is_inline: true,
            content: c,
            version,
            timing,
            args,
        });
    }
    /// Creates a new `HandlerContext` for the given player slot.
    pub fn new(player_index: usize, nickname: String) -> Self {
        Self {
            player_index,
            player_id: (player_index + 1) as u64,
            nickname,
            width: 1280,
            height: 720,
            monitor_width: 1920,
            monitor_height: 1080,
            number_of_players: 1,
            is_active: true,
            ..Default::default()
        }
    }

    /// Calculates greatest common divisor for aspect ratio formatting.
    fn gcd(a: i32, b: i32) -> i32 {
        if b == 0 {
            a.abs()
        } else {
            Self::gcd(b, a % b)
        }
    }

    /// Returns string representation of aspect ratio (e.g. "16:9").
    pub fn aspect_ratio(&self) -> String {
        let w = if self.width > 0 { self.width } else { 16 };
        let h = if self.height > 0 { self.height } else { 9 };
        let d = Self::gcd(w, h);
        let d = if d == 0 { 1 } else { d };
        format!("{}:{}", w / d, h / d)
    }

    /// Returns floating-point aspect ratio value (width / height).
    pub fn aspect_ratio_decimal(&self) -> f64 {
        if self.height > 0 {
            self.width as f64 / self.height as f64
        } else {
            1.7777777777777777
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handler_context_is_active_default_and_serde() {
        let ctx = HandlerContext::new(0, "Player1".to_string());
        assert!(ctx.is_active);

        // Missing is_active in JSON should default to true
        let parsed: HandlerContext = serde_json::from_str(r#"{"player_index":0,"player_id":1,"nickname":"P1","instance_folder":"","root_install_folder":"","root_folder":"","orig_root_folder":"","exe_path":"","save_path":null,"start_arguments":"","width":1280,"height":720,"pos_x":0,"pos_y":0,"monitor_width":1920,"monitor_height":1080,"is_keyboard_player":false,"gamepad_id":0,"gamepad_guid":null,"monitor_index":0,"dpi_handling":"True","number_of_players":1,"handler_guid":"","process_id":null,"window_handle":null}"#).unwrap();
        assert!(parsed.is_active);

        let parsed_inactive: HandlerContext = serde_json::from_str(r#"{"player_index":1,"player_id":2,"nickname":"P2","instance_folder":"","root_install_folder":"","root_folder":"","orig_root_folder":"","exe_path":"","save_path":null,"start_arguments":"","width":1280,"height":720,"pos_x":0,"pos_y":0,"monitor_width":1920,"monitor_height":1080,"is_keyboard_player":false,"gamepad_id":0,"gamepad_guid":null,"monitor_index":0,"dpi_handling":"True","number_of_players":1,"handler_guid":"","process_id":null,"window_handle":null,"is_active":false}"#).unwrap();
        assert!(!parsed_inactive.is_active);
    }
}

