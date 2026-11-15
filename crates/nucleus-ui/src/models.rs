//! Data models and representations for the graphical user interface.

pub use crate::window::ActiveTab;

/// Progress message sent from asynchronous handler download/installation to the UI.
#[derive(Debug, Clone)]
pub enum HubInstallProgressMsg {
    /// Progress update with normalized ratio from 0.0 to 1.0.
    Progress(f32),
    /// Installation completed with optional installed HandlerItem.
    Done(Option<HandlerItem>),
}

/// Metadata representation of an installed or discovered game handler for UI display.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandlerItem {
    /// Handler unique identifier.
    pub guid: String,
    /// Game title.
    pub game_name: String,
    /// Specific handler title or maintainer note.
    pub handler_title: String,
    /// Game executable filename.
    pub executable_name: String,
    /// Maximum number of players supported.
    pub max_players: i32,
    /// Full instructions, setup notes, and handler description.
    pub description: String,
    /// Optional IGDB cover art URL or local image path.
    pub cover_image_url: Option<String>,
    /// Categorized tags for filtering (e.g. Controller support, Shooter).
    pub tags: Vec<String>,
    /// Whether the game executable is located and verified on disk.
    pub is_installed: bool,
    /// Absolute path to the located game executable on disk.
    pub executable_path: Option<String>,
    /// Total play time accumulated in seconds.
    pub play_time_seconds: u64,
    /// Last played UNIX timestamp in seconds.
    pub last_played_timestamp: Option<u64>,
    /// Whether user marked this game as a favorite.
    pub is_favorite: bool,
    /// Whether this handler supports gamepads.
    pub supports_gamepad: bool,
    /// Whether this handler supports XInput controllers.
    pub supports_xinput: bool,
    /// Whether this handler supports DirectInput controllers.
    pub supports_dinput: bool,
    /// Whether this handler supports RawInput (ProtoInput / multiple keyboards and mice).
    pub supports_raw_input: bool,
    /// Whether this handler supports single keyboard/mouse.
    pub supports_keyboard: bool,
    /// Whether this handler supports multiple keyboards and mice (e.g. ProtoInput).
    pub supports_multiple_keyboards_and_mice: bool,
    /// Custom questions / options defined by Game.AddOption.
    pub options: Vec<nucleus_script::HandlerOption>,
    /// Profile settings: assign gamepads by device index.
    pub assign_gamepads_by_index: bool,
    /// Profile settings: assign gamepads on button press.
    pub assign_gamepads_on_button_press: bool,
    /// Profile settings: keep instance content folder after exit.
    pub keep_instances_content_folder: bool,
    /// Profile settings: disable custom profile.
    pub disable_profile: bool,
    /// Profile settings: disable handler updates.
    pub disable_handler_update: bool,
    /// Custom game icon path if overridden.
    pub custom_game_icon: Option<String>,
    /// Steam language override.
    pub steam_language: Option<String>,
    /// Path to handler script file or folder.
    pub handler_path: Option<String>,
    /// Formatted date string of last played session from metaInfo.json.
    pub last_played_str: Option<String>,
    /// Whether handler is officially verified by the SplitscreenMe team.
    pub is_verified: bool,
}

impl HandlerItem {
    /// Creates a new `HandlerItem`.
    pub fn new(
        guid: impl Into<String>,
        game_name: impl Into<String>,
        executable_name: impl Into<String>,
        max_players: i32,
        description: impl Into<String>,
    ) -> Self {
        let name_str = game_name.into();
        Self {
            guid: guid.into(),
            handler_title: format!("{} Community Handler", name_str),
            game_name: name_str,
            executable_name: executable_name.into(),
            max_players,
            description: description.into(),
            cover_image_url: None,
            tags: vec![
                "Controller support".to_string(),
                "Keyboard/mouse support".to_string(),
            ],
            is_installed: false,
            executable_path: None,
            play_time_seconds: 0,
            last_played_timestamp: None,
            is_favorite: false,
            supports_gamepad: true,
            supports_xinput: true,
            supports_dinput: false,
            supports_raw_input: false,
            supports_keyboard: false,
            supports_multiple_keyboards_and_mice: false,
            options: Vec::new(),
            assign_gamepads_by_index: false,
            assign_gamepads_on_button_press: false,
            keep_instances_content_folder: false,
            disable_profile: false,
            disable_handler_update: false,
            custom_game_icon: None,
            steam_language: None,
            handler_path: None,
            last_played_str: None,
            is_verified: true,
        }
    }

    /// Builder to set handler verification status.
    pub fn with_verified(mut self, verified: bool) -> Self {
        self.is_verified = verified;
        self
    }

    /// Builder to set last played string from metaInfo.json.
    pub fn with_last_played_str(mut self, date_str: Option<String>) -> Self {
        self.last_played_str = date_str;
        self
    }

    /// Builder to set handler subtitle.
    pub fn with_handler_title(mut self, title: impl Into<String>) -> Self {
        self.handler_title = title.into();
        self
    }

    /// Builder to set cover image URL.
    pub fn with_cover(mut self, url: Option<String>) -> Self {
        self.cover_image_url = url;
        self
    }

    /// Builder to set tags.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Builder to set installed state.
    pub fn with_installed(mut self, installed: bool) -> Self {
        self.is_installed = installed;
        self
    }

    /// Builder to set executable path.
    pub fn with_executable_path(mut self, path: Option<String>) -> Self {
        self.executable_path = path;
        self
    }

    /// Builder to set play time.
    pub fn with_play_time(mut self, seconds: u64) -> Self {
        self.play_time_seconds = seconds;
        self
    }

    /// Builder to set last played timestamp.
    pub fn with_last_played(mut self, timestamp: Option<u64>) -> Self {
        self.last_played_timestamp = timestamp;
        self
    }

    /// Builder to set favorite state.
    pub fn with_favorite(mut self, favorite: bool) -> Self {
        self.is_favorite = favorite;
        self
    }

    /// Builder to set supported input types.
    pub fn with_input_support(mut self, gamepad: bool, keyboard: bool, multi_km: bool) -> Self {
        self.supports_gamepad = gamepad;
        self.supports_xinput = gamepad;
        self.supports_dinput = false;
        self.supports_raw_input = multi_km;
        self.supports_keyboard = keyboard;
        self.supports_multiple_keyboards_and_mice = multi_km;
        self
    }

    /// Builder to set exact supported input types (XInput, DirectInput, RawInput/ProtoInput, Keyboard/Mouse).
    pub fn with_exact_input_support(
        mut self,
        xinput: bool,
        dinput: bool,
        raw_input: bool,
        keyboard: bool,
    ) -> Self {
        self.supports_xinput = xinput;
        self.supports_dinput = dinput;
        self.supports_raw_input = raw_input;
        self.supports_gamepad = xinput || dinput;
        self.supports_keyboard = keyboard;
        self.supports_multiple_keyboards_and_mice = raw_input;
        self
    }

    /// Builder to set custom handler questions / options.
    pub fn with_options(mut self, options: Vec<nucleus_script::HandlerOption>) -> Self {
        self.options = options;
        self
    }

    /// Builder to set handler path.
    pub fn with_handler_path(mut self, path: Option<String>) -> Self {
        self.handler_path = path;
        self
    }

    /// Returns human-readable accumulated play time (e.g. "1h 24m", "45m", "0m").
    pub fn formatted_play_time(&self) -> String {
        if self.play_time_seconds == 0 {
            "0m".to_string()
        } else if self.play_time_seconds < 3600 {
            let mins = self.play_time_seconds / 60;
            let secs = self.play_time_seconds % 60;
            if mins == 0 {
                format!("{}s", secs)
            } else {
                format!("{}m {:02}s", mins, secs)
            }
        } else {
            let hours = self.play_time_seconds / 3600;
            let mins = (self.play_time_seconds % 3600) / 60;
            format!("{}h {:02}m", hours, mins)
        }
    }

    /// Returns human-readable last played string (e.g. "9/20/2025", "Today", "2 days ago", "Never").
    pub fn formatted_last_played(&self) -> String {
        if let Some(ref s) = self.last_played_str {
            let trimmed = s.trim();
            if !trimmed.is_empty() && trimmed != "..." {
                return trimmed
                    .split_whitespace()
                    .next()
                    .unwrap_or(trimmed)
                    .to_string();
            }
        }

        let ts = match self.last_played_timestamp {
            Some(t) if t > 0 => t,
            _ => return "Never".to_string(),
        };

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if ts > now {
            return "Just now".to_string();
        }

        let diff = now - ts;
        if diff < 120 {
            "Just now".to_string()
        } else if diff < 3600 {
            format!("{}m ago", diff / 60)
        } else if diff < 86400 {
            format!("{}h ago", diff / 3600)
        } else if diff < 172800 {
            "Yesterday".to_string()
        } else {
            let days = diff / 86400;
            if days < 30 {
                format!("{}d ago", days)
            } else {
                format!("{}mo ago", days / 30)
            }
        }
    }
}

/// Sorting criteria for game handlers in the Library view.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibrarySortOption {
    #[default]
    Alphabetical,
    LastPlayed,
    PlayTime,
    MaxPlayers,
    DateAdded,
}

impl LibrarySortOption {
    /// Returns the UI display label for this sorting option.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Alphabetical => "Alphabetical",
            Self::LastPlayed => "Last Played",
            Self::PlayTime => "Play Time",
            Self::MaxPlayers => "Max Players",
            Self::DateAdded => "Date Added",
        }
    }

    /// Returns the icon associated with this sorting option.
    pub fn icon(&self) -> gpui_kit_assets::IconName {
        match self {
            Self::Alphabetical => gpui_kit_assets::IconName::ArrowDownAZ,
            Self::LastPlayed => gpui_kit_assets::IconName::Clock,
            Self::PlayTime => gpui_kit_assets::IconName::Timer,
            Self::MaxPlayers => gpui_kit_assets::IconName::Users,
            Self::DateAdded => gpui_kit_assets::IconName::Calendar,
        }
    }
}

/// Sort direction (ascending or descending).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibrarySortDirection {
    #[default]
    Ascending,
    Descending,
}

impl LibrarySortDirection {
    /// Returns display icon / text for direction toggle button.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ascending => "Asc",
            Self::Descending => "Desc",
        }
    }

    /// Returns the direction icon associated with this sort direction.
    pub fn icon(&self) -> gpui_kit_assets::IconName {
        match self {
            Self::Ascending => gpui_kit_assets::IconName::ArrowUp,
            Self::Descending => gpui_kit_assets::IconName::ArrowDown,
        }
    }

    /// Toggles the direction.
    pub fn toggle(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

/// Representation of a physical controller or input device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceItem {
    /// Device identifier or port index.
    pub id: usize,
    /// Friendly name reported by system or driver.
    pub name: String,
    /// Hardware GUID string.
    pub guid: String,
    /// Whether device is currently active/connected.
    pub is_connected: bool,
    /// Whether this is a virtual/mock device.
    pub is_virtual: bool,
    /// Device category (Gamepad, Keyboard, Mouse).
    pub device_type: nucleus_common::types::DeviceType,
}

impl DeviceItem {
    /// Creates a new physical `DeviceItem`.
    pub fn new(
        id: usize,
        name: impl Into<String>,
        guid: impl Into<String>,
        is_connected: bool,
    ) -> Self {
        let name_str = name.into();
        let guid_str = guid.into();
        let dev_type = if name_str.to_lowercase().contains("mouse")
            || guid_str.to_lowercase().contains("mouse")
        {
            nucleus_common::types::DeviceType::Mouse
        } else if name_str.to_lowercase().contains("keyboard")
            || guid_str.to_lowercase().contains("keyboard")
        {
            nucleus_common::types::DeviceType::Keyboard
        } else {
            nucleus_common::types::DeviceType::XInputGamepad
        };
        Self {
            id,
            name: name_str,
            guid: guid_str,
            is_connected,
            is_virtual: false,
            device_type: dev_type,
        }
    }

    /// Creates a new virtual/mock `DeviceItem`.
    pub fn new_virtual(id: usize, name: impl Into<String>, guid: impl Into<String>) -> Self {
        let name_str = name.into();
        let guid_str = guid.into();
        let dev_type = if name_str.to_lowercase().contains("mouse")
            || guid_str.to_lowercase().contains("mouse")
        {
            nucleus_common::types::DeviceType::Mouse
        } else if name_str.to_lowercase().contains("keyboard")
            || guid_str.to_lowercase().contains("keyboard")
        {
            nucleus_common::types::DeviceType::Keyboard
        } else {
            nucleus_common::types::DeviceType::XInputGamepad
        };
        Self {
            id,
            name: name_str,
            guid: guid_str,
            is_connected: true,
            is_virtual: true,
            device_type: dev_type,
        }
    }

    /// Creates a new `DeviceItem` with an explicit `DeviceType`.
    pub fn new_with_type(
        id: usize,
        name: impl Into<String>,
        guid: impl Into<String>,
        is_connected: bool,
        device_type: nucleus_common::types::DeviceType,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            guid: guid.into(),
            is_connected,
            is_virtual: false,
            device_type,
        }
    }
}

/// A community handler package listed in the Hub browser.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HubPackageItem {
    /// Package ID in SplitscreenMe hub.
    pub id: String,
    /// Handler package title.
    pub title: String,
    /// Official game title.
    pub game_name: String,
    /// Package version string.
    pub version: String,
    /// Script author username.
    pub author: String,
    /// Detailed description and notes (Markdown).
    pub description: String,
    /// Player capacity label (e.g. "4 Players").
    pub max_players: String,
    /// Numeric max players count.
    pub max_players_count: i32,
    /// Numeric remote package version number.
    pub version_number: i32,
    /// Numeric local installed version number, if installed.
    pub installed_version: Option<i32>,
    /// Whether an update is available (remote version > installed version).
    pub has_update: bool,
    /// Whether the package is installed and matches or exceeds the remote version.
    pub is_up_to_date: bool,
    /// Whether this package is already downloaded locally.
    pub is_installed: bool,
    /// IGDB cover image URL.
    pub cover_image_url: Option<String>,
    /// Number of likes / stars (hotness).
    pub likes: u32,
    /// Total download count.
    pub downloads: u64,
    /// Total comments count.
    pub comments_count: u32,
    /// Categorized tags (controllers, genres, player count).
    pub tags: Vec<String>,
    /// Creation timestamp string.
    pub created_at: String,
    /// Last updated timestamp string.
    pub updated_at: String,
    /// Whether this package is officially verified by the SplitscreenMe team.
    pub is_verified: bool,
    /// Whether handler supports gamepads / controllers.
    pub playable_controllers: bool,
    /// Whether handler supports keyboard and mouse.
    pub playable_mouse_keyboard: bool,
    /// Whether handler supports multiple keyboards and mice.
    pub playable_multi_mouse_keyboard: bool,
}

impl HubPackageItem {
    /// Creates a basic `HubPackageItem`.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        version: impl Into<String>,
        author: impl Into<String>,
        description: impl Into<String>,
        max_players: impl Into<String>,
        is_installed: bool,
    ) -> Self {
        let t_str = title.into();
        Self {
            id: id.into(),
            game_name: t_str.clone(),
            title: t_str,
            version: version.into(),
            author: author.into(),
            description: description.into(),
            max_players: max_players.into(),
            max_players_count: 4,
            version_number: 1,
            installed_version: if is_installed { Some(1) } else { None },
            has_update: false,
            is_up_to_date: is_installed,
            is_installed,
            cover_image_url: None,
            likes: 0,
            downloads: 0,
            comments_count: 0,
            tags: Vec::new(),
            created_at: String::new(),
            updated_at: String::new(),
            is_verified: true,
            playable_controllers: true,
            playable_mouse_keyboard: false,
            playable_multi_mouse_keyboard: false,
        }
    }

    /// Builder to set package verification status.
    pub fn with_verification(mut self, verified: bool) -> Self {
        self.is_verified = verified;
        self
    }

    /// Builder to configure input device capabilities.
    pub fn with_inputs(mut self, controllers: bool, mouse_kb: bool, multi_mouse_kb: bool) -> Self {
        self.playable_controllers = controllers;
        self.playable_mouse_keyboard = mouse_kb;
        self.playable_multi_mouse_keyboard = multi_mouse_kb;
        self
    }

    /// Builder to configure version tracking and update availability.
    pub fn with_version_info(
        mut self,
        version_number: i32,
        installed_version: Option<i32>,
    ) -> Self {
        self.version_number = version_number;
        self.installed_version = installed_version;
        if let Some(local_ver) = installed_version {
            self.is_installed = true;
            if version_number > local_ver {
                self.has_update = true;
                self.is_up_to_date = false;
            } else {
                self.has_update = false;
                self.is_up_to_date = true;
            }
        } else {
            self.has_update = false;
            self.is_up_to_date = false;
        }
        self
    }

    /// Builder to set rich metadata from Splitscreen.Me hub response.
    pub fn with_meta(
        mut self,
        game_name: impl Into<String>,
        cover_image_url: Option<String>,
        likes: u32,
        downloads: u64,
        comments_count: u32,
        tags: Vec<String>,
        created_at: impl Into<String>,
        updated_at: impl Into<String>,
    ) -> Self {
        self.game_name = game_name.into();
        self.cover_image_url = cover_image_url;
        self.likes = likes;
        self.downloads = downloads;
        self.comments_count = comments_count;
        self.tags = tags;
        self.created_at = created_at.into();
        self.updated_at = updated_at.into();
        self
    }
}

/// Target action for a modal confirmation dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfirmAction {
    /// Reset all settings to defaults.
    ResetDefaults,
    /// Delete a saved numbered profile.
    DeleteNumberedProfile(String, usize),
    /// Remove game from library list.
    RemoveGame(String),
    /// Delete user profile save path on disk.
    DeleteSavePath(String),
}

/// State representation for an active user confirmation dialog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmDialogState {
    /// Modal dialog header title.
    pub title: String,
    /// Detailed warning or explanation message.
    pub message: String,
    /// Confirmation button label (e.g. "Reset", "Delete").
    pub confirm_label: String,
    /// Whether this action is destructive (renders red button).
    pub is_destructive: bool,
    /// Action to execute on confirm.
    pub action: ConfirmAction,
}

/// Interactive launch prompt request dispatched from engine staging to UI.
#[derive(Debug)]
pub struct LaunchPromptRequest {
    /// 0-based player index requiring action.
    pub player_index: usize,
    /// Total player count for this session.
    pub player_count: usize,
    /// Handler instruction message text.
    pub message: String,
    /// Optional auto-proceed countdown in seconds.
    pub auto_proceed_seconds: Option<u32>,
    /// One-shot response channel: true = proceed, false = cancel launch.
    pub response_tx: tokio::sync::oneshot::Sender<bool>,
}
