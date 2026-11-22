//! Global application settings and user preferences.

use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Global application configuration and preferences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    // --- General & Paths ---
    /// Directory where handlers are stored.
    pub handlers_directory: String,
    /// Active UI theme name.
    pub theme: String,
    /// Theme accent color palette ("Blue", "Purple", "Emerald", "Rose", "Amber", "Cyan", "Indigo").
    pub accent_color: String,
    /// Default window width in pixels.
    pub window_width: u32,
    /// Default window height in pixels.
    pub window_height: u32,
    /// Automatically scan storage drives for games when the app starts.
    pub auto_scan_on_start: bool,
    /// Custom data directory for Nucleus Coop storage (empty string for default).
    pub nucleus_data_dir: String,
    /// Close Nucleus Coop application when game instances are launched.
    pub close_on_launch: bool,
    /// Periodically check for application and handler updates.
    pub check_for_updates: bool,
    /// Preferred UI language identifier.
    pub language: String,
    /// Base URL for the Splitscreen.Me Community Hub.
    pub hub_api_url: String,
    /// Automatically restore saved Hub session token on startup.
    pub hub_remember_session: bool,
    /// Enable detailed debug logging output.
    pub debug_log: bool,
    /// Minimize application to system tray on start.
    pub minimize_to_tray: bool,
    /// Automatically reposition game windows based on screen layout.
    pub auto_position_windows: bool,

    // --- Language & Network (C# Parity) ---
    /// Steam language setting passed to emulator or game launcher.
    pub steam_lang: String,
    /// Epic games language setting.
    pub epic_lang: String,
    /// Preferred network interface name or IP ("Automatic" by default).
    pub network: String,

    // --- Behavior, Accounts & Isolation (C# Parity) ---
    /// Whether to inject custom nicknames into game instances.
    pub use_nicks_in_game: bool,
    /// Keep created user accounts and directories between launches.
    pub keep_accounts: bool,
    /// Password used for created Windows user accounts.
    pub nucleus_account_password: String,
    /// Suppress input lock reminder notification on launch.
    pub ignore_input_lock_reminder: bool,
    /// Automatically adjust monitor DPI desktop scaling.
    pub auto_desktop_scaling: bool,
    /// Allow multiple concurrent Nucleus Coop launcher processes.
    pub nucleus_multi_instances: bool,
    /// Monotonic schema version for settings migrations.
    #[serde(default)]
    pub settings_schema_version: u32,
    /// Disable persistent game profiles.
    pub disable_game_profiles: bool,
    /// Custom text editor path for opening handler scripts.
    pub text_editor_path: String,

    // --- Gamepad & Input Device Assignment (C# Parity) ---
    /// Use raw XInput controller indices instead of DInput/SDL ordering.
    pub use_xinput_index: bool,
    /// Automatically assign gamepads to player slots by first button press.
    pub profile_assign_gamepad_by_button_press: bool,
    /// Virtual Gamepad Mode only (disables physical controller assignment).
    pub vgm_only: bool,

    // --- OSD & Visual Preferences (C# Parity) ---
    /// Foreground and border color of On-Screen Display (RGB "r,g,b" or hex).
    pub osd_color: String,
    /// Whether the OSD overlay is enabled.
    pub osd_enabled: bool,
    /// Blur strength for background overlay when dialogs are shown.
    pub blur: u32,

    // --- Hotkeys (C# Parity) ---
    /// Key used to toggle input lock (default "End").
    pub lock_inputs_key: String,
    /// Shortcut to close game instances (default "Ctrl+Q").
    pub hotkey_close: String,
    /// Shortcut to stop the active session (default "Ctrl+E").
    pub hotkey_stop: String,
    /// Shortcut to toggle game window top-most flag (default "Ctrl+T").
    pub hotkey_topmost: String,
    /// Shortcut to unfocus / set focus to game window (default "Ctrl+H").
    pub hotkey_set_focus: String,
    /// Shortcut to reposition and reset game windows (default "Ctrl+R").
    pub hotkey_reset_windows: String,
    /// Shortcut to toggle cutscenes mode (default "Ctrl+M").
    pub hotkey_cutscenes: String,
    /// Shortcut to switch splitscreen layouts (default "Ctrl+N").
    pub hotkey_switch: String,
    /// Shortcut to show the on-screen keyboard shortcuts reminder (default "Alt+R").
    pub hotkey_shortcuts_reminder: String,
    /// Shortcut to switch active window merger child focus (default "Alt+A").
    pub hotkey_merger_switch: String,
    /// Shortcut to toggle Couch in-game Big Picture OSD menu (default "F10").
    pub hotkey_couch_osd: String,

    // --- Controller & Gamepad Shortcuts (C# Parity: App_GamePadShortcuts & App_GamePadNavigation) ---
    /// Whether gamepad shortcuts and controller navigation are enabled.
    pub gamepad_shortcuts_enabled: bool,
    /// Controller button glyph style: "Xbox" or "Playstation".
    pub gamepad_type: String,
    /// Analog deadzone threshold for controller UI navigation (default 10000).
    pub gamepad_deadzone: i32,
    /// Drag and Drop button (default "RB").
    pub gamepad_drag_drop: String,
    /// Right click button (default "LT").
    pub gamepad_right_click: String,
    /// Left click button (default "RT").
    pub gamepad_left_click: String,
    /// On-screen keyboard shortcut combo (default "Guide+DPadUp").
    pub gamepad_shortcut_osk: String,
    /// Unfocus game windows shortcut combo (default "Guide+Y").
    pub gamepad_shortcut_unfocus: String,
    /// Close Nucleus shortcut combo (default "Guide+Start").
    pub gamepad_shortcut_close: String,
    /// Stop session shortcut combo (default "Guide+Back").
    pub gamepad_shortcut_stop: String,
    /// Toggle topmost shortcut combo (default "Guide+X").
    pub gamepad_shortcut_topmost: String,
    /// Reset windows shortcut combo (default "Guide+DPadRight").
    pub gamepad_shortcut_reset_windows: String,
    /// Cutscenes mode shortcut combo (default "Guide+B").
    pub gamepad_shortcut_cutscenes: String,
    /// Switch layouts shortcut combo (default "Guide+RB").
    pub gamepad_shortcut_switch_layout: String,
    /// Lock inputs shortcut combo (default "Guide+RS").
    pub gamepad_shortcut_lock_inputs: String,
    /// Alt+Tab cursor release shortcut combo (default "Guide+LB").
    pub gamepad_shortcut_release_cursor: String,
    /// Lock UI control shortcut combo (default "Guide+DPadDown").
    pub gamepad_shortcut_lock_ui: String,

    // --- Player Slot Nicknames and Steam IDs (32 Players, C# Parity) ---
    /// Nicknames for player slots 1 through 32.
    pub player_nicknames: Vec<String>,
    /// Custom Steam IDs for player slots 1 through 32.
    pub player_steam_ids: Vec<String>,

    // --- Audio Routing (C# Parity) ---
    /// Whether custom per-instance audio device routing is enabled.
    pub audio_custom: bool,
    /// Audio endpoint device names assigned to instance 1 through 8.
    pub audio_instances: Vec<String>,

    // --- Custom Layout & Windows Merger (C# Parity) ---
    /// Enable Windows Merger tool for single-window composition.
    pub windows_merger: bool,
    /// Enable Lossless hook for Windows Merger.
    pub lossless_hook: bool,
    /// Custom resolution for Windows Merger viewport.
    pub merger_resolution: String,
    /// Enable visual divider lines between splitscreen viewports.
    pub split_div: bool,
    /// Color of divider lines ("Black", "White", etc.).
    pub split_div_color: String,
    /// Hide Windows desktop background during active session.
    pub hide_desktop: bool,
    /// Number of horizontal divider lines.
    pub horizontal_lines: u32,
    /// Number of vertical divider lines.
    pub vertical_lines: u32,
    /// Maximum players supported in custom layout.
    pub max_players: u32,
    /// Cutscenes: keep aspect ratio.
    pub cts_keep_aspect_ratio: bool,
    /// Cutscenes: mute audio only on unfocused viewports.
    pub cts_mute_audio_only: bool,
    /// Cutscenes: unfocus viewport during cutscene.
    pub cts_unfocus: bool,
    /// Hardware device GUIDs to ignore / blacklist from couch controller auto-assignment.
    #[serde(default)]
    pub ignored_device_guids: Vec<String>,

    // --- Network Loopback IP & Goldberg Emulation ---
    /// Enable loopback IP isolation (127.0.0.X).
    pub loopback_isolation_enabled: bool,
    /// Base loopback IP address (default "127.0.0.2").
    pub loopback_base_ip: String,
    /// Goldberg broadcast IP address (default "127.0.0.255").
    pub goldberg_broadcast_ip: String,
    /// Goldberg listen port (default 47584).
    pub goldberg_listen_port: u16,
    /// Goldberg offline mode force flag.
    pub goldberg_offline_mode: bool,
    /// Custom Steam language override.
    pub steam_language_override: Option<String>,
    /// Whether launch countdown prompt auto-proceeds.
    pub auto_proceed_countdown: bool,
    /// Launch countdown seconds.
    pub auto_proceed_seconds: u32,

    // --- Ludusavi Savegame Management ---
    /// Custom path to ludusavi executable.
    #[serde(default)]
    pub ludusavi_path: Option<String>,
    /// Whether Ludusavi savegame management is enabled.
    #[serde(default = "default_true")]
    pub ludusavi_enabled: bool,
    /// Maximum backup snapshots kept per save slot.
    #[serde(default = "default_max_backups")]
    pub ludusavi_max_backups_per_slot: u32,
    /// Backup host PC saves before launching a game session.
    #[serde(default = "default_true")]
    pub ludusavi_backup_host_before_launch: bool,

    // --- AutoHotkey Runtime ---
    /// Custom directory containing AutoHotkey v1/v2 runtimes.
    #[serde(default)]
    pub autohotkey_custom_path: Option<String>,

    // --- Extensions, Add-ons & Plugins ---
    /// Whether plugin and addon extensions are enabled.
    #[serde(default = "default_true")]
    pub plugins_enabled: bool,

    // --- Handler Debugger ---
    /// Automatically attach debugger on handler launch.
    #[serde(default)]
    pub debugger_auto_attach: bool,

    // --- Big Picture / Couch Mode ---
    /// Whether to start Nucleus Co-op directly in Big Picture mode on application launch.
    #[serde(default)]
    pub start_in_big_picture: bool,
    /// Interface scale preference ("Normal", "Large", "Large · TV").
    #[serde(default = "default_interface_scale")]
    pub interface_scale: String,
    /// Whether interface navigation/confirmation sounds are enabled.
    #[serde(default = "default_true")]
    pub interface_sounds: bool,
    /// Preferred display monitor index (0 = primary).
    #[serde(default)]
    pub preferred_display: usize,
    /// Custom key-value preferences for individual game handlers.
    #[serde(default)]
    pub custom_preferences: std::collections::HashMap<String, String>,

    // --- PeerParty Online Multiplayer ---
    /// Whether PeerParty P2P multiplayer emulation is enabled globally.
    #[serde(default = "default_true")]
    pub peerparty_enabled: bool,
    /// Default room code to host or join with PeerParty.
    #[serde(default)]
    pub peerparty_default_room: Option<String>,
    /// Whether to announce and discover sessions publicly via BitTorrent DHT.
    #[serde(default)]
    pub peerparty_public_sessions: bool,
    /// Whether PeerParty in-game voice chat is enabled.
    #[serde(default)]
    pub peerparty_voice_chat: bool,
    /// Whether PeerParty in-game overlay is enabled.
    #[serde(default = "default_true")]
    pub peerparty_overlay: bool,
    /// Preferred multiplayer emulator backend ("Auto", "Goldberg", "Nemirtingas", "PeerParty").
    #[serde(default = "default_preferred_emulator")]
    pub preferred_emulator: String,

    // --- Developer Mode ---
    /// Enable developer tools, mock device injection, and advanced diagnostics.
    #[serde(default)]
    pub developer_mode: bool,
}

fn default_preferred_emulator() -> String {
    "Auto".to_string()
}

fn default_interface_scale() -> String {
    "Large · TV".to_string()
}

fn default_true() -> bool {
    true
}

fn default_max_backups() -> u32 {
    5
}

impl Default for AppSettings {
    fn default() -> Self {
        let mut nicknames = Vec::with_capacity(32);
        let mut steam_ids = Vec::with_capacity(32);
        for i in 1..=32 {
            nicknames.push(format!("Player{}", i));
            steam_ids.push(format!("765611980000000{:02}", i));
        }

        let mut audio_instances = Vec::with_capacity(8);
        for _ in 0..8 {
            audio_instances.push("Default".to_string());
        }

        Self {
            handlers_directory: "handlers".to_string(),
            theme: "dark".to_string(),
            accent_color: "Blue".to_string(),
            window_width: 1200,
            window_height: 800,
            auto_scan_on_start: false,
            nucleus_data_dir: String::new(),
            close_on_launch: false,
            check_for_updates: true,
            language: "en".to_string(),
            hub_api_url: "https://hub.splitscreen.me".to_string(),
            hub_remember_session: true,
            debug_log: false,
            minimize_to_tray: false,
            auto_position_windows: true,

            steam_lang: "Automatic".to_string(),
            epic_lang: "English".to_string(),
            network: "Automatic".to_string(),

            use_nicks_in_game: true,
            keep_accounts: false,
            nucleus_account_password: "12345".to_string(),
            ignore_input_lock_reminder: true,
            auto_desktop_scaling: false,
            nucleus_multi_instances: false,
            settings_schema_version: 1,
            disable_game_profiles: false,
            text_editor_path: "Default".to_string(),

            use_xinput_index: false,
            profile_assign_gamepad_by_button_press: false,
            vgm_only: false,

            osd_color: "255,255,255".to_string(),
            osd_enabled: true,
            blur: 4,

            lock_inputs_key: "End".to_string(),
            hotkey_close: "Ctrl+Q".to_string(),
            hotkey_stop: "Ctrl+E".to_string(),
            hotkey_topmost: "Ctrl+T".to_string(),
            hotkey_set_focus: "Ctrl+H".to_string(),
            hotkey_reset_windows: "Ctrl+R".to_string(),
            hotkey_cutscenes: "Ctrl+M".to_string(),
            hotkey_switch: "Ctrl+N".to_string(),
            hotkey_shortcuts_reminder: "Alt+R".to_string(),
            hotkey_merger_switch: "Alt+A".to_string(),
            hotkey_couch_osd: "F10".to_string(),

            gamepad_shortcuts_enabled: true,
            gamepad_type: "Xbox".to_string(),
            gamepad_deadzone: 10000,
            gamepad_drag_drop: "RB".to_string(),
            gamepad_right_click: "LT".to_string(),
            gamepad_left_click: "RT".to_string(),
            gamepad_shortcut_osk: "Guide+DPadUp".to_string(),
            gamepad_shortcut_unfocus: "Guide+Y".to_string(),
            gamepad_shortcut_close: "Guide+Start".to_string(),
            gamepad_shortcut_stop: "Guide+Back".to_string(),
            gamepad_shortcut_topmost: "Guide+X".to_string(),
            gamepad_shortcut_reset_windows: "Guide+DPadRight".to_string(),
            gamepad_shortcut_cutscenes: "Guide+B".to_string(),
            gamepad_shortcut_switch_layout: "Guide+RB".to_string(),
            gamepad_shortcut_lock_inputs: "Guide+RS".to_string(),
            gamepad_shortcut_release_cursor: "Guide+LB".to_string(),
            gamepad_shortcut_lock_ui: "Guide+DPadDown".to_string(),

            player_nicknames: nicknames,
            player_steam_ids: steam_ids,

            audio_custom: false,
            audio_instances,

            windows_merger: false,
            lossless_hook: true,
            merger_resolution: String::new(),
            split_div: false,
            split_div_color: "Black".to_string(),
            hide_desktop: false,
            horizontal_lines: 1,
            vertical_lines: 1,
            max_players: 4,
            cts_keep_aspect_ratio: false,
            cts_mute_audio_only: false,
            cts_unfocus: false,
            ignored_device_guids: Vec::new(),

            loopback_isolation_enabled: true,
            loopback_base_ip: "127.0.0.2".to_string(),
            goldberg_broadcast_ip: "127.0.0.255".to_string(),
            goldberg_listen_port: 47584,
            goldberg_offline_mode: true,
            steam_language_override: None,
            auto_proceed_countdown: true,
            auto_proceed_seconds: 10,

            ludusavi_path: None,
            ludusavi_enabled: true,
            ludusavi_max_backups_per_slot: 5,
            ludusavi_backup_host_before_launch: true,

            autohotkey_custom_path: None,
            plugins_enabled: true,
            debugger_auto_attach: false,

            start_in_big_picture: false,
            interface_scale: "Large · TV".to_string(),
            interface_sounds: true,
            preferred_display: 0,
            custom_preferences: std::collections::HashMap::new(),

            peerparty_enabled: true,
            peerparty_default_room: None,
            peerparty_public_sessions: false,
            peerparty_voice_chat: false,
            peerparty_overlay: true,
            preferred_emulator: "Auto".to_string(),
            developer_mode: false,
        }
    }
}

impl AppSettings {
    /// Current schema version for [`AppSettings`] JSON files.
    pub const CURRENT_SCHEMA_VERSION: u32 = 1;

    /// Migrates legacy settings payloads in place to [`Self::CURRENT_SCHEMA_VERSION`].
    pub fn migrate_in_place(&mut self) {
        if self.settings_schema_version < Self::CURRENT_SCHEMA_VERSION {
            self.nucleus_multi_instances = false;
            self.settings_schema_version = Self::CURRENT_SCHEMA_VERSION;
        }
    }

    /// Parses settings from a JSON string and applies any pending schema migrations.
    pub fn from_json_str(content: &str) -> NucleusResult<Self> {
        let mut settings: Self = serde_json::from_str(content)?;
        settings.migrate_in_place();
        Ok(settings)
    }

    /// Returns the path to the settings file: %APPDATA%\NucleusCoop\settings.json
    pub fn settings_path() -> std::path::PathBuf {
        let appdata = std::env::var("APPDATA")
            .unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().to_string());
        std::path::PathBuf::from(appdata)
            .join("NucleusCoop")
            .join("settings.json")
    }

    /// Load settings from disk, returning defaults if the file doesn't exist or fails to parse.
    pub fn load() -> Self {
        let path = Self::settings_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(settings) = Self::from_json_str(&content) {
                    return settings;
                }
            }
        }
        Self::default()
    }

    /// Save settings to disk, creating directories as needed.
    pub fn save(&self) -> nucleus_common::error::NucleusResult<()> {
        let path = Self::settings_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Loads application settings from a JSON file, returning defaults if not found.
    pub fn load_from_file(path: &Path) -> NucleusResult<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = file_util::read_text_file(path)?;
        Self::from_json_str(&content)
    }

    /// Saves application settings to a JSON file.
    pub fn save_to_file(&self, path: &Path) -> NucleusResult<()> {
        let content = serde_json::to_string_pretty(self)?;
        file_util::write_text_file(path, &content)?;
        Ok(())
    }

    /// Retrieves the player nickname for slot `player_index` (0-based).
    pub fn get_player_nickname(&self, player_index: usize) -> String {
        if player_index < self.player_nicknames.len()
            && !self.player_nicknames[player_index].is_empty()
        {
            self.player_nicknames[player_index].clone()
        } else {
            format!("Player{}", player_index + 1)
        }
    }

    /// Sets the player nickname for slot `player_index` (0-based).
    pub fn set_player_nickname(&mut self, player_index: usize, nickname: impl Into<String>) {
        if player_index >= self.player_nicknames.len() {
            self.player_nicknames
                .resize(player_index + 1, String::new());
        }
        self.player_nicknames[player_index] = nickname.into();
    }

    /// Retrieves the Steam ID for slot `player_index` (0-based).
    pub fn get_player_steam_id(&self, player_index: usize) -> String {
        if player_index < self.player_steam_ids.len()
            && !self.player_steam_ids[player_index].is_empty()
        {
            self.player_steam_ids[player_index].clone()
        } else {
            format!("765611980000000{:02}", player_index + 1)
        }
    }

    /// Sets the Steam ID for slot `player_index` (0-based).
    pub fn set_player_steam_id(&mut self, player_index: usize, steam_id: impl Into<String>) {
        if player_index >= self.player_steam_ids.len() {
            self.player_steam_ids
                .resize(player_index + 1, String::new());
        }
        self.player_steam_ids[player_index] = steam_id.into();
    }

    /// Retrieves the assigned audio device for an instance index (0-based).
    pub fn get_audio_instance(&self, instance_index: usize) -> &str {
        if instance_index < self.audio_instances.len() {
            &self.audio_instances[instance_index]
        } else {
            "Default"
        }
    }

    /// Sets the assigned audio device for an instance index (0-based).
    pub fn set_audio_instance(&mut self, instance_index: usize, device: impl Into<String>) {
        if instance_index >= self.audio_instances.len() {
            self.audio_instances
                .resize(instance_index + 1, "Default".to_string());
        }
        self.audio_instances[instance_index] = device.into();
    }

    /// Parses a hotkey string such as "Ctrl+Q", "Ctrl+N", "Shift+F1", "Ctrl+Tab", etc. into (modifier_mask, virtual_key_code).
    /// Modifier mask: 1 = Ctrl, 2 = Alt, 4 = Shift.
    pub fn parse_hotkey(hotkey_str: &str) -> Option<(u32, u32)> {
        let parts: Vec<&str> = hotkey_str.split('+').map(|s| s.trim()).collect();
        if parts.is_empty() {
            return None;
        }

        let mut modifier = 0u32;
        let mut key_part = None;

        for (idx, part) in parts.iter().enumerate() {
            let lower = part.to_lowercase();
            if idx < parts.len() - 1
                || lower == "ctrl"
                || lower == "control"
                || lower == "alt"
                || lower == "menu"
                || lower == "shift"
            {
                match lower.as_str() {
                    "ctrl" | "control" => modifier |= 1,
                    "alt" | "menu" => modifier |= 2,
                    "shift" => modifier |= 4,
                    _ => {
                        if idx == parts.len() - 1 {
                            key_part = Some(*part);
                        } else {
                            return None;
                        }
                    }
                }
            } else {
                key_part = Some(*part);
            }
        }

        let key_str = key_part?.trim();
        let key_lower = key_str.to_lowercase();
        let key_code = match key_lower.as_str() {
            "tab" => 0x09,
            "enter" | "return" => 0x0D,
            "space" => 0x20,
            "backspace" => 0x08,
            "escape" | "esc" => 0x1B,
            "pageup" | "prior" => 0x21,
            "pagedown" | "next" => 0x22,
            "end" => 0x23,
            "home" => 0x24,
            "left" => 0x25,
            "up" => 0x26,
            "right" => 0x27,
            "down" => 0x28,
            "insert" | "ins" => 0x2D,
            "delete" | "del" => 0x2E,
            "f1" => 0x70,
            "f2" => 0x71,
            "f3" => 0x72,
            "f4" => 0x73,
            "f5" => 0x74,
            "f6" => 0x75,
            "f7" => 0x76,
            "f8" => 0x77,
            "f9" => 0x78,
            "f10" => 0x79,
            "f11" => 0x7A,
            "f12" => 0x7B,
            _ => {
                if key_str.len() == 1 {
                    let c = key_str.chars().next()?.to_ascii_uppercase();
                    if c.is_ascii_alphanumeric() {
                        c as u32
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
        };

        Some((modifier, key_code))
    }

    /// Parses a lock key string such as "End", "Home", "F1", etc. into its virtual key code.
    pub fn parse_lock_key(key_str: &str) -> u32 {
        match key_str.trim().to_lowercase().as_str() {
            "end" => 0x23,
            "home" => 0x24,
            "delete" | "del" => 0x2E,
            "multiply" | "*" => 0x6A,
            "f1" => 0x70,
            "f2" => 0x71,
            "f3" => 0x72,
            "f4" => 0x73,
            "f5" => 0x74,
            "f6" => 0x75,
            "f7" => 0x76,
            "f8" => 0x77,
            "f9" => 0x78,
            "f10" => 0x79,
            "f11" => 0x7A,
            "f12" => 0x7B,
            "+" => 0xBB,
            "-" => 0xBD,
            "numpad 0" | "num0" => 0x60,
            "numpad 1" | "num1" => 0x61,
            "numpad 2" | "num2" => 0x62,
            "numpad 3" | "num3" => 0x63,
            "numpad 4" | "num4" => 0x64,
            "numpad 5" | "num5" => 0x65,
            "numpad 6" | "num6" => 0x66,
            "numpad 7" | "num7" => 0x67,
            "numpad 8" | "num8" => 0x68,
            "numpad 9" | "num9" => 0x69,
            _ => 0x23, // Default to VK_END
        }
    }

    /// Returns true if the string is a valid 17-digit Steam ID (matching C# Settings.cs:656).
    pub fn is_valid_steam_id(id: &str) -> bool {
        id.len() == 17 && id.chars().all(|c| c.is_ascii_digit())
    }

    /// Supported input lock toggle keys matching C# App_Hotkeys.lockKeysDict.
    pub const LOCK_KEYS: [&'static str; 16] = [
        "End", "Home", "Delete", "Multiply", "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9",
        "F10", "F11", "F12",
    ];

    /// Cycles the input lock toggle key to the next available option.
    pub fn cycle_lock_key(&mut self) {
        let cur = self.lock_inputs_key.as_str();
        let idx = Self::LOCK_KEYS
            .iter()
            .position(|&k| k.eq_ignore_ascii_case(cur))
            .unwrap_or(0);
        let next = Self::LOCK_KEYS[(idx + 1) % Self::LOCK_KEYS.len()];
        self.lock_inputs_key = next.to_string();
    }

    /// Formats a modifier mask and virtual key code into a display string (e.g. "Ctrl+Q", "F10").
    pub fn format_hotkey(modifier: u32, key_code: u32) -> String {
        let key_str = if (0x41..=0x5A).contains(&key_code) || (0x30..=0x39).contains(&key_code) {
            char::from_u32(key_code).unwrap_or('Q').to_string()
        } else if (0x70..=0x7B).contains(&key_code) {
            format!("F{}", key_code - 0x70 + 1)
        } else {
            "Q".to_string()
        };
        match modifier {
            0 => key_str,
            1 => format!("Ctrl+{}", key_str),
            2 => format!("Alt+{}", key_str),
            4 => format!("Shift+{}", key_str),
            5 => format!("Ctrl+Shift+{}", key_str),
            _ => format!("Ctrl+{}", key_str),
        }
    }

    /// Cycles the modifier of a hotkey string (Ctrl -> Alt -> Shift -> Ctrl, or None -> Ctrl).
    pub fn cycle_modifier(hotkey_str: &str) -> String {
        if let Some((m, k)) = Self::parse_hotkey(hotkey_str) {
            let next_m = match m {
                0 => 1, // None -> Ctrl
                1 => 2, // Ctrl -> Alt
                2 => 4, // Alt -> Shift
                4 => 1, // Shift -> Ctrl
                _ => 1,
            };
            Self::format_hotkey(next_m, k)
        } else {
            "Ctrl+Q".to_string()
        }
    }

    /// Cycles the key of a hotkey string through standard hotkey keys.
    pub fn cycle_key(hotkey_str: &str) -> String {
        let keys = [
            'Q', 'E', 'T', 'H', 'R', 'M', 'N', 'A', 'Z', 'X', 'C', 'P', 'L', 'K', 'J', '1', '2',
            '3', '4',
        ];
        if let Some((m, k)) = Self::parse_hotkey(hotkey_str) {
            if (0x70..=0x7B).contains(&k) {
                let next_f = if k >= 0x7B { 0x70 } else { k + 1 };
                Self::format_hotkey(m, next_f)
            } else {
                let cur_char = char::from_u32(k).unwrap_or('Q');
                let idx = keys.iter().position(|&c| c == cur_char).unwrap_or(0);
                let next_char = keys[(idx + 1) % keys.len()];
                Self::format_hotkey(m, next_char as u32)
            }
        } else {
            "Ctrl+Q".to_string()
        }
    }

    /// Cycles the modifier for a named hotkey field.
    pub fn cycle_hotkey_modifier(&mut self, hotkey_name: &str) {
        match hotkey_name {
            "close" => self.hotkey_close = Self::cycle_modifier(&self.hotkey_close),
            "stop" => self.hotkey_stop = Self::cycle_modifier(&self.hotkey_stop),
            "topmost" => self.hotkey_topmost = Self::cycle_modifier(&self.hotkey_topmost),
            "set_focus" => self.hotkey_set_focus = Self::cycle_modifier(&self.hotkey_set_focus),
            "reset_windows" => {
                self.hotkey_reset_windows = Self::cycle_modifier(&self.hotkey_reset_windows)
            }
            "cutscenes" => self.hotkey_cutscenes = Self::cycle_modifier(&self.hotkey_cutscenes),
            "switch" => self.hotkey_switch = Self::cycle_modifier(&self.hotkey_switch),
            "shortcuts_reminder" => {
                self.hotkey_shortcuts_reminder =
                    Self::cycle_modifier(&self.hotkey_shortcuts_reminder)
            }
            "merger_switch" => {
                self.hotkey_merger_switch = Self::cycle_modifier(&self.hotkey_merger_switch)
            }
            "couch_osd" => self.hotkey_couch_osd = Self::cycle_modifier(&self.hotkey_couch_osd),
            _ => {}
        }
    }

    /// Cycles the key for a named hotkey field.
    pub fn cycle_hotkey_key(&mut self, hotkey_name: &str) {
        match hotkey_name {
            "close" => self.hotkey_close = Self::cycle_key(&self.hotkey_close),
            "stop" => self.hotkey_stop = Self::cycle_key(&self.hotkey_stop),
            "topmost" => self.hotkey_topmost = Self::cycle_key(&self.hotkey_topmost),
            "set_focus" => self.hotkey_set_focus = Self::cycle_key(&self.hotkey_set_focus),
            "reset_windows" => {
                self.hotkey_reset_windows = Self::cycle_key(&self.hotkey_reset_windows)
            }
            "cutscenes" => self.hotkey_cutscenes = Self::cycle_key(&self.hotkey_cutscenes),
            "switch" => self.hotkey_switch = Self::cycle_key(&self.hotkey_switch),
            "shortcuts_reminder" => {
                self.hotkey_shortcuts_reminder = Self::cycle_key(&self.hotkey_shortcuts_reminder)
            }
            "merger_switch" => {
                self.hotkey_merger_switch = Self::cycle_key(&self.hotkey_merger_switch)
            }
            "couch_osd" => self.hotkey_couch_osd = Self::cycle_key(&self.hotkey_couch_osd),
            _ => {}
        }
    }

    /// Returns the virtual key code for this instance's configured `lock_inputs_key`.
    pub fn lock_key_code(&self) -> u32 {
        Self::parse_lock_key(&self.lock_inputs_key)
    }

    /// Returns the configured binding string for a named controller action.
    pub fn get_gamepad_binding(&self, action: &str) -> &str {
        match action {
            "unfocus" => &self.gamepad_shortcut_unfocus,
            "close" => &self.gamepad_shortcut_close,
            "stop" => &self.gamepad_shortcut_stop,
            "topmost" => &self.gamepad_shortcut_topmost,
            "reset_windows" => &self.gamepad_shortcut_reset_windows,
            "cutscenes" => &self.gamepad_shortcut_cutscenes,
            "switch_layout" => &self.gamepad_shortcut_switch_layout,
            "lock_inputs" => &self.gamepad_shortcut_lock_inputs,
            "release_cursor" => &self.gamepad_shortcut_release_cursor,
            "lock_ui" => &self.gamepad_shortcut_lock_ui,
            "osk" => &self.gamepad_shortcut_osk,
            "drag_drop" => &self.gamepad_drag_drop,
            "right_click" => &self.gamepad_right_click,
            "left_click" => &self.gamepad_left_click,
            _ => "",
        }
    }

    /// Updates the configured binding string for a named controller action.
    pub fn set_gamepad_binding(&mut self, action: &str, binding: String) {
        match action {
            "unfocus" => self.gamepad_shortcut_unfocus = binding,
            "close" => self.gamepad_shortcut_close = binding,
            "stop" => self.gamepad_shortcut_stop = binding,
            "topmost" => self.gamepad_shortcut_topmost = binding,
            "reset_windows" => self.gamepad_shortcut_reset_windows = binding,
            "cutscenes" => self.gamepad_shortcut_cutscenes = binding,
            "switch_layout" => self.gamepad_shortcut_switch_layout = binding,
            "lock_inputs" => self.gamepad_shortcut_lock_inputs = binding,
            "release_cursor" => self.gamepad_shortcut_release_cursor = binding,
            "lock_ui" => self.gamepad_shortcut_lock_ui = binding,
            "osk" => self.gamepad_shortcut_osk = binding,
            "drag_drop" => self.gamepad_drag_drop = binding,
            "right_click" => self.gamepad_right_click = binding,
            "left_click" => self.gamepad_left_click = binding,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_single_instance_with_schema_v1() {
        let settings = AppSettings::default();
        assert!(!settings.nucleus_multi_instances);
        assert_eq!(settings.settings_schema_version, 1);
    }

    #[test]
    fn test_legacy_json_migrates_multi_instances_to_false_preserving_fields() {
        let legacy_json = r#"{
            "theme": "nord",
            "handlers_directory": "D:\\CustomHandlers",
            "lock_inputs_key": "F8",
            "hotkey_close": "Alt+F4",
            "nucleus_multi_instances": true,
            "close_on_launch": true
        }"#;

        let migrated = AppSettings::from_json_str(legacy_json).unwrap();
        assert!(!migrated.nucleus_multi_instances);
        assert_eq!(migrated.settings_schema_version, 1);
        assert_eq!(migrated.theme, "nord");
        assert_eq!(migrated.handlers_directory, "D:\\CustomHandlers");
        assert_eq!(migrated.lock_inputs_key, "F8");
        assert_eq!(migrated.hotkey_close, "Alt+F4");
        assert!(migrated.close_on_launch);
    }

    #[test]
    fn test_current_schema_v1_preserves_explicit_multi_instances_opt_in() {
        let current_json = r#"{
            "settings_schema_version": 1,
            "nucleus_multi_instances": true,
            "theme": "dracula"
        }"#;

        let loaded = AppSettings::from_json_str(current_json).unwrap();
        assert!(loaded.nucleus_multi_instances);
        assert_eq!(loaded.settings_schema_version, 1);
        assert_eq!(loaded.theme, "dracula");
    }

    #[test]
    fn test_load_from_file_migrates_legacy_json() {
        let temp_dir = std::env::temp_dir().join(format!(
            "nucleus_settings_mig_{}_{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();
        let path = temp_dir.join("settings.json");

        std::fs::write(
            &path,
            r#"{"theme": "solarized", "nucleus_multi_instances": true}"#,
        )
        .unwrap();

        let loaded = AppSettings::load_from_file(&path).unwrap();
        assert!(!loaded.nucleus_multi_instances);
        assert_eq!(loaded.settings_schema_version, 1);
        assert_eq!(loaded.theme, "solarized");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
