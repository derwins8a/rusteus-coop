use nucleus_platform::ProcessManager;
use nucleus_platform_windows::WindowsProcessManager;
use std::process::Command;

#[test]
fn test_suspend_and_resume_process_lifecycle() {
    let mgr = WindowsProcessManager::new();
    let mut child = Command::new("cmd.exe")
        .args(["/c", "pause"])
        .spawn()
        .unwrap();
    let pid = child.id();

    // Suspend should succeed
    assert!(mgr.suspend_process(pid).is_ok());

    // Resume should succeed
    assert!(mgr.resume_process(pid).is_ok());

    // Termination after resume cleans up cleanly
    let _ = child.kill();
}
