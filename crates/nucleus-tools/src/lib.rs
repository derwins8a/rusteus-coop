//! Utility tools, emulation layers, and game manipulation modules for Nucleus Coop.
//!
//! Provides modular tools for binary patching, save/config backup, controller remapping,
//! Steam/Epic/Galaxy emulation, network isolation, window management, and input redirection.

pub mod audio;
pub mod autohotkey;
pub mod backup;
pub mod cursor;
pub mod dev_reorder;
pub mod dinput_blocker;
pub mod dpi_scaling;
pub mod dx9_wrapper;
pub mod eac_bypass;
pub mod elevator;
pub mod epic_emu;
pub mod flawless_widescreen;
pub mod galaxy_emu;
pub mod game_starter;
pub mod hex_edit;
pub mod ludusavi;
pub mod ludusavi_config;
pub mod network;
pub mod peerparty_types;
pub mod sdl2;
pub mod steam_emu;
pub mod users;
pub mod window_methods;
pub mod x360ce;
pub mod xinput_plus;

pub use audio::{AudioRouter, AudioTool};
pub use autohotkey::{AhkLaunchSpec, AhkManager, AhkPreludeVars, AhkSource, AhkTiming, AhkVersion};
pub use backup::BackupTool;
pub use cursor::CursorTool;
pub use dev_reorder::DevReorderTool;
pub use dinput_blocker::DinputBlockerTool;
pub use dpi_scaling::DpiScalingTool;
pub use dx9_wrapper::{Dx9WrapperConfig, Dx9WrapperTool};
pub use eac_bypass::EacBypassTool;
pub use elevator::ElevatorTool;
pub use epic_emu::{EpicEmuConfig, EpicEmuTool, get_epic_language};
pub use flawless_widescreen::FlawlessWidescreenTool;
pub use galaxy_emu::{GalaxyEmuConfig, GalaxyEmuTool};
pub use game_starter::{GameStarterTool, LaunchSpec};
pub use hex_edit::HexEditTool;
pub use ludusavi::{
    LudusaviApiReport, LudusaviDriver, LudusaviFileReport, LudusaviGameReport, LudusaviOverallStats,
};
pub use ludusavi_config::{LudusaviInstanceTarget, LudusaviRedirect, write_isolated_config};
pub use network::{ForceBindIpOptions, NetworkInterfaceInfo, NetworkTool};
pub use peerparty_types::{PeerPartyConfig, PeerPartyLaunchMode, PeerPartyService};
pub use sdl2::Sdl2Tool;
pub use steam_emu::{
    GoldbergDeployOptions, SmartSteamEmuDeployResult, SmartSteamEmuOptions, SteamClientLaunchInfo,
    SteamEmuTool,
};
pub use users::UsersTool;
pub use window_methods::WindowMethodsTool;
pub use x360ce::X360ceTool;
pub use xinput_plus::{CustomDllHookOptions, XInputPlusIniOptions, XInputPlusTool};
