//! Data models, service enums, and configuration structures for PeerParty multiplayer emulation.

use nucleus_common::dll_injection::{
    DllInjectionConfig, InjectionMethod, InjectionTiming, TargetArchitecture, TargetInstanceFilter,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

/// Supported multiplayer middleware SDKs emulated by PeerParty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PeerPartyService {
    /// Valve Steamworks (ISteamNetworking, ISteamMatchmaking, P2P, auth tickets).
    Steamworks,
    /// Epic Online Services (EOS Connect/Auth, Lobbies, Sessions, P2P, RTC voice).
    EpicOnlineServices,
    /// Microsoft PlayFab (Party, Lobby, Matchmaking, C SDK, Web REST API).
    PlayFab,
    /// GOG Galaxy (SDK 1.57+ vtables, lobbies, user, friends, cloud saves).
    GogGalaxy,
    /// Exit Games Photon (Realtime, PUN, Bolt, Fusion over UDP, TCP, WebSocket).
    Photon,
    /// Unity Gaming Services (Lobby with Wire events, Relay, Cloud Save, Vivox voice).
    UnityGamingServices,
    /// Activision Demonware (Lobbies, matchmaking, server time, address checks).
    Demonware,
    /// Microsoft Xbox Live SDK and Xbox App wrappers.
    XboxLive,
}

impl PeerPartyService {
    /// Canonical lowercase identifier for environment variables and CLI parameters.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Steamworks => "steam",
            Self::EpicOnlineServices => "eos",
            Self::PlayFab => "playfab",
            Self::GogGalaxy => "galaxy",
            Self::Photon => "photon",
            Self::UnityGamingServices => "ugs",
            Self::Demonware => "demonware",
            Self::XboxLive => "xboxlive",
        }
    }
}

impl fmt::Display for PeerPartyService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for PeerPartyService {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "steam" | "steamworks" => Ok(Self::Steamworks),
            "eos" | "epic" | "epiconlineservices" => Ok(Self::EpicOnlineServices),
            "playfab" | "party" => Ok(Self::PlayFab),
            "galaxy" | "gog" | "goggalaxy" => Ok(Self::GogGalaxy),
            "photon" | "pun" | "bolt" | "fusion" => Ok(Self::Photon),
            "ugs" | "unity" | "unitygamingservices" => Ok(Self::UnityGamingServices),
            "demonware" | "dw" | "cod" => Ok(Self::Demonware),
            "xbox" | "xboxlive" => Ok(Self::XboxLive),
            other => Err(format!("Unknown PeerParty service: {}", other)),
        }
    }
}

/// Execution and injection mode for PeerParty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PeerPartyLaunchMode {
    /// Injected hook DLL via nucleus-hook (preferred for maximum control).
    InjectedHook,
    /// Standalone peerparty.exe launcher wrapper.
    LauncherWrapper,
}

impl Default for PeerPartyLaunchMode {
    fn default() -> Self {
        Self::InjectedHook
    }
}

/// Comprehensive configuration specification for a PeerParty multi-instance session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerPartyConfig {
    /// Whether PeerParty emulation is active.
    pub enabled: bool,
    /// List of middleware services emulated by PeerParty.
    pub services: Vec<PeerPartyService>,
    /// Optional private BitTorrent DHT room code for internet multiplayer.
    pub room_code: Option<String>,
    /// Whether to enable public session discovery on the BitTorrent DHT.
    pub public_sessions: bool,
    /// Whether multi-instance mode is enabled (Unity single-instance mutex renames).
    pub multi_instance: bool,
    /// Whether to copy first player's saves to subsequent instances.
    pub copy_saves: bool,
    /// Whether in-game Opus voice chat is enabled.
    pub voice_enabled: bool,
    /// Whether in-game DXGI/D3D overlay is enabled.
    pub overlay_enabled: bool,
    /// The launch and injection strategy.
    pub launch_mode: PeerPartyLaunchMode,
}

impl Default for PeerPartyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            services: vec![
                PeerPartyService::Steamworks,
                PeerPartyService::EpicOnlineServices,
                PeerPartyService::Photon,
                PeerPartyService::UnityGamingServices,
            ],
            room_code: None,
            public_sessions: false,
            multi_instance: true,
            copy_saves: false,
            voice_enabled: false,
            overlay_enabled: false,
            launch_mode: PeerPartyLaunchMode::InjectedHook,
        }
    }
}

impl PeerPartyConfig {
    /// Creates a new active PeerParty configuration with default settings.
    pub fn new() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Adds a service to the emulated services list if not already present.
    pub fn with_service(mut self, service: PeerPartyService) -> Self {
        if !self.services.contains(&service) {
            self.services.push(service);
        }
        self
    }

    /// Sets the private room code for internet DHT routing.
    pub fn with_room_code(mut self, code: impl Into<String>) -> Self {
        let trimmed = code.into().trim().to_string();
        if trimmed.is_empty() {
            self.room_code = None;
        } else {
            self.room_code = Some(trimmed);
        }
        self
    }

    /// Toggles public session discovery.
    pub fn with_public_sessions(mut self, public: bool) -> Self {
        self.public_sessions = public;
        self
    }

    /// Toggles multi-instance support.
    pub fn with_multi_instance(mut self, multi: bool) -> Self {
        self.multi_instance = multi;
        self
    }

    /// Toggles per-player save copying.
    pub fn with_copy_saves(mut self, copy: bool) -> Self {
        self.copy_saves = copy;
        self
    }

    /// Returns `true` if PeerParty is configured to emulate the specified service.
    pub fn handles_service(&self, service: PeerPartyService) -> bool {
        self.enabled && self.services.contains(&service)
    }

    /// Formats the active services as a comma-separated string suitable for `PEERPARTY_SERVICES`.
    pub fn to_services_env(&self) -> String {
        self.services
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Converts this configuration into a standard `DllInjectionConfig` for `peerparty_hook.dll`.
    pub fn to_dll_injection_config(&self, dll_path: PathBuf) -> DllInjectionConfig {
        let mut config = DllInjectionConfig::new(dll_path.to_string_lossy().to_string());
        config.timing = InjectionTiming::AtStartupSuspended;
        config.method = InjectionMethod::QueueUserApc;
        config.architecture = TargetArchitecture::Auto;
        config.target_instances = TargetInstanceFilter::All;
        config.eject_on_close = false;
        config.required = true;
        config
    }
}
