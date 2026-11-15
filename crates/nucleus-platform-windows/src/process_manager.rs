use std::ffi::c_void;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use windows::Win32::Foundation::{CloseHandle, HANDLE, STILL_ACTIVE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows::Win32::System::Memory::{VirtualProtectEx, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS};
use windows::Win32::System::Threading::{
    CreateProcessW, GetExitCodeProcess, OpenProcess, QueryFullProcessImageNameW, SetPriorityClass,
    SetProcessAffinityMask, TerminateProcess, WaitForSingleObject, ABOVE_NORMAL_PRIORITY_CLASS,
    BELOW_NORMAL_PRIORITY_CLASS, CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT, HIGH_PRIORITY_CLASS,
    IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS, PROCESS_CREATION_FLAGS, PROCESS_INFORMATION,
    PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SET_INFORMATION,
    PROCESS_SUSPEND_RESUME, PROCESS_SYNCHRONIZE, PROCESS_TERMINATE, PROCESS_VM_OPERATION,
    PROCESS_VM_WRITE, REALTIME_PRIORITY_CLASS, STARTUPINFOW,
};

use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_common::types::{ProcessConfig, ProcessHandle, ProcessInfo, ProcessPriority};
use nucleus_platform::ProcessManager;

/// Windows process lifecycle and management implementation.
#[derive(Default)]
pub struct WindowsProcessManager;

impl WindowsProcessManager {
    /// Creates a new `WindowsProcessManager`.
    pub fn new() -> Self {
        Self
    }
}

impl ProcessManager for WindowsProcessManager {
    fn start_process(&self, config: &ProcessConfig) -> NucleusResult<ProcessHandle> {
        self.start_process_extended(config, false)
    }

    fn start_process_extended(&self, config: &ProcessConfig, suspended: bool) -> NucleusResult<ProcessHandle> {
        let exe_str = config.exe_path.to_string_lossy();
        let mut cmd_str = format!("\"{}\"", exe_str);
        for arg in &config.args {
            if arg.contains(' ') || arg.contains('\t') {
                cmd_str.push_str(&format!(" \"{}\"", arg));
            } else {
                cmd_str.push_str(&format!(" {}", arg));
            }
        }

        let mut cmd_wide: Vec<u16> = cmd_str.encode_utf16().chain(std::iter::once(0)).collect();

        let work_dir_buf: Option<Vec<u16>> = if let Some(ref dir) = config.working_dir {
            if !dir.as_os_str().is_empty() {
                Some(dir.as_os_str().encode_wide().chain(std::iter::once(0)).collect())
            } else {
                None
            }
        } else if let Some(parent) = config.exe_path.parent() {
            if !parent.as_os_str().is_empty() {
                Some(parent.as_os_str().encode_wide().chain(std::iter::once(0)).collect())
            } else {
                None
            }
        } else {
            None
        };

        let lp_current_dir = work_dir_buf.as_ref().map(|v| v.as_ptr()).unwrap_or(std::ptr::null());

        // Environment block: double-null terminated block of UTF-16 "KEY=VALUE\0" strings
        let mut env_block: Vec<u16> = Vec::new();
        let mut all_vars: std::collections::BTreeMap<String, String> = std::env::vars().collect();
        for (k, v) in &config.environment {
            all_vars.insert(k.clone(), v.clone());
        }
        for (k, v) in &all_vars {
            let entry = format!("{}={}\0", k, v);
            env_block.extend(entry.encode_utf16());
        }
        env_block.push(0);

        let mut creation_flags = CREATE_UNICODE_ENVIRONMENT;
        if suspended {
            creation_flags |= CREATE_SUSPENDED;
        }

        let mut si = STARTUPINFOW::default();
        si.cb = std::mem::size_of::<STARTUPINFOW>() as u32;
        let mut pi = PROCESS_INFORMATION::default();

        unsafe {
            let res = CreateProcessW(
                None,
                windows::core::PWSTR(cmd_wide.as_mut_ptr()),
                None,
                None,
                false,
                creation_flags,
                Some(env_block.as_ptr() as *const c_void),
                if lp_current_dir.is_null() {
                    windows::core::PCWSTR::null()
                } else {
                    windows::core::PCWSTR(lp_current_dir)
                },
                &si,
                &mut pi,
            );

            if let Err(err) = res {
                return Err(NucleusError::Platform(format!(
                    "CreateProcessW failed for '{}': {}",
                    config.exe_path.display(),
                    err
                )));
            }

            let pid = pi.dwProcessId;
            let _ = CloseHandle(pi.hThread);
            let _ = CloseHandle(pi.hProcess);

            let name = config
                .exe_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "process.exe".to_string());

            Ok(ProcessHandle::new(pid, name))
        }
    }

    fn write_process_memory(&self, pid: u32, address: usize, data: &[u8]) -> NucleusResult<usize> {
        unsafe {
            let handle = OpenProcess(PROCESS_VM_WRITE | PROCESS_VM_OPERATION, false, pid)
                .map_err(|e| NucleusError::Platform(format!("OpenProcess failed: {e}")))?;

            let mut old_protect = PAGE_PROTECTION_FLAGS::default();
            let _ = VirtualProtectEx(
                handle,
                address as *const c_void,
                data.len(),
                PAGE_EXECUTE_READWRITE,
                &mut old_protect,
            );

            let mut bytes_written = 0usize;
            let res = WriteProcessMemory(
                handle,
                address as *const c_void,
                data.as_ptr() as *const c_void,
                data.len(),
                Some(&mut bytes_written),
            );

            let mut dummy = PAGE_PROTECTION_FLAGS::default();
            let _ = VirtualProtectEx(
                handle,
                address as *const c_void,
                data.len(),
                old_protect,
                &mut dummy,
            );

            let _ = CloseHandle(handle);

            res.map_err(|e| NucleusError::Platform(format!("WriteProcessMemory failed: {e}")))?;
            Ok(bytes_written)
        }
    }

    fn kill_process(&self, pid: u32) -> NucleusResult<()> {
        unsafe {
            let handle = OpenProcess(PROCESS_TERMINATE, false, pid)
                .map_err(|e| NucleusError::Platform(e.to_string()))?;
            let res = TerminateProcess(handle, 1);
            let _ = CloseHandle(handle);
            res.map_err(|e| NucleusError::Platform(e.to_string()))?;
        }
        Ok(())
    }

    fn get_process_info(&self, pid: u32) -> NucleusResult<ProcessInfo> {
        unsafe {
            let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                Ok(h) => h,
                Err(_) => {
                    return Ok(ProcessInfo {
                        pid,
                        name: format!("process_{}", pid),
                        exe_path: PathBuf::new(),
                    });
                }
            };

            let mut buf = vec![0u16; 1024];
            let mut size = buf.len() as u32;
            let success = QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_FORMAT(0),
                windows::core::PWSTR(buf.as_mut_ptr()),
                &mut size,
            );
            let _ = CloseHandle(handle);

            if success.is_ok() && size > 0 {
                let path_str = String::from_utf16_lossy(&buf[..size as usize]);
                let p = PathBuf::from(path_str);
                let name = p
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| format!("process_{}", pid));
                Ok(ProcessInfo {
                    pid,
                    name,
                    exe_path: p,
                })
            } else {
                Ok(ProcessInfo {
                    pid,
                    name: format!("process_{}", pid),
                    exe_path: PathBuf::new(),
                })
            }
        }
    }

    fn is_process_running(&self, pid: u32) -> bool {
        unsafe {
            if let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                let mut exit_code = 0u32;
                let res = GetExitCodeProcess(handle, &mut exit_code);
                let _ = CloseHandle(handle);
                if res.is_ok() && exit_code == STILL_ACTIVE.0 as u32 {
                    return true;
                }
            }
        }
        false
    }

    fn set_affinity(&self, pid: u32, mask: u64) -> NucleusResult<()> {
        unsafe {
            let handle = OpenProcess(PROCESS_SET_INFORMATION, false, pid)
                .map_err(|e| NucleusError::Platform(e.to_string()))?;
            let res = SetProcessAffinityMask(handle, mask as usize);
            let _ = CloseHandle(handle);
            res.map_err(|e| NucleusError::Platform(e.to_string()))?;
        }
        Ok(())
    }

    fn set_priority(&self, pid: u32, priority: ProcessPriority) -> NucleusResult<()> {
        let flag: PROCESS_CREATION_FLAGS = match priority {
            ProcessPriority::Idle => IDLE_PRIORITY_CLASS,
            ProcessPriority::BelowNormal => BELOW_NORMAL_PRIORITY_CLASS,
            ProcessPriority::Normal => NORMAL_PRIORITY_CLASS,
            ProcessPriority::AboveNormal => ABOVE_NORMAL_PRIORITY_CLASS,
            ProcessPriority::High => HIGH_PRIORITY_CLASS,
            ProcessPriority::RealTime => REALTIME_PRIORITY_CLASS,
        };

        unsafe {
            let handle = OpenProcess(PROCESS_SET_INFORMATION, false, pid)
                .map_err(|e| NucleusError::Platform(e.to_string()))?;
            let res = SetPriorityClass(handle, flag);
            let _ = CloseHandle(handle);
            res.map_err(|e| NucleusError::Platform(e.to_string()))?;
        }
        Ok(())
    }

    fn get_processes_by_name(&self, name: &str) -> NucleusResult<Vec<ProcessInfo>> {
        let mut results = Vec::new();
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)
                .map_err(|e| NucleusError::Platform(e.to_string()))?;

            let mut entry = PROCESSENTRY32W::default();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(snapshot, &mut entry).is_ok() {
                loop {
                    let null_pos = entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len());
                    let proc_name = String::from_utf16_lossy(&entry.szExeFile[..null_pos]);

                    let target_clean = name.trim_end_matches(".exe");
                    let current_clean = proc_name.trim_end_matches(".exe");

                    if current_clean.eq_ignore_ascii_case(target_clean) || proc_name.eq_ignore_ascii_case(name) {
                        results.push(ProcessInfo {
                            pid: entry.th32ProcessID,
                            name: proc_name,
                            exe_path: PathBuf::new(),
                        });
                    }

                    if Process32NextW(snapshot, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(snapshot);
        }
        Ok(results)
    }

    fn wait_for_process(&self, pid: u32, timeout_ms: u32) -> NucleusResult<bool> {
        unsafe {
            let handle = OpenProcess(PROCESS_SYNCHRONIZE, false, pid)
                .map_err(|e| NucleusError::Platform(e.to_string()))?;
            let wait_res = WaitForSingleObject(handle, timeout_ms);
            let _ = CloseHandle(handle);

            if wait_res == WAIT_OBJECT_0 {
                Ok(true)
            } else if wait_res == WAIT_TIMEOUT {
                Ok(false)
            } else {
                Err(NucleusError::Platform("WaitForSingleObject failed".to_string()))
            }
        }
    }

    fn suspend_process(&self, pid: u32) -> NucleusResult<()> {
        unsafe { call_nt_process(pid, windows::core::s!("NtSuspendProcess"), "NtSuspendProcess") }
    }

    fn resume_process(&self, pid: u32) -> NucleusResult<()> {
        unsafe { call_nt_process(pid, windows::core::s!("NtResumeProcess"), "NtResumeProcess") }
    }
}

unsafe fn call_nt_process(pid: u32, proc_name: windows::core::PCSTR, name_str: &str) -> NucleusResult<()> {
    unsafe {
        let handle = OpenProcess(PROCESS_SUSPEND_RESUME, false, pid)
            .map_err(|e| NucleusError::Platform(format!("OpenProcess failed: {e}")))?;

        let ntdll = match GetModuleHandleA(windows::core::s!("ntdll.dll")) {
            Ok(h) => h,
            Err(e) => {
                let _ = CloseHandle(handle);
                return Err(NucleusError::Platform(format!("Failed to get ntdll handle: {e}")));
            }
        };

        let proc = GetProcAddress(ntdll, proc_name);
        let status = match proc {
            Some(p) => {
                let nt_fn: unsafe extern "system" fn(HANDLE) -> i32 = std::mem::transmute(p);
                nt_fn(handle)
            }
            None => {
                let _ = CloseHandle(handle);
                return Err(NucleusError::Platform(format!("{name_str} not found in ntdll.dll")));
            }
        };

        let _ = CloseHandle(handle);
        if status == 0 {
            Ok(())
        } else {
            Err(NucleusError::Platform(format!(
                "{name_str} failed with NTSTATUS: 0x{:08X}",
                status as u32
            )))
        }
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn test_process_info_and_enumeration() {
        let mgr = WindowsProcessManager::new();
        let our_pid = std::process::id();

        assert!(mgr.is_process_running(our_pid));

        let info = mgr.get_process_info(our_pid).unwrap();
        assert_eq!(info.pid, our_pid);
        assert!(!info.name.is_empty());

        let procs = mgr.get_processes_by_name("cargo").unwrap_or_default();
        println!("Found cargo processes: {:?}", procs);
    }

    #[test]
    fn test_start_process_extended_suspended_and_resume() {
        let mgr = WindowsProcessManager::new();
        let mut config = ProcessConfig::default();
        config.exe_path = PathBuf::from("cmd.exe");
        config.args = vec!["/c".to_string(), "timeout".to_string(), "2".to_string()];

        let handle = mgr.start_process_extended(&config, true).expect("Failed to start process suspended");
        assert!(mgr.is_process_running(handle.pid));

        // Test writing process memory
        let test_bytes = [0x90u8; 4];
        let write_res = mgr.write_process_memory(handle.pid, 0x10000, &test_bytes);
        // Address might or might not be mapped in cmd.exe, so error or success is acceptable
        let _ = write_res;

        // Resume and wait
        mgr.resume_process(handle.pid).expect("Failed to resume process");
        let _ = mgr.kill_process(handle.pid);
    }
}


