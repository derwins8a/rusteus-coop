//! Disk scanner for discovering game executables and matching them against handlers.

use nucleus_common::error::NucleusResult;
use nucleus_script::HandlerConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Information about a discovered game executable on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoundGame {
    /// Friendly game name or executable base name.
    pub name: String,
    /// Absolute path to the detected game executable.
    pub exe_path: PathBuf,
    /// Optional path to an extracted or cached icon file.
    pub icon_path: Option<PathBuf>,
    /// Unique handler GUID if matched to an installed handler.
    pub handler_guid: Option<String>,
    /// Handler display name if matched.
    pub handler_name: Option<String>,
    /// Confidence score for handler match (0–100).
    pub match_score: u8,
}

/// Scanner responsible for searching drives and directories for supported game binaries.
#[derive(Default)]
pub struct DiskScanner {
    /// Directory paths to exclude during scans.
    excluded_paths: Vec<PathBuf>,
    /// Known handler configs for matching, indexed by GUID.
    handler_catalog: HashMap<String, HandlerConfig>,
}

impl DiskScanner {
    /// Creates a new `DiskScanner` with default exclusion rules.
    pub fn new() -> Self {
        Self {
            excluded_paths: Vec::new(),
            handler_catalog: HashMap::new(),
        }
    }

    /// Adds a directory path to exclude from scanning.
    pub fn add_excluded_path(&mut self, path: impl Into<PathBuf>) {
        self.excluded_paths.push(path.into());
    }

    /// Loads a set of handler configs that the scanner uses for matching.
    pub fn load_handlers(&mut self, handlers: &HashMap<String, HandlerConfig>) {
        self.handler_catalog = handlers.clone();
    }

    /// Attempts to match a discovered executable against known handler configs.
    /// Returns `(guid, handler_name, score)` where score is 0–100.
    fn match_handler(&self, exe_path: &Path) -> Option<(String, String, u8)> {
        let file_name = exe_path
            .file_name()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let file_stem = exe_path
            .file_stem()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let parent_dir = exe_path
            .parent()
            .map(|p| p.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let mut best: Option<(String, String, u8)> = None;

        for (guid, config) in &self.handler_catalog {
            let mut score: u8 = 0;

            // Primary executable name match (highest confidence)
            if !config.executable_name.is_empty()
                && config.executable_name.to_lowercase() == file_name
            {
                score = score.max(90);
            }

            // Alternate executable names
            if let Some(ref alt_names) = config.executable_names {
                for alt in alt_names {
                    if alt.to_lowercase() == file_name {
                        score = score.max(85);
                        break;
                    }
                }
            }

            // Game name matches file stem (moderate confidence)
            if !config.game_name.is_empty()
                && config.game_name.to_lowercase() == file_stem
            {
                score = score.max(70);
            }

            // Contextual directory hints (e.g. "steamapps\\common\\Portal 2")
            if let Some(ref contexts) = config.executable_context {
                for ctx in contexts {
                    if parent_dir.contains(&ctx.to_lowercase()) {
                        score = score.max(60);
                        break;
                    }
                }
            }

            // Binaries subfolder match
            if !config.binaries_folder.is_empty()
                && parent_dir.ends_with(&config.binaries_folder.to_lowercase())
            {
                score = score.max(50);
            }

            if score > 0 {
                if best.as_ref().map_or(true, |(_, _, best_score)| score > *best_score) {
                    best = Some((guid.clone(), config.game_name.clone(), score));
                }
            }
        }

        // Only return matches with reasonable confidence
        best.filter(|(_, _, score)| *score >= 50)
    }

    /// Scans the provided list of search paths for game executables up to max depth.
    pub fn scan(&self, search_paths: &[&Path]) -> NucleusResult<Vec<FoundGame>> {
        let mut found = Vec::new();
        for &root in search_paths {
            if root.exists() && root.is_dir() {
                self.scan_dir(root, 0, 5, &mut found);
            }
        }

        // Sort: matched handlers first, then alphabetical
        found.sort_by(|a, b| {
            b.match_score
                .cmp(&a.match_score)
                .then(a.name.cmp(&b.name))
        });

        info!("DiskScanner: found {} executables ({} with handler matches)",
            found.len(),
            found.iter().filter(|g| g.handler_guid.is_some()).count()
        );
        Ok(found)
    }

    /// Internal recursive scanner with depth limiter.
    fn scan_dir(&self, dir: &Path, depth: usize, max_depth: usize, found: &mut Vec<FoundGame>) {
        if depth > max_depth {
            return;
        }

        // Check exclusions
        for ex in &self.excluded_paths {
            if dir == ex {
                return;
            }
        }

        let dir_name = dir.file_name().unwrap_or_default().to_string_lossy();

        // Skip system and hidden directories
        if dir_name.starts_with('$')
            || dir_name.starts_with('.')
            || dir_name.eq_ignore_ascii_case("System Volume Information")
            || dir_name.eq_ignore_ascii_case("Windows")
            || dir_name.eq_ignore_ascii_case("WinSxS")
            || dir_name.eq_ignore_ascii_case("WinSide")
            || dir_name.eq_ignore_ascii_case(".git")
            || dir_name.eq_ignore_ascii_case("node_modules")
            || dir_name.eq_ignore_ascii_case("__pycache__")
        {
            return;
        }

        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                self.scan_dir(&path, depth + 1, max_depth, found);
            } else if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext.eq_ignore_ascii_case("exe") {
                        let stem = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "Unknown".to_string());

                        // Ignore known non-game helper executables
                        if is_ignored_executable(&stem) {
                            continue;
                        }

                        let (handler_guid, handler_name, match_score) = match self.match_handler(&path) {
                            Some((guid, name, score)) => {
                                debug!("DiskScanner: matched '{}' -> handler '{}' (score {})", stem, name, score);
                                (Some(guid), Some(name), score)
                            }
                            None => (None, None, 0),
                        };

                        found.push(FoundGame {
                            name: stem,
                            exe_path: path,
                            icon_path: None,
                            handler_guid,
                            handler_name,
                            match_score,
                        });
                    }
                }
            }
        }
    }

    /// Scans recognized game libraries and local drives for game installations.
    pub fn scan_drives(&self) -> NucleusResult<Vec<FoundGame>> {
        let mut search_paths: Vec<PathBuf> = Vec::new();

        #[cfg(windows)]
        {
            // Scan drives from C: to Z:
            for drive_letter in b'C'..=b'Z' {
                let root = PathBuf::from(format!("{}:\\", drive_letter as char));
                if root.exists() {
                    // Standard game library folders
                    let candidates = [
                        root.join("Games"),
                        root.join("GOG Games"),
                        root.join("Epic Games"),
                        root.join("Program Files (x86)").join("Steam").join("steamapps").join("common"),
                        root.join("Program Files").join("Steam").join("steamapps").join("common"),
                        root.join("SteamLibrary").join("steamapps").join("common"),
                        root.join("Program Files (x86)").join("GOG Galaxy").join("Games"),
                        root.join("Program Files").join("Epic Games"),
                        root.join("XboxGames"),
                    ];

                    for candidate in &candidates {
                        if candidate.exists() {
                            search_paths.push(candidate.clone());
                        }
                    }
                }
            }
        }

        #[cfg(not(windows))]
        {
            if let Ok(home) = std::env::var("HOME") {
                let home_path = PathBuf::from(home);
                let candidates = [
                    home_path.join(".steam").join("steam").join("steamapps").join("common"),
                    home_path.join(".local").join("share").join("Steam").join("steamapps").join("common"),
                    home_path.join(".local").join("share").join("applications"),
                    PathBuf::from("/usr/games"),
                    PathBuf::from("/opt"),
                ];
                for candidate in &candidates {
                    if candidate.exists() {
                        search_paths.push(candidate.clone());
                    }
                }
            }
        }

        let refs: Vec<&Path> = search_paths.iter().map(|p| p.as_path()).collect();
        self.scan(&refs)
    }
}

/// Returns true if the executable stem should be ignored (non-game helpers/tools).
fn is_ignored_executable(stem: &str) -> bool {
    const IGNORED: &[&str] = &[
        "unins000",
        "uninstall",
        "uninst",
        "crashpad_handler",
        "UnityCrashHandler64",
        "UnityCrashHandler32",
        "CrashReportClient",
        "EpicWebHelper",
        "EasyAntiCheat",
        "BattlEye",
        "dotnetfx",
        "vcredist_x64",
        "vcredist_x86",
        "dxsetup",
        "DirectXSetup",
        "setup",
        "install",
        "installer",
        "redist",
        "prerequisite",
        "physx",
        "openal",
    ];
    IGNORED
        .iter()
        .any(|ignored| stem.eq_ignore_ascii_case(ignored))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_scanner_finds_executables() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_scan_{}", std::process::id()));
        let game_sub = temp_dir.join("MyGame").join("Binaries");
        std::fs::create_dir_all(&game_sub).unwrap();

        let dummy_exe = game_sub.join("game.exe");
        std::fs::write(&dummy_exe, b"MZ dummy").unwrap();

        let unins_exe = game_sub.join("uninstall.exe");
        std::fs::write(&unins_exe, b"MZ unins").unwrap();

        let scanner = DiskScanner::new();
        let found = scanner.scan(&[&temp_dir]).unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "game");
        assert_eq!(found[0].exe_path, dummy_exe);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_disk_scanner_handler_matching() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_match_{}", std::process::id()));
        let game_dir = temp_dir.join("Portal 2");
        std::fs::create_dir_all(&game_dir).unwrap();

        // Write the game exe with the exact name the handler expects
        let portal_exe = game_dir.join("portal2.exe");
        std::fs::write(&portal_exe, b"MZ portal").unwrap();

        // Other random exe that won't match
        let other_exe = game_dir.join("updater.exe");
        std::fs::write(&other_exe, b"MZ updater").unwrap();

        let mut scanner = DiskScanner::new();

        // Register a handler that expects "portal2.exe"
        let mut handler = HandlerConfig::default();
        handler.guid = "portal2-guid".to_string();
        handler.game_name = "Portal 2".to_string();
        handler.executable_name = "portal2.exe".to_string();

        let mut catalog = HashMap::new();
        catalog.insert("portal2-guid".to_string(), handler);
        scanner.load_handlers(&catalog);

        let found = scanner.scan(&[&temp_dir]).unwrap();

        // Should find 2 executables
        assert_eq!(found.len(), 2, "Expected portal2.exe and updater.exe");

        // The matched one should be first (highest score)
        let matched = found.iter().find(|g| g.exe_path == portal_exe);
        assert!(matched.is_some(), "portal2.exe should be found");
        let matched = matched.unwrap();
        assert_eq!(matched.handler_guid, Some("portal2-guid".to_string()));
        assert_eq!(matched.handler_name, Some("Portal 2".to_string()));
        assert!(matched.match_score >= 90, "Score should be high for exact exe name match");

        // The unmatched one should have no handler
        let unmatched = found.iter().find(|g| g.exe_path == other_exe);
        assert!(unmatched.is_some());
        let unmatched = unmatched.unwrap();
        assert!(unmatched.handler_guid.is_none());
        assert_eq!(unmatched.match_score, 0);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_ignored_executables() {
        assert!(is_ignored_executable("unins000"));
        assert!(is_ignored_executable("CrashReportClient"));
        assert!(is_ignored_executable("UnityCrashHandler64"));
        assert!(!is_ignored_executable("portal2"));
        assert!(!is_ignored_executable("GameEXE"));
    }
}
