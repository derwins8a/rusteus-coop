//! Compatibility bridge and parser for legacy C# Nucleus Co-op `pluginsInfo.txt`.

use nucleus_common::error::NucleusResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Represents an entry parsed from legacy `pluginsInfo.txt`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyPluginInfo {
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub binary_path: String,
    pub extra_info: String,
}

/// Known built-in tool equivalents implemented natively in safe Rust.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuiltinToolEquivalent {
    GoldbergEmulator,
    NemirtingasEpic,
    NemirtingasGalaxy,
    DevReorder,
    X360ce,
    ProtoInput,
    AutoHotkey,
    Ludusavi,
}

impl LegacyPluginInfo {
    /// Checks if this legacy entry maps to a high-performance native Rust tool in `nucleus-tools`.
    pub fn builtin_rust_equivalent(&self) -> Option<BuiltinToolEquivalent> {
        let n = self.name.to_lowercase();
        let b = self.binary_path.to_lowercase();
        if n.contains("goldberg") || b.contains("goldberg") {
            Some(BuiltinToolEquivalent::GoldbergEmulator)
        } else if n.contains("nemirtingas epic") || b.contains("epic") {
            Some(BuiltinToolEquivalent::NemirtingasEpic)
        } else if n.contains("nemirtingas galaxy") || b.contains("galaxy") {
            Some(BuiltinToolEquivalent::NemirtingasGalaxy)
        } else if n.contains("devreorder") || b.contains("devreorder") {
            Some(BuiltinToolEquivalent::DevReorder)
        } else if n.contains("x360ce") || b.contains("x360ce") {
            Some(BuiltinToolEquivalent::X360ce)
        } else if n.contains("protoinput") || b.contains("protoinput") {
            Some(BuiltinToolEquivalent::ProtoInput)
        } else if n.contains("autohotkey") || b.contains("autohotkey") {
            Some(BuiltinToolEquivalent::AutoHotkey)
        } else if n.contains("ludusavi") || b.contains("ludusavi") {
            Some(BuiltinToolEquivalent::Ludusavi)
        } else {
            None
        }
    }
}

/// Parses a legacy `pluginsInfo.txt` file.
pub fn parse_legacy_plugins_txt(content: &str) -> Vec<LegacyPluginInfo> {
    let mut results = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with(';') {
            continue;
        }

        let parts: Vec<&str> = trimmed.split('#').map(|s| s.trim()).collect();
        if parts.len() >= 5 {
            results.push(LegacyPluginInfo {
                name: parts[0].to_string(),
                author: parts[1].to_string(),
                version: parts[2].to_string(),
                description: parts[3].to_string(),
                binary_path: parts[4].to_string(),
                extra_info: parts.get(5).unwrap_or(&"").to_string(),
            });
        }
    }

    results
}

/// Scans a directory for legacy `pluginsInfo.txt`.
pub fn find_and_parse_legacy_plugins(dir: &Path) -> NucleusResult<Vec<LegacyPluginInfo>> {
    let file_path = dir.join("pluginsInfo.txt");
    if !file_path.exists() {
        return Ok(Vec::new());
    }

    let text = fs::read_to_string(&file_path)?;
    Ok(parse_legacy_plugins_txt(&text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_legacy_plugins_txt() {
        let sample = r#"
// Legacy Nucleus pluginsInfo
DevReorder#Briank#1.0.3#Device reordering hook for DInput#devreorder.dll#dinput
Goldberg Emulator#Mr_Goldberg#0.2.5#Steam emulator wrapper#steam_api.dll#steam
CustomTool#Community#2.0.0#Some standalone tool#custom.exe#standalone
"#;
        let parsed = parse_legacy_plugins_txt(sample);
        assert_eq!(parsed.len(), 3);
        assert_eq!(parsed[0].name, "DevReorder");
        assert_eq!(
            parsed[0].builtin_rust_equivalent(),
            Some(BuiltinToolEquivalent::DevReorder)
        );
        assert_eq!(
            parsed[1].builtin_rust_equivalent(),
            Some(BuiltinToolEquivalent::GoldbergEmulator)
        );
        assert_eq!(parsed[2].builtin_rust_equivalent(), None);
    }
}
