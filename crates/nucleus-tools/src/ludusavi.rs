//! Typed driver and JSON report parsing for the Ludusavi save manager CLI.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::{Deserialize, Serialize};
use nucleus_common::error::{NucleusError, NucleusResult};
use crate::ludusavi_config::LudusaviInstanceTarget;

/// Overall summary statistics in a Ludusavi `--api` JSON report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LudusaviOverallStats {
    pub total_games: usize,
    pub processed_games: usize,
    pub total_bytes: u64,
    pub changed_bytes: u64,
}

/// Information about a single file in a Ludusavi report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LudusaviFileReport {
    pub bytes: u64,
    pub decision: String,
}

/// Game-specific backup/restore report.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LudusaviGameReport {
    #[serde(default)]
    pub files: HashMap<String, LudusaviFileReport>,
}

/// Full machine-readable report returned by Ludusavi `--api` on stdout.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LudusaviApiReport {
    pub overall: LudusaviOverallStats,
    #[serde(default)]
    pub games: HashMap<String, LudusaviGameReport>,
    #[serde(default)]
    pub errors: HashMap<String, serde_json::Value>,
}

/// Match returned by `ludusavi find`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LudusaviGameMatch {
    pub name: String,
}

/// Driver for discovering and invoking the local Ludusavi CLI tool.
#[derive(Debug, Clone)]
pub struct LudusaviDriver {
    exe_path: PathBuf,
}

impl LudusaviDriver {
    /// Discovers `ludusavi.exe` from a custom path, `<utils_path>/Ludusavi/`, or the system PATH.
    pub fn discover(custom_path: Option<&Path>) -> NucleusResult<Self> {
        if let Some(custom) = custom_path {
            if custom.exists() && custom.is_file() {
                return Ok(Self { exe_path: custom.to_path_buf() });
            }
            let direct = custom.join("ludusavi.exe");
            if direct.exists() {
                return Ok(Self { exe_path: direct });
            }
        }

        let utils_exe = if let Some(globals) = nucleus_common::Globals::try_get() {
            globals.utils_path.join("Ludusavi").join("ludusavi.exe")
        } else {
            PathBuf::from("utils").join("Ludusavi").join("ludusavi.exe")
        };

        if utils_exe.exists() {
            return Ok(Self { exe_path: utils_exe });
        }

        // Try PATH
        if let Ok(output) = Command::new("ludusavi").arg("--version").output() {
            if output.status.success() {
                return Ok(Self { exe_path: PathBuf::from("ludusavi") });
            }
        }

        Err(NucleusError::Platform(
            "Ludusavi CLI executable not found. Place ludusavi.exe in utils/Ludusavi/ or add to PATH.".to_string(),
        ))
    }

    /// Creates a driver explicitly wrapping an existing path (e.g. for testing).
    pub fn from_path(exe_path: PathBuf) -> Self {
        Self { exe_path }
    }

    /// Searches the Ludusavi title database/manifest for games matching the hint.
    pub fn find_game(&self, title_hint: &str) -> NucleusResult<Vec<String>> {
        let mut cmd = Command::new(&self.exe_path);
        cmd.arg("find").arg("--api").arg(title_hint);

        let output = cmd.output().map_err(|e| {
            NucleusError::Process(format!("Failed to execute 'ludusavi find': {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NucleusError::Process(format!("Ludusavi find failed: {}", stderr)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Output can be a JSON array of strings or game objects
        if let Ok(titles) = serde_json::from_str::<Vec<String>>(&stdout) {
            return Ok(titles);
        }
        if let Ok(matches) = serde_json::from_str::<Vec<LudusaviGameMatch>>(&stdout) {
            return Ok(matches.into_iter().map(|m| m.name).collect());
        }

        Ok(Vec::new())
    }

    /// Backs up save data for the specified target instance into target.backup_repo_dir.
    pub fn backup_instance(
        &self,
        config_dir: &Path,
        target: &LudusaviInstanceTarget,
        preview: bool,
    ) -> NucleusResult<LudusaviApiReport> {
        let mut cmd = Command::new(&self.exe_path);
        cmd.arg("--config").arg(config_dir);
        cmd.arg("backup");
        cmd.arg("--force");
        cmd.arg("--api");
        cmd.arg("--path").arg(&target.backup_repo_dir);

        if preview {
            cmd.arg("--preview");
        }

        cmd.arg(&target.game_title);

        tracing::info!(
            "LudusaviDriver: Executing backup for '{}' to '{:?}'",
            target.game_title,
            target.backup_repo_dir
        );

        let output = cmd.output().map_err(|e| {
            NucleusError::Process(format!("Failed to execute 'ludusavi backup': {}", e))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NucleusError::Process(format!("Ludusavi backup returned empty stdout: {}", stderr)));
        }

        let report: LudusaviApiReport = serde_json::from_str(&stdout).map_err(|e| {
            NucleusError::Json(e)
        })?;

        if !report.errors.is_empty() {
            tracing::warn!("Ludusavi backup reported non-fatal errors: {:?}", report.errors);
        }

        Ok(report)
    }

    /// Restores save data from target.backup_repo_dir into the target instance paths.
    pub fn restore_instance(
        &self,
        config_dir: &Path,
        target: &LudusaviInstanceTarget,
        preview: bool,
    ) -> NucleusResult<LudusaviApiReport> {
        let mut cmd = Command::new(&self.exe_path);
        cmd.arg("--config").arg(config_dir);
        cmd.arg("restore");
        cmd.arg("--force");
        cmd.arg("--api");
        cmd.arg("--path").arg(&target.backup_repo_dir);

        if preview {
            cmd.arg("--preview");
        }

        cmd.arg(&target.game_title);

        tracing::info!(
            "LudusaviDriver: Executing restore for '{}' from '{:?}'",
            target.game_title,
            target.backup_repo_dir
        );

        let output = cmd.output().map_err(|e| {
            NucleusError::Process(format!("Failed to execute 'ludusavi restore': {}", e))
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.trim().is_empty() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(NucleusError::Process(format!("Ludusavi restore returned empty stdout: {}", stderr)));
        }

        let report: LudusaviApiReport = serde_json::from_str(&stdout).map_err(|e| {
            NucleusError::Json(e)
        })?;

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ludusavi_api_json_report() {
        let json = r#"{
            "overall": {
                "totalGames": 1,
                "processedGames": 1,
                "totalBytes": 14520,
                "changedBytes": 14520
            },
            "games": {
                "My Game": {
                    "files": {
                        "C:/Users/Player1/save.dat": {
                            "bytes": 14520,
                            "decision": "Copied"
                        }
                    }
                }
            },
            "errors": {}
        }"#;

        let report: LudusaviApiReport = serde_json::from_str(json).unwrap();
        assert_eq!(report.overall.total_games, 1);
        assert_eq!(report.overall.total_bytes, 14520);
        let game = report.games.get("My Game").unwrap();
        let file = game.files.get("C:/Users/Player1/save.dat").unwrap();
        assert_eq!(file.bytes, 14520);
        assert_eq!(file.decision, "Copied");
        assert!(report.errors.is_empty());
    }
}
