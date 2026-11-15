//! Generator for isolated, ephemeral Ludusavi `config.yaml` files.

use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use nucleus_common::error::NucleusResult;

/// A path redirection mapping for Ludusavi backup/restore cycles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LudusaviRedirect {
    pub source: PathBuf,
    pub target: PathBuf,
}

/// Custom game specification for games without standard PCGamingWiki manifest definitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LudusaviCustomGame {
    pub name: String,
    #[serde(default)]
    pub files: Vec<String>,
    #[serde(default)]
    pub registry: Vec<String>,
}

/// Specification for an isolated Ludusavi instance backup/restore target.
#[derive(Debug, Clone)]
pub struct LudusaviInstanceTarget {
    pub game_title: String,
    pub backup_repo_dir: PathBuf,
    pub redirects: Vec<LudusaviRedirect>,
    pub custom_files: Vec<String>,
    pub custom_registry: Vec<String>,
    pub full_backup_limit: u32,
}

/// Structure representing a complete isolated `config.yaml` for Ludusavi.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct LudusaviConfigYaml {
    #[serde(skip_serializing_if = "Option::is_none")]
    backup: Option<PathConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    restore: Option<PathConfig>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    redirects: Vec<RedirectYaml>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    custom_games: Vec<CustomGameYaml>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PathConfig {
    path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RedirectYaml {
    source: String,
    target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CustomGameYaml {
    name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    files: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    registry: Vec<String>,
}

/// Writes an isolated `config.yaml` to the specified directory.
pub fn write_isolated_config(
    config_dir: &Path,
    target: &LudusaviInstanceTarget,
) -> NucleusResult<PathBuf> {
    let _ = fs::create_dir_all(config_dir);
    let config_file = config_dir.join("config.yaml");

    let repo_path = target.backup_repo_dir.to_string_lossy().replace('\\', "/");

    let mut redirects_yaml = Vec::new();
    for r in &target.redirects {
        redirects_yaml.push(RedirectYaml {
            source: r.source.to_string_lossy().replace('\\', "/"),
            target: r.target.to_string_lossy().replace('\\', "/"),
        });
    }

    let mut custom_games_yaml = Vec::new();
    if !target.custom_files.is_empty() || !target.custom_registry.is_empty() {
        custom_games_yaml.push(CustomGameYaml {
            name: target.game_title.clone(),
            files: target.custom_files.iter().map(|f| f.replace('\\', "/")).collect(),
            registry: target.custom_registry.clone(),
        });
    }

    let config = LudusaviConfigYaml {
        backup: Some(PathConfig { path: repo_path.clone() }),
        restore: Some(PathConfig { path: repo_path }),
        redirects: redirects_yaml,
        custom_games: custom_games_yaml,
    };

    let yaml_str = serde_yaml::to_string(&config).map_err(|e| {
        nucleus_common::error::NucleusError::Other(format!("Failed to serialize Ludusavi config: {}", e))
    })?;

    fs::write(&config_file, yaml_str)?;
    Ok(config_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_isolated_config_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_lud_cfg_test_{}", std::process::id()));
        let _ = fs::create_dir_all(&temp_dir);

        let target = LudusaviInstanceTarget {
            game_title: "Halo Reach".to_string(),
            backup_repo_dir: PathBuf::from("C:\\Saves\\Halo"),
            redirects: vec![LudusaviRedirect {
                source: PathBuf::from("C:\\Users\\Main\\AppData\\Roaming\\Game"),
                target: PathBuf::from("C:\\Users\\Main\\NucleusCoop\\Player1\\AppData\\Roaming\\Game"),
            }],
            custom_files: vec!["C:/Games/Halo/Instance1/save.bin".to_string()],
            custom_registry: vec!["HKEY_CURRENT_USER\\Software\\Game".to_string()],
            full_backup_limit: 5,
        };

        let cfg_path = write_isolated_config(&temp_dir, &target).unwrap();
        assert!(cfg_path.exists());

        let content = fs::read_to_string(&cfg_path).unwrap();
        assert!(content.contains("C:/Saves/Halo"));
        assert!(content.contains("Player1"));
        assert!(content.contains("Halo Reach"));

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
