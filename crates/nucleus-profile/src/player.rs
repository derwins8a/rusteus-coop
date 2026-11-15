//! Player information for slot mapping and controller assignment.

use serde::{Deserialize, Serialize};

/// Player session information defining controller assignments and slot properties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PlayerInfo {
    /// Zero-based player slot index.
    pub player_index: usize,
    /// Player display nickname.
    pub nickname: String,
    /// Whether this player is assigned standard paired keyboard and mouse input.
    pub is_keyboard: bool,
    /// Whether this player is assigned an individual ProtoInput/RawInput keyboard.
    #[serde(default)]
    pub is_raw_keyboard: bool,
    /// Hardware GUID for assigned individual ProtoInput keyboard.
    #[serde(default)]
    pub raw_keyboard_guid: Option<String>,
    /// Whether this player is assigned an individual ProtoInput/RawInput mouse.
    #[serde(default)]
    pub is_raw_mouse: bool,
    /// Hardware GUID for assigned individual ProtoInput mouse.
    #[serde(default)]
    pub raw_mouse_guid: Option<String>,
    /// Controller hardware GUID or device ID assigned to this player.
    pub gamepad_guid: Option<String>,
    /// Custom Steam ID or virtual user ID.
    pub steam_id: Option<u64>,
    /// Assigned device index / ID.
    #[serde(default)]
    pub device_id: usize,
}

impl PlayerInfo {
    /// Creates a new `PlayerInfo` with the given index and nickname.
    pub fn new(player_index: usize, nickname: impl Into<String>) -> Self {
        Self {
            player_index,
            nickname: nickname.into(),
            ..Default::default()
        }
    }
}
