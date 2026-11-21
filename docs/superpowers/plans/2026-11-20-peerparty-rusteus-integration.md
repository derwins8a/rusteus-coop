# PeerParty Integration Swarm Implementation Plan (Unified DLL Injection Alignment)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Integrate the MaximumFreak/PeerParty multiplayer middleware emulation and BitTorrent DHT P2P mesh layer into Rusteus via a clean Git submodule, automated build pipeline, unified DLL injection infrastructure, and SplitscreenMe JavaScript handler bridge without code duplication or destructive DLL replacement.

**Architecture:** PeerParty is maintained as a decoupled upstream submodule (`vendor/PeerParty`) compiled into 64-bit and 32-bit hook DLLs (`peerparty_hook.dll` / `peerparty_hook32.dll`) staged under `nucleus-coop/utils/PeerParty/`. Rusteus's `nucleus-tools` provides a high-level configuration and environment driver that emits standard `DllInjectionConfig` models with `timing: AtStartupSuspended`. The existing unified `nucleus-hook::DllInjector` (from commit `63c0720`) performs the in-memory injection (with newly added `QueueUserApc` support), `nucleus-script` bridges `Game.UsePeerParty` into `Game.InjectedDlls`, and `nucleus-handler` executes early injection and multi-instance staging via its existing `AtStartupSuspended` lifecycle (commit `c95f63d`).

**Tech Stack:** Rust (Edition 2024), `windows-sys` (0.59), `windows` (0.58), `boa_engine` (0.20), `serde` / `serde_json`, PowerShell pipeline automation, BitTorrent DHT / STUN P2P networking, `sccache` compiler caching, `rtk` command proxying, `headroom` context optimization.

**Spec:** [`docs/superpowers/specs/2026-11-20-peerparty-rusteus-integration-design.md`](file:///c:/Users/Derwins8A/Desktop/NucleusCoopRustRewrite/nucleus-coop/docs/superpowers/specs/2026-11-20-peerparty-rusteus-integration-design.md)

---

## Global Constraints & Performance Invariants

- **Unified Hooking Alignment**: Do NOT build a separate or parallel injector. All DLL injections must flow through `nucleus-hook::DllInjector::inject_advanced` and `nucleus-handler::game_handler.rs:1460-1505`.
- **Zero Blind Code Duplication**: Do NOT copy-paste PeerParty source into `nucleus-coop/crates/`. Upstream lives at `vendor/PeerParty` as a git submodule.
- **Non-Destructive Hooking Invariant**: Never physically overwrite original game DLLs on disk when PeerParty is enabled. Injection must occur in memory via `peerparty_hook.dll`.
- **Zero Stubbing Invariant**: All ported structures, functions, and interfaces must contain genuine, production-grade logic. No `todo!()`, `unimplemented!()`, or dummy empty returns.
- **Compute & Disk Efficiency**:
  - Always enable `sccache` for compilation: `$env:RUSTC_WRAPPER = "sccache"`.
  - Always use `rtk` proxying for shell commands (e.g., `rtk cargo`, `rtk git`) to minimize token consumption.
  - Delete temporary worker target directories (`target_wave_*`) upon completion of each wave to protect disk space.
- **Windows Toolchain Lock Isolation**: Every parallel worker subagent must compile within an isolated target directory: `$env:CARGO_TARGET_DIR = "target_wave_<id>"`.
- **Backward Compatibility**: Existing game handlers using Goldberg (`Game.UseGoldberg = true`) or Nemirtingas (`Game.UseNemirtingas = true`) must remain 100% functional without regressions.

---

## Review Focus

1. **Architecture Bitness Mismatch**: 32-bit WOW64 games injected with 64-bit hook DLL or vice-versa causes immediate target process crash (`STATUS_INVALID_IMAGE_FORMAT`). Verify `PeInspector::verify_compatibility` selects `x64/peerparty_hook.dll` vs `x86/peerparty_hook32.dll`.
2. **Early Injection Synchronization**: Game processes attempting to initialize Steamworks or EOS in `DllMain` before `CreateRemoteThread` runs will fail. Verify that `QueueUserApc` or suspended thread APC runs while target thread is suspended before the main entry point.
3. **Emulator Mutual Exclusion**: If a handler enables both Goldberg and PeerParty for Steam, deploying both `steam_api.dll` (Goldberg) and `peerparty_hook.dll` causes duplicate hook interference. Verify mutual exclusion suppression.
4. **Multi-Instance Environment Collision**: Running 4 local instances must yield 4 distinct player names, LAN IDs, and non-overlapping loopback port allocations.
5. **Path Canonicalization on Windows**: Long paths, junctions, and spaces in game folder paths must not break `PEERPARTY_GAME_ROOT` or `PEERPARTY_SET` environment arguments.

---

## Swarm Waves & File-Ownership Conflict Matrix

```
┌────────────────────────────────────────────────────────────────────────┐
│                      PEERPARTY INTEGRATION SWARM                       │
├────────────────────────────────────────────────────────────────────────┤
│ Wave 1: Submodule Pipeline & Shared Foundation Types                   │
│   Worker 1.1: Git Submodule & Build Automation Scripts                 │
│   Worker 1.2: Foundation Types & Configuration Models in nucleus-tools │
│   [Gate: rtk cargo test -p nucleus-common -p nucleus-tools]            │
├────────────────────────────────────────────────────────────────────────┤
│ Wave 2: Unified Hook Enhancement & Tool Runtime Driver                 │
│   Worker 2.1: nucleus-hook QueueUserApc Support in DllInjector         │
│   Worker 2.2: nucleus-tools::peerparty Driver & DllInjectionConfig Gen │
│   [Gate: rtk cargo test -p nucleus-hook -p nucleus-tools]              │
├────────────────────────────────────────────────────────────────────────┤
│ Wave 3: Handler Bridge & Multi-Instance Staging                        │
│   Worker 3.1: nucleus-script Boa JS Bridge (Game.UsePeerParty)         │
│   Worker 3.2: nucleus-handler Multi-Instance Staging & Startup Wiring  │
│   [Gate: rtk cargo test -p nucleus-script -p nucleus-handler]          │
├────────────────────────────────────────────────────────────────────────┤
│ Wave 4: Engine Session Controller & Profile Settings                   │
│   Worker 4.1: nucleus-profile Settings & GameProfile PeerParty Models  │
│   Worker 4.2: nucleus-engine Session Orchestration & Mutex Isolation  │
│   [Gate: rtk cargo test -p nucleus-profile -p nucleus-engine]          │
├────────────────────────────────────────────────────────────────────────┤
│ Wave 5: UI Configuration Controls & End-to-End Verification Gate       │
│   Worker 5.1: nucleus-ui Splitscreen Setup & Settings View Controls    │
│   Worker 5.2: Workspace Full Suite Regression Verification             │
│   [Gate: rtk cargo test --workspace --exclude nucleus-app]             │
└────────────────────────────────────────────────────────────────────────┘
```

| File | Exclusive Owner | Wave | Responsibilities |
|---|---|---|---|
| `scripts/build_peerparty.ps1` | Worker 1.1 | Wave 1 | Automated submodule checkout, MSVC target builds, artifact staging |
| `scripts/download_peerparty.ps1` | Worker 1.1 | Wave 1 | Release asset fallback fetcher for end-user binary distribution |
| `nucleus-coop/crates/nucleus-tools/src/peerparty_types.rs` | Worker 1.2 | Wave 1 | Service enums, bitmasks, configuration serialization models |
| `nucleus-coop/crates/nucleus-hook/src/injector.rs` | Worker 2.1 | Wave 2 | Add `InjectionMethod::QueueUserApc` to `DllInjector::inject_advanced` |
| `nucleus-coop/crates/nucleus-tools/src/peerparty.rs` | Worker 2.2 | Wave 2 | Utility path discovery, environment generator, `DllInjectionConfig` |
| `nucleus-coop/crates/nucleus-script/src/handler_context.rs` | Worker 3.1 | Wave 3 | Context options, `Context.PeerParty` properties |
| `nucleus-coop/crates/nucleus-script/src/lib.rs` | Worker 3.1 | Wave 3 | `Game.UsePeerParty`, auto-append to `Game.InjectedDlls` |
| `nucleus-coop/crates/nucleus-handler/src/instance.rs` | Worker 3.2 | Wave 3 | Instance folder staging, emulator mutual exclusion rules |
| `nucleus-coop/crates/nucleus-handler/src/launch.rs` | Worker 3.2 | Wave 3 | Verify `AtStartupSuspended` injection through `game_handler.rs:1460` |
| `nucleus-coop/crates/nucleus-profile/src/settings.rs` | Worker 4.1 | Wave 4 | Global PeerParty preferences (room code, DHT public sessions) |
| `nucleus-coop/crates/nucleus-profile/src/game_profile.rs` | Worker 4.1 | Wave 4 | Per-game preferred emulator provider selection |
| `nucleus-coop/crates/nucleus-engine/src/session.rs` | Worker 4.2 | Wave 4 | Session initialization, heartbeat monitoring, clean shutdown |
| `nucleus-coop/crates/nucleus-ui/src/setup_screen.rs` | Worker 5.1 | Wave 5 | Emulator backend picker combobox, Room Code entry field |
| `nucleus-coop/crates/nucleus-ui/src/settings_view.rs` | Worker 5.1 | Wave 5 | PeerParty global preferences toggle and configuration cards |

---

## Detailed Task Specifications

### Wave 1: Submodule Pipeline & Shared Foundation Types

#### Task 1: Git Submodule Setup & Build Automation Scripts
**Files:**
- Create: `scripts/build_peerparty.ps1`
- Create: `scripts/download_peerparty.ps1`
- Modify: `.gitmodules`
- Test: `tests/scripts/test_peerparty_pipeline.ps1`

**Interfaces:**
- Consumes: Upstream Git repo `https://github.com/MaximumFreak/PeerParty.git` (tag: `v0.4.7`)
- Produces: Compiled artifacts in `nucleus-coop/utils/PeerParty/{x64,x86}/peerparty_hook.dll`

- [ ] **Step 1: Write verification test for pipeline output directory structure**
  Verify that scripts produce or validate expected output artifact paths:
  `utils/PeerParty/x64/peerparty_hook.dll`, `utils/PeerParty/x86/peerparty_hook32.dll`, `utils/PeerParty/peerparty.exe`.
- [ ] **Step 2: Add git submodule at `vendor/PeerParty`**
  ```powershell
  git submodule add --force https://github.com/MaximumFreak/PeerParty.git vendor/PeerParty
  git submodule update --init --recursive
  ```
- [ ] **Step 3: Implement `scripts/build_peerparty.ps1`**
  Builds x64 and x86 targets using cargo MSVC targets with `sccache` enabled and copies artifacts to `nucleus-coop/utils/PeerParty/`.
- [ ] **Step 4: Implement `scripts/download_peerparty.ps1`**
  Fetches pre-compiled GitHub release assets with SHA-256 verification when local Rust cross-compiler is unavailable.
- [ ] **Step 5: Verify build automation script executes and stages artifacts**
  Run: `powershell -ExecutionPolicy Bypass -File scripts/download_peerparty.ps1 -VerifyOnly`
  Expected: PASS

---

#### Task 2: Foundation Types & Configuration Models
**Files:**
- Create: `nucleus-coop/crates/nucleus-tools/src/peerparty_types.rs`
- Modify: `nucleus-coop/crates/nucleus-tools/src/lib.rs`
- Test: `nucleus-coop/crates/nucleus-tools/tests/peerparty_types_tests.rs`

**Interfaces:**
- Consumes: `serde`, `serde_json`, `nucleus-common::dll_injection::DllInjectionConfig`
- Produces: `PeerPartyConfig`, `PeerPartyService`, `PeerPartyLaunchMode`

- [ ] **Step 1: Write failing unit test for `PeerPartyConfig` serialization & service flags**
  ```rust
  #[test]
  fn test_peerparty_config_roundtrip() {
      let config = PeerPartyConfig::default()
          .with_service(PeerPartyService::Steamworks)
          .with_service(PeerPartyService::Photon)
          .with_room_code("TESTROOM123");
      let json = serde_json::to_string(&config).unwrap();
      let deserialized: PeerPartyConfig = serde_json::from_str(&json).unwrap();
      assert_eq!(config, deserialized);
  }
  ```
- [ ] **Step 2: Run test to verify it fails**
  Run: `$env:CARGO_TARGET_DIR = "target_wave_1_2"; $env:RUSTC_WRAPPER = "sccache"; rtk cargo test --manifest-path nucleus-coop/Cargo.toml -p nucleus-tools --test peerparty_types_tests`
  Expected: FAIL with module not found
- [ ] **Step 3: Implement `peerparty_types.rs`**
  Implement full enum variants (`Steamworks`, `EpicOnlineServices`, `PlayFab`, `GogGalaxy`, `Photon`, `UnityGamingServices`, `Demonware`, `XboxLive`), flag bitmasks, and conversion to `DllInjectionConfig`.
- [ ] **Step 4: Run test to verify it passes and cleanup target dir**
  Run: `rtk cargo test --manifest-path nucleus-coop/Cargo.toml -p nucleus-tools --test peerparty_types_tests`
  Expected: PASS (1 passed, 0 failed)
  Cleanup: `Remove-Item -Recurse -Force target_wave_1_2 -ErrorAction SilentlyContinue`

---

### Wave 2: Unified Hook Enhancement & Tool Runtime Driver

#### Task 3: Add `QueueUserApc` Support to Unified `DllInjector`
**Files:**
- Modify: `nucleus-coop/crates/nucleus-hook/src/injector.rs`
- Test: `nucleus-coop/crates/nucleus-hook/tests/apc_injection_tests.rs`

**Interfaces:**
- Consumes: `windows::Win32::System::Threading::QueueUserAPC`, `RemoteMemoryGuard`, `PeInspector`
- Produces: Enhanced `DllInjector::inject_advanced` supporting `InjectionMethod::QueueUserApc`

- [ ] **Step 1: Write failing test verifying APC injection method dispatch**
  ```rust
  #[test]
  fn test_injection_method_apc_dispatch() {
      let mut config = DllInjectionConfig::new("dummy.dll".into());
      config.method = InjectionMethod::QueueUserApc;
      // Verifies validation passes and correct injection branch is dispatched
  }
  ```
- [ ] **Step 2: Run test to verify it fails**
  Run: `$env:CARGO_TARGET_DIR = "target_wave_2_1"; $env:RUSTC_WRAPPER = "sccache"; rtk cargo test --manifest-path nucleus-coop/Cargo.toml -p nucleus-hook --test apc_injection_tests`
  Expected: FAIL
- [ ] **Step 3: Implement `QueueUserApc` branch in `injector.rs`**
  When `config.method == InjectionMethod::QueueUserApc`:
  Enumerate the target suspended process thread, allocate memory via `RemoteMemoryGuard`, write DLL path, and call `QueueUserAPC(load_library_w, thread_handle, remote_ptr)`.
- [ ] **Step 4: Run test to verify it passes and cleanup target dir**
  Run: `rtk cargo test --manifest-path nucleus-coop/Cargo.toml -p nucleus-hook --test apc_injection_tests`
  Expected: PASS
  Cleanup: `Remove-Item -Recurse -Force target_wave_2_1 -ErrorAction SilentlyContinue`

---

#### Task 4: Tool Runtime Driver & DllInjectionConfig Generator
**Files:**
- Create: `nucleus-coop/crates/nucleus-tools/src/peerparty.rs`
- Modify: `nucleus-coop/crates/nucleus-tools/src/lib.rs`
- Test: `nucleus-coop/crates/nucleus-tools/tests/peerparty_driver_tests.rs`

**Interfaces:**
- Consumes: `PeerPartyConfig`, `DllInjectionConfig`, `InjectionTiming::AtStartupSuspended`
- Produces: `PeerPartyTool::prepare_instance()`, `PeerPartyTool::get_injection_config()`, `PeerPartyTool::generate_env_vars()`

- [ ] **Step 1: Write failing unit test for `DllInjectionConfig` and environment generation**
  ```rust
  #[test]
  fn test_peerparty_injection_config_generation() {
      let tool = PeerPartyTool::new(PathBuf::from("utils/PeerParty"));
      let inj_cfg = tool.get_injection_config(true).unwrap();
      assert_eq!(inj_cfg.timing, InjectionTiming::AtStartupSuspended);
      assert_eq!(inj_cfg.method, InjectionMethod::QueueUserApc);
  }
  ```
- [ ] **Step 2: Run test to verify it fails**
  Run: `$env:CARGO_TARGET_DIR = "target_wave_2_2"; $env:RUSTC_WRAPPER = "sccache"; rtk cargo test --manifest-path nucleus-coop/Cargo.toml -p nucleus-tools --test peerparty_driver_tests`
  Expected: FAIL
- [ ] **Step 3: Implement `PeerPartyTool` in `nucleus-coop/crates/nucleus-tools/src/peerparty.rs`**
  Implement path resolution for x64/x86 DLLs, generation of `DllInjectionConfig`, `generate_env_vars`, and mutual exclusion checks against Goldberg/Nemirtingas.
- [ ] **Step 4: Run test to verify it passes and cleanup target dir**
  Run: `rtk cargo test --manifest-path nucleus-coop/Cargo.toml -p nucleus-tools --test peerparty_driver_tests`
  Expected: PASS
  Cleanup: `Remove-Item -Recurse -Force target_wave_2_2 -ErrorAction SilentlyContinue`

---

### Wave 3: Handler Bridge & Multi-Instance Staging

#### Task 5: SplitscreenMe JavaScript Handler Bridge
**Files:**
- Modify: `nucleus-coop/crates/nucleus-script/src/handler_context.rs`
- Modify: `nucleus-coop/crates/nucleus-script/src/lib.rs`
- Modify: `nucleus-coop/crates/nucleus-script/src/enums.rs`
- Test: `nucleus-coop/crates/nucleus-script/tests/peerparty_script_tests.rs`

**Interfaces:**
- Consumes: `boa_engine::Context`, `PeerPartyConfig`, `Game.InjectedDlls`
- Produces: JavaScript global `Game.UsePeerParty`, `Game.PeerPartyServices`, `Game.PeerPartyRoom`

- [ ] **Step 1: Write failing test evaluating handler script with `Game.UsePeerParty = true`**
  ```rust
  #[test]
  fn test_handler_script_peerparty_properties() {
      let script = r#"
          Game.UsePeerParty = true;
          Game.PeerPartyServices = ["Steam", "Photon"];
          Game.PeerPartyRoom = "ROOM42";
      "#;
      let engine = ScriptEngine::new().unwrap();
      let handler = engine.evaluate_handler(script).unwrap();
      assert!(handler.use_peerparty);
      assert_eq!(handler.peerparty_room, Some("ROOM42".to_string()));
      assert!(handler.injected_dlls.iter().any(|d| d.path.contains("peerparty_hook.dll")));
  }
  ```
- [ ] **Step 2: Run test to verify it fails**
  Run: `$env:CARGO_TARGET_DIR = "target_wave_3_1"; $env:RUSTC_WRAPPER = "sccache"; rtk cargo test --manifest-path nucleus-coop/Cargo.toml -p nucleus-script --test peerparty_script_tests`
  Expected: FAIL
- [ ] **Step 3: Implement Boa JS property bridging in `nucleus-script`**
  Register `Game.UsePeerParty`, `Game.PeerPartyServices`, `Game.PeerPartyRoom`, `Game.PeerPartyPublic`, and automatically inject the hook DLL into `handler.injected_dlls`.
- [ ] **Step 4: Run test to verify it passes and cleanup target dir**
  Run: `rtk cargo test --manifest-path nucleus-coop/Cargo.toml -p nucleus-script --test peerparty_script_tests`
  Expected: PASS
  Cleanup: `Remove-Item -Recurse -Force target_wave_3_1 -ErrorAction SilentlyContinue`

---

#### Task 6: Multi-Instance Staging & Startup Wiring
**Files:**
- Modify: `nucleus-coop/crates/nucleus-handler/src/instance.rs`
- Modify: `nucleus-coop/crates/nucleus-handler/src/launch.rs`
- Modify: `nucleus-coop/crates/nucleus-handler/src/game_handler.rs`
- Test: `nucleus-coop/crates/nucleus-handler/tests/peerparty_handler_tests.rs`

**Interfaces:**
- Consumes: `PeerPartyTool`, `game_handler.rs:1460-1505` `AtStartupSuspended` lifecycle
- Produces: Orchestrated multi-instance staging with zero file collisions and suspended early injection

- [ ] **Step 1: Write failing test verifying instance staging suppresses Goldberg when PeerParty handles Steam**
  ```rust
  #[test]
  fn test_staging_suppresses_conflicting_emulators() {
      let context = create_test_context_with_peerparty(true, vec![PeerPartyService::Steamworks]);
      let staging = InstanceStager::new(&context);
      assert!(!staging.should_deploy_goldberg());
  }
  ```
- [ ] **Step 2: Run test to verify it fails**
  Run: `$env:CARGO_TARGET_DIR = "target_wave_3_2"; $env:RUSTC_WRAPPER = "sccache"; rtk cargo test --manifest-path nucleus-coop/Cargo.toml -p nucleus-handler --test peerparty_handler_tests`
  Expected: FAIL
- [ ] **Step 3: Implement staging mutual exclusion and verify launch loop**
  In `instance.rs`, skip deploying conflicting Goldberg or Nemirtingas DLLs when PeerParty is active for their services. Confirm `game_handler.rs:1460-1505` picks up the startup DLL.
- [ ] **Step 4: Run test to verify it passes and cleanup target dir**
  Run: `rtk cargo test --manifest-path nucleus-coop/Cargo.toml -p nucleus-handler --test peerparty_handler_tests`
  Expected: PASS
  Cleanup: `Remove-Item -Recurse -Force target_wave_3_2 -ErrorAction SilentlyContinue`

---

### Wave 4: Engine Session Controller & Profile Settings

#### Task 7: Profile Models & Persistent Configuration
**Files:**
- Modify: `nucleus-coop/crates/nucleus-profile/src/settings.rs`
- Modify: `nucleus-coop/crates/nucleus-profile/src/game_profile.rs`
- Test: `nucleus-coop/crates/nucleus-profile/tests/peerparty_profile_tests.rs`

**Interfaces:**
- Consumes: `serde`
- Produces: `AppSettings.peerparty_default_room`, `AppSettings.peerparty_public_sessions`, `GameProfile.preferred_emulator`

- [ ] **Step 1: Write failing test verifying settings migration and persistence**
  ```rust
  #[test]
  fn test_settings_peerparty_defaults() {
      let settings = AppSettings::default();
      assert_eq!(settings.peerparty_public_sessions, false);
      assert_eq!(settings.peerparty_default_room, None);
  }
  ```
- [ ] **Step 2: Run test to verify it fails**
  Run: `$env:CARGO_TARGET_DIR = "target_wave_4_1"; $env:RUSTC_WRAPPER = "sccache"; rtk cargo test --manifest-path nucleus-coop/Cargo.toml -p nucleus-profile --test peerparty_profile_tests`
  Expected: FAIL
- [ ] **Step 3: Implement profile fields and schema migrations in `nucleus-profile`**
  Add settings fields with `#[serde(default)]` backward-compatibility decorators.
- [ ] **Step 4: Run test to verify it passes and cleanup target dir**
  Run: `rtk cargo test --manifest-path nucleus-coop/Cargo.toml -p nucleus-profile --test peerparty_profile_tests`
  Expected: PASS
  Cleanup: `Remove-Item -Recurse -Force target_wave_4_1 -ErrorAction SilentlyContinue`

---

#### Task 8: Engine Session Coordination & Teardown Cleanup
**Files:**
- Modify: `nucleus-coop/crates/nucleus-engine/src/game_manager.rs`
- Modify: `nucleus-coop/crates/nucleus-engine/src/session.rs`
- Test: `nucleus-coop/crates/nucleus-engine/tests/peerparty_engine_tests.rs`

**Interfaces:**
- Consumes: `GameSession`, `PeerPartyTool`
- Produces: Clean multi-instance startup, heartbeat mutex monitoring, and session teardown

- [ ] **Step 1: Write failing test verifying session teardown cleans up PeerParty state**
  ```rust
  #[test]
  fn test_session_cleanup_peerparty() {
      let mut session = GameSession::new(test_game(), test_players());
      session.set_peerparty_active(true);
      session.stop().unwrap();
      assert!(!session.is_active());
  }
  ```
- [ ] **Step 2: Run test to verify it fails**
  Run: `$env:CARGO_TARGET_DIR = "target_wave_4_2"; $env:RUSTC_WRAPPER = "sccache"; rtk cargo test --manifest-path nucleus-coop/Cargo.toml -p nucleus-engine --test peerparty_engine_tests`
  Expected: FAIL
- [ ] **Step 3: Implement session lifecycle orchestration in `nucleus-engine`**
  Wire `PeerPartyTool` initialization into `GameManager::launch_game` and add cleanup routines on session stop.
- [ ] **Step 4: Run test to verify it passes and cleanup target dir**
  Run: `rtk cargo test --manifest-path nucleus-coop/Cargo.toml -p nucleus-engine --test peerparty_engine_tests`
  Expected: PASS
  Cleanup: `Remove-Item -Recurse -Force target_wave_4_2 -ErrorAction SilentlyContinue`

---

### Wave 5: UI Controls & End-to-End Verification Gate

#### Task 9: Splitscreen Setup & Settings UI Controls
**Files:**
- Modify: `nucleus-coop/crates/nucleus-ui/src/setup_screen.rs`
- Modify: `nucleus-coop/crates/nucleus-ui/src/settings_view.rs`
- Test: `nucleus-coop/crates/nucleus-ui/tests/peerparty_ui_tests.rs`

**Interfaces:**
- Consumes: `gpui`, `nucleus-profile::AppSettings`, `GameProfile`
- Produces: UI selector for Emulator Backend and Room Code input

- [ ] **Step 1: Write UI component test checking presence of PeerParty options in setup screen**
  ```rust
  #[test]
  fn test_setup_screen_renders_peerparty_selector() {
      // GPUI test context verifying emulator combobox options
  }
  ```
- [ ] **Step 2: Implement UI controls in `setup_screen.rs` and `settings_view.rs`**
  Add styled dropdowns using `gpui-kit` components for selecting between Goldberg, Nemirtingas, and PeerParty, plus a room code entry field.
- [ ] **Step 3: Run UI test to verify it passes and cleanup target dir**
  Run: `rtk cargo test --manifest-path nucleus-coop/Cargo.toml -p nucleus-ui --test peerparty_ui_tests`
  Expected: PASS
  Cleanup: `Remove-Item -Recurse -Force target_wave_5_1 -ErrorAction SilentlyContinue`

---

#### Task 10: End-to-End Workspace Verification Gate
**Files:**
- Entire workspace

- [ ] **Step 1: Compile all crates in release configuration with sccache**
  Run: `$env:RUSTC_WRAPPER = "sccache"; rtk cargo check --manifest-path nucleus-coop/Cargo.toml --workspace`
  Expected: PASS with 0 errors
- [ ] **Step 2: Execute full workspace unit and integration test suite**
  Run: `rtk cargo test --manifest-path nucleus-coop/Cargo.toml --workspace --exclude nucleus-app`
  Expected: 100% tests PASS across all crates
- [ ] **Step 3: Commit and push full integration changes to live repository**
  ```bash
  rtk git -C nucleus-coop add -A
  rtk git -C nucleus-coop commit -m "feat(peerparty): integrate PeerParty P2P multiplayer middleware emulation"
  rtk git -C nucleus-coop push origin main
  ```
