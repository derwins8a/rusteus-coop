//! Data models, enums, and configuration structures for arbitrary DLL injection
//! and inter-process interaction across multi-instance game sessions.

use serde::{Deserialize, Serialize};

/// Specifies the precise lifecycle trigger point when a DLL should be injected into a target instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InjectionTiming {
    /// Injected immediately upon suspended process creation, before any game code or CRT runs.
    AtStartupSuspended,
    /// Injected immediately after the process is spawned and running.
    PostStartup,
    /// Injected once the primary viewport HWND has been discovered and initial styles/bounds applied.
    OnWindowCreated,
    /// Injected after a specific delay (in milliseconds) following window discovery.
    AfterDelayMs(u64),
    /// Injected only when the handler script explicitly triggers injection via `Context.InjectDll()`.
    Manual,
    /// Injected when input lock (VK_END / ProtoInput) is toggled.
    OnInputLock,
}

impl Default for InjectionTiming {
    fn default() -> Self {
        Self::OnWindowCreated
    }
}

/// The underlying mechanism used to inject the dynamic library into the target address space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InjectionMethod {
    /// Standard Win32 `CreateRemoteThread` invoking `LoadLibraryW`.
    CreateRemoteThread,
    /// Early thread execution via `QueueUserAPC` (ideal for suspended processes).
    QueueUserApc,
    /// Injected via ProtoInput EasyHook / RemoteLoad runtime.
    EasyHook,
    /// Manual memory mapping / reflective injection avoiding on-disk module registration.
    ManualMap,
}

impl Default for InjectionMethod {
    fn default() -> Self {
        Self::CreateRemoteThread
    }
}

/// Filter determining which player instances receive the injected DLL.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetInstanceFilter {
    /// Injected into all player instances in the session.
    All,
    /// Injected into Player 0 (the first instance) only.
    First,
    /// Injected into all instances except Player 0.
    ExceptFirst,
    /// Injected into specific zero-based player slot indices.
    Specific(Vec<usize>),
}

impl Default for TargetInstanceFilter {
    fn default() -> Self {
        Self::All
    }
}

impl TargetInstanceFilter {
    /// Returns `true` if this filter applies to the specified player slot index.
    pub fn applies_to(&self, player_index: usize, total_instances: usize) -> bool {
        let _ = total_instances;
        match self {
            Self::All => true,
            Self::First => player_index == 0,
            Self::ExceptFirst => player_index > 0,
            Self::Specific(indices) => indices.contains(&player_index),
        }
    }
}

/// Target process architecture expected for the DLL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TargetArchitecture {
    /// Automatically detected by pre-inspecting the target process bitness and PE headers.
    Auto,
    /// 64-bit AMD64 architecture.
    X64,
    /// 32-bit i386 / WOW64 architecture.
    X86,
}

impl Default for TargetArchitecture {
    fn default() -> Self {
        Self::Auto
    }
}

/// Real-time Inter-Process Communication channel to establish with the injected DLL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IpcChannelKind {
    /// No IPC channel is established.
    None,
    /// Windows Named Pipe (`\\.\pipe\...`) with length-prefixed JSON framing.
    NamedPipe,
    /// Memory-mapped file (`Local\...`) with Win32 event synchronization.
    SharedMemory,
    /// Win32 window message communication (`WM_COPYDATA`).
    WindowMessage,
}

impl Default for IpcChannelKind {
    fn default() -> Self {
        Self::None
    }
}

/// Comprehensive configuration specification for injecting a dynamic library into a game instance.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DllInjectionConfig {
    /// Path to the DLL file (relative to handler directory or absolute).
    pub path: String,
    /// Which player instances receive this injection.
    #[serde(default)]
    pub target_instances: TargetInstanceFilter,
    /// When this DLL should be injected into the process.
    #[serde(default)]
    pub timing: InjectionTiming,
    /// Optional post-window delay in milliseconds.
    #[serde(default)]
    pub delay_ms: u64,
    /// Expected target binary architecture.
    #[serde(default)]
    pub architecture: TargetArchitecture,
    /// Injection mechanism.
    #[serde(default)]
    pub method: InjectionMethod,
    /// Optional exported function name to execute after `LoadLibraryW` succeeds.
    #[serde(default)]
    pub export_name: Option<String>,
    /// Optional string or JSON payload passed to the exported function.
    #[serde(default)]
    pub export_argument: Option<String>,
    /// Whether failure to inject this DLL halts the entire game launch.
    #[serde(default)]
    pub required: bool,
    /// IPC mechanism to establish with the injected DLL.
    #[serde(default)]
    pub ipc_channel: IpcChannelKind,
    /// Custom named pipe or shared memory name format (e.g. `\\.\pipe\Nucleus_{GameGuid}_{PlayerIndex}`).
    #[serde(default)]
    pub pipe_name: Option<String>,
    /// Whether to explicitly call `FreeLibrary` on session teardown.
    #[serde(default)]
    pub eject_on_close: bool,
}

impl DllInjectionConfig {
    /// Creates a default injection configuration for the given DLL path.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            target_instances: TargetInstanceFilter::All,
            timing: InjectionTiming::OnWindowCreated,
            delay_ms: 0,
            architecture: TargetArchitecture::Auto,
            method: InjectionMethod::CreateRemoteThread,
            export_name: None,
            export_argument: None,
            required: false,
            ipc_channel: IpcChannelKind::None,
            pipe_name: None,
            eject_on_close: false,
        }
    }
}
