//! Centralized AutoHotkey v1/v2 runtime management, prelude generation, and subprocess lifecycle.

use nucleus_common::error::{NucleusError, NucleusResult};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

/// AutoHotkey major version target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AhkVersion {
    #[default]
    Auto,
    V1,
    V2,
}

/// Execution timing for an AutoHotkey script.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AhkTiming {
    Immediate,
    PreLaunch,
    #[default]
    PostWindow,
}

/// Source representation of an AutoHotkey script.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AhkSource {
    File {
        path: PathBuf,
    },
    Inline {
        label: String,
        code: String,
        working_dir: PathBuf,
    },
}

/// Runtime variables injected into the AutoHotkey script prelude.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AhkPreludeVars {
    pub player_index: usize,
    pub player_id: u64,
    pub nickname: String,
    pub process_id: u32,
    pub window_handle: usize,
    pub pos_x: i32,
    pub pos_y: i32,
    pub width: i32,
    pub height: i32,
    pub instance_folder: String,
    pub root_install_folder: String,
    pub handler_folder: String,
    pub is_keyboard_player: bool,
    pub gamepad_id: i32,
}

/// Specification for launching an AutoHotkey script.
#[derive(Debug, Clone)]
pub struct AhkLaunchSpec {
    pub id: String,
    pub source: AhkSource,
    pub version: AhkVersion,
    pub timing: AhkTiming,
    pub vars: AhkPreludeVars,
    pub custom_runtime: Option<PathBuf>,
    pub extra_args: Vec<String>,
}

/// Active AutoHotkey process handle with captured stderr diagnostics.
pub struct ActiveAhkScript {
    pub id: String,
    pub child: Child,
    pub pid: u32,
}

/// Manages AutoHotkey discovery, prelude generation, out-of-process execution, and teardown.
#[derive(Default)]
pub struct AhkManager {
    active_scripts: Arc<Mutex<HashMap<String, ActiveAhkScript>>>,
    stderr_logs: Arc<Mutex<VecDeque<String>>>,
    custom_runtime_path: Option<PathBuf>,
}

impl AhkManager {
    /// Creates a new `AhkManager`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets an optional custom directory for AutoHotkey binaries.
    pub fn with_custom_runtime(mut self, path: Option<PathBuf>) -> Self {
        self.custom_runtime_path = path;
        self
    }

    /// Returns a shared reference to the collected stderr logs buffer.
    pub fn stderr_logs(&self) -> Arc<Mutex<VecDeque<String>>> {
        Arc::clone(&self.stderr_logs)
    }

    /// Automatically detects whether script source targets AutoHotkey v1 or v2.
    pub fn detect_version(code: &str, requested: AhkVersion) -> AhkVersion {
        if requested != AhkVersion::Auto {
            return requested;
        }

        let lower = code.to_lowercase();
        if lower.contains("#requires autohotkey v2") || lower.contains("#requires autohotkey >=2") {
            return AhkVersion::V2;
        }
        if lower.contains("#requires autohotkey v1") || lower.contains("#requires autohotkey <2") {
            return AhkVersion::V1;
        }

        // Heuristics for v1 syntax (legacy command-style with commas, e.g. "MsgBox,", "Run,", "EnvGet,")
        if lower.contains("msgbox,")
            || lower.contains("run,")
            || lower.contains("send,")
            || lower.contains("setworkingdir,")
        {
            return AhkVersion::V1;
        }

        // Default to v2 for modern scripts
        AhkVersion::V2
    }

    /// Escapes backslashes and double quotes for safe embedding in AutoHotkey string literals.
    fn escape_ahk_string(s: &str) -> String {
        s.replace('`', "``").replace('"', "`\"")
    }

    /// Renders a syntax-safe prelude injecting Nucleus instance variables.
    pub fn render_prelude(version: AhkVersion, vars: &AhkPreludeVars) -> String {
        let is_v2 = matches!(version, AhkVersion::V2 | AhkVersion::Auto);
        let mut out = String::new();

        out.push_str("; --- Nucleus Co-op AutoHotkey Injected Prelude ---\n");
        // Ensure scripts can run concurrently without single-instance dialog prompts
        out.push_str("#SingleInstance Off\n");

        if is_v2 {
            out.push_str(&format!(
                "global Nucleus_PlayerIndex := {}\n",
                vars.player_index
            ));
            out.push_str(&format!("global Nucleus_PlayerID := {}\n", vars.player_id));
            out.push_str(&format!(
                "global Nucleus_Nickname := \"{}\"\n",
                Self::escape_ahk_string(&vars.nickname)
            ));
            out.push_str(&format!("global Nucleus_PID := {}\n", vars.process_id));
            out.push_str(&format!("global Nucleus_HWND := {}\n", vars.window_handle));
            out.push_str(&format!("global Nucleus_PosX := {}\n", vars.pos_x));
            out.push_str(&format!("global Nucleus_PosY := {}\n", vars.pos_y));
            out.push_str(&format!("global Nucleus_Width := {}\n", vars.width));
            out.push_str(&format!("global Nucleus_Height := {}\n", vars.height));
            out.push_str(&format!(
                "global Nucleus_InstanceFolder := \"{}\"\n",
                Self::escape_ahk_string(&vars.instance_folder)
            ));
            out.push_str(&format!(
                "global Nucleus_RootInstallFolder := \"{}\"\n",
                Self::escape_ahk_string(&vars.root_install_folder)
            ));
            out.push_str(&format!(
                "global Nucleus_HandlerFolder := \"{}\"\n",
                Self::escape_ahk_string(&vars.handler_folder)
            ));
            out.push_str(&format!(
                "global Nucleus_IsKeyboardPlayer := {}\n",
                if vars.is_keyboard_player { 1 } else { 0 }
            ));
            out.push_str(&format!(
                "global Nucleus_GamepadId := {}\n",
                vars.gamepad_id
            ));
        } else {
            out.push_str(&format!("Nucleus_PlayerIndex := {}\n", vars.player_index));
            out.push_str(&format!("Nucleus_PlayerID := {}\n", vars.player_id));
            out.push_str(&format!(
                "Nucleus_Nickname := \"{}\"\n",
                Self::escape_ahk_string(&vars.nickname)
            ));
            out.push_str(&format!("Nucleus_PID := {}\n", vars.process_id));
            out.push_str(&format!("Nucleus_HWND := {}\n", vars.window_handle));
            out.push_str(&format!("Nucleus_PosX := {}\n", vars.pos_x));
            out.push_str(&format!("Nucleus_PosY := {}\n", vars.pos_y));
            out.push_str(&format!("Nucleus_Width := {}\n", vars.width));
            out.push_str(&format!("Nucleus_Height := {}\n", vars.height));
            out.push_str(&format!(
                "Nucleus_InstanceFolder := \"{}\"\n",
                Self::escape_ahk_string(&vars.instance_folder)
            ));
            out.push_str(&format!(
                "Nucleus_RootInstallFolder := \"{}\"\n",
                Self::escape_ahk_string(&vars.root_install_folder)
            ));
            out.push_str(&format!(
                "Nucleus_HandlerFolder := \"{}\"\n",
                Self::escape_ahk_string(&vars.handler_folder)
            ));
            out.push_str(&format!(
                "Nucleus_IsKeyboardPlayer := {}\n",
                if vars.is_keyboard_player { 1 } else { 0 }
            ));
            out.push_str(&format!("Nucleus_GamepadId := {}\n", vars.gamepad_id));
        }

        out.push_str("; --- End Injected Prelude ---\n\n");
        out
    }

    /// Resolves and validates a script file relative to handler or instance directories,
    /// strictly blocking directory traversal attacks (`..`).
    pub fn resolve_handler_ahk_file(
        handler_folder: &Path,
        instance_folder: &Path,
        rel_or_name: &str,
    ) -> NucleusResult<PathBuf> {
        let clean_rel = Path::new(rel_or_name);
        for comp in clean_rel.components() {
            if let std::path::Component::ParentDir = comp {
                return Err(NucleusError::Handler(format!(
                    "Path traversal ('..') is not permitted in AutoHotkey script path: '{}'",
                    rel_or_name
                )));
            }
        }

        let candidate_handler = handler_folder.join(clean_rel);
        if candidate_handler.exists() && candidate_handler.is_file() {
            return Ok(candidate_handler);
        }

        let candidate_inst = instance_folder.join(clean_rel);
        if candidate_inst.exists() && candidate_inst.is_file() {
            return Ok(candidate_inst);
        }

        Err(NucleusError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "AutoHotkey script '{}' not found in handler or instance directories",
                rel_or_name
            ),
        )))
    }

    /// Discovers an AutoHotkey executable for the requested version.
    pub fn discover_runtime(
        &self,
        version: AhkVersion,
        prefer_64bit: bool,
    ) -> NucleusResult<PathBuf> {
        let is_v2 = matches!(version, AhkVersion::V2 | AhkVersion::Auto);
        let bitness_exe = if prefer_64bit {
            if is_v2 {
                "AutoHotkey64.exe"
            } else {
                "AutoHotkeyU64.exe"
            }
        } else {
            if is_v2 {
                "AutoHotkey32.exe"
            } else {
                "AutoHotkeyU32.exe"
            }
        };

        // 1. Check custom runtime path
        if let Some(ref custom_dir) = self.custom_runtime_path {
            let candidate = custom_dir.join(bitness_exe);
            if candidate.exists() {
                return Ok(candidate);
            }
            let sub_v = if is_v2 {
                custom_dir.join("v2")
            } else {
                custom_dir.join("v1")
            };
            let sub_cand = sub_v.join(bitness_exe);
            if sub_cand.exists() {
                return Ok(sub_cand);
            }
        }

        // 2. Check Nucleus utils/AutoHotkey directory
        let utils_dir = if let Some(globals) = nucleus_common::Globals::try_get() {
            globals.utils_path.join("AutoHotkey")
        } else {
            PathBuf::from("utils").join("AutoHotkey")
        };

        let sub_v = if is_v2 {
            utils_dir.join("v2")
        } else {
            utils_dir.join("v1")
        };
        let candidate = sub_v.join(bitness_exe);
        if candidate.exists() {
            return Ok(candidate);
        }

        let direct = utils_dir.join(bitness_exe);
        if direct.exists() {
            return Ok(direct);
        }

        // 3. Fallback to standard Program Files system install
        #[cfg(windows)]
        {
            let program_files =
                std::env::var("ProgramFiles").unwrap_or_else(|_| "C:\\Program Files".to_string());
            let sys_v2 = Path::new(&program_files)
                .join("AutoHotkey")
                .join("v2")
                .join(bitness_exe);
            if sys_v2.exists() {
                return Ok(sys_v2);
            }
            let sys_v1 = Path::new(&program_files)
                .join("AutoHotkey")
                .join(bitness_exe);
            if sys_v1.exists() {
                return Ok(sys_v1);
            }
        }

        Err(NucleusError::Platform(format!(
            "AutoHotkey {} runtime ({}) not found. Place it in utils/AutoHotkey/ or configure in Settings.",
            if is_v2 { "v2" } else { "v1" },
            bitness_exe
        )))
    }

    /// Spawns an AutoHotkey script according to the given launch specification.
    pub fn spawn_script(&self, spec: &AhkLaunchSpec) -> NucleusResult<u32> {
        let version = match spec.source {
            AhkSource::Inline { ref code, .. } => Self::detect_version(code, spec.version),
            AhkSource::File { ref path } => {
                let code = fs::read_to_string(path).unwrap_or_default();
                Self::detect_version(&code, spec.version)
            }
        };

        let is_64bit =
            nucleus_common::is_64bit_executable(Path::new(&spec.vars.root_install_folder))
                .unwrap_or(true);
        let runtime_exe = if let Some(ref custom) = spec.custom_runtime {
            custom.clone()
        } else {
            self.discover_runtime(version, is_64bit)?
        };

        let prelude = Self::render_prelude(version, &spec.vars);

        let (mut cmd, pipe_input) = match &spec.source {
            AhkSource::Inline {
                code, working_dir, ..
            } => {
                // If code contains #Include, stage it into .nucleus_ahk to preserve relative path resolution
                if code.contains("#Include") || code.contains("#include") {
                    let ahk_dir = Path::new(&spec.vars.instance_folder).join(".nucleus_ahk");
                    let _ = fs::create_dir_all(&ahk_dir);
                    let staged_file = ahk_dir.join(format!("{}.ahk", spec.id));
                    let full_code = format!("{}\n{}", prelude, code);
                    fs::write(&staged_file, &full_code)?;

                    let mut c = Command::new(&runtime_exe);
                    c.arg("/ErrorStdOut").arg(&staged_file);
                    c.current_dir(working_dir);
                    (c, None)
                } else {
                    // Pipe directly to stdin via '*'
                    let mut c = Command::new(&runtime_exe);
                    c.arg("/ErrorStdOut").arg("*");
                    c.current_dir(working_dir);
                    c.stdin(Stdio::piped());
                    let full_code = format!("{}\n{}", prelude, code);
                    (c, Some(full_code))
                }
            }
            AhkSource::File { path } => {
                let ahk_dir = Path::new(&spec.vars.instance_folder).join(".nucleus_ahk");
                let _ = fs::create_dir_all(&ahk_dir);
                let staged_file = ahk_dir.join(format!("{}_wrapper.ahk", spec.id));
                let user_code = fs::read_to_string(path)?;
                let full_code = format!("{}\n{}", prelude, user_code);
                fs::write(&staged_file, &full_code)?;

                let mut c = Command::new(&runtime_exe);
                c.arg("/ErrorStdOut").arg(&staged_file);
                if let Some(parent) = path.parent() {
                    c.current_dir(parent);
                }
                (c, None)
            }
        };

        for arg in &spec.extra_args {
            cmd.arg(arg);
        }

        cmd.stderr(Stdio::piped());

        tracing::info!(
            "AhkManager: Spawning AutoHotkey script '{}' with {:?}",
            spec.id,
            runtime_exe
        );

        let mut child = cmd.spawn().map_err(|e| {
            NucleusError::Process(format!(
                "Failed to spawn AutoHotkey runtime at {:?}: {}",
                runtime_exe, e
            ))
        })?;

        if let Some(input) = pipe_input {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(input.as_bytes());
            }
        }

        let pid = child.id();
        let mut active = self
            .active_scripts
            .lock()
            .map_err(|e| NucleusError::Platform(format!("Lock poisoned: {}", e)))?;

        active.insert(
            spec.id.clone(),
            ActiveAhkScript {
                id: spec.id.clone(),
                child,
                pid,
            },
        );

        Ok(pid)
    }

    /// Stops a specific running AutoHotkey script by ID.
    pub fn stop_by_id(&self, script_id: &str) -> NucleusResult<()> {
        let mut active = self
            .active_scripts
            .lock()
            .map_err(|e| NucleusError::Platform(format!("Lock poisoned: {}", e)))?;

        if let Some(mut script) = active.remove(script_id) {
            let _ = script.child.kill();
            let _ = script.child.wait();
        }
        Ok(())
    }

    /// Terminates all currently active AutoHotkey scripts.
    pub fn stop_all(&self) {
        if let Ok(mut active) = self.active_scripts.lock() {
            for (_id, mut script) in active.drain() {
                tracing::info!(
                    "AhkManager: Terminating AHK script '{}' (PID {})",
                    script.id,
                    script.pid
                );
                let _ = script.child.kill();
                let _ = script.child.wait();
            }
        }
    }
}

impl Drop for AhkManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_detection() {
        assert_eq!(
            AhkManager::detect_version("#Requires AutoHotkey v2.0\nMsgBox 'hi'", AhkVersion::Auto),
            AhkVersion::V2
        );
        assert_eq!(
            AhkManager::detect_version("#Requires AutoHotkey v1.1\nMsgBox, hi", AhkVersion::Auto),
            AhkVersion::V1
        );
        assert_eq!(
            AhkManager::detect_version("MsgBox, legacy v1 comma syntax", AhkVersion::Auto),
            AhkVersion::V1
        );
        assert_eq!(
            AhkManager::detect_version("f() => 42", AhkVersion::Auto),
            AhkVersion::V2
        );
    }

    #[test]
    fn test_prelude_generation_syntax_and_escaping() {
        let vars = AhkPreludeVars {
            player_index: 1,
            player_id: 2,
            nickname: "Player \"Co-op\" 2".to_string(),
            process_id: 9999,
            window_handle: 0x1234,
            pos_x: 0,
            pos_y: 540,
            width: 1920,
            height: 540,
            instance_folder: "C:\\Games\\Halo \"Test\"\\Instance2".to_string(),
            root_install_folder: "C:\\Games\\Halo \"Test\"".to_string(),
            handler_folder: "handlers\\halo".to_string(),
            is_keyboard_player: true,
            gamepad_id: -1,
        };

        let prelude_v2 = AhkManager::render_prelude(AhkVersion::V2, &vars);
        assert!(prelude_v2.contains("global Nucleus_PlayerIndex := 1"));
        assert!(prelude_v2.contains("global Nucleus_Nickname := \"Player `\"Co-op`\" 2\""));
        assert!(prelude_v2.contains(
            "global Nucleus_InstanceFolder := \"C:\\Games\\Halo `\"Test`\"\\Instance2\""
        ));
        assert!(prelude_v2.contains("#SingleInstance Off"));

        let prelude_v1 = AhkManager::render_prelude(AhkVersion::V1, &vars);
        assert!(prelude_v1.contains("Nucleus_PlayerIndex := 1"));
        assert!(!prelude_v1.contains("global Nucleus_PlayerIndex"));
    }

    #[test]
    fn test_path_traversal_blocked() {
        let temp = std::env::temp_dir();
        let res =
            AhkManager::resolve_handler_ahk_file(&temp, &temp, "../../Windows/System32/cmd.ahk");
        assert!(res.is_err());
    }
}
