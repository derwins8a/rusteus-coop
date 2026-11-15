//! Instance directory preparation and symlink deployment.

use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;
use std::path::{Path, PathBuf};

/// Options for configuring instance file and directory linking.
#[derive(Debug, Clone, Default)]
pub struct LinkOptions {
    /// Whether to symlink folders instead of copying/linking files individually when possible.
    pub symlink_folders: bool,
    /// Whether to prefer NTFS hardlinks over symlinks for files.
    pub hardlink: bool,
    /// Whether to physically copy all files instead of symlinking or hardlinking (HardcopyGame).
    pub hardcopy: bool,
    /// Whether to symlink the executable or copy it.
    pub symlink_exe: bool,
    /// Directory paths/names that cannot be junctioned as a whole unit, but must be recursed into.
    pub dir_symlink_exclusions: Vec<String>,
    /// Directory paths/names to truly skip (`direxskip`).
    pub dir_exclusions: Vec<String>,
    /// Directories whose files must all be copied rather than linked.
    pub dir_symlink_copy_instead: Vec<String>,
    /// Whether DirSymlinkCopyInstead copies all subfolders recursively (true) or links child subfolders as junctions (false).
    pub dir_symlink_copy_instead_include_sub_folders: bool,
    /// File names/patterns to skip linking.
    pub file_symlink_exclusions: Vec<String>,
    /// File names/patterns to copy instead of link.
    pub file_symlink_copy_instead: Vec<String>,
    /// Files copied directly from root game directory to instance directory.
    pub copy_files: Vec<String>,
    /// Primary executable name.
    pub exe_name: String,
}

/// Manages directory structure creation, hardlinking, and symlink isolation per instance.
pub struct InstanceSetup {
    /// Root directory containing all instances for the current session.
    pub session_root: PathBuf,
}

impl InstanceSetup {
    /// Creates a new `InstanceSetup` with the given session root folder.
    pub fn new(session_root: impl Into<PathBuf>) -> Self {
        Self {
            session_root: session_root.into(),
        }
    }

    /// Prepares an isolated directory for a specific instance using instant junctions/hardlinks.
    pub fn prepare_instance_folder(&self, instance_index: usize, game_root: &Path) -> NucleusResult<PathBuf> {
        let instance_folder = self.session_root.join(format!("Instance{}", instance_index));
        file_util::ensure_dir(&instance_folder)?;

        if game_root.exists() && game_root.is_dir() {
            Self::symlink_or_copy_game_dir(game_root, &instance_folder, &[], &[], true)?;
        }

        Ok(instance_folder)
    }

    /// Creates an instance folder at `<base_dir>/Instance<index>`.
    pub fn create_instance_folder(base_dir: &Path, index: usize) -> NucleusResult<PathBuf> {
        let instance_folder = base_dir.join(format!("Instance{}", index));
        file_util::ensure_dir(&instance_folder)?;
        Ok(instance_folder)
    }

    /// Recursively strips readonly permissions from all files and directories in a subtree.
    pub fn strip_readonly_recursive(path: &Path) {
        if !path.exists() {
            return;
        }
        if path.is_file() {
            if let Ok(metadata) = std::fs::metadata(path) {
                let mut perms = metadata.permissions();
                if perms.readonly() {
                    perms.set_readonly(false);
                    let _ = std::fs::set_permissions(path, perms);
                }
            }
        } else if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let child_path = entry.path();
                    Self::strip_readonly_recursive(&child_path);
                }
            }
            if let Ok(metadata) = std::fs::metadata(path) {
                let mut perms = metadata.permissions();
                if perms.readonly() {
                    perms.set_readonly(false);
                    let _ = std::fs::set_permissions(path, perms);
                }
            }
        }
    }

    /// Cleans up a specific instance folder, ensuring read-only attributes are stripped before deletion.
    pub fn cleanup_instance_folder(instance_folder: &Path) -> NucleusResult<()> {
        if instance_folder.exists() {
            Self::strip_readonly_recursive(instance_folder);
            if let Err(e) = std::fs::remove_dir_all(instance_folder) {
                tracing::warn!("remove_dir_all failed for {:?}: {}. Retrying after stripping readonly...", instance_folder, e);
                Self::strip_readonly_recursive(instance_folder);
                let _ = std::fs::remove_dir_all(instance_folder);
            }
        }
        Ok(())
    }

    /// Fast directory linking using symlink_dir or NTFS junction (instant, 0 disk space).
    pub fn link_or_copy_dir(src: &Path, dst: &Path) -> NucleusResult<()> {
        if dst.exists() {
            return Ok(());
        }
        #[cfg(windows)]
        {
            if std::os::windows::fs::symlink_dir(src, dst).is_ok() {
                return Ok(());
            }
            // mklink /J creates an NTFS junction which requires NO administrative privileges
            use std::os::windows::process::CommandExt;
            if let Ok(status) = std::process::Command::new("cmd")
                .args(["/C", "mklink", "/J"])
                .arg(dst.as_os_str())
                .arg(src.as_os_str())
                .creation_flags(0x08000000) // CREATE_NO_WINDOW
                .status()
            {
                if status.success() && dst.exists() {
                    return Ok(());
                }
            }
        }
        #[cfg(unix)]
        {
            if std::os::unix::fs::symlink(src, dst).is_ok() {
                return Ok(());
            }
        }
        // Fall back to directory copy only if linking is impossible (e.g., across exotic filesystems)
        file_util::copy_directory(src, dst, true)
    }

    /// Fast file linking using hardlinks with resilient symlink/copy fallback.
    pub fn link_or_copy_file(src: &Path, dst: &Path) -> NucleusResult<()> {
        Self::link_file_with_fallback(src, dst, true)
    }

    /// Links a file with fallback from hardlink to symlink to copy.
    pub fn link_file_with_fallback(src: &Path, dst: &Path, hardlink: bool) -> NucleusResult<()> {
        if dst.exists() {
            return Ok(());
        }
        if let Some(parent) = dst.parent() {
            file_util::ensure_dir(parent)?;
        }

        // 1. If hardlink was explicitly requested, try hardlink first
        if hardlink && std::fs::hard_link(src, dst).is_ok() {
            return Ok(());
        }

        // 2. Try symlink (succeeds if running as Admin or with Developer Mode enabled)
        #[cfg(windows)]
        {
            if std::os::windows::fs::symlink_file(src, dst).is_ok() {
                return Ok(());
            }
        }
        #[cfg(unix)]
        {
            if std::os::unix::fs::symlink(src, dst).is_ok() {
                return Ok(());
            }
        }

        // 3. Resilient fallback: on Windows (and Unix), if symlink failed (typical for non-admin users),
        // ALWAYS try hardlink before falling back to physical copying!
        // Hardlinks require no administrative privileges on NTFS, take 0 extra disk space,
        // and link instantaneously (microseconds instead of multi-gigabyte copying).
        if std::fs::hard_link(src, dst).is_ok() {
            return Ok(());
        }

        // 4. Only if both symlink and hardlink fail (e.g. crossing volume/drive boundaries), copy file
        std::fs::copy(src, dst)
            .map(|_| ())
            .map_err(|e| {
                tracing::warn!("Failed to link or copy {:?} to {:?}: {}", src, dst, e);
                nucleus_common::error::NucleusError::Io(e)
            })
    }

    /// Copies a single file directly with directory creation.
    pub fn copy_file_with_fallback(src: &Path, dst: &Path) -> NucleusResult<()> {
        if dst.exists() {
            return Ok(());
        }
        if let Some(parent) = dst.parent() {
            file_util::ensure_dir(parent)?;
        }
        std::fs::copy(src, dst)
            .map(|_| ())
            .map_err(|e| {
                tracing::warn!("Failed to copy {:?} to {:?}: {}", src, dst, e);
                nucleus_common::error::NucleusError::Io(e)
            })
    }

    /// Symlinks or copies a game directory into an instance folder with full fidelity to C# WinDirectoryUtil.
    pub fn link_game_directory(
        game_dir: &Path,
        instance_dir: &Path,
        options: &LinkOptions,
    ) -> NucleusResult<()> {
        if !game_dir.exists() || !game_dir.is_dir() {
            return Ok(());
        }

        file_util::ensure_dir(instance_dir)?;

        Self::link_directory_recursive(game_dir, game_dir, instance_dir, options, true)?;

        // Ensure all files specified in copy_files are copied from root if present
        for file in &options.copy_files {
            let src_file = game_dir.join(file);
            let dst_file = instance_dir.join(file);
            if src_file.exists() && !dst_file.exists() {
                if let Some(parent) = dst_file.parent() {
                    file_util::ensure_dir(parent)?;
                }
                let _ = std::fs::copy(&src_file, &dst_file);
            }
        }

        Ok(())
    }

    /// Recursive directory linking matching C# WinDirectoryUtil.LinkDirectory.
    fn link_directory_recursive(
        root: &Path,
        current_dir: &Path,
        destination: &Path,
        options: &LinkOptions,
        first_run: bool,
    ) -> NucleusResult<()> {
        let rel_path = current_dir
            .strip_prefix(root)
            .unwrap_or(current_dir)
            .to_string_lossy()
            .to_lowercase()
            .replace('/', "\\");
        let dir_name = current_dir
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        // 1. Check if directory is in dir_exclusions (direxskip - truly skip!)
        if !options.dir_exclusions.is_empty() {
            for ex in &options.dir_exclusions {
                let ex_clean = ex
                    .strip_prefix("direxskip")
                    .unwrap_or(ex.as_str())
                    .to_lowercase()
                    .replace('/', "\\");
                if (!rel_path.is_empty() && (rel_path == ex_clean || rel_path.starts_with(&format!("{}\\", ex_clean))))
                    || dir_name == ex_clean
                {
                    tracing::debug!("Skipping excluded directory: {:?}", current_dir);
                    return Ok(());
                }
            }
        }

        // 2. Check if directory is "special" (in dir_symlink_exclusions - cannot be junctioned as whole unit)
        let mut special = false;
        let mut is_copy_dir_root = false;
        let mut is_copy_dir_sub = false;

        if !options.dir_symlink_exclusions.is_empty() {
            for ex in &options.dir_symlink_exclusions {
                let ex_clean = ex.to_lowercase().replace('/', "\\");
                if (!rel_path.is_empty() && (rel_path == ex_clean || ex_clean.starts_with(&rel_path)))
                    || dir_name == ex_clean
                {
                    special = true;
                    break;
                }
            }
        }

        // Check if directory is in dir_symlink_copy_instead
        if !options.dir_symlink_copy_instead.is_empty() {
            for copy_entry in &options.dir_symlink_copy_instead {
                let copy_clean = copy_entry.to_lowercase().replace('/', "\\");
                if (!rel_path.is_empty() && rel_path == copy_clean) || dir_name == copy_clean {
                    is_copy_dir_root = true;
                    break;
                } else if !rel_path.is_empty() && rel_path.starts_with(&format!("{}\\", copy_clean)) {
                    is_copy_dir_sub = true;
                    break;
                }
            }
        }

        let copy_dir = is_copy_dir_root || (is_copy_dir_sub && options.dir_symlink_copy_instead_include_sub_folders);
        if copy_dir || options.hardcopy {
            special = true;
        }

        // 3. Symlink entire directory if symlink_folders is enabled and not special and not root and not hardcopy
        if options.symlink_folders && !options.hardcopy && !special && !first_run {
            if Self::link_or_copy_dir(current_dir, destination).is_ok() {
                return Ok(());
            }
        }

        // 4. Create physical directory and process contents
        file_util::ensure_dir(destination)?;

        if let Ok(entries) = std::fs::read_dir(current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let file_name = entry.file_name().to_string_lossy().to_string();
                let file_name_lower = file_name.to_lowercase();
                let dest = destination.join(&file_name);

                if path.is_dir() {
                    if copy_dir && !options.dir_symlink_copy_instead_include_sub_folders && !options.hardcopy {
                        // If false, only files directly in that directory are copied and child subfolders are linked as junctions
                        Self::link_or_copy_dir(&path, &dest)?;
                    } else {
                        Self::link_directory_recursive(root, &path, &dest, options, false)?;
                    }
                } else if path.is_file() {
                    // Check file exclusions
                    let mut file_excluded = false;
                    for fex in &options.file_symlink_exclusions {
                        let fex_clean = fex.to_lowercase();
                        if file_name_lower == fex_clean || file_name_lower.contains(&fex_clean) {
                            file_excluded = true;
                            break;
                        }
                    }
                    if file_excluded {
                        continue;
                    }

                    // Check if file should be copied instead of linked
                    let mut should_copy = options.hardcopy || copy_dir;
                    if !should_copy {
                        for cpy in &options.file_symlink_copy_instead {
                            let cpy_clean = cpy.to_lowercase();
                            if file_name_lower == cpy_clean || file_name_lower.contains(&cpy_clean) {
                                should_copy = true;
                                break;
                            }
                        }
                    }
                    if !should_copy {
                        for cpy in &options.copy_files {
                            if file_name_lower == cpy.to_lowercase() {
                                should_copy = true;
                                break;
                            }
                        }
                    }
                    if !should_copy && !options.symlink_exe && !options.exe_name.is_empty() && file_name_lower == options.exe_name.to_lowercase() {
                        should_copy = true;
                    }

                    if should_copy {
                        Self::copy_file_with_fallback(&path, &dest)?;
                    } else {
                        Self::link_file_with_fallback(&path, &dest, options.hardlink)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Legacy helper: symlinks or copies a game directory to an instance directory, respecting exclusions.
    pub fn symlink_or_copy_game_dir(
        game_dir: &Path,
        target_dir: &Path,
        dir_exclusions: &[String],
        file_exclusions: &[String],
        symlink_exe: bool,
    ) -> NucleusResult<()> {
        let options = LinkOptions {
            symlink_folders: false,
            hardlink: true,
            hardcopy: false,
            symlink_exe,
            dir_symlink_exclusions: dir_exclusions.to_vec(),
            dir_exclusions: Vec::new(),
            dir_symlink_copy_instead: Vec::new(),
            dir_symlink_copy_instead_include_sub_folders: false,
            file_symlink_exclusions: file_exclusions.to_vec(),
            file_symlink_copy_instead: Vec::new(),
            copy_files: Vec::new(),
            exe_name: String::new(),
        };
        Self::link_game_directory(game_dir, target_dir, &options)
    }

    /// Cleans up created instance directories, ensuring read-only attributes are stripped.
    pub fn cleanup(&self) -> NucleusResult<()> {
        if self.session_root.exists() {
            Self::strip_readonly_recursive(&self.session_root);
            let _ = std::fs::remove_dir_all(&self.session_root);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_setup_folder_creation_and_cleanup() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_inst_{}", std::process::id()));
        let game_dir = temp_dir.join("MockGame");
        let session_dir = temp_dir.join("SessionRoot");

        file_util::ensure_dir(&game_dir).unwrap();
        file_util::write_text_file(&game_dir.join("game.exe"), "dummy_exe").unwrap();

        let setup = InstanceSetup::new(&session_dir);
        let inst0 = setup.prepare_instance_folder(0, &game_dir).unwrap();
        assert!(inst0.exists());
        assert!(inst0.join("game.exe").exists());

        setup.cleanup().unwrap();
        assert!(!session_dir.exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_instance_linking_dir_symlink_exclusions_preserves_files() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_symlink_ex_{}", std::process::id()));
        let game_dir = temp_dir.join("L4D2Mock");
        let inst_dir = temp_dir.join("Instance0");

        file_util::ensure_dir(&game_dir.join("bin")).unwrap();
        file_util::write_text_file(&game_dir.join("bin").join("client.dll"), "dll_content").unwrap();
        file_util::ensure_dir(&game_dir.join("left4dead2")).unwrap();
        file_util::write_text_file(&game_dir.join("left4dead2").join("gameinfo.txt"), "gameinfo").unwrap();

        let options = LinkOptions {
            symlink_folders: false,
            hardlink: true,
            hardcopy: false,
            symlink_exe: true,
            dir_symlink_exclusions: vec!["bin".to_string(), "left4dead2".to_string()],
            dir_exclusions: vec!["crash_logs".to_string()],
            dir_symlink_copy_instead: vec![],
            dir_symlink_copy_instead_include_sub_folders: false,
            file_symlink_exclusions: vec![],
            file_symlink_copy_instead: vec![],
            copy_files: vec![],
            exe_name: "left4dead2.exe".to_string(),
        };

        InstanceSetup::link_game_directory(&game_dir, &inst_dir, &options).unwrap();

        // Crucial test: bin and left4dead2 must NOT have been skipped!
        assert!(inst_dir.join("bin").exists(), "bin folder should exist in instance");
        assert!(inst_dir.join("bin").join("client.dll").exists(), "client.dll should exist in instance/bin");
        assert!(inst_dir.join("left4dead2").exists(), "left4dead2 folder should exist in instance");
        assert!(inst_dir.join("left4dead2").join("gameinfo.txt").exists(), "gameinfo.txt should exist in instance/left4dead2");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_instance_linking_dir_exclusions_skips_folder() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_direx_{}", std::process::id()));
        let game_dir = temp_dir.join("Game");
        let inst_dir = temp_dir.join("Instance0");

        file_util::ensure_dir(&game_dir.join("crash_logs")).unwrap();
        file_util::write_text_file(&game_dir.join("crash_logs").join("dump.dmp"), "dump").unwrap();
        file_util::write_text_file(&game_dir.join("game.exe"), "exe").unwrap();

        let options = LinkOptions {
            symlink_folders: false,
            hardlink: true,
            hardcopy: false,
            symlink_exe: true,
            dir_symlink_exclusions: vec![],
            dir_exclusions: vec!["crash_logs".to_string()],
            dir_symlink_copy_instead: vec![],
            dir_symlink_copy_instead_include_sub_folders: false,
            file_symlink_exclusions: vec![],
            file_symlink_copy_instead: vec![],
            copy_files: vec![],
            exe_name: "game.exe".to_string(),
        };

        InstanceSetup::link_game_directory(&game_dir, &inst_dir, &options).unwrap();

        assert!(inst_dir.join("game.exe").exists());
        assert!(!inst_dir.join("crash_logs").exists(), "crash_logs folder should have been skipped");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_instance_linking_copy_files_and_file_exclusions() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_fileex_{}", std::process::id()));
        let game_dir = temp_dir.join("Game");
        let inst_dir = temp_dir.join("Instance0");

        file_util::ensure_dir(&game_dir).unwrap();
        file_util::write_text_file(&game_dir.join("skip.me"), "skip").unwrap();
        file_util::write_text_file(&game_dir.join("copy.cfg"), "setting=1").unwrap();
        file_util::write_text_file(&game_dir.join("copied_root.txt"), "hello").unwrap();
        file_util::write_text_file(&game_dir.join("normal.bin"), "data").unwrap();

        let options = LinkOptions {
            symlink_folders: false,
            hardlink: true,
            hardcopy: false,
            symlink_exe: true,
            dir_symlink_exclusions: vec![],
            dir_exclusions: vec![],
            dir_symlink_copy_instead: vec![],
            dir_symlink_copy_instead_include_sub_folders: false,
            file_symlink_exclusions: vec!["skip.me".to_string()],
            file_symlink_copy_instead: vec!["copy.cfg".to_string()],
            copy_files: vec!["copied_root.txt".to_string()],
            exe_name: "normal.bin".to_string(),
        };

        InstanceSetup::link_game_directory(&game_dir, &inst_dir, &options).unwrap();

        assert!(!inst_dir.join("skip.me").exists(), "skip.me should be excluded");
        assert!(inst_dir.join("copy.cfg").exists(), "copy.cfg should be present");
        assert!(inst_dir.join("copied_root.txt").exists(), "copied_root.txt should be present");
        assert!(inst_dir.join("normal.bin").exists(), "normal.bin should be present");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_dir_symlink_copy_instead_include_sub_folders_behavior() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_subfolders_{}", std::process::id()));
        let game_dir = temp_dir.join("Game");
        let inst_false = temp_dir.join("InstFalse");
        let inst_true = temp_dir.join("InstTrue");

        // Structure: Game/data/config.ini, Game/data/sub/nested.txt
        let data_dir = game_dir.join("data");
        let sub_dir = data_dir.join("sub");
        file_util::ensure_dir(&sub_dir).unwrap();
        file_util::write_text_file(&data_dir.join("config.ini"), "config=true").unwrap();
        file_util::write_text_file(&sub_dir.join("nested.txt"), "nested_val").unwrap();

        // 1. With dir_symlink_copy_instead_include_sub_folders = false
        let opts_false = LinkOptions {
            symlink_folders: false,
            hardlink: true,
            hardcopy: false,
            symlink_exe: true,
            dir_symlink_exclusions: vec![],
            dir_exclusions: vec![],
            dir_symlink_copy_instead: vec!["data".to_string()],
            dir_symlink_copy_instead_include_sub_folders: false,
            file_symlink_exclusions: vec![],
            file_symlink_copy_instead: vec![],
            copy_files: vec![],
            exe_name: String::new(),
        };
        InstanceSetup::link_game_directory(&game_dir, &inst_false, &opts_false).unwrap();
        assert!(inst_false.join("data").join("config.ini").exists());
        assert!(inst_false.join("data").join("sub").exists());
        assert!(inst_false.join("data").join("sub").join("nested.txt").exists());

        // 2. With dir_symlink_copy_instead_include_sub_folders = true
        let opts_true = LinkOptions {
            symlink_folders: false,
            hardlink: true,
            hardcopy: false,
            symlink_exe: true,
            dir_symlink_exclusions: vec![],
            dir_exclusions: vec![],
            dir_symlink_copy_instead: vec!["data".to_string()],
            dir_symlink_copy_instead_include_sub_folders: true,
            file_symlink_exclusions: vec![],
            file_symlink_copy_instead: vec![],
            copy_files: vec![],
            exe_name: String::new(),
        };
        InstanceSetup::link_game_directory(&game_dir, &inst_true, &opts_true).unwrap();
        assert!(inst_true.join("data").join("config.ini").exists());
        assert!(inst_true.join("data").join("sub").exists());
        assert!(inst_true.join("data").join("sub").join("nested.txt").exists());

        // Verify independent copying: modify inst_true nested file, game_dir file should remain intact
        file_util::write_text_file(&inst_true.join("data").join("sub").join("nested.txt"), "modified_true").unwrap();
        let orig_content = std::fs::read_to_string(&sub_dir.join("nested.txt")).unwrap();
        assert_eq!(orig_content, "nested_val", "Original file in game directory must remain unmodified");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_cleanup_instance_folder_strips_readonly() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_test_readonly_{}", std::process::id()));
        let inst_dir = temp_dir.join("InstanceRO");
        let sub_dir = inst_dir.join("subfolder");
        file_util::ensure_dir(&sub_dir).unwrap();

        let ro_file1 = inst_dir.join("locked1.txt");
        let ro_file2 = sub_dir.join("locked2.txt");
        file_util::write_text_file(&ro_file1, "ro_content_1").unwrap();
        file_util::write_text_file(&ro_file2, "ro_content_2").unwrap();

        // Mark files as readonly
        let mut perm1 = std::fs::metadata(&ro_file1).unwrap().permissions();
        perm1.set_readonly(true);
        std::fs::set_permissions(&ro_file1, perm1).unwrap();

        let mut perm2 = std::fs::metadata(&ro_file2).unwrap().permissions();
        perm2.set_readonly(true);
        std::fs::set_permissions(&ro_file2, perm2).unwrap();

        assert!(std::fs::metadata(&ro_file1).unwrap().permissions().readonly());
        assert!(std::fs::metadata(&ro_file2).unwrap().permissions().readonly());

        // cleanup_instance_folder must strip readonly and successfully delete instance folder
        InstanceSetup::cleanup_instance_folder(&inst_dir).expect("cleanup_instance_folder should succeed on readonly files");
        assert!(!inst_dir.exists(), "Instance folder should have been deleted");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}

