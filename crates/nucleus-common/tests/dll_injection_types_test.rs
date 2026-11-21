use nucleus_common::dll_injection::{
    DllInjectionConfig, InjectionMethod, InjectionTiming, IpcChannelKind, TargetArchitecture,
    TargetInstanceFilter,
};

#[test]
fn test_dll_injection_config_json_roundtrip() {
    let cfg = DllInjectionConfig {
        path: "mods/test.dll".to_string(),
        target_instances: TargetInstanceFilter::Specific(vec![0, 1]),
        timing: InjectionTiming::OnWindowCreated,
        delay_ms: 1000,
        architecture: TargetArchitecture::Auto,
        method: InjectionMethod::CreateRemoteThread,
        export_name: Some("InitMod".to_string()),
        export_argument: Some("instance_id=1".to_string()),
        required: true,
        ipc_channel: IpcChannelKind::NamedPipe,
        pipe_name: Some(r"\\.\pipe\TestPipe".to_string()),
        eject_on_close: true,
    };
    let json = serde_json::to_string(&cfg).expect("Failed to serialize DllInjectionConfig");
    let deserialized: DllInjectionConfig =
        serde_json::from_str(&json).expect("Failed to deserialize DllInjectionConfig");
    assert_eq!(cfg, deserialized);
}

#[test]
fn test_dll_injection_defaults_and_filters() {
    let cfg = DllInjectionConfig::new("test.dll");
    assert_eq!(cfg.path, "test.dll");
    assert_eq!(cfg.timing, InjectionTiming::OnWindowCreated);
    assert_eq!(cfg.method, InjectionMethod::CreateRemoteThread);
    assert_eq!(cfg.target_instances, TargetInstanceFilter::All);
    assert_eq!(cfg.ipc_channel, IpcChannelKind::None);
    assert!(!cfg.eject_on_close);

    assert!(TargetInstanceFilter::All.applies_to(0, 4));
    assert!(TargetInstanceFilter::All.applies_to(3, 4));
    assert!(TargetInstanceFilter::First.applies_to(0, 4));
    assert!(!TargetInstanceFilter::First.applies_to(1, 4));
    assert!(!TargetInstanceFilter::ExceptFirst.applies_to(0, 4));
    assert!(TargetInstanceFilter::ExceptFirst.applies_to(1, 4));
    assert!(TargetInstanceFilter::Specific(vec![1, 2]).applies_to(1, 4));
    assert!(!TargetInstanceFilter::Specific(vec![1, 2]).applies_to(0, 4));
}
