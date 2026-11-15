//! Steam emulation tooling and Goldberg emulator configuration.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use nucleus_common::error::NucleusResult;
use nucleus_common::file_util;

/// Options for deploying Goldberg Steam emulator to a player instance directory.
#[derive(Debug, Clone)]
pub struct GoldbergDeployOptions<'a> {
    pub instance_dir: &'a Path,
    pub orig_game_dir: &'a Path,
    pub utils_dir: &'a Path,
    pub steam_id: &'a str,
    pub player_index: usize,
    pub nickname: &'a str,
    pub is_64bit: bool,
    pub language: Option<&'a str>,
    pub listen_port: Option<u16>,
    pub no_local_save: bool,
    pub create_steam_appid_by_exe: bool,
    pub experimental: bool,
    pub ignore_steam_appid: bool,
    pub need_steam_interface: bool,
    pub use_nucleus_environment: bool,
    pub custom_steam_api_dll_path: Option<&'a [String]>,
    pub handler_folder: Option<&'a Path>,
    pub experimental_steam_client: bool,
    pub experimental_rename: bool,
    pub orig_steam_dll_path: Option<&'a Path>,
    pub exe_name: Option<&'a str>,
    pub start_arguments: Option<&'a str>,
}

/// Launch information returned when Goldberg experimental steam client loader is configured.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamClientLaunchInfo {
    /// Path to the `steamclient_loader.exe` in the instance directory.
    pub loader_exe: PathBuf,
    /// Working directory for the loader process.
    pub working_dir: PathBuf,
    /// Arguments to pass to `steamclient_loader.exe`.
    pub launch_args: String,
    /// List of all deployed files.
    pub deployed_files: Vec<PathBuf>,
}

/// Options for deploying SmartSteamEmu to a player instance directory.
#[derive(Debug, Clone)]
pub struct SmartSteamEmuOptions<'a> {
    pub instance_dir: &'a Path,
    pub utils_dir: &'a Path,
    pub target_exe: &'a Path,
    pub start_in: Option<&'a Path>,
    pub cmd_line: &'a str,
    pub app_id: &'a str,
    pub steam_id: u64,
    pub nickname: &'a str,
    pub is_64bit: bool,
    pub language: Option<&'a str>,
    pub additional_lines: Option<&'a [String]>,
}

/// Deployment result for SmartSteamEmu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmartSteamEmuDeployResult {
    pub loader_exe: PathBuf,
    pub ini_path: PathBuf,
    pub emu_dir: PathBuf,
}

/// Tool for configuring and deploying Steam emulators (such as Goldberg Steam Emulator and SmartSteamEmu)
/// to enable offline local LAN multiplayer across instances.
#[derive(Debug, Default, Clone)]
pub struct SteamEmuTool;

impl SteamEmuTool {
    /// Writes `steam_appid.txt` into the target directory containing the specified App ID.
    pub fn write_steam_appid(target_dir: &Path, app_id: &str) -> NucleusResult<()> {
        file_util::ensure_dir(target_dir)?;
        let appid_file = target_dir.join("steam_appid.txt");
        file_util::write_text_file(&appid_file, app_id.trim())?;
        Ok(())
    }

    /// Sets up Goldberg emulator configuration (appid and settings) in one step (legacy wrapper).
    pub fn setup_goldberg(
        game_instance_dir: &Path,
        steam_id: &str,
        account_name: &str,
        listen_port: Option<u16>,
    ) -> NucleusResult<()> {
        Self::write_steam_appid(game_instance_dir, steam_id)?;
        let id_num = steam_id.parse::<u64>().unwrap_or(480);
        Self::setup_goldberg_settings(game_instance_dir, id_num, account_name, None, listen_port)
    }

    /// Configures Goldberg settings (`steam_settings` directory) for a specific player instance.
    pub fn setup_goldberg_settings(
        game_instance_dir: &Path,
        steam_id: u64,
        account_name: &str,
        language: Option<&str>,
        listen_port: Option<u16>,
    ) -> NucleusResult<()> {
        let settings_dir = game_instance_dir.join("steam_settings");
        file_util::ensure_dir(&settings_dir)?;

        // Write user steam id
        let user_id_file = settings_dir.join("user_steam_id.txt");
        file_util::write_text_file(&user_id_file, &steam_id.to_string())?;

        // Write account name
        let account_file = settings_dir.join("account_name.txt");
        file_util::write_text_file(&account_file, account_name)?;

        // Write language if specified
        if let Some(lang) = language {
            let lang_file = settings_dir.join("language.txt");
            file_util::write_text_file(&lang_file, lang)?;
        }

        // Write custom listen port if provided
        if let Some(port) = listen_port {
            let port_file = settings_dir.join("listen_port.txt");
            file_util::write_text_file(&port_file, &port.to_string())?;
        }

        Ok(())
    }

    /// Generates `ColdClientLoader.ini` according to C# NucleusCoop specification.
    pub fn generate_cold_client_loader_ini(
        exe_name: &str,
        cmd_line: &str,
        app_id: &str,
    ) -> String {
        let mut s = String::new();
        s.push_str("#My own modified version of ColdClientLoader originally by Rat431\r\n");
        s.push_str("[SteamClient]\r\n");
        s.push_str(&format!("Exe={}\r\n", exe_name));
        s.push_str("ExeRunDir=.\r\n");
        s.push_str(&format!("ExeCommandLine={}\r\n", cmd_line));
        s.push_str(&format!("AppId={}\r\n\r\n", app_id));
        s.push_str("SteamClientDll=steamclient.dll\r\n");
        s.push_str("SteamClient64Dll=steamclient64.dll\r\n");
        s
    }

    /// Deploys Goldberg Experimental Steam Client files (`steamclient.dll`, `steamclient64.dll`,
    /// `steamclient_loader.exe`) and generates `ColdClientLoader.ini`.
    pub fn deploy_goldberg_experimental_steam_client(
        instance_exe_folder: &Path,
        utils_dir: &Path,
        exe_name: &str,
        cmd_line: &str,
        app_id: &str,
        nickname: &str,
        steam_id: u64,
        language: Option<&str>,
        no_local_save: bool,
        use_nucleus_environment: bool,
    ) -> NucleusResult<SteamClientLaunchInfo> {
        file_util::ensure_dir(instance_exe_folder)?;
        let mut deployed = Vec::new();

        let goldberg_dir = utils_dir.join("GoldbergEmu");
        let exp_client_dir = goldberg_dir.join("experimental_steamclient");
        let exp_dir = goldberg_dir.join("experimental");
        let source_dir = if exp_client_dir.exists() {
            exp_client_dir
        } else if exp_dir.exists() {
            exp_dir
        } else {
            goldberg_dir
        };

        for dll_name in &["steamclient.dll", "steamclient64.dll", "steamclient_loader.exe"] {
            let src = source_dir.join(dll_name);
            let dst = instance_exe_folder.join(dll_name);
            if src.exists() {
                let _ = file_util::copy_file(&src, &dst, true);
                deployed.push(dst);
            } else {
                tracing::warn!("Goldberg experimental steamclient source file not found: {}", src.display());
            }
        }

        let ini_content = Self::generate_cold_client_loader_ini(exe_name, cmd_line, app_id);
        let ini_path = instance_exe_folder.join("ColdClientLoader.ini");
        file_util::write_text_file(&ini_path, &ini_content)?;
        deployed.push(ini_path);

        let lang = language.unwrap_or("english");
        let settings_folder = if no_local_save {
            if use_nucleus_environment {
                dirs::home_dir()
                    .map(|h| h.join("NucleusCoop").join(nickname).join("AppData").join("Roaming").join("Goldberg SteamEmu Saves").join("settings"))
                    .unwrap_or_else(|| instance_exe_folder.join("settings"))
            } else {
                dirs::data_dir()
                    .map(|d| d.join("Goldberg SteamEmu Saves").join("settings"))
                    .or_else(|| dirs::home_dir().map(|h| h.join("AppData").join("Roaming").join("Goldberg SteamEmu Saves").join("settings")))
                    .unwrap_or_else(|| instance_exe_folder.join("settings"))
            }
        } else {
            let local_save_file = instance_exe_folder.join("local_save.txt");
            let _ = file_util::write_text_file(&local_save_file, "");
            deployed.push(local_save_file);
            instance_exe_folder.join("settings")
        };

        let _ = file_util::ensure_dir(&settings_folder);
        let user_id_file = settings_folder.join("user_steam_id.txt");
        let _ = file_util::write_text_file(&user_id_file, &steam_id.to_string());
        deployed.push(user_id_file);

        let account_file = settings_folder.join("account_name.txt");
        let _ = file_util::write_text_file(&account_file, nickname);
        deployed.push(account_file);

        let lang_file = settings_folder.join("language.txt");
        let _ = file_util::write_text_file(&lang_file, lang);
        deployed.push(lang_file);

        // Also write into steam_settings in instance_exe_folder for compatibility
        let steam_settings = instance_exe_folder.join("steam_settings");
        let _ = file_util::ensure_dir(&steam_settings);
        let _ = file_util::write_text_file(&steam_settings.join("user_steam_id.txt"), &steam_id.to_string());
        let _ = file_util::write_text_file(&steam_settings.join("account_name.txt"), nickname);
        let _ = file_util::write_text_file(&steam_settings.join("language.txt"), lang);

        let loader_exe = instance_exe_folder.join("steamclient_loader.exe");
        Ok(SteamClientLaunchInfo {
            loader_exe,
            working_dir: instance_exe_folder.to_path_buf(),
            launch_args: String::new(),
            deployed_files: deployed,
        })
    }

    /// Renames existing `steam_api.dll` to `cracksteam_api.dll` (or `steam_api64.dll` to `cracksteam_api64.dll`)
    /// within `instance_dll_dir` before deploying Goldberg emulator DLLs.
    pub fn rename_to_crack_steam_api(
        instance_dll_dir: &Path,
        is_64bit: bool,
    ) -> NucleusResult<Option<PathBuf>> {
        let orig_name = if is_64bit { "steam_api64.dll" } else { "steam_api.dll" };
        let crack_name = if is_64bit { "cracksteam_api64.dll" } else { "cracksteam_api.dll" };
        let orig_path = instance_dll_dir.join(orig_name);
        let crack_path = instance_dll_dir.join(crack_name);
        if orig_path.exists() {
            if crack_path.exists() {
                let _ = std::fs::remove_file(&crack_path);
            }
            std::fs::rename(&orig_path, &crack_path)?;
            Ok(Some(crack_path))
        } else {
            Ok(None)
        }
    }

    /// Rewrites updated nickname and user Steam ID into all `account_name.txt` and `user_steam_id.txt`
    /// files found under `instance_dir` (both in `settings` and `steam_settings`), matching C#
    /// `SteamFunctions.GoldbergWriteSteamIDAndAccount`.
    pub fn write_steam_id_and_account(
        instance_dir: &Path,
        nickname: &str,
        steam_id: u64,
    ) -> NucleusResult<Vec<PathBuf>> {
        Self::write_steam_id_and_account_extended(instance_dir, nickname, steam_id, false)
    }

    /// Extended version supporting per-user Nucleus sandbox roaming directories.
    pub fn write_steam_id_and_account_extended(
        instance_dir: &Path,
        nickname: &str,
        steam_id: u64,
        use_nucleus_environment: bool,
    ) -> NucleusResult<Vec<PathBuf>> {
        let mut updated = Vec::new();
        let mut account_files = Vec::new();
        let mut steam_id_files = Vec::new();

        find_files_by_name(instance_dir, "account_name.txt", &mut account_files, 0, 5);
        find_files_by_name(instance_dir, "user_steam_id.txt", &mut steam_id_files, 0, 5);

        if account_files.is_empty() {
            for sub in &["settings", "steam_settings"] {
                let folder = instance_dir.join(sub);
                let _ = file_util::ensure_dir(&folder);
                account_files.push(folder.join("account_name.txt"));
            }
        }
        if steam_id_files.is_empty() {
            for sub in &["settings", "steam_settings"] {
                let folder = instance_dir.join(sub);
                let _ = file_util::ensure_dir(&folder);
                steam_id_files.push(folder.join("user_steam_id.txt"));
            }
        }

        if let Some(home) = dirs::home_dir() {
            let default_roaming = home
                .join("AppData")
                .join("Roaming")
                .join("Goldberg SteamEmu Saves")
                .join("settings");
            let def_acc = default_roaming.join("account_name.txt");
            let def_id = default_roaming.join("user_steam_id.txt");
            if def_acc.exists() && !account_files.contains(&def_acc) {
                account_files.push(def_acc);
            }
            if def_id.exists() && !steam_id_files.contains(&def_id) {
                steam_id_files.push(def_id);
            }

            if use_nucleus_environment {
                let nuc_roaming = home
                    .join("NucleusCoop")
                    .join(nickname)
                    .join("AppData")
                    .join("Roaming")
                    .join("Goldberg SteamEmu Saves")
                    .join("settings");
                let nuc_acc = nuc_roaming.join("account_name.txt");
                let nuc_id = nuc_roaming.join("user_steam_id.txt");
                if nuc_acc.exists() && !account_files.contains(&nuc_acc) {
                    account_files.push(nuc_acc);
                }
                if nuc_id.exists() && !steam_id_files.contains(&nuc_id) {
                    steam_id_files.push(nuc_id);
                }
            }
        }

        for acc in account_files {
            if let Some(parent) = acc.parent() {
                let _ = file_util::ensure_dir(parent);
            }
            file_util::write_text_file(&acc, nickname)?;
            updated.push(acc);
        }

        let id_str = steam_id.to_string();
        for id_file in steam_id_files {
            if let Some(parent) = id_file.parent() {
                let _ = file_util::ensure_dir(parent);
            }
            file_util::write_text_file(&id_file, &id_str)?;
            updated.push(id_file);
        }

        Ok(updated)
    }

    /// Extracts Steamworks interface version strings embedded in a steam_api DLL binary.
    pub fn extract_steam_interfaces_from_dll(dll_bytes: &[u8]) -> Vec<String> {
        let mut interfaces = std::collections::BTreeSet::new();

        const PREFIXES: &[&str] = &[
            "SteamClient",
            "SteamUser",
            "SteamFriends",
            "SteamUtils",
            "SteamMatchmaking",
            "SteamMatchmakingServers",
            "SteamUserStats",
            "SteamApps",
            "SteamNetworking",
            "SteamNetworkingUtils",
            "SteamNetworkingSockets",
            "SteamNetworkingMessages",
            "SteamRemoteStorage",
            "SteamScreenshots",
            "SteamHTTP",
            "SteamController",
            "SteamUGC",
            "SteamAppList",
            "SteamMusic",
            "SteamMusicRemote",
            "SteamHTMLSurface",
            "SteamInventory",
            "SteamVideo",
            "SteamParentalSettings",
            "SteamInput",
            "STEAMUSERSTATS_INTERFACE_VERSION",
            "STEAMAPPS_INTERFACE_VERSION",
            "STEAMUTILS_INTERFACE_VERSION",
            "STEAMFRIENDS_INTERFACE_VERSION",
            "STEAMUSER_INTERFACE_VERSION",
            "STEAMCLIENT_INTERFACE_VERSION",
        ];

        let len = dll_bytes.len();
        let mut i = 0;
        while i < len {
            if dll_bytes[i].is_ascii_alphanumeric() || dll_bytes[i] == b'_' {
                let start = i;
                while i < len && (dll_bytes[i].is_ascii_alphanumeric() || dll_bytes[i] == b'_') {
                    i += 1;
                }
                let slice = &dll_bytes[start..i];
                if slice.len() >= 8 && slice.len() <= 50 {
                    if let Ok(s) = std::str::from_utf8(slice) {
                        for prefix in PREFIXES {
                            if s.starts_with(prefix) {
                                let suffix = &s[prefix.len()..];
                                if suffix.chars().all(|c| c.is_ascii_digit()) && !suffix.is_empty() {
                                    interfaces.insert(s.to_string());
                                }
                            }
                        }
                    }
                }
            } else {
                i += 1;
            }
        }

        interfaces.into_iter().collect()
    }

    /// Generates `steam_interfaces.txt` by running `utils/GoldbergEmu/tools/generate_interfaces_file.exe`
    /// or by extracting interface version strings directly from the original `steam_api.dll`.
    pub fn generate_interfaces_file(
        orig_steam_dll: &Path,
        output_file: &Path,
        utils_dir: Option<&Path>,
    ) -> NucleusResult<PathBuf> {
        if let Some(parent) = output_file.parent() {
            file_util::ensure_dir(parent)?;
        }

        let mut ran_tool = false;
        if let Some(u_dir) = utils_dir {
            let tool_exe = u_dir
                .join("GoldbergEmu")
                .join("tools")
                .join("generate_interfaces_file.exe");
            if tool_exe.is_file() && orig_steam_dll.is_file() {
                let tools_dir = tool_exe.parent().unwrap();
                let mut cmd = std::process::Command::new(&tool_exe);
                cmd.arg(orig_steam_dll).current_dir(tools_dir);
                if let Ok(status) = cmd.status() {
                    if status.success() {
                        let tool_out = tools_dir.join("steam_interfaces.txt");
                        if tool_out.is_file() {
                            let _ = file_util::copy_file(&tool_out, output_file, true)?;
                            let _ = std::fs::remove_file(tool_out);
                            ran_tool = true;
                        }
                    }
                }
            }
        }

        if !ran_tool {
            let dll_bytes = if orig_steam_dll.is_file() {
                std::fs::read(orig_steam_dll)?
            } else {
                Vec::new()
            };

            let mut ifaces = Self::extract_steam_interfaces_from_dll(&dll_bytes);
            if ifaces.is_empty() {
                ifaces = vec![
                    "SteamClient017".to_string(),
                    "SteamUser019".to_string(),
                    "SteamFriends015".to_string(),
                    "SteamUtils009".to_string(),
                    "SteamMatchmaking009".to_string(),
                    "SteamMatchmakingServers002".to_string(),
                    "SteamUserStats011".to_string(),
                    "SteamApps007".to_string(),
                    "SteamNetworking005".to_string(),
                    "SteamRemoteStorage014".to_string(),
                    "SteamScreenshots003".to_string(),
                    "SteamHTTP002".to_string(),
                    "SteamController006".to_string(),
                    "SteamUGC012".to_string(),
                ];
            }

            let content = ifaces.join("\r\n") + "\r\n";
            file_util::write_text_file(output_file, &content)?;
        }

        Ok(output_file.to_path_buf())
    }

    /// Spawns `utils/GoldbergEmu/lobby_connect/lobby_connect.exe`, reads lobby connection ID from stdout,
    /// and returns the connection argument (e.g. `+connect_lobby <id>`).
    pub fn run_goldberg_lobby_connect(
        utils_dir: &Path,
        timeout_ms: u64,
    ) -> NucleusResult<Option<String>> {
        let lobby_exe = utils_dir
            .join("GoldbergEmu")
            .join("lobby_connect")
            .join("lobby_connect.exe");

        if !lobby_exe.is_file() {
            tracing::warn!("lobby_connect.exe not found at {}", lobby_exe.display());
            return Ok(None);
        }

        let mut child = std::process::Command::new(&lobby_exe)
            .stdout(std::process::Stdio::piped())
            .spawn()?;

        let stdout = match child.stdout.take() {
            Some(s) => s,
            None => return Ok(None),
        };

        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(stdout);
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_millis(timeout_ms.max(100));

        let mut found_lobby: Option<String> = None;

        for line_res in reader.lines() {
            if start_time.elapsed() >= timeout {
                break;
            }
            if let Ok(line) = line_res {
                let trimmed = line.trim();
                if trimmed.contains("+connect_lobby") {
                    found_lobby = Some(trimmed.to_string());
                    break;
                } else if trimmed.starts_with("Lobby ID:") || trimmed.starts_with("Lobby:") {
                    let id = trimmed.split(':').nth(1).unwrap_or("").trim();
                    if !id.is_empty() {
                        found_lobby = Some(format!("+connect_lobby {}", id));
                        break;
                    }
                }
            }
        }

        let _ = child.kill();
        let _ = child.wait();

        Ok(found_lobby)
    }

    /// Helper to format a lobby connection argument for game command lines.
    pub fn format_lobby_connect_arg(lobby_id: &str) -> String {
        let clean = lobby_id.trim();
        if clean.starts_with("+connect_lobby") {
            clean.to_string()
        } else {
            format!("+connect_lobby {}", clean)
        }
    }

    /// Generates `SmartSteamEmu.ini` with standard sections `[Launcher]` and `[SmartSteamEmu]`,
    /// appending any handler `SSEAdditionalLines` (supporting `Section|Key=Value` or `Key=Value`).
    pub fn generate_smart_steam_emu_ini(
        target_exe: &Path,
        start_in: &Path,
        cmd_line: &str,
        app_id: &str,
        steam_id: u64,
        nickname: &str,
        language: &str,
        emu_dir: &Path,
        additional_lines: Option<&[String]>,
    ) -> String {
        let mut sections: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();

        let launcher = vec![
            ("Target".to_string(), target_exe.to_string_lossy().to_string()),
            ("StartIn".to_string(), start_in.to_string_lossy().to_string()),
            ("CommandLine".to_string(), cmd_line.to_string()),
            ("SteamClientPath".to_string(), emu_dir.join("SmartSteamEmu.dll").to_string_lossy().to_string()),
            ("SteamClientPath64".to_string(), emu_dir.join("SmartSteamEmu64.dll").to_string_lossy().to_string()),
            ("InjectDll".to_string(), "1".to_string()),
        ];
        sections.insert("Launcher".to_string(), launcher);

        let sse = vec![
            ("AppId".to_string(), app_id.to_string()),
            ("SteamIdGeneration".to_string(), "Manual".to_string()),
            ("ManualSteamId".to_string(), steam_id.to_string()),
            ("Language".to_string(), language.to_string()),
            ("PersonaName".to_string(), nickname.to_string()),
            ("DisableOverlay".to_string(), "1".to_string()),
            ("SeparateStorageByName".to_string(), "1".to_string()),
        ];
        sections.insert("SmartSteamEmu".to_string(), sse);

        if let Some(lines) = additional_lines {
            for line in lines {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
                    continue;
                }
                if let Some(pipe_pos) = trimmed.find('|') {
                    let section = trimmed[..pipe_pos].trim().to_string();
                    let remainder = trimmed[pipe_pos + 1..].trim();
                    if let Some(eq_pos) = remainder.find('=') {
                        let key = remainder[..eq_pos].trim().to_string();
                        let val = remainder[eq_pos + 1..].trim().to_string();
                        let entries = sections.entry(section).or_default();
                        if let Some(entry) = entries.iter_mut().find(|(k, _)| k.eq_ignore_ascii_case(&key)) {
                            entry.1 = val;
                        } else {
                            entries.push((key, val));
                        }
                    }
                } else if let Some(eq_pos) = trimmed.find('=') {
                    let key = trimmed[..eq_pos].trim().to_string();
                    let val = trimmed[eq_pos + 1..].trim().to_string();
                    let entries = sections.entry("SmartSteamEmu".to_string()).or_default();
                    if let Some(entry) = entries.iter_mut().find(|(k, _)| k.eq_ignore_ascii_case(&key)) {
                        entry.1 = val;
                    } else {
                        entries.push((key, val));
                    }
                }
            }
        }

        let mut out = String::new();
        let ordered_keys: Vec<String> = {
            let mut keys = Vec::new();
            if sections.contains_key("Launcher") {
                keys.push("Launcher".to_string());
            }
            if sections.contains_key("SmartSteamEmu") {
                keys.push("SmartSteamEmu".to_string());
            }
            for k in sections.keys() {
                if k != "Launcher" && k != "SmartSteamEmu" {
                    keys.push(k.clone());
                }
            }
            keys
        };

        for sec_name in ordered_keys {
            if let Some(entries) = sections.get(&sec_name) {
                out.push_str(&format!("[{}]\r\n", sec_name));
                for (k, v) in entries {
                    out.push_str(&format!("{}={}\r\n", k, v));
                }
                out.push_str("\r\n");
            }
        }

        out
    }

    /// Deploys SmartSteamEmu from `utils/SmartSteamEmu/` to `instance_dir/SmartSteamLoader/`,
    /// generating `SmartSteamEmu.ini` and selecting the appropriate loader executable.
    pub fn deploy_smart_steam_emu(
        options: &SmartSteamEmuOptions,
    ) -> NucleusResult<SmartSteamEmuDeployResult> {
        let emu_dir = options.instance_dir.join("SmartSteamLoader");
        file_util::ensure_dir(&emu_dir)?;

        let source_sse = options.utils_dir.join("SmartSteamEmu");
        if source_sse.exists() {
            let _ = copy_dir_all(&source_sse, &emu_dir);
        } else {
            tracing::warn!("SmartSteamEmu source folder not found: {}", source_sse.display());
        }

        let loader_filename = if options.is_64bit {
            "SmartSteamLoader_x64.exe"
        } else {
            "SmartSteamLoader.exe"
        };
        let loader_exe = emu_dir.join(loader_filename);

        let start_in = options
            .start_in
            .map(|p| p.to_path_buf())
            .or_else(|| options.target_exe.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| options.instance_dir.to_path_buf());

        let lang = options.language.unwrap_or("english");
        let ini_content = Self::generate_smart_steam_emu_ini(
            options.target_exe,
            &start_in,
            options.cmd_line,
            options.app_id,
            options.steam_id,
            options.nickname,
            lang,
            &emu_dir,
            options.additional_lines,
        );

        let ini_path = emu_dir.join("SmartSteamEmu.ini");
        file_util::write_text_file(&ini_path, &ini_content)?;

        Ok(SmartSteamEmuDeployResult {
            loader_exe,
            ini_path,
            emu_dir,
        })
    }

    /// Deploys `winmm.dll` from `utils/Steam Stub DRM Patcher/<arch>/winmm.dll` to `instance_exe_folder`.
    pub fn deploy_steam_stub_drm_patcher(
        instance_exe_folder: &Path,
        utils_dir: &Path,
        arch: &str,
    ) -> NucleusResult<PathBuf> {
        file_util::ensure_dir(instance_exe_folder)?;

        let normalized_arch = {
            let trimmed = arch.trim().to_ascii_lowercase();
            if trimmed == "64" || trimmed == "x64" {
                "x64"
            } else if trimmed == "86" || trimmed == "32" || trimmed == "x86" {
                "x86"
            } else if trimmed.starts_with('x') {
                arch.trim()
            } else {
                "x86"
            }
        };

        let src_dll = utils_dir
            .join("Steam Stub DRM Patcher")
            .join(normalized_arch)
            .join("winmm.dll");

        let dst_dll = instance_exe_folder.join("winmm.dll");

        if src_dll.is_file() {
            let _ = file_util::copy_file(&src_dll, &dst_dll, true)?;
            tracing::info!("Deployed Steam Stub DRM Patcher: {} -> {}", src_dll.display(), dst_dll.display());
        } else {
            tracing::warn!("Steam Stub DRM Patcher winmm.dll not found: {}", src_dll.display());
        }

        Ok(dst_dll)
    }

    /// Runs Steamless (`utils/Steamless/Steamless.CLI.exe`) on `target_exe` to unpack SteamStub DRM.
    /// If `<target_exe>.unpacked.exe` is produced, replaces `target_exe` with the unpacked binary.
    pub fn run_steamless(
        utils_dir: &Path,
        target_exe: &Path,
        steamless_args: Option<&str>,
        timing_ms: u64,
    ) -> NucleusResult<bool> {
        let steamless_exe = utils_dir
            .join("Steamless")
            .join("Steamless.CLI.exe");

        let target_dir = target_exe.parent().unwrap_or(Path::new("."));
        let exe_name = target_exe.file_name().and_then(|n| n.to_str()).unwrap_or("");

        let unpacked_exe = target_dir.join(format!("{}.unpacked.exe", exe_name));

        if steamless_exe.is_file() && target_exe.is_file() {
            let mut cmd = std::process::Command::new(&steamless_exe);
            cmd.current_dir(target_dir);

            if let Some(args) = steamless_args {
                for arg in args.split_whitespace() {
                    if !arg.is_empty() {
                        cmd.arg(arg);
                    }
                }
            }
            cmd.arg(target_exe);

            let mut child = cmd.spawn()?;
            let wait_dur = std::time::Duration::from_millis(timing_ms.max(500));
            let start = std::time::Instant::now();

            loop {
                if let Ok(Some(_)) = child.try_wait() {
                    break;
                }
                if start.elapsed() >= wait_dur {
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }

        if unpacked_exe.is_file() {
            tracing::info!("Steamless patch applied: replacing original exe with unpacked version");
            let _ = std::fs::remove_file(target_exe);
            std::thread::sleep(std::time::Duration::from_millis(100));
            std::fs::rename(&unpacked_exe, target_exe)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Reads the Steam client language from the Windows Registry (`HKCU\Software\Valve\Steam\steamglobal`,
    /// `HKCU\Software\Valve\Steam`, or `HKLM\SOFTWARE\Valve\Steam`), defaulting to `"english"`.
    pub fn get_user_steam_client_language() -> Option<String> {
        #[cfg(windows)]
        {
            use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
            use winreg::RegKey;

            if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(r"Software\Valve\Steam\steamglobal", KEY_READ) {
                if let Ok(lang) = hkcu.get_value::<String, _>("Language") {
                    if !lang.trim().is_empty() {
                        return Some(lang.to_lowercase());
                    }
                }
            }

            if let Ok(hkcu) = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(r"Software\Valve\Steam", KEY_READ) {
                if let Ok(lang) = hkcu.get_value::<String, _>("Language") {
                    if !lang.trim().is_empty() {
                        return Some(lang.to_lowercase());
                    }
                }
            }

            if let Ok(hklm) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey_with_flags(r"SOFTWARE\Valve\Steam", KEY_READ) {
                if let Ok(lang) = hklm.get_value::<String, _>("Language") {
                    if !lang.trim().is_empty() {
                        return Some(lang.to_lowercase());
                    }
                }
            }
        }

        Some("english".to_string())
    }

    /// Fast Steam API DLL discovery:
    /// Tier 1: Check known standard relative paths (bin, Binaries/Win64, root, etc.) -> 0.05ms
    /// Tier 2: Check immediate subdirectories at depth 1 -> 1ms
    /// Tier 3: Bounded recursive search (max depth 3), pruning asset directories -> fast fallback
    pub fn find_steam_dlls_fast(dir: &Path, found: &mut Vec<PathBuf>) {
        const COMMON_PATHS: &[&str] = &[
            "bin\\steam_api.dll",
            "bin\\steam_api64.dll",
            "steam_api.dll",
            "steam_api64.dll",
            "bin64\\steam_api64.dll",
            "bin32\\steam_api.dll",
            "bin\\x64\\steam_api64.dll",
            "bin\\x86\\steam_api.dll",
            "bin\\win64\\steam_api64.dll",
            "bin\\win32\\steam_api.dll",
            "Binaries\\Win64\\steam_api64.dll",
            "Binaries\\Win32\\steam_api.dll",
            "Binaries\\Win64\\steam_api.dll",
            "Binaries\\Win32\\steam_api.dll",
        ];

        // Tier 1: Check common well-known locations
        for rel in COMMON_PATHS {
            let candidate = dir.join(rel);
            if candidate.is_file() {
                found.push(candidate);
            }
        }

        if !found.is_empty() {
            return;
        }

        // Tier 2 & 3: Bounded search pruning heavy asset directories
        Self::find_steam_dlls_bounded(dir, found, 0, 3);
    }

    fn find_steam_dlls_bounded(dir: &Path, found: &mut Vec<PathBuf>, depth: usize, max_depth: usize) {
        if depth > max_depth {
            return;
        }

        const ASSET_DIRS: &[&str] = &[
            "materials", "models", "sound", "sounds", "music", "audio", "maps", "media", "scenes",
            "textures", "shaders", "addons", "vpk", "content", "movies", "video", "docs", "documentation",
            "save", "saves", "cache", ".git", ".svn"
        ];

        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut subdirs = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        let lower = file_name.to_ascii_lowercase();
                        if lower == "steam_api.dll" || lower == "steam_api64.dll" {
                            found.push(path);
                        }
                    }
                } else if path.is_dir() && depth < max_depth {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        let lower = name.to_ascii_lowercase();
                        if !lower.ends_with(".bak")
                            && !lower.starts_with('.')
                            && !ASSET_DIRS.contains(&lower.as_str())
                            && !lower.starts_with("dlc")
                        {
                            subdirs.push(path);
                        }
                    }
                }
            }

            for subdir in subdirs {
                Self::find_steam_dlls_bounded(&subdir, found, depth + 1, max_depth);
            }
        }
    }

    /// Recursively searches a directory for `steam_api.dll` and `steam_api64.dll`.
    pub fn find_steam_dlls_recursive(dir: &Path, found: &mut Vec<PathBuf>) {
        Self::find_steam_dlls_fast(dir, found);
    }

    /// Faithfully deploys Goldberg emulator DLLs, `steam_appid.txt`, `local_save.txt`,
    /// `steam_interfaces.txt`, and per-player settings mirroring C# `SteamFunctions.UseGoldberg`.
    pub fn deploy_goldberg(options: &GoldbergDeployOptions) -> NucleusResult<Vec<PathBuf>> {
        let mut deployed = Vec::new();

        // If GoldbergExperimentalSteamClient is requested, delegate to experimental steam client deployment
        if options.experimental_steam_client {
            let exe_name = options.exe_name.unwrap_or("game.exe");
            let cmd_line = options.start_arguments.unwrap_or("");
            let steam_id_num = options.steam_id.parse::<u64>().unwrap_or(480);
            let launch_info = Self::deploy_goldberg_experimental_steam_client(
                options.instance_dir,
                options.utils_dir,
                exe_name,
                cmd_line,
                options.steam_id,
                options.nickname,
                steam_id_num,
                options.language,
                options.no_local_save,
                options.use_nucleus_environment,
            )?;
            deployed.extend(launch_info.deployed_files);
            return Ok(deployed);
        }

        let goldberg_src_dir = options.utils_dir.join("GoldbergEmu");

        let src_32_candidate = if options.experimental {
            goldberg_src_dir.join("experimental").join("steam_api.dll")
        } else {
            goldberg_src_dir.join("steam_api.dll")
        };
        let src_64_candidate = if options.experimental {
            goldberg_src_dir.join("experimental").join("steam_api64.dll")
        } else {
            goldberg_src_dir.join("steam_api64.dll")
        };

        let (src_32, src_64) = if !src_32_candidate.exists() && !src_64_candidate.exists() {
            let alt_dir = Path::new(r"C:\Games\NucleusCoop242\utils\GoldbergEmu");
            if alt_dir.exists() {
                (alt_dir.join("steam_api.dll"), alt_dir.join("steam_api64.dll"))
            } else {
                (src_32_candidate, src_64_candidate)
            }
        } else {
            (src_32_candidate, src_64_candidate)
        };

        // Determine destination relative paths for steam_api*.dll
        let mut target_rel_paths: Vec<PathBuf> = Vec::new();
        if let Some(custom_paths) = options.custom_steam_api_dll_path {
            for p in custom_paths {
                target_rel_paths.push(PathBuf::from(p));
            }
        }

        if target_rel_paths.is_empty() {
            let mut found_dlls = Vec::new();
            Self::find_steam_dlls_recursive(options.orig_game_dir, &mut found_dlls);
            for full_p in found_dlls {
                if let Ok(rel) = full_p.strip_prefix(options.orig_game_dir) {
                    target_rel_paths.push(rel.to_path_buf());
                }
            }
        }

        if target_rel_paths.is_empty() {
            let def_name = if options.is_64bit { "steam_api64.dll" } else { "steam_api.dll" };
            target_rel_paths.push(PathBuf::from(def_name));
            if options.orig_game_dir.join("bin").exists() || options.instance_dir.join("bin").exists() {
                target_rel_paths.push(PathBuf::from("bin").join(def_name));
            }
        }

        let player_steam_id = 76561198000000000u64 + (options.player_index as u64) + 1;
        let lang = options.language.unwrap_or("english");

        for rel in &target_rel_paths {
            let dest_file = options.instance_dir.join(rel);
            let dest_dir = dest_file.parent().unwrap_or(options.instance_dir);
            file_util::ensure_dir(dest_dir)?;

            let is_64 = dest_file
                .file_name()
                .map(|n| n.to_string_lossy().to_ascii_lowercase().contains("64"))
                .unwrap_or(options.is_64bit);

            // GoldbergExperimentalRename support
            if options.experimental && options.experimental_rename {
                if let Ok(Some(renamed)) = Self::rename_to_crack_steam_api(dest_dir, is_64) {
                    tracing::info!("GoldbergExperimentalRename: renamed to {}", renamed.display());
                    deployed.push(renamed);
                }
            }

            let src_file = if is_64 {
                if src_64.exists() { &src_64 } else { &src_32 }
            } else {
                if src_32.exists() { &src_32 } else { &src_64 }
            };

            if src_file.exists() {
                let _ = file_util::copy_file(src_file, &dest_file, true);
                tracing::info!("Deployed Goldberg Steam API: {} -> {}", src_file.display(), dest_file.display());
                deployed.push(dest_file.clone());
            } else {
                tracing::warn!("Goldberg source DLL not found: {}", src_file.display());
            }

            // Write steam_appid.txt in the steam dll directory
            if !options.ignore_steam_appid {
                let appid_file = dest_dir.join("steam_appid.txt");
                let _ = file_util::write_text_file(&appid_file, options.steam_id);
                deployed.push(appid_file);
            }

            // local_save.txt
            if !options.no_local_save {
                let local_save_file = dest_dir.join("local_save.txt");
                let _ = file_util::write_text_file(&local_save_file, "");
                deployed.push(local_save_file);
            }

            // Write settings in dest_dir (both settings and steam_settings for maximum compatibility)
            for sub_settings in &["settings", "steam_settings"] {
                let settings_folder = dest_dir.join(sub_settings);
                let _ = file_util::ensure_dir(&settings_folder);
                let _ = file_util::write_text_file(&settings_folder.join("account_name.txt"), options.nickname);
                let _ = file_util::write_text_file(&settings_folder.join("user_steam_id.txt"), &player_steam_id.to_string());
                let _ = file_util::write_text_file(&settings_folder.join("language.txt"), lang);
                if let Some(port) = options.listen_port {
                    let _ = file_util::write_text_file(&settings_folder.join("listen_port.txt"), &port.to_string());
                }
            }

            // If UseNucleusEnvironment && GoldbergNoLocalSave:
            if options.use_nucleus_environment && options.no_local_save {
                if let Some(home) = dirs::home_dir() {
                    let user_settings = home
                        .join("NucleusCoop")
                        .join(options.nickname)
                        .join("AppData")
                        .join("Roaming")
                        .join("Goldberg SteamEmu Saves")
                        .join("settings");
                    let _ = file_util::ensure_dir(&user_settings);
                    let _ = file_util::write_text_file(&user_settings.join("account_name.txt"), options.nickname);
                    let _ = file_util::write_text_file(&user_settings.join("user_steam_id.txt"), &player_steam_id.to_string());
                    let _ = file_util::write_text_file(&user_settings.join("language.txt"), lang);
                }
            }

            // steam_interfaces.txt
            if options.need_steam_interface {
                let iface_dest = dest_dir.join("steam_interfaces.txt");
                let candidates = [
                    goldberg_src_dir.join("tools").join("steam_interfaces.txt"),
                    options.orig_game_dir.join("steam_interfaces.txt"),
                    options.handler_folder.map(|h| h.join("steam_interfaces.txt")).unwrap_or_default(),
                ];
                let mut found_existing = false;
                for cand in &candidates {
                    if cand.is_file() {
                        let _ = file_util::copy_file(cand, &iface_dest, true);
                        deployed.push(iface_dest.clone());
                        found_existing = true;
                        break;
                    }
                }
                if !found_existing {
                    let orig_dll = options.orig_steam_dll_path
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| options.orig_game_dir.join(rel));
                    let _ = Self::generate_interfaces_file(&orig_dll, &iface_dest, Some(options.utils_dir));
                    deployed.push(iface_dest);
                }
            }
        }

        if options.create_steam_appid_by_exe {
            let exe_appid = options.instance_dir.join("steam_appid.txt");
            let _ = file_util::write_text_file(&exe_appid, options.steam_id);
            deployed.push(exe_appid);
        }

        Ok(deployed)
    }
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}

fn find_files_by_name(dir: &Path, target_name: &str, found: &mut Vec<PathBuf>, depth: usize, max_depth: usize) {
    if depth > max_depth {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.eq_ignore_ascii_case(target_name) {
                        found.push(path);
                    }
                }
            } else if path.is_dir() && depth < max_depth {
                find_files_by_name(&path, target_name, found, depth + 1, max_depth);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goldberg_deploy_recursive_discovery_and_settings() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_goldberg_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let orig_game = temp_dir.join("orig_game");
        let bin_dir = orig_game.join("bin");
        std::fs::create_dir_all(&bin_dir).unwrap();
        std::fs::write(bin_dir.join("steam_api.dll"), "dummy original steam_api.dll").unwrap();

        let utils_dir = temp_dir.join("utils");
        let goldberg_dir = utils_dir.join("GoldbergEmu");
        std::fs::create_dir_all(&goldberg_dir).unwrap();
        std::fs::write(goldberg_dir.join("steam_api.dll"), "goldberg emu steam_api dll").unwrap();

        let instance_dir = temp_dir.join("instance0");
        std::fs::create_dir_all(&instance_dir).unwrap();

        let opts = GoldbergDeployOptions {
            instance_dir: &instance_dir,
            orig_game_dir: &orig_game,
            utils_dir: &utils_dir,
            steam_id: "550",
            player_index: 0,
            nickname: "Player1",
            is_64bit: false,
            language: Some("english"),
            listen_port: None,
            no_local_save: true,
            create_steam_appid_by_exe: true,
            experimental: false,
            ignore_steam_appid: false,
            need_steam_interface: false,
            use_nucleus_environment: true,
            custom_steam_api_dll_path: None,
            handler_folder: None,
            experimental_steam_client: false,
            experimental_rename: false,
            orig_steam_dll_path: None,
            exe_name: Some("left4dead2.exe"),
            start_arguments: None,
        };

        let deployed = SteamEmuTool::deploy_goldberg(&opts).unwrap();
        assert!(!deployed.is_empty());

        let target_dll = instance_dir.join("bin").join("steam_api.dll");
        assert!(target_dll.exists(), "Instance bin/steam_api.dll should be created");
        let content = std::fs::read_to_string(&target_dll).unwrap();
        assert_eq!(content, "goldberg emu steam_api dll");

        let appid_bin = instance_dir.join("bin").join("steam_appid.txt");
        assert!(appid_bin.exists());
        assert_eq!(std::fs::read_to_string(&appid_bin).unwrap(), "550");

        let appid_exe = instance_dir.join("steam_appid.txt");
        assert!(appid_exe.exists());
        assert_eq!(std::fs::read_to_string(&appid_exe).unwrap(), "550");

        let settings_acc = instance_dir.join("bin").join("settings").join("account_name.txt");
        assert!(settings_acc.exists());
        assert_eq!(std::fs::read_to_string(&settings_acc).unwrap(), "Player1");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_cold_client_loader_ini_generation() {
        let ini = SteamEmuTool::generate_cold_client_loader_ini("game.exe", "-w 1920 -h 1080", "480");
        assert!(ini.contains("#My own modified version of ColdClientLoader originally by Rat431"));
        assert!(ini.contains("[SteamClient]"));
        assert!(ini.contains("Exe=game.exe"));
        assert!(ini.contains("ExeRunDir=."));
        assert!(ini.contains("ExeCommandLine=-w 1920 -h 1080"));
        assert!(ini.contains("AppId=480"));
        assert!(ini.contains("SteamClientDll=steamclient.dll"));
        assert!(ini.contains("SteamClient64Dll=steamclient64.dll"));
    }

    #[test]
    fn test_smart_steam_emu_ini_and_additional_lines() {
        let target_exe = Path::new(r"C:\Games\Portal2\portal2.exe");
        let start_in = Path::new(r"C:\Games\Portal2");
        let emu_dir = Path::new(r"C:\Games\Portal2\Instance0\SmartSteamLoader");
        let add_lines = vec![
            "SmartSteamEmu|ManualSteamId=76561198000000005".to_string(),
            "SmartSteamEmu|CustomKey=CustomValue".to_string(),
            "CustomSection|TestParam=123".to_string(),
        ];

        let ini = SteamEmuTool::generate_smart_steam_emu_ini(
            target_exe,
            start_in,
            "-novid",
            "620",
            76561198000000001,
            "Gamer1",
            "russian",
            emu_dir,
            Some(&add_lines),
        );

        assert!(ini.contains("[Launcher]"));
        assert!(ini.contains("Target=C:\\Games\\Portal2\\portal2.exe"));
        assert!(ini.contains("StartIn=C:\\Games\\Portal2"));
        assert!(ini.contains("CommandLine=-novid"));
        assert!(ini.contains("InjectDll=1"));

        assert!(ini.contains("[SmartSteamEmu]"));
        assert!(ini.contains("AppId=620"));
        assert!(ini.contains("PersonaName=Gamer1"));
        assert!(ini.contains("Language=russian"));
        // Overridden by additional line
        assert!(ini.contains("ManualSteamId=76561198000000005"));
        assert!(ini.contains("CustomKey=CustomValue"));

        // Additional custom section
        assert!(ini.contains("[CustomSection]"));
        assert!(ini.contains("TestParam=123"));
    }

    #[test]
    fn test_write_steam_id_and_account() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_write_account_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Create pre-existing account_name.txt and user_steam_id.txt in nested bin/settings
        let bin_settings = temp_dir.join("bin").join("settings");
        std::fs::create_dir_all(&bin_settings).unwrap();
        std::fs::write(bin_settings.join("account_name.txt"), "OldNick").unwrap();
        std::fs::write(bin_settings.join("user_steam_id.txt"), "000000").unwrap();

        let updated = SteamEmuTool::write_steam_id_and_account(&temp_dir, "SuperPlayer", 76561198012345678).unwrap();
        assert!(!updated.is_empty());

        let new_nick = std::fs::read_to_string(bin_settings.join("account_name.txt")).unwrap();
        assert_eq!(new_nick, "SuperPlayer");

        let new_id = std::fs::read_to_string(bin_settings.join("user_steam_id.txt")).unwrap();
        assert_eq!(new_id, "76561198012345678");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_extract_steam_interfaces_and_generate_file() {
        let dummy_dll_data = b"Some random binary header \x00 SteamClient017 \x00 SteamUser019 \x00 junk bytes \x00 SteamFriends015 \x00";
        let ifaces = SteamEmuTool::extract_steam_interfaces_from_dll(dummy_dll_data);
        assert!(ifaces.contains(&"SteamClient017".to_string()));
        assert!(ifaces.contains(&"SteamUser019".to_string()));
        assert!(ifaces.contains(&"SteamFriends015".to_string()));

        let temp_dir = std::env::temp_dir().join(format!("nucleus_iface_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let dummy_dll = temp_dir.join("dummy_steam_api.dll");
        std::fs::write(&dummy_dll, dummy_dll_data).unwrap();

        let out_file = temp_dir.join("steam_settings").join("steam_interfaces.txt");
        let generated = SteamEmuTool::generate_interfaces_file(&dummy_dll, &out_file, None).unwrap();
        assert!(generated.is_file());

        let content = std::fs::read_to_string(&generated).unwrap();
        assert!(content.contains("SteamClient017"));
        assert!(content.contains("SteamUser019"));
        assert!(content.contains("SteamFriends015"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_rename_to_crack_steam_api() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_crack_rename_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let steam_dll = temp_dir.join("steam_api.dll");
        std::fs::write(&steam_dll, "original 32bit dll").unwrap();

        let steam64_dll = temp_dir.join("steam_api64.dll");
        std::fs::write(&steam64_dll, "original 64bit dll").unwrap();

        let renamed32 = SteamEmuTool::rename_to_crack_steam_api(&temp_dir, false).unwrap();
        assert_eq!(renamed32, Some(temp_dir.join("cracksteam_api.dll")));
        assert!(!steam_dll.exists());
        assert!(temp_dir.join("cracksteam_api.dll").exists());

        let renamed64 = SteamEmuTool::rename_to_crack_steam_api(&temp_dir, true).unwrap();
        assert_eq!(renamed64, Some(temp_dir.join("cracksteam_api64.dll")));
        assert!(!steam64_dll.exists());
        assert!(temp_dir.join("cracksteam_api64.dll").exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_deploy_steam_stub_drm_patcher() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_drm_patch_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let utils_dir = temp_dir.join("utils");
        let x86_dir = utils_dir.join("Steam Stub DRM Patcher").join("x86");
        let x64_dir = utils_dir.join("Steam Stub DRM Patcher").join("x64");
        std::fs::create_dir_all(&x86_dir).unwrap();
        std::fs::create_dir_all(&x64_dir).unwrap();
        std::fs::write(x86_dir.join("winmm.dll"), "winmm 32bit").unwrap();
        std::fs::write(x64_dir.join("winmm.dll"), "winmm 64bit").unwrap();

        let inst_exe_dir = temp_dir.join("inst_exe");
        std::fs::create_dir_all(&inst_exe_dir).unwrap();

        let deployed = SteamEmuTool::deploy_steam_stub_drm_patcher(&inst_exe_dir, &utils_dir, "64").unwrap();
        assert!(deployed.exists());
        assert_eq!(std::fs::read_to_string(&deployed).unwrap(), "winmm 64bit");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_goldberg_experimental_steam_client_deploy() {
        let temp_dir = std::env::temp_dir().join(format!("nucleus_exp_client_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let utils_dir = temp_dir.join("utils");
        let exp_dir = utils_dir.join("GoldbergEmu").join("experimental_steamclient");
        std::fs::create_dir_all(&exp_dir).unwrap();
        std::fs::write(exp_dir.join("steamclient.dll"), "steamclient dll").unwrap();
        std::fs::write(exp_dir.join("steamclient64.dll"), "steamclient64 dll").unwrap();
        std::fs::write(exp_dir.join("steamclient_loader.exe"), "loader exe").unwrap();

        let inst_dir = temp_dir.join("instance0");
        std::fs::create_dir_all(&inst_dir).unwrap();

        let launch_info = SteamEmuTool::deploy_goldberg_experimental_steam_client(
            &inst_dir,
            &utils_dir,
            "game.exe",
            "-windowed",
            "1000",
            "PlayerOne",
            76561198000000001,
            Some("english"),
            false,
            false,
        ).unwrap();

        assert_eq!(launch_info.loader_exe, inst_dir.join("steamclient_loader.exe"));
        assert!(inst_dir.join("steamclient.dll").exists());
        assert!(inst_dir.join("steamclient64.dll").exists());
        assert!(inst_dir.join("ColdClientLoader.ini").exists());
        assert!(inst_dir.join("local_save.txt").exists());
        assert!(inst_dir.join("settings").join("account_name.txt").exists());

        let ini_txt = std::fs::read_to_string(inst_dir.join("ColdClientLoader.ini")).unwrap();
        assert!(ini_txt.contains("Exe=game.exe"));
        assert!(ini_txt.contains("AppId=1000"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_get_user_steam_client_language() {
        let lang = SteamEmuTool::get_user_steam_client_language();
        assert!(lang.is_some());
        let l = lang.unwrap();
        assert!(!l.is_empty());
    }
}
