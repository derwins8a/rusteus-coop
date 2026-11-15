//! User profile isolation, save synchronization, and Windows Explorer shell folder redirection.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_common::file_util;

/// Subkey for Windows Explorer User Shell Folders under HKEY_CURRENT_USER.
pub const USER_SHELL_FOLDERS_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders";
/// Name of the registry value controlling the user's Documents folder.
pub const PERSONAL_VALUE_NAME: &str = "Personal";

/// Represents the isolated filesystem hierarchy and process environment for a single player instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerEnvironment {
    /// Isolated user profile root (%USERPROFILE%\NucleusCoop\<PlayerNickname>).
    pub root: PathBuf,
    /// Isolated Documents folder (%USERPROFILE%\NucleusCoop\<PlayerNickname>\Documents).
    pub documents: PathBuf,
    /// Isolated AppData Roaming folder (%USERPROFILE%\NucleusCoop\<PlayerNickname>\AppData\Roaming).
    pub app_data_roaming: PathBuf,
    /// Isolated AppData Local folder (%USERPROFILE%\NucleusCoop\<PlayerNickname>\AppData\Local).
    pub app_data_local: PathBuf,
    /// Virtual HOMEPATH for process environment.
    pub home_path: String,
    /// Specific isolated config path if configured.
    pub config_path: Option<PathBuf>,
    /// Specific isolated save path if configured.
    pub save_path: Option<PathBuf>,
}

impl PlayerEnvironment {
    /// Creates a new `PlayerEnvironment` instance.
    pub fn new(
        root: PathBuf,
        documents: PathBuf,
        app_data_roaming: PathBuf,
        app_data_local: PathBuf,
        home_path: String,
    ) -> Self {
        Self {
            root,
            documents,
            app_data_roaming,
            app_data_local,
            home_path,
            config_path: None,
            save_path: None,
        }
    }

    /// Sets the specific isolated config path.
    pub fn with_config_path(mut self, path: PathBuf) -> Self {
        self.config_path = Some(path);
        self
    }

    /// Sets the specific isolated save path.
    pub fn with_save_path(mut self, path: PathBuf) -> Self {
        self.save_path = Some(path);
        self
    }

    /// Applies the player's isolated directory paths to the target environment variables map.
    pub fn apply_to_env(&self, env: &mut HashMap<String, String>) {
        env.insert("USERPROFILE".to_string(), self.root.to_string_lossy().to_string());
        env.insert("HOMEPATH".to_string(), self.home_path.clone());
        env.insert("APPDATA".to_string(), self.app_data_roaming.to_string_lossy().to_string());
        env.insert("LOCALAPPDATA".to_string(), self.app_data_local.to_string_lossy().to_string());
    }
}

/// RAII guard for restoring the Windows Explorer `Personal` shell folder registry value.
/// Automatically restores the original registry value when dropped or when `.restore()` is called.
#[derive(Debug)]
pub struct PersonalFolderGuard {
    target_key: String,
    original_value: Option<String>,
    #[cfg(windows)]
    original_type: Option<winreg::enums::RegType>,
    backup_file: Option<PathBuf>,
    restored: bool,
}

impl PersonalFolderGuard {
    /// Manually restores the original `Personal` registry value.
    pub fn restore(&mut self) -> NucleusResult<()> {
        if self.restored {
            return Ok(());
        }
        self.restored = true;
        UsersTool::restore_personal_folder_internal(
            &self.target_key,
            self.original_value.as_deref(),
            #[cfg(windows)]
            self.original_type.clone(),
            self.backup_file.as_deref(),
        )
    }

    /// Returns the original registry value if one was backed up.
    pub fn original_value(&self) -> Option<&str> {
        self.original_value.as_deref()
    }

    /// Returns the path to the .reg backup file if one was written.
    pub fn backup_file(&self) -> Option<&Path> {
        self.backup_file.as_deref()
    }

    /// Disarms the guard so it will not attempt to restore on drop.
    pub fn disarm(&mut self) {
        self.restored = true;
    }
}

impl Drop for PersonalFolderGuard {
    fn drop(&mut self) {
        if !self.restored {
            if let Err(err) = self.restore() {
                tracing::error!("Failed to restore Personal shell folder on drop: {}", err);
            }
        }
    }
}

/// Matches a string against a pattern supporting `*` and `?` wildcards (case-insensitive).
pub fn wildcard_match(pattern: &str, text: &str) -> bool {
    let p_lower = pattern.to_lowercase();
    let t_lower = text.to_lowercase();

    let p_chars: Vec<char> = p_lower.chars().collect();
    let t_chars: Vec<char> = t_lower.chars().collect();

    let mut p_idx = 0;
    let mut t_idx = 0;
    let mut star_idx = None;
    let mut match_idx = 0;

    while t_idx < t_chars.len() {
        if p_idx < p_chars.len() && (p_chars[p_idx] == '?' || p_chars[p_idx] == t_chars[t_idx]) {
            p_idx += 1;
            t_idx += 1;
        } else if p_idx < p_chars.len() && p_chars[p_idx] == '*' {
            star_idx = Some(p_idx);
            match_idx = t_idx;
            p_idx += 1;
        } else if let Some(s_idx) = star_idx {
            p_idx = s_idx + 1;
            match_idx += 1;
            t_idx = match_idx;
        } else {
            return false;
        }
    }

    while p_idx < p_chars.len() && p_chars[p_idx] == '*' {
        p_idx += 1;
    }

    p_idx == p_chars.len()
}

/// Tool for managing isolated user profile directories, shell folder redirection, and save data.
#[derive(Debug, Default, Clone)]
pub struct UsersTool;

impl UsersTool {
    /// Creates the full %USERPROFILE%\NucleusCoop\<PlayerNickname> directory tree on disk
    /// including Documents, AppData\Roaming, and AppData\Local.
    pub fn create_player_environment(player_nickname: &str) -> NucleusResult<PlayerEnvironment> {
        let user_profile = dirs::home_dir().unwrap_or_else(|| PathBuf::from("C:\\Users\\Default"));
        let username = std::env::var("USERNAME").unwrap_or_else(|_| "Default".to_string());

        let root = user_profile.join("NucleusCoop").join(player_nickname);
        let documents = root.join("Documents");
        let app_data_roaming = root.join("AppData").join("Roaming");
        let app_data_local = root.join("AppData").join("Local");
        let home_path = format!("\\Users\\{}\\NucleusCoop\\{}", username, player_nickname);

        file_util::ensure_dir(&root)?;
        file_util::ensure_dir(&documents)?;
        file_util::ensure_dir(&app_data_roaming)?;
        file_util::ensure_dir(&app_data_local)?;

        Ok(PlayerEnvironment::new(
            root,
            documents,
            app_data_roaming,
            app_data_local,
            home_path,
        ))
    }

    /// Replicates UserProfileConfigPath from the user's real user profile to the player's isolated profile.
    pub fn copy_user_profile_config_path(
        player_env: &PlayerEnvironment,
        relative_path: &str,
        force_overwrite: bool,
    ) -> NucleusResult<PathBuf> {
        let real_user_dir = dirs::home_dir().unwrap_or_default();
        Self::copy_user_profile_config(
            &real_user_dir,
            &player_env.root.parent().unwrap_or(&player_env.root),
            player_env.root.file_name().and_then(|n| n.to_str()).unwrap_or_default(),
            relative_path,
            force_overwrite,
        )
    }

    /// Replicates DocumentsConfigPath or DocumentsSavePath from the user's real Documents to the player's isolated Documents.
    pub fn copy_documents_path(
        player_env: &PlayerEnvironment,
        relative_path: &str,
        force_overwrite: bool,
    ) -> NucleusResult<PathBuf> {
        let real_docs = dirs::document_dir().unwrap_or_else(|| {
            dirs::home_dir().map(|h| h.join("Documents")).unwrap_or_default()
        });
        let src_path = real_docs.join(relative_path);
        let dest_path = player_env.documents.join(relative_path);

        if src_path.exists() && (!dest_path.exists() || force_overwrite) {
            if src_path.is_file() {
                if let Some(parent) = dest_path.parent() {
                    file_util::ensure_dir(parent)?;
                }
                file_util::copy_file(&src_path, &dest_path, true)?;
            } else if src_path.is_dir() {
                file_util::ensure_dir(&dest_path)?;
                file_util::copy_directory(&src_path, &dest_path, true)?;
            }
        }

        Ok(dest_path)
    }

    /// Constructs the base user profile environment root directory for a specific player nickname.
    pub fn get_player_environment_root(base_dir: &Path, player_nickname: &str) -> PathBuf {
        base_dir.join("NucleusCoop").join(player_nickname)
    }

    /// Isolates and copies user configuration files from the real user directory to an isolated player directory.
    pub fn copy_user_profile_config(
        real_user_dir: &Path,
        isolated_base_dir: &Path,
        player_nickname: &str,
        relative_config_path: &str,
        force_overwrite: bool,
    ) -> NucleusResult<PathBuf> {
        let player_env = Self::get_player_environment_root(isolated_base_dir, player_nickname);
        let dest_path = player_env.join(relative_config_path);
        let src_path = real_user_dir.join(relative_config_path);

        if src_path.exists() && (!dest_path.exists() || force_overwrite) {
            if src_path.is_file() {
                if let Some(parent) = dest_path.parent() {
                    file_util::ensure_dir(parent)?;
                }
                file_util::copy_file(&src_path, &dest_path, true)?;
            } else if src_path.is_dir() {
                file_util::ensure_dir(&dest_path)?;
                file_util::copy_directory(&src_path, &dest_path, true)?;
            }
        }

        Ok(dest_path)
    }

    /// Isolates and copies game save data for a player instance.
    pub fn copy_user_save_data(
        real_user_dir: &Path,
        isolated_base_dir: &Path,
        player_nickname: &str,
        relative_save_path: &str,
        force_overwrite: bool,
    ) -> NucleusResult<PathBuf> {
        Self::copy_user_profile_config(
            real_user_dir,
            isolated_base_dir,
            player_nickname,
            relative_save_path,
            force_overwrite,
        )
    }

    // =========================================================================
    // 1. Windows Explorer Personal Shell Folder Redirection & Backup
    // =========================================================================

    /// Redirects the Windows Explorer `Personal` shell folder to the isolated Documents folder.
    ///
    /// Backs up the current registry value and writes `User Shell Folders.reg` to `backup_dir`
    /// (defaulting to `./utils/backup/User Shell Folders.reg`) if not already present.
    /// Returns an RAII `PersonalFolderGuard` that will restore the original registry value on drop or when `restore()` is called.
    pub fn redirect_personal_folder(
        documents_path: &Path,
        backup_dir: Option<&Path>,
    ) -> NucleusResult<PersonalFolderGuard> {
        Self::redirect_personal_folder_with_key(USER_SHELL_FOLDERS_KEY, documents_path, backup_dir)
    }

    /// Redirects a specified registry key's `Personal` value (useful for tests or custom subkeys).
    pub fn redirect_personal_folder_with_key(
        key_subpath: &str,
        documents_path: &Path,
        backup_dir: Option<&Path>,
    ) -> NucleusResult<PersonalFolderGuard> {
        // 1. Write User Shell Folders.reg backup if not already present
        let backup_reg_path = if let Some(dir) = backup_dir {
            let reg_file = dir.join("User Shell Folders.reg");
            if !reg_file.exists() {
                let _ = Self::export_registry_key(key_subpath, &reg_file);
            }
            Some(reg_file)
        } else {
            let default_dir = Path::new("utils").join("backup");
            let reg_file = default_dir.join("User Shell Folders.reg");
            if !reg_file.exists() {
                let _ = Self::export_registry_key(key_subpath, &reg_file);
            }
            Some(reg_file)
        };

        // 2. Read the existing Personal value
        #[cfg(windows)]
        let (original_value, original_type) = {
            match Self::get_winreg_value(key_subpath, PERSONAL_VALUE_NAME) {
                Some((val, vtype)) => (Some(val), Some(vtype)),
                None => (None, None),
            }
        };

        #[cfg(not(windows))]
        let original_value = Self::get_mock_registry_value(key_subpath, PERSONAL_VALUE_NAME);

        // 3. Set the new Personal value pointing to isolated Documents folder
        let docs_str = documents_path.to_string_lossy().to_string();
        #[cfg(windows)]
        {
            Self::set_winreg_value(
                key_subpath,
                PERSONAL_VALUE_NAME,
                &docs_str,
                winreg::enums::REG_EXPAND_SZ,
            )?;
        }
        #[cfg(not(windows))]
        {
            Self::set_mock_registry_value(key_subpath, PERSONAL_VALUE_NAME, &docs_str);
        }

        tracing::info!(
            "Redirected Personal shell folder for key '{}' to: {}",
            key_subpath,
            docs_str
        );

        Ok(PersonalFolderGuard {
            target_key: key_subpath.to_string(),
            original_value,
            #[cfg(windows)]
            original_type,
            backup_file: backup_reg_path,
            restored: false,
        })
    }

    /// Backs up the current `Personal` shell folder registry value and exports `.reg` file if not present.
    pub fn backup_personal_folder(backup_dir: Option<&Path>) -> NucleusResult<Option<String>> {
        Self::backup_personal_folder_with_key(USER_SHELL_FOLDERS_KEY, backup_dir)
    }

    /// Backs up the specified registry key's `Personal` shell folder value and exports `.reg` file if not present.
    pub fn backup_personal_folder_with_key(
        key_subpath: &str,
        backup_dir: Option<&Path>,
    ) -> NucleusResult<Option<String>> {
        if let Some(dir) = backup_dir {
            let reg_file = dir.join("User Shell Folders.reg");
            if !reg_file.exists() {
                let _ = Self::export_registry_key(key_subpath, &reg_file);
            }
        }

        #[cfg(windows)]
        {
            Ok(Self::get_winreg_value(key_subpath, PERSONAL_VALUE_NAME).map(|(v, _)| v))
        }
        #[cfg(not(windows))]
        {
            Ok(Self::get_mock_registry_value(key_subpath, PERSONAL_VALUE_NAME))
        }
    }

    /// Restores the `Personal` shell folder to the specified original value or by importing the backup .reg file.
    pub fn restore_personal_folder(
        original_value: Option<&str>,
        backup_file: Option<&Path>,
    ) -> NucleusResult<()> {
        Self::restore_personal_folder_with_key(USER_SHELL_FOLDERS_KEY, original_value, backup_file)
    }

    /// Restores the specified registry key's `Personal` value.
    pub fn restore_personal_folder_with_key(
        key_subpath: &str,
        original_value: Option<&str>,
        backup_file: Option<&Path>,
    ) -> NucleusResult<()> {
        Self::restore_personal_folder_internal(
            key_subpath,
            original_value,
            #[cfg(windows)]
            None,
            backup_file,
        )
    }

    pub(crate) fn restore_personal_folder_internal(
        key_subpath: &str,
        original_value: Option<&str>,
        #[cfg(windows)]
        original_type: Option<winreg::enums::RegType>,
        backup_file: Option<&Path>,
    ) -> NucleusResult<()> {
        let mut restored = false;

        if let Some(val) = original_value {
            #[cfg(windows)]
            {
                let vtype = original_type.unwrap_or(winreg::enums::REG_EXPAND_SZ);
                if Self::set_winreg_value(key_subpath, PERSONAL_VALUE_NAME, val, vtype).is_ok() {
                    restored = true;
                }
            }
            #[cfg(not(windows))]
            {
                Self::set_mock_registry_value(key_subpath, PERSONAL_VALUE_NAME, val);
                restored = true;
            }
        }

        // If direct registry write was not possible or failed, attempt import from .reg backup
        if !restored {
            if let Some(file) = backup_file {
                if file.is_file() {
                    let _ = Self::import_registry_file(file);
                    restored = true;
                }
            }
        }

        if restored {
            tracing::info!("Restored Personal shell folder for key '{}'", key_subpath);
        }

        Ok(())
    }

    /// Imports `./utils/backup/User Shell Folders.reg` if it exists.
    pub fn restore_user_shell_folders_from_backup(backup_dir: &Path) -> NucleusResult<bool> {
        let reg_file = backup_dir.join("User Shell Folders.reg");
        if reg_file.is_file() {
            Self::import_registry_file(&reg_file)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Exports a registry key under HKEY_CURRENT_USER to a `.reg` file using `reg.exe export`.
    pub fn export_registry_key(key_subpath: &str, output_path: &Path) -> NucleusResult<()> {
        if let Some(parent) = output_path.parent() {
            file_util::ensure_dir(parent)?;
        }

        #[cfg(windows)]
        {
            let full_key = format!(r"HKEY_CURRENT_USER\{}", key_subpath);
            let output = std::process::Command::new("reg.exe")
                .args(["export", &full_key, &output_path.to_string_lossy(), "/y"])
                .output()
                .map_err(|e| NucleusError::Platform(format!("Failed to spawn reg.exe export: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("reg.exe export of '{}' returned warning: {}", full_key, stderr.trim());
            }
        }

        #[cfg(not(windows))]
        {
            let dummy_content = format!("Windows Registry Editor Version 5.00\n\n[HKEY_CURRENT_USER\\{}]\n", key_subpath);
            let _ = std::fs::write(output_path, dummy_content);
        }

        Ok(())
    }

    /// Imports a `.reg` file into the system registry using `reg.exe import`.
    pub fn import_registry_file(reg_path: &Path) -> NucleusResult<()> {
        if !reg_path.exists() {
            return Err(NucleusError::Platform(format!(
                "Registry backup file does not exist: {:?}",
                reg_path
            )));
        }

        #[cfg(windows)]
        {
            let output = std::process::Command::new("reg.exe")
                .args(["import", &reg_path.to_string_lossy()])
                .output()
                .map_err(|e| NucleusError::Platform(format!("Failed to spawn reg.exe import: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                tracing::warn!("reg.exe import of '{:?}' returned warning: {}", reg_path, stderr.trim());
            }
        }

        #[cfg(not(windows))]
        {
            let _ = reg_path;
        }

        Ok(())
    }

    #[cfg(windows)]
    pub(crate) fn get_winreg_value(
        sub_key: &str,
        value_name: &str,
    ) -> Option<(String, winreg::enums::RegType)> {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let key = hkcu.open_subkey(sub_key).ok()?;
        let raw = key.get_raw_value(value_name).ok()?;
        let val_str = match raw.vtype {
            winreg::enums::REG_SZ | winreg::enums::REG_EXPAND_SZ => {
                let u16_slice: &[u16] = unsafe {
                    std::slice::from_raw_parts(raw.bytes.as_ptr() as *const u16, raw.bytes.len() / 2)
                };
                let mut s = String::from_utf16_lossy(u16_slice);
                if s.ends_with('\0') {
                    s.pop();
                }
                s
            }
            _ => key.get_value(value_name).ok()?,
        };
        Some((val_str, raw.vtype))
    }

    #[cfg(windows)]
    pub(crate) fn set_winreg_value(
        sub_key: &str,
        value_name: &str,
        value: &str,
        vtype: winreg::enums::RegType,
    ) -> NucleusResult<()> {
        let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey(sub_key)
            .map_err(|e| NucleusError::Platform(format!("Failed to create/open registry key {}: {}", sub_key, e)))?;

        if vtype == winreg::enums::REG_EXPAND_SZ {
            let mut utf16: Vec<u16> = value.encode_utf16().collect();
            utf16.push(0);
            let bytes = unsafe {
                std::slice::from_raw_parts(utf16.as_ptr() as *const u8, utf16.len() * 2)
            };
            let reg_val = winreg::RegValue {
                vtype: winreg::enums::REG_EXPAND_SZ,
                bytes: bytes.to_vec(),
            };
            key.set_raw_value(value_name, &reg_val)
                .map_err(|e| NucleusError::Platform(format!("Failed to set REG_EXPAND_SZ value {}: {}", value_name, e)))?;
        } else {
            key.set_value(value_name, &value)
                .map_err(|e| NucleusError::Platform(format!("Failed to set string value {}: {}", value_name, e)))?;
        }
        Ok(())
    }

    #[cfg(not(windows))]
    fn get_mock_registry_value(sub_key: &str, value_name: &str) -> Option<String> {
        let store = MOCK_REGISTRY.lock().ok()?;
        store.get(&(sub_key.to_string(), value_name.to_string())).cloned()
    }

    #[cfg(not(windows))]
    fn set_mock_registry_value(sub_key: &str, value_name: &str, value: &str) {
        if let Ok(mut store) = MOCK_REGISTRY.lock() {
            store.insert((sub_key.to_string(), value_name.to_string()), value.to_string());
        }
    }

    // =========================================================================
    // 2. Isolated Save/Config File Purging
    // =========================================================================

    /// Purges matching glob patterns/filenames inside the player's isolated config directory.
    /// If `player_env.config_path` is set, searches that directory; otherwise searches `player_env.root`.
    pub fn delete_files_in_config_path(
        player_env: &PlayerEnvironment,
        patterns: &[String],
    ) -> NucleusResult<usize> {
        if let Some(ref config_dir) = player_env.config_path {
            if config_dir.exists() {
                return Self::delete_files_in_directory(config_dir, patterns);
            }
        }
        Self::delete_files_in_directory(&player_env.root, patterns)
    }

    /// Purges matching glob patterns/filenames inside the player's isolated save directory.
    /// If `player_env.save_path` is set, searches that directory; otherwise searches `player_env.documents`.
    pub fn delete_files_in_save_path(
        player_env: &PlayerEnvironment,
        patterns: &[String],
    ) -> NucleusResult<usize> {
        if let Some(ref save_dir) = player_env.save_path {
            if save_dir.exists() {
                return Self::delete_files_in_directory(save_dir, patterns);
            }
        }
        if player_env.documents.exists() {
            let docs_deleted = Self::delete_files_in_directory(&player_env.documents, patterns)?;
            let root_deleted = Self::delete_files_in_directory(&player_env.root, patterns)?;
            return Ok(docs_deleted + root_deleted);
        }
        Self::delete_files_in_directory(&player_env.root, patterns)
    }

    /// Purges matching glob patterns/filenames inside a specific relative path under the player's root.
    pub fn delete_files_in_config_path_relative(
        player_env: &PlayerEnvironment,
        relative_path: &str,
        patterns: &[String],
    ) -> NucleusResult<usize> {
        let target = player_env.root.join(relative_path);
        Self::delete_files_in_directory(&target, patterns)
    }

    /// Purges matching glob patterns/filenames inside a specific relative path under the player's documents.
    pub fn delete_files_in_save_path_relative(
        player_env: &PlayerEnvironment,
        relative_path: &str,
        patterns: &[String],
    ) -> NucleusResult<usize> {
        let target = player_env.documents.join(relative_path);
        Self::delete_files_in_directory(&target, patterns)
    }

    /// Deletes all files within `base_dir` and its subdirectories matching any of the specified patterns.
    /// Strips read-only attributes before deletion if necessary.
    pub fn delete_files_in_directory(
        base_dir: &Path,
        patterns: &[String],
    ) -> NucleusResult<usize> {
        if !base_dir.exists() || patterns.is_empty() {
            return Ok(0);
        }

        let mut deleted = 0;
        let mut stack = vec![base_dir.to_path_buf()];

        while let Some(current_dir) = stack.pop() {
            let entries = match std::fs::read_dir(&current_dir) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.is_file() {
                    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    let rel_path = path
                        .strip_prefix(base_dir)
                        .map(|p| p.to_string_lossy().replace('\\', "/"))
                        .unwrap_or_default();

                    let matches = patterns.iter().any(|pat| {
                        let pat_norm = pat.replace('\\', "/");
                        wildcard_match(&pat_norm, file_name) || wildcard_match(&pat_norm, &rel_path)
                    });

                    if matches {
                        let remove_res = std::fs::remove_file(&path);
                        if remove_res.is_err() {
                            if let Ok(metadata) = std::fs::metadata(&path) {
                                let mut perms = metadata.permissions();
                                perms.set_readonly(false);
                                let _ = std::fs::set_permissions(&path, perms);
                                let _ = std::fs::remove_file(&path);
                            }
                        }

                        if !path.exists() {
                            tracing::info!("Purged matching file: {:?}", path);
                            deleted += 1;
                        }
                    }
                }
            }
        }

        Ok(deleted)
    }

    // =========================================================================
    // 3. Transfer Account Profiles
    // =========================================================================

    /// Copies existing saved data from `%USERPROFILE%\NucleusCoop\<source_nickname>` to
    /// `%USERPROFILE%\NucleusCoop\<target_nickname>`, enabling profile data replication between players.
    pub fn transfer_nucleus_user_account_profiles(
        source_nickname: &str,
        target_nickname: &str,
    ) -> NucleusResult<usize> {
        if source_nickname == target_nickname {
            return Ok(0);
        }

        let user_profile = dirs::home_dir().unwrap_or_else(|| PathBuf::from("C:\\Users\\Default"));
        let nuc_root = user_profile.join("NucleusCoop");
        let source_dir = nuc_root.join(source_nickname);
        let target_dir = nuc_root.join(target_nickname);

        Self::transfer_account_profiles_between_dirs(&source_dir, &target_dir)
    }

    /// Recursively copies all files and directories from `source_dir` into `target_dir`,
    /// creating parent directories as needed and overwriting existing files.
    pub fn transfer_account_profiles_between_dirs(
        source_dir: &Path,
        target_dir: &Path,
    ) -> NucleusResult<usize> {
        if !source_dir.exists() || !source_dir.is_dir() {
            return Ok(0);
        }
        if source_dir == target_dir {
            return Ok(0);
        }

        let mut copied_count = 0;
        let mut stack = vec![source_dir.to_path_buf()];

        while let Some(current_dir) = stack.pop() {
            let entries = match std::fs::read_dir(&current_dir) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                let rel = match path.strip_prefix(source_dir) {
                    Ok(r) => r,
                    Err(_) => continue,
                };
                let target_path = target_dir.join(rel);

                if path.is_dir() {
                    file_util::ensure_dir(&target_path)?;
                    stack.push(path);
                } else if path.is_file() {
                    if let Some(parent) = target_path.parent() {
                        file_util::ensure_dir(parent)?;
                    }
                    file_util::copy_file(&path, &target_path, true)?;
                    copied_count += 1;
                }
            }
        }

        tracing::info!(
            "Transferred {} files from {:?} to {:?}",
            copied_count,
            source_dir,
            target_dir
        );

        Ok(copied_count)
    }
}

#[cfg(not(windows))]
use std::sync::Mutex;
#[cfg(not(windows))]
static MOCK_REGISTRY: std::sync::LazyLock<Mutex<HashMap<(String, String), String>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_env_root() {
        let base = Path::new("C:\\Users\\Test");
        let env = UsersTool::get_player_environment_root(base, "Player1");
        assert_eq!(env, PathBuf::from("C:\\Users\\Test\\NucleusCoop\\Player1"));
    }

    #[test]
    fn test_copy_profile_config() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_users_test_{}", std::process::id()));
        let real_dir = temp_dir.join("real");
        let isolated_dir = temp_dir.join("isolated");

        let src_cfg = real_dir.join("game").join("config.ini");
        file_util::ensure_dir(src_cfg.parent().unwrap()).unwrap();
        file_util::write_text_file(&src_cfg, "volume=100").unwrap();

        let dest = UsersTool::copy_user_profile_config(
            &real_dir,
            &isolated_dir,
            "Player1",
            "game/config.ini",
            true,
        ).unwrap();

        assert!(dest.exists());
        let content = file_util::read_text_file(&dest).unwrap();
        assert_eq!(content, "volume=100");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_player_environment_hierarchy_and_env_vars() {
        let env = UsersTool::create_player_environment("TestPlayer").unwrap();
        assert!(env.root.ends_with("NucleusCoop\\TestPlayer") || env.root.ends_with("NucleusCoop/TestPlayer"));
        assert!(env.documents.ends_with("Documents"));
        assert!(env.app_data_roaming.ends_with("Roaming"));
        assert!(env.app_data_local.ends_with("Local"));

        assert!(env.root.is_dir());
        assert!(env.documents.is_dir());
        assert!(env.app_data_roaming.is_dir());
        assert!(env.app_data_local.is_dir());

        let mut vars = HashMap::new();
        env.apply_to_env(&mut vars);
        assert_eq!(vars.get("USERPROFILE").unwrap(), &env.root.to_string_lossy().to_string());
        assert_eq!(vars.get("APPDATA").unwrap(), &env.app_data_roaming.to_string_lossy().to_string());
        assert_eq!(vars.get("LOCALAPPDATA").unwrap(), &env.app_data_local.to_string_lossy().to_string());
        assert!(vars.contains_key("HOMEPATH"));
    }

    #[test]
    fn test_wildcard_matching() {
        assert!(wildcard_match("*.ini", "config.ini"));
        assert!(wildcard_match("*.INI", "CONFIG.ini"));
        assert!(wildcard_match("settings_*.cfg", "settings_player1.cfg"));
        assert!(wildcard_match("save??.dat", "save01.dat"));
        assert!(!wildcard_match("save??.dat", "save1.dat"));
        assert!(wildcard_match("*", "anything.txt"));
        assert!(wildcard_match("*.*", "anything.txt"));
        assert!(!wildcard_match("exact.bin", "other.bin"));
        assert!(wildcard_match("exact.bin", "exact.bin"));
    }

    #[test]
    fn test_delete_files_in_config_path_and_purging() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_purge_test_{}", std::process::id()));
        let root = temp_dir.join("NucleusCoop").join("Player1");
        let docs = root.join("Documents");
        let roaming = root.join("AppData").join("Roaming");
        let local = root.join("AppData").join("Local");

        file_util::ensure_dir(&docs).unwrap();
        file_util::ensure_dir(&roaming).unwrap();
        file_util::ensure_dir(&local).unwrap();

        let cfg1 = roaming.join("game").join("settings.cfg");
        let cfg2 = roaming.join("game").join("video.ini");
        let save1 = docs.join("My Games").join("SaveGame.sav");
        let keep_file = roaming.join("game").join("important_keep.txt");

        file_util::write_text_file(&cfg1, "bind=w").unwrap();
        file_util::write_text_file(&cfg2, "res=1920x1080").unwrap();
        file_util::write_text_file(&save1, "level=5").unwrap();
        file_util::write_text_file(&keep_file, "keep this!").unwrap();

        let p_env = PlayerEnvironment::new(
            root.clone(),
            docs.clone(),
            roaming.clone(),
            local.clone(),
            "\\Users\\Test\\NucleusCoop\\Player1".to_string(),
        );

        // Delete .cfg and .ini
        let patterns = vec!["*.cfg".to_string(), "*.ini".to_string()];
        let deleted = UsersTool::delete_files_in_config_path(&p_env, &patterns).unwrap();
        assert_eq!(deleted, 2);
        assert!(!cfg1.exists());
        assert!(!cfg2.exists());
        assert!(keep_file.exists());
        assert!(save1.exists());

        // Delete save files
        let save_patterns = vec!["*.sav".to_string()];
        let save_deleted = UsersTool::delete_files_in_save_path(&p_env, &save_patterns).unwrap();
        assert_eq!(save_deleted, 1);
        assert!(!save1.exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_transfer_nucleus_user_account_profiles() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_transfer_test_{}", std::process::id()));
        let p1_dir = temp_dir.join("Player1");
        let p2_dir = temp_dir.join("Player2");

        let p1_save = p1_dir.join("Documents").join("game").join("save.dat");
        let p1_cfg = p1_dir.join("AppData").join("Local").join("game").join("prefs.json");
        file_util::write_text_file(&p1_save, "player1_save_data").unwrap();
        file_util::write_text_file(&p1_cfg, "{\"volume\": 80}").unwrap();

        let count = UsersTool::transfer_account_profiles_between_dirs(&p1_dir, &p2_dir).unwrap();
        assert_eq!(count, 2);

        let p2_save = p2_dir.join("Documents").join("game").join("save.dat");
        let p2_cfg = p2_dir.join("AppData").join("Local").join("game").join("prefs.json");

        assert!(p2_save.exists());
        assert!(p2_cfg.exists());
        assert_eq!(file_util::read_text_file(&p2_save).unwrap(), "player1_save_data");
        assert_eq!(file_util::read_text_file(&p2_cfg).unwrap(), "{\"volume\": 80}");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_personal_folder_redirection_guard_and_restore() {
        let test_key = r"Software\NucleusCoopUnitTest\User Shell Folders";
        let temp_dir = std::env::temp_dir().join(format!("nucleus_reg_test_{}", std::process::id()));
        let backup_dir = temp_dir.join("backup");
        let isolated_docs = temp_dir.join("Player1").join("Documents");
        file_util::ensure_dir(&isolated_docs).unwrap();

        // 1. Prepopulate test key with original value
        #[cfg(windows)]
        {
            UsersTool::set_winreg_value(
                test_key,
                PERSONAL_VALUE_NAME,
                "%USERPROFILE%\\Documents",
                winreg::enums::REG_EXPAND_SZ,
            ).unwrap();
        }
        #[cfg(not(windows))]
        {
            UsersTool::set_mock_registry_value(test_key, PERSONAL_VALUE_NAME, "%USERPROFILE%\\Documents");
        }

        // 2. Redirect with RAII guard
        {
            let guard = UsersTool::redirect_personal_folder_with_key(
                test_key,
                &isolated_docs,
                Some(&backup_dir),
            ).unwrap();

            assert_eq!(guard.original_value(), Some("%USERPROFILE%\\Documents"));

            // Check redirected value
            #[cfg(windows)]
            {
                let (cur_val, _) = UsersTool::get_winreg_value(test_key, PERSONAL_VALUE_NAME).unwrap();
                assert_eq!(cur_val, isolated_docs.to_string_lossy());
            }
            #[cfg(not(windows))]
            {
                let cur_val = UsersTool::get_mock_registry_value(test_key, PERSONAL_VALUE_NAME).unwrap();
                assert_eq!(cur_val, isolated_docs.to_string_lossy());
            }
            // Guard dropped here, triggering automatic restore
        }

        // 3. Verify it was restored back to original
        #[cfg(windows)]
        {
            let (restored_val, _) = UsersTool::get_winreg_value(test_key, PERSONAL_VALUE_NAME).unwrap();
            assert_eq!(restored_val, "%USERPROFILE%\\Documents");

            // Clean up test key
            let hkcu = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER);
            let _ = hkcu.delete_subkey_all(r"Software\NucleusCoopUnitTest");
        }
        #[cfg(not(windows))]
        {
            let restored_val = UsersTool::get_mock_registry_value(test_key, PERSONAL_VALUE_NAME).unwrap();
            assert_eq!(restored_val, "%USERPROFILE%\\Documents");
        }

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
