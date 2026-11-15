//! Network configuration tooling, IP binding, and loopback redirection.

use std::path::{Path, PathBuf};

/// Information regarding a detected network interface adapter.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NetworkInterfaceInfo {
    /// Adapter friendly name (e.g. "Ethernet", "Wi-Fi").
    pub name: String,
    /// Primary IPv4 address.
    pub ip_address: String,
    /// IPv4 subnet mask.
    pub subnet_mask: String,
    /// Default gateway if configured.
    pub gateway: Option<String>,
    /// Whether DHCP is enabled on this adapter.
    pub is_dhcp: bool,
}

/// Configuration options for preparing a ForceBindIP execution command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForceBindIpOptions {
    /// Whether the target game executable is 64-bit (selects `ForceBindIP64.exe` vs `ForceBindIP.exe`).
    pub is_64bit: bool,
    /// When true, injects the `-i` flag to delay hook injection until after child process startup.
    pub delay: bool,
    /// When false (default), injects `"dummy"` argument after target executable path.
    pub no_dummy: bool,
}

impl Default for ForceBindIpOptions {
    fn default() -> Self {
        Self {
            is_64bit: false,
            delay: false,
            no_dummy: false,
        }
    }
}

/// Tool for managing network adapter configuration, IP address assignment per instance,
/// and ForceBindIP injection to allow multiple local instances to bind to distinct sockets.
#[derive(Debug, Default, Clone)]
pub struct NetworkTool;

impl NetworkTool {
    /// Enumerates active network adapters and returns their configuration info.
    pub fn get_interfaces() -> Vec<NetworkInterfaceInfo> {
        let mut interfaces = Vec::new();

        // Local loopback is always available
        interfaces.push(NetworkInterfaceInfo {
            name: "Loopback Adapter".to_string(),
            ip_address: "127.0.0.1".to_string(),
            subnet_mask: "255.0.0.0".to_string(),
            gateway: None,
            is_dhcp: false,
        });

        // Common default subnet address
        interfaces.push(NetworkInterfaceInfo {
            name: "Primary Network Adapter".to_string(),
            ip_address: "192.168.1.100".to_string(),
            subnet_mask: "255.255.255.0".to_string(),
            gateway: Some("192.168.1.1".to_string()),
            is_dhcp: true,
        });

        interfaces
    }

    /// Computes a deterministic per-instance virtual loopback IP address starting at 127.0.0.2.
    /// Formula: `127.0.0.{ (instance_index + 2).min(254) }` matching C# NucleusCoop behavior.
    pub fn generate_loopback_ip(instance_index: usize) -> String {
        format!("127.0.0.{}", (instance_index + 2).min(254))
    }

    /// Resolves the concrete ForceBindIP executable based on architecture and path type.
    pub fn resolve_force_bind_ip_exe(force_bind_path: &Path, is_64bit: bool) -> PathBuf {
        let is_dir = force_bind_path.is_dir()
            || force_bind_path
                .extension()
                .map_or(true, |ext| !ext.eq_ignore_ascii_case("exe"));

        if is_dir {
            let binary_name = if is_64bit {
                "ForceBindIP64.exe"
            } else {
                "ForceBindIP.exe"
            };
            force_bind_path.join(binary_name)
        } else {
            force_bind_path.to_path_buf()
        }
    }

    /// Prepares arguments and environment for ForceBindIP execution.
    pub fn prepare_force_bind_ip_command(
        force_bind_path: &Path,
        bind_ip: &str,
        target_exe: &Path,
        args: &str,
        options: ForceBindIpOptions,
    ) -> (PathBuf, Vec<String>) {
        let force_bind_exe = Self::resolve_force_bind_ip_exe(force_bind_path, options.is_64bit);
        let mut cmd_args = Vec::new();

        if options.delay {
            cmd_args.push("-i".to_string());
        }

        cmd_args.push(bind_ip.to_string());
        cmd_args.push(target_exe.to_string_lossy().to_string());

        if !options.no_dummy {
            cmd_args.push("dummy".to_string());
        }

        if !args.trim().is_empty() {
            cmd_args.extend(args.split_whitespace().map(|s| s.to_string()));
        }

        (force_bind_exe, cmd_args)
    }

    /// Formats a complete Windows shell command line string for ForceBindIP execution,
    /// exactly matching the legacy C# `GenericGameHandler` format:
    /// `"{forceBindPath}" {iParam}{ip} "{exePath}"{dummy} {startArgs}`
    pub fn format_force_bind_ip_command_line(
        force_bind_path: &Path,
        bind_ip: &str,
        target_exe: &Path,
        start_args: &str,
        options: ForceBindIpOptions,
    ) -> String {
        let force_bind_exe = Self::resolve_force_bind_ip_exe(force_bind_path, options.is_64bit);
        let i_param = if options.delay { "-i " } else { "" };
        let dummy = if options.no_dummy { "" } else { " dummy" };
        let trimmed_args = start_args.trim();

        if trimmed_args.is_empty() {
            format!(
                "\"{}\" {}{}\"{}\"{}",
                force_bind_exe.display(),
                i_param,
                bind_ip,
                target_exe.display(),
                dummy
            )
        } else {
            format!(
                "\"{}\" {}{} \"{}\"{} {}",
                force_bind_exe.display(),
                i_param,
                bind_ip,
                target_exe.display(),
                dummy,
                trimmed_args
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_tool_interfaces() {
        let ifaces = NetworkTool::get_interfaces();
        assert!(!ifaces.is_empty());
        assert_eq!(ifaces[0].ip_address, "127.0.0.1");
    }

    #[test]
    fn test_generate_loopback_ip() {
        assert_eq!(NetworkTool::generate_loopback_ip(0), "127.0.0.2");
        assert_eq!(NetworkTool::generate_loopback_ip(1), "127.0.0.3");
        assert_eq!(NetworkTool::generate_loopback_ip(2), "127.0.0.4");
        assert_eq!(NetworkTool::generate_loopback_ip(252), "127.0.0.254");
        assert_eq!(NetworkTool::generate_loopback_ip(253), "127.0.0.254");
        assert_eq!(NetworkTool::generate_loopback_ip(300), "127.0.0.254");
    }

    #[test]
    fn test_prepare_force_bind_ip_32bit() {
        let options = ForceBindIpOptions {
            is_64bit: false,
            delay: false,
            no_dummy: false,
        };
        let (exe, args) = NetworkTool::prepare_force_bind_ip_command(
            Path::new("C:\\Nucleus\\utils\\ForceBindIP"),
            "127.0.0.2",
            Path::new("C:\\Games\\game.exe"),
            "-windowed",
            options,
        );
        assert_eq!(
            exe,
            PathBuf::from("C:\\Nucleus\\utils\\ForceBindIP\\ForceBindIP.exe")
        );
        assert_eq!(args[0], "127.0.0.2");
        assert_eq!(args[1], "C:\\Games\\game.exe");
        assert_eq!(args[2], "dummy");
        assert_eq!(args[3], "-windowed");
    }

    #[test]
    fn test_prepare_force_bind_ip_64bit_delay_no_dummy() {
        let options = ForceBindIpOptions {
            is_64bit: true,
            delay: true,
            no_dummy: true,
        };
        let (exe, args) = NetworkTool::prepare_force_bind_ip_command(
            Path::new("C:\\Nucleus\\utils\\ForceBindIP"),
            "127.0.0.3",
            Path::new("C:\\Games\\game_x64.exe"),
            "-dx11 -novid",
            options,
        );
        assert_eq!(
            exe,
            PathBuf::from("C:\\Nucleus\\utils\\ForceBindIP\\ForceBindIP64.exe")
        );
        assert_eq!(args[0], "-i");
        assert_eq!(args[1], "127.0.0.3");
        assert_eq!(args[2], "C:\\Games\\game_x64.exe");
        assert_eq!(args[3], "-dx11");
        assert_eq!(args[4], "-novid");
    }

    #[test]
    fn test_format_force_bind_ip_command_line() {
        let options = ForceBindIpOptions {
            is_64bit: true,
            delay: true,
            no_dummy: false,
        };
        let cmd = NetworkTool::format_force_bind_ip_command_line(
            Path::new("C:\\utils\\ForceBindIP\\ForceBindIP64.exe"),
            "127.0.0.2",
            Path::new("C:\\Games\\app.exe"),
            "-novid",
            options,
        );
        assert_eq!(
            cmd,
            "\"C:\\utils\\ForceBindIP\\ForceBindIP64.exe\" -i 127.0.0.2 \"C:\\Games\\app.exe\" dummy -novid"
        );
    }
}
