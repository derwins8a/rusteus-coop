//! Game-specific profile data and launch configurations.

use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Persistent configuration and state associated with a configured game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct GameProfile {
    /// Unique profile identifier.
    pub id: String,
    /// Unique GUID of the handler associated with this game profile.
    pub handler_guid: String,
    /// Absolute filesystem path to the main game executable.
    pub executable_path: String,
    /// File path to the handler script package.
    pub handler_path: String,
    /// Custom user-specified launch arguments.
    pub custom_arguments: Option<String>,
    /// Last launch timestamp in UTC seconds.
    pub last_played_timestamp: Option<u64>,
    /// Total play time accumulated in seconds.
    #[serde(default)]
    pub total_play_time_seconds: u64,
    /// Configured player count for splitscreen session.
    #[serde(default = "default_player_count")]
    pub player_count: usize,
    /// Monitor layout configuration preference (e.g. "Auto", "Vertical", "Horizontal", "Grid4").
    #[serde(default = "default_layout")]
    pub layout: String,
    /// Selected script options key-value pairs (e.g. "Native" -> "No", "Scale" -> "NoScaled").
    #[serde(default)]
    pub options: std::collections::HashMap<String, String>,
    /// Whether user marked this game as a favorite.
    #[serde(default)]
    pub favorite: bool,
    /// Assign gamepads by device index instead of button presses.
    #[serde(default)]
    pub assign_gamepads_by_index: bool,
    /// Assign gamepads on button press.
    #[serde(default)]
    pub assign_gamepads_on_button_press: bool,
    /// Keep instanced content directories after session exit.
    #[serde(default)]
    pub keep_instances_content_folder: bool,
    /// Disable custom profile settings.
    #[serde(default)]
    pub disable_profile: bool,
    /// Disable automatic handler updates.
    #[serde(default)]
    pub disable_handler_update: bool,
    /// Custom game icon path if overridden by user.
    #[serde(default)]
    pub custom_game_icon: Option<String>,
    /// Steam language override (e.g. "English", "Spanish", "French").
    #[serde(default)]
    pub steam_language: Option<String>,
    /// Optional display title or name for this profile (e.g. "Profile 1", "Co-op 2P").
    #[serde(default)]
    pub profile_name: Option<String>,
    /// Device assignments mapped to player slots (e.g. "Gamepad #0 (Xbox)", "Keyboard & Mouse Slot").
    #[serde(default)]
    pub player_device_assignments: Vec<String>,
    /// Monitor display index assignments per player (0-based display index).
    #[serde(default)]
    pub player_monitor_assignments: Vec<usize>,
    /// Audio endpoint device ID assignments per player.
    #[serde(default)]
    pub player_audio_device_assignments: Vec<String>,
    /// Active Ludusavi save slot ID assigned per player index.
    #[serde(default)]
    pub player_save_slots: Vec<Option<String>>,
    /// Whether Ludusavi automatic save backup and restore is enabled for this profile.
    #[serde(default = "default_true")]
    pub ludusavi_auto_sync: bool,
}

fn default_true() -> bool {
    true
}

fn default_player_count() -> usize {
    2
}

fn default_layout() -> String {
    "Auto".to_string()
}

impl GameProfile {
    /// Creates a new `GameProfile` for the given executable and handler.
    pub fn new(
        id: impl Into<String>,
        handler_guid: impl Into<String>,
        executable_path: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            handler_guid: handler_guid.into(),
            executable_path: executable_path.into(),
            ..Default::default()
        }
    }

    /// Loads a game profile from a JSON file.
    pub fn load_from_file(path: &Path) -> NucleusResult<Self> {
        let content = file_util::read_text_file(path)?;
        let profile = serde_json::from_str(&content)?;
        Ok(profile)
    }

    /// Saves the game profile to a JSON file.
    pub fn save_to_file(&self, path: &Path) -> NucleusResult<()> {
        let content = serde_json::to_string_pretty(self)?;
        file_util::write_text_file(path, &content)?;
        Ok(())
    }
}
