use nucleus_common::dll_injection::{DllInjectionConfig, TargetArchitecture};
use nucleus_hook::injector::DllInjector;
use std::path::Path;

#[test]
fn test_inject_nonexistent_dll_fails_cleanly() {
    let dummy_path = Path::new("C:\\nonexistent\\missing_library_12345.dll");
    let current_pid = std::process::id();
    let res = DllInjector::inject_dll(current_pid, dummy_path);
    assert!(res.is_err());
}

#[test]
fn test_inject_advanced_nonexistent_fails() {
    let cfg = DllInjectionConfig::new("C:\\nonexistent\\invalid.dll");
    let current_pid = std::process::id();
    let res = DllInjector::inject_advanced(current_pid, &cfg);
    assert!(res.is_err());
}

#[test]
fn test_inject_advanced_architecture_mismatch_rejected_before_allocation() {
    // Attempting to inject a known 64-bit DLL (kernel32.dll) into an invalid bitness
    // or passing an explicit invalid architecture configuration
    let mut cfg = DllInjectionConfig::new("C:\\Windows\\System32\\kernel32.dll");
    cfg.architecture = TargetArchitecture::X86; // Force wrong architecture expectation
    let current_pid = std::process::id();
    if cfg!(target_arch = "x86_64") {
        let res = DllInjector::inject_advanced(current_pid, &cfg);
        // It must either reject due to configuration architecture mismatch or succeed if Auto
        assert!(res.is_err(), "Expected architecture mismatch error");
    }
}
