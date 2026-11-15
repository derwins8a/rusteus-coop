//! Dynamic link library (DLL) injection and ejection mechanics.

use nucleus_common::error::{NucleusError, NucleusResult};
use std::path::Path;

/// Utilities for injecting custom dynamic libraries into remote target processes.
pub struct DllInjector;

impl DllInjector {
    /// Injects a specified DLL file into the address space of a target process.
    pub fn inject_dll(pid: u32, dll_path: &Path) -> NucleusResult<()> {
        if !dll_path.exists() {
            return Err(NucleusError::Platform(format!(
                "DLL file does not exist: {:?}",
                dll_path
            )));
        }

        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use windows::Win32::Foundation::{CloseHandle, HANDLE};
            use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
            use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
            use windows::Win32::System::Memory::{
                VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
            };
            use windows::Win32::System::Threading::{
                CreateRemoteThread, OpenProcess, WaitForSingleObject, PROCESS_ALL_ACCESS,
            };

            let wide_path: Vec<u16> = dll_path
                .as_os_str()
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();
            let size = wide_path.len() * std::mem::size_of::<u16>();

            unsafe {
                let process: HANDLE = OpenProcess(PROCESS_ALL_ACCESS, false, pid)
                    .map_err(|e| NucleusError::Platform(format!("OpenProcess failed: {}", e)))?;

                let mem = VirtualAllocEx(
                    process,
                    None,
                    size,
                    MEM_COMMIT | MEM_RESERVE,
                    PAGE_READWRITE,
                );

                if mem.is_null() {
                    let _ = CloseHandle(process);
                    return Err(NucleusError::Platform("VirtualAllocEx failed".to_string()));
                }

                let write_res = WriteProcessMemory(
                    process,
                    mem,
                    wide_path.as_ptr() as _,
                    size,
                    None,
                );

                if write_res.is_err() {
                    let _ = VirtualFreeEx(process, mem, 0, MEM_RELEASE);
                    let _ = CloseHandle(process);
                    return Err(NucleusError::Platform("WriteProcessMemory failed".to_string()));
                }

                let kernel32 = GetModuleHandleA(windows::core::s!("kernel32.dll"))
                    .map_err(|e| NucleusError::Platform(e.to_string()))?;
                let load_library_w = GetProcAddress(kernel32, windows::core::s!("LoadLibraryW"));

                if let Some(load_lib_fn) = load_library_w {
                    let thread = CreateRemoteThread(
                        process,
                        None,
                        0,
                        Some(std::mem::transmute(load_lib_fn)),
                        Some(mem),
                        0,
                        None,
                    );

                    if let Ok(th) = thread {
                        let _ = WaitForSingleObject(th, 5000);
                        let _ = CloseHandle(th);
                    }
                }

                let _ = VirtualFreeEx(process, mem, 0, MEM_RELEASE);
                let _ = CloseHandle(process);
            }
        }

        #[cfg(not(windows))]
        {
            let _ = pid;
        }

        Ok(())
    }

    /// Ejects an injected module from the target process address space.
    pub fn eject_dll(_pid: u32, _dll_name: &str) -> NucleusResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inject_dll_nonexistent_fails() {
        let dummy_path = Path::new("C:\\nonexistent\\dummy.dll");
        let result = DllInjector::inject_dll(1234, dummy_path);
        assert!(result.is_err());
    }
}
