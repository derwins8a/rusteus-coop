//! Enums exposed to the JavaScript handler scripts, matching C# originals.

use serde::{Deserialize, Serialize};

/// Type of game save modification or format.
/// Matches C# `SaveType`: None = 0, INI = 1, CFG = 2, SCR = 3, RegKey = 4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(i32)]
pub enum SaveType {
    /// No save modification specified.
    #[default]
    None = 0,
    /// INI configuration file format.
    INI = 1,
    /// CFG configuration file format.
    CFG = 2,
    /// SCR configuration file format.
    SCR = 3,
    /// Windows Registry key save.
    RegKey = 4,
}

impl From<i32> for SaveType {
    fn from(val: i32) -> Self {
        match val {
            1 => SaveType::INI,
            2 => SaveType::CFG,
            3 => SaveType::SCR,
            4 => SaveType::RegKey,
            _ => SaveType::None,
        }
    }
}

/// Search method used when matching strings or patterns.
/// Matches C# `SearchType`: Contains = 0, Full = 1, StartsWith = 2, EndsWith = 3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(i32)]
pub enum SearchType {
    /// Target must contain pattern.
    #[default]
    Contains = 0,
    /// Target must match pattern exactly (C# `SearchType.Full`).
    Full = 1,
    /// Target must start with pattern.
    StartsWith = 2,
    /// Target must end with pattern.
    EndsWith = 3,
}

impl From<i32> for SearchType {
    fn from(val: i32) -> Self {
        match val {
            0 => SearchType::Contains,
            1 => SearchType::Full,
            2 => SearchType::StartsWith,
            3 => SearchType::EndsWith,
            _ => SearchType::Contains,
        }
    }
}

/// Target folder location used in handler operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(i32)]
pub enum Folder {
    /// User Documents directory.
    #[default]
    Documents = 0,
    /// Main root game installation folder.
    MainGameFolder = 1,
    /// Instanced game copy/symlink directory.
    InstancedGameFolder = 2,
}

impl From<i32> for Folder {
    fn from(val: i32) -> Self {
        match val {
            0 => Folder::Documents,
            1 => Folder::MainGameFolder,
            2 => Folder::InstancedGameFolder,
            _ => Folder::Documents,
        }
    }
}

/// DPI scaling handling strategy for launched instances.
/// Matches C# `DPIHandling`: True = 0, Scaled = 1, InvScaled = 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(i32)]
pub enum DPIHandling {
    /// DPI handling is enabled (default).
    #[default]
    True = 0,
    /// Scaled resolution handling.
    Scaled = 1,
    /// Inversely scaled resolution handling.
    InvScaled = 2,
}

impl From<i32> for DPIHandling {
    fn from(val: i32) -> Self {
        match val {
            0 => DPIHandling::True,
            1 => DPIHandling::Scaled,
            2 => DPIHandling::InvScaled,
            _ => DPIHandling::True,
        }
    }
}

/// Type of custom step prompt presented to user prior to game launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(i32)]
pub enum CustomStepType {
    /// Multiple choice option step.
    #[default]
    OptionStep = 0,
    /// Text entry input step.
    TextEntry = 1,
    /// Player number selection step.
    PlayerNumber = 2,
}

/// Windows Registry value data type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(i32)]
pub enum RegType {
    /// String registry value (REG_SZ).
    #[default]
    String = 1,
    /// Expandable string registry value (REG_EXPAND_SZ).
    ExpandString = 2,
    /// Binary registry value (REG_BINARY).
    Binary = 3,
    /// 32-bit integer registry value (REG_DWORD).
    DWord = 4,
    /// Multi-string registry value (REG_MULTI_SZ).
    MultiString = 7,
    /// 64-bit integer registry value (REG_QWORD).
    QWord = 11,
}
