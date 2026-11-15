//! Common shared types used across NucleusCoop crates.

use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// Two-dimensional integer vector representing coordinates, sizes, or offsets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct IntVector2 {
    /// X coordinate or horizontal value.
    pub x: i32,
    /// Y coordinate or vertical value.
    pub y: i32,
}

impl IntVector2 {
    /// Zero vector `(0, 0)`.
    pub const ZERO: Self = Self { x: 0, y: 0 };

    /// Creates a new `IntVector2`.
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Rectangle defined by an origin `(x, y)` and dimensions `(width, height)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Rect {
    /// X coordinate of the top-left corner.
    pub x: i32,
    /// Y coordinate of the top-left corner.
    pub y: i32,
    /// Width of the rectangle.
    pub width: i32,
    /// Height of the rectangle.
    pub height: i32,
}

impl Rect {
    /// Zero-sized rectangle at `(0, 0)`.
    pub const ZERO: Self = Self {
        x: 0,
        y: 0,
        width: 0,
        height: 0,
    };

    /// Creates a new `Rect`.
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Checks if a point is contained within this rectangle.
    pub fn contains_point(&self, point: IntVector2) -> bool {
        point.x >= self.x
            && point.x < self.x + self.width
            && point.y >= self.y
            && point.y < self.y + self.height
    }
}

/// Platform-agnostic window handle.
///
/// Stores `HWND` as `u64` on Windows or `Window` / `XID` / Wayland surface identifier on Linux.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct WindowHandle(pub u64);

impl WindowHandle {
    /// Invalid / null window handle.
    pub const INVALID: Self = Self(0);

    /// Creates a new window handle wrapper.
    pub const fn new(handle: u64) -> Self {
        Self(handle)
    }

    /// Returns `true` if the handle is non-zero.
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

/// Lightweight reference to an operating system process.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessHandle {
    /// Operating system process identifier.
    pub pid: u32,
    /// Process executable name.
    pub name: String,
}

impl ProcessHandle {
    /// Creates a new `ProcessHandle`.
    pub fn new(pid: u32, name: impl Into<String>) -> Self {
        Self {
            pid,
            name: name.into(),
        }
    }
}

/// Configuration used to spawn or configure a game process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessConfig {
    /// Path to the executable.
    pub exe_path: PathBuf,
    /// Command line arguments.
    pub args: Vec<String>,
    /// Working directory for the process.
    pub working_dir: Option<PathBuf>,
    /// Environment variables to set.
    pub environment: HashMap<String, String>,
    /// Whether to request administrator privileges.
    pub run_as_admin: bool,
    /// Specific OS user account to run under, if applicable.
    pub run_as_user: Option<String>,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            exe_path: PathBuf::new(),
            args: Vec::new(),
            working_dir: None,
            environment: HashMap::new(),
            run_as_admin: false,
            run_as_user: None,
        }
    }
}

/// Information about a running process.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessInfo {
    /// Operating system process ID.
    pub pid: u32,
    /// Process name.
    pub name: String,
    /// Full path to the executable file.
    pub exe_path: PathBuf,
}

/// Operating system process scheduling priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ProcessPriority {
    /// Idle / lowest priority.
    Idle,
    /// Below normal priority.
    BelowNormal,
    /// Normal priority.
    #[default]
    Normal,
    /// Above normal priority.
    AboveNormal,
    /// High priority.
    High,
    /// Real-time / critical priority.
    RealTime,
}

/// Information about a connected display / monitor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Display {
    /// Display device index or unique identifier.
    pub id: u32,
    /// Display device name or description.
    pub name: String,
    /// Display bounding rectangle in desktop coordinate space.
    pub bounds: Rect,
    /// Whether this is the primary monitor.
    pub is_primary: bool,
    /// Display scaling factor (e.g. 1.0 = 100%, 1.25 = 125%).
    pub scale_factor: f64,
}

/// Window styling attributes and behavior configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowStyle {
    /// Whether the window has normal borders.
    pub borders: bool,
    /// Whether the title bar caption is shown.
    pub title_bar: bool,
    /// Whether the window should stay topmost.
    pub topmost: bool,
    /// Whether the window can be resized by the user.
    pub resizable: bool,
    /// Whether the minimize box is enabled.
    pub minimize_box: bool,
    /// Whether the maximize box is enabled.
    pub maximize_box: bool,
}

impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            borders: true,
            title_bar: true,
            topmost: false,
            resizable: true,
            minimize_box: true,
            maximize_box: true,
        }
    }
}

/// Information about a local network interface adapter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Adapter name or alias.
    pub name: String,
    /// IPv4 address string.
    pub ip: String,
    /// Subnet mask string.
    pub subnet: String,
    /// Whether this adapter is a virtual or loopback interface.
    pub is_virtual: bool,
}

/// Backup of Windows registry keys or values before modifications.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RegistryBackup {
    /// Root hive name (e.g., "HKEY_CURRENT_USER").
    pub root_key: String,
    /// Registry subkey path.
    pub sub_key: String,
    /// Backed up value entries.
    pub values: Vec<RegistryValueBackup>,
}

/// Backed up individual registry value entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegistryValueBackup {
    /// Value name.
    pub name: String,
    /// Value type (e.g. REG_SZ, REG_DWORD, REG_BINARY).
    pub value_type: String,
    /// Raw binary data of the value.
    pub data: Vec<u8>,
}

/// Search matching mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SearchMode {
    /// Matches if target string starts with pattern.
    #[default]
    StartsWith,
    /// Matches if target string contains pattern.
    Contains,
    /// Matches if target string ends with pattern.
    EndsWith,
    /// Matches if target string exactly equals pattern.
    Exact,
}

/// Input device category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum DeviceType {
    /// Standard keyboard.
    Keyboard,
    /// Mouse pointing device.
    Mouse,
    /// XInput-compatible gamepad (Xbox style).
    #[default]
    XInputGamepad,
    /// DirectInput / generic HID gamepad or joystick.
    DInputGamepad,
    /// Unidentified device.
    Unknown,
}

/// Input device descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputDevice {
    /// Device identifier or handle.
    pub handle: u64,
    /// Friendly device name.
    pub name: String,
    /// Type of input device.
    pub device_type: DeviceType,
}

/// Runtime tracking data for a game instance process.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ProcessData {
    /// Process ID, if running.
    pub pid: Option<u32>,
    /// Main window handle for the process.
    pub window_handle: Option<WindowHandle>,
    /// Target or current window position.
    pub position: IntVector2,
    /// Target or current window size.
    pub size: IntVector2,
    /// Whether the process has finished or exited.
    pub finished: bool,
    /// Whether window handle detection should retry.
    pub hwnd_retry: bool,
    /// Whether window positioning has already been set.
    pub setted: bool,
    /// Whether mutexes have been killed for this instance.
    pub killed_mutexes: bool,
    /// Instance lifecycle or status code.
    pub status: i32,
    /// Audio volume level override (0.0 to 1.0).
    pub audio_volume: Option<f32>,
}

/// Information representing an emulated player/user.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct UserInfo {
    /// Player display nickname.
    pub nickname: String,
    /// Emulated 64-bit Steam ID or account identifier.
    pub steam_id: u64,
    /// Local or virtual IP address assigned to this user instance.
    pub ip_address: Option<String>,
}
