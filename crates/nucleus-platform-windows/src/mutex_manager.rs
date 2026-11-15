//! Windows implementation of `MutexManager` using NtQuerySystemInformation to enumerate
//! and kill named kernel mutex objects owned by a process — the core mechanism used by
//! NucleusCoop to allow multiple game instances to coexist.

use nucleus_common::error::NucleusResult;
use nucleus_platform::MutexManager;
use tracing::{debug, info, warn};

#[cfg(windows)]
use windows::Win32::Foundation::CloseHandle;

/// Windows named mutex manipulation manager.
/// Uses NtQuerySystemInformation (SYSTEM_HANDLE_INFORMATION class) to enumerate
/// all kernel handles across the system, find mutex handles belonging to the target
/// process, then duplicate and close them to break the mutex lock.
#[derive(Default)]
pub struct WindowsMutexManager;

impl WindowsMutexManager {
    /// Creates a new `WindowsMutexManager`.
    pub fn new() -> Self {
        Self
    }
}

#[cfg(windows)]
mod win_impl {
    use super::*;
    use std::ffi::c_void;
    use windows::Win32::Foundation::{DuplicateHandle, FALSE, HANDLE};
    use windows::Win32::System::Threading::{
        GetCurrentProcess, OpenProcess, PROCESS_DUP_HANDLE, PROCESS_QUERY_INFORMATION,
    };

    // NT status / NTSTATUS typedef
    type NTSTATUS = i32;
    const STATUS_INFO_LENGTH_MISMATCH: NTSTATUS = 0xC0000004_u32 as i32;
    const SYSTEM_HANDLE_INFORMATION_CLASS: u32 = 16;

    /// Raw SYSTEM_HANDLE_TABLE_ENTRY_INFO structure from ntdll
    #[repr(C)]
    #[allow(non_snake_case, dead_code)]
    struct SystemHandleEntry {
        UniqueProcessId: u16,
        CreatorBackTraceIndex: u16,
        ObjectTypeIndex: u8,
        HandleAttributes: u8,
        HandleValue: u16,
        Object: *mut c_void,
        GrantedAccess: u32,
    }

    /// SYSTEM_HANDLE_INFORMATION buffer header
    #[repr(C)]
    #[allow(non_snake_case)]
    struct SystemHandleInformation {
        NumberOfHandles: u32,
        _reserved: u32,
        // Entries follow immediately after — accessed via pointer arithmetic
    }

    #[allow(non_snake_case)]
    unsafe extern "system" {
        fn NtQuerySystemInformation(
            SystemInformationClass: u32,
            SystemInformation: *mut c_void,
            SystemInformationLength: u32,
            ReturnLength: *mut u32,
        ) -> NTSTATUS;

        fn NtQueryObject(
            Handle: HANDLE,
            ObjectInformationClass: u32,
            ObjectInformation: *mut c_void,
            ObjectInformationLength: u32,
            ReturnLength: *mut u32,
        ) -> NTSTATUS;
    }

    #[repr(C)]
    #[allow(non_snake_case)]
    struct UnicodeString {
        Length: u16,
        MaximumLength: u16,
        Buffer: *mut u16,
    }

    /// PUBLIC_OBJECT_NAME_INFORMATION (ObjectNameInformation = 1)
    #[repr(C)]
    #[allow(non_snake_case)]
    struct ObjectNameInformation {
        Name: UnicodeString,
        // Name.Buffer points to WCHAR data following the struct
    }

    /// PUBLIC_OBJECT_TYPE_INFORMATION (ObjectTypeInformation = 2)
    #[repr(C)]
    #[allow(non_snake_case)]
    struct ObjectTypeInformation {
        TypeName: UnicodeString,
    }

    use std::collections::HashMap;
    use std::sync::RwLock;

    /// Read a wide string from a UNICODE_STRING buffer embedded in a byte slice.
    unsafe fn read_unicode_string(us: &UnicodeString, _buf: &[u8]) -> Option<String> {
        if us.Buffer.is_null() || us.Length == 0 {
            return None;
        }
        let char_count = (us.Length / 2) as usize;
        let slice = unsafe { std::slice::from_raw_parts(us.Buffer, char_count) };
        Some(String::from_utf16_lossy(slice))
    }

    /// Dynamic kernel object type index cache mapping type names ("mutant", "semaphore", "event")
    /// to their system ObjectTypeIndex.
    static TYPE_INDEX_CACHE: std::sync::LazyLock<RwLock<HashMap<String, u8>>> =
        std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

    /// Enumerate all system handles of the specified kernel type for a given PID, returning (handle_value, name) pairs.
    pub fn enumerate_handles_for_pid(target_pid: u32, expected_type: &str) -> Vec<(u16, String)> {
        let mut result = Vec::new();
        let expected_type_norm = if expected_type.is_empty() {
            "Mutant".to_string()
        } else {
            expected_type.to_string()
        };

        // Check if we already have the type index cached for this object type
        let cached_type_idx = {
            let cache = TYPE_INDEX_CACHE.read().unwrap();
            cache.get(&expected_type_norm.to_ascii_lowercase()).copied()
        };

        // Grow buffer until NtQuerySystemInformation succeeds
        let mut buf_len: u32 = 1 << 20; // start with 1 MB
        let mut buf: Vec<u8>;
        let status;
        loop {
            buf = vec![0u8; buf_len as usize];
            let mut ret_len: u32 = 0;
            let s = unsafe {
                NtQuerySystemInformation(
                    SYSTEM_HANDLE_INFORMATION_CLASS,
                    buf.as_mut_ptr() as *mut c_void,
                    buf_len,
                    &mut ret_len,
                )
            };
            if s == STATUS_INFO_LENGTH_MISMATCH {
                buf_len = (ret_len + 0x10000).max(buf_len * 2);
                continue;
            }
            status = s;
            break;
        }

        if status != 0 {
            warn!("NtQuerySystemInformation failed: 0x{:08X}", status as u32);
            return result;
        }

        let info = unsafe { &*(buf.as_ptr() as *const SystemHandleInformation) };
        let count = info.NumberOfHandles as usize;
        let entries_ptr = unsafe {
            (buf.as_ptr() as *const SystemHandleInformation).add(1) as *const SystemHandleEntry
        };

        // Open target process for handle duplication
        let proc = unsafe {
            match OpenProcess(
                PROCESS_DUP_HANDLE | PROCESS_QUERY_INFORMATION,
                FALSE,
                target_pid,
            ) {
                Ok(h) => h,
                Err(e) => {
                    warn!("OpenProcess({}) failed: {}", target_pid, e);
                    return result;
                }
            }
        };

        let current_proc = unsafe { GetCurrentProcess() };

        for i in 0..count {
            let entry = unsafe { std::ptr::read_unaligned(entries_ptr.add(i)) };

            if entry.UniqueProcessId as u32 != target_pid {
                continue;
            }

            // Fast path: If type index is cached, filter out non-matching handles immediately
            if let Some(type_idx) = cached_type_idx {
                if entry.ObjectTypeIndex != type_idx {
                    continue;
                }
            }

            // Duplicate handle into our process so we can query its type and name
            let mut dup_handle = HANDLE::default();
            let dup_ok = unsafe {
                DuplicateHandle(
                    proc,
                    HANDLE(entry.HandleValue as *mut c_void),
                    current_proc,
                    &mut dup_handle,
                    0,
                    FALSE,
                    windows::Win32::Foundation::DUPLICATE_SAME_ACCESS,
                )
            };

            if dup_ok.is_err() {
                continue;
            }

            // If not cached, query ObjectTypeInformation to discover and cache type index
            if cached_type_idx.is_none() {
                let mut type_buf = vec![0u8; 1024];
                let mut ret_len: u32 = 0;
                let type_status = unsafe {
                    NtQueryObject(
                        dup_handle,
                        2, // ObjectTypeInformation
                        type_buf.as_mut_ptr() as *mut c_void,
                        type_buf.len() as u32,
                        &mut ret_len,
                    )
                };

                if type_status != 0 {
                    let _ = unsafe { CloseHandle(dup_handle) };
                    continue;
                }

                let type_info = unsafe { &*(type_buf.as_ptr() as *const ObjectTypeInformation) };
                let type_name = unsafe { read_unicode_string(&type_info.TypeName, &type_buf) };

                let is_match = match type_name.as_deref() {
                    Some(t) => t.eq_ignore_ascii_case(&expected_type_norm),
                    None => false,
                };

                if !is_match {
                    let _ = unsafe { CloseHandle(dup_handle) };
                    continue;
                }

                if let Some(ref t) = type_name {
                    let mut cache = TYPE_INDEX_CACHE.write().unwrap();
                    cache.insert(t.to_ascii_lowercase(), entry.ObjectTypeIndex);
                    debug!(
                        "Discovered kernel {} ObjectTypeIndex: {}",
                        t, entry.ObjectTypeIndex
                    );
                }
            }

            // Query ObjectNameInformation safely
            let mut name_buf = vec![0u8; 1024];
            let mut name_ret_len: u32 = 0;
            let name_status = unsafe {
                NtQueryObject(
                    dup_handle,
                    1, // ObjectNameInformation
                    name_buf.as_mut_ptr() as *mut c_void,
                    name_buf.len() as u32,
                    &mut name_ret_len,
                )
            };

            // Close duplicated handle
            let _ = unsafe { CloseHandle(dup_handle) };

            if name_status != 0 {
                continue;
            }

            let name_info = unsafe { &*(name_buf.as_ptr() as *const ObjectNameInformation) };
            if let Some(name) = unsafe { read_unicode_string(&name_info.Name, &name_buf) } {
                if !name.is_empty() {
                    result.push((entry.HandleValue, name));
                }
            }
        }

        let _ = unsafe { CloseHandle(proc) };
        result
    }

    /// Kill (close) a specific handle in the target process identified by handle value.
    pub fn close_handle_in_process(target_pid: u32, handle_value: u16) -> bool {
        unsafe {
            let proc = match OpenProcess(PROCESS_DUP_HANDLE, FALSE, target_pid) {
                Ok(h) => h,
                Err(_) => return false,
            };
            let current_proc = GetCurrentProcess();
            let mut dup = HANDLE::default();

            let result = DuplicateHandle(
                proc,
                HANDLE(handle_value as *mut c_void),
                current_proc,
                &mut dup,
                0,
                FALSE,
                windows::Win32::Foundation::DUPLICATE_CLOSE_SOURCE,
            );

            let _ = CloseHandle(proc);
            if result.is_ok() {
                let _ = CloseHandle(dup);
                true
            } else {
                false
            }
        }
    }
}

/// Helper to parse optional `type:<TypeName>|<MutexName>` prefix.
/// If found, returns `(parsed_type, parsed_name)`. Otherwise returns `(default_type, name)`.
pub fn parse_type_and_name<'a>(name: &'a str, default_type: &'a str) -> (&'a str, &'a str) {
    let lower = name.to_ascii_lowercase();
    if lower.starts_with("type:") {
        if let Some(pipe_pos) = name.find('|') {
            if pipe_pos >= 5 {
                let parsed_type = &name[5..pipe_pos];
                let parsed_name = &name[pipe_pos + 1..];
                let dtype = if parsed_type.is_empty() {
                    if default_type.is_empty() {
                        "Mutant"
                    } else {
                        default_type
                    }
                } else {
                    parsed_type
                };
                return (dtype, parsed_name);
            }
        }
    }
    (
        if default_type.is_empty() {
            "Mutant"
        } else {
            default_type
        },
        name,
    )
}

/// Helper to match handle object name against target name.
/// When `partial == false`, matches if `local_name` equals or `object_name` ends with `target_name`.
/// When `partial == true`, matches if `local_name` or `object_name` contains `target_name`.
pub fn is_name_match(object_name: &str, target_name: &str, partial: bool) -> bool {
    let local_name = object_name
        .trim_start_matches('\\')
        .rsplit('\\')
        .next()
        .unwrap_or(object_name);

    if partial {
        local_name
            .to_lowercase()
            .contains(&target_name.to_lowercase())
            || object_name
                .to_lowercase()
                .contains(&target_name.to_lowercase())
    } else {
        local_name.eq_ignore_ascii_case(target_name)
            || object_name.eq_ignore_ascii_case(target_name)
            || object_name
                .to_lowercase()
                .ends_with(&target_name.to_lowercase())
    }
}

impl MutexManager for WindowsMutexManager {
    fn kill_mutex(&self, pid: u32, mutex_name: &str, mutex_type: &str) -> NucleusResult<bool> {
        self.kill_mutex_ext(pid, mutex_name, mutex_type, false)
    }

    fn kill_mutex_ext(
        &self,
        pid: u32,
        mutex_name: &str,
        mutex_type: &str,
        partial: bool,
    ) -> NucleusResult<bool> {
        #[cfg(windows)]
        {
            let (target_type, target_name) = parse_type_and_name(mutex_name, mutex_type);
            info!(
                "MutexManager: killing mutex '{}' (type: {}, partial: {}) for PID {}",
                target_name, target_type, partial, pid
            );

            // Up to 3 attempts with 150ms sleep between attempts (parity with C# ProcessUtil.cs:220)
            for attempt in 1..=3 {
                debug!("MutexManager: attempt {}/3 for PID {}", attempt, pid);
                let all_handles = win_impl::enumerate_handles_for_pid(pid, target_type);
                let mut killed = false;

                for (handle_val, name) in &all_handles {
                    if is_name_match(name, target_name, partial) {
                        debug!(
                            "MutexManager: found {} '{}' (handle 0x{:X}), closing...",
                            target_type, name, handle_val
                        );
                        if win_impl::close_handle_in_process(pid, *handle_val) {
                            info!(
                                "MutexManager: killed {} '{}' successfully",
                                target_type, name
                            );
                            killed = true;
                        } else {
                            warn!(
                                "MutexManager: failed to close handle for {} '{}'",
                                target_type, name
                            );
                        }
                    }
                }

                if killed {
                    return Ok(true);
                }

                if attempt < 3 {
                    std::thread::sleep(std::time::Duration::from_millis(150));
                }
            }

            Ok(false)
        }
        #[cfg(not(windows))]
        {
            let _ = (pid, mutex_name, mutex_type, partial);
            Ok(false)
        }
    }

    fn rename_mutex(&self, pid: u32, old_name: &str, new_name: &str) -> NucleusResult<bool> {
        let (target_type, target_name) = parse_type_and_name(old_name, "Mutant");
        info!(
            "MutexManager: rename_mutex '{}' -> '{}' for PID {} (killing original)",
            old_name, new_name, pid
        );
        self.kill_mutex_ext(pid, target_name, target_type, false)
    }

    fn find_mutex(&self, pid: u32, pattern: &str, partial: bool) -> NucleusResult<Vec<String>> {
        #[cfg(windows)]
        {
            let (target_type, target_name) = parse_type_and_name(pattern, "Mutant");
            let all_handles = win_impl::enumerate_handles_for_pid(pid, target_type);
            let matched: Vec<String> = all_handles
                .into_iter()
                .filter_map(|(_, name)| {
                    if is_name_match(&name, target_name, partial) {
                        Some(name)
                    } else {
                        None
                    }
                })
                .collect();
            Ok(matched)
        }
        #[cfg(not(windows))]
        {
            let _ = (pid, pattern, partial);
            Ok(vec![])
        }
    }
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    #[test]
    fn test_mutex_manager_creation() {
        let mgr = WindowsMutexManager::new();
        // find_mutex on PID 0 should not crash, just return empty or error
        let result = mgr.find_mutex(0, "NonExistentMutex", false);
        // Either Ok(empty) or an error — just ensure it doesn't panic
        match result {
            Ok(v) => assert!(v.is_empty()),
            Err(_) => {} // acceptable
        }
    }

    #[test]
    fn test_kill_nonexistent_mutex_returns_false() {
        let mgr = WindowsMutexManager::new();
        // Kill a mutex on our own PID that doesn't exist — should return Ok(false)
        let our_pid = std::process::id();
        let result = mgr.kill_mutex(our_pid, "Nucleus_NonExistentMutex_XYZ_12345", "Mutant");
        assert!(matches!(result, Ok(false)));
    }

    #[test]
    fn test_find_and_kill_real_mutex() {
        use windows::Win32::Foundation::CloseHandle;
        use windows::Win32::System::Threading::CreateMutexW;
        use windows::core::HSTRING;

        let mutex_name = format!("NucleusTestMutex_{}", std::process::id());
        let h_name = HSTRING::from(&mutex_name);
        let handle =
            unsafe { CreateMutexW(None, windows::Win32::Foundation::FALSE, &h_name) }.unwrap();
        assert!(!handle.is_invalid());

        let mgr = WindowsMutexManager::new();
        let our_pid = std::process::id();

        println!("Created test mutex '{}' for PID {}", mutex_name, our_pid);
        let found = mgr.find_mutex(our_pid, &mutex_name, true).unwrap();
        println!("Found mutexes: {:?}", found);

        let killed = mgr.kill_mutex(our_pid, &mutex_name, "Mutant").unwrap();
        println!("Killed result: {}", killed);

        let _ = unsafe { CloseHandle(handle) };
        assert!(
            killed,
            "Should have successfully found and killed the created mutex"
        );
    }

    #[test]
    fn test_parse_type_and_name() {
        assert_eq!(
            parse_type_and_name("MyMutex", "Mutant"),
            ("Mutant", "MyMutex")
        );
        assert_eq!(
            parse_type_and_name("type:Event|MyEvent", "Mutant"),
            ("Event", "MyEvent")
        );
        assert_eq!(
            parse_type_and_name("TYPE:Semaphore|MySem", "Mutant"),
            ("Semaphore", "MySem")
        );
        assert_eq!(
            parse_type_and_name("type:Mutant|hl2_singleton_mutex", "Mutant"),
            ("Mutant", "hl2_singleton_mutex")
        );
    }

    #[test]
    fn test_is_name_match() {
        let full = r"\Sessions\1\BaseNamedObjects\hl2_singleton_mutex";
        // Exact / EndsWith
        assert!(is_name_match(full, "hl2_singleton_mutex", false));
        assert!(is_name_match(
            full,
            r"\BaseNamedObjects\hl2_singleton_mutex",
            false
        ));
        assert!(!is_name_match(full, "hl2_singleton", false));

        // Partial / Contains
        assert!(is_name_match(full, "hl2_singleton", true));
        assert!(is_name_match(full, "SINGLETON", true));
        assert!(!is_name_match(full, "portal2", true));
    }
}
