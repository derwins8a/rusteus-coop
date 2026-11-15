# Rusteus-Coop — Contributing Guide (Human & LLM-Agent Workflows)

Welcome to the **Rusteus-Coop** developer guide. Whether you are a human software engineer writing idiomatic Rust by hand or an LLM-based autonomous agent executing implementation plans, this document defines the engineering standards, invariants, architectural boundaries, and verification runbooks required to maintain production parity.

---

## 1. Architectural Invariants (Non-Negotiable Red Lines)

Every contribution—whether authored by human or machine—must strictly comply with the following architectural invariants. PRs or patches violating these rules will be rejected:

### Invariant 1: The Zero-Stubbing Policy
- **No Stubs Policy**: Never introduce empty functions, dummy return values (`Ok(())`, `true`, `None`, `""`), placeholder mocks, or `todo!()` / `unimplemented!()` macros into non-scaffold crates.
- Every function, method, or enum match arm must contain real, working, production-ready logic matching or improving upon legacy C# NucleusCoop behavior.

### Invariant 2: Windows Target Directory Isolation
- **The Rule**: When executing `cargo check`, `cargo test`, or `cargo build` on Windows, **ALWAYS** specify an isolated target directory via environment variable:
  ```powershell
  $env:CARGO_TARGET_DIR = "target_<feature_name>"
  cargo test --manifest-path nucleus-coop/Cargo.toml -p <crate>
  ```
- **Why**: GPUI, `boa_engine`, and macro-heavy crates produce massive symbol databases and PDBs during compilation. Sharing a single `target/` directory between concurrent processes or test runs triggers fatal Windows file-lock errors (`Access is denied (os error 5)`).

### Invariant 3: Boa ECMAScript Engine Thread Safety (`!Send`)
- **The Rule**: Never store a persistent `ScriptEngine` or Boa `boa_engine::Context` inside structs that cross thread boundaries (e.g., `GameManager`, `GameSession`, or UI models).
- **Why**: Boa's garbage collector and symbol interner use non-atomic reference-counted `Rc` pointers, making `boa_engine::Context` strictly `!Send`.
- **Implementation**: Store handler scripts as UTF-8 strings (`script_source: Option<String>`). Instantiate short-lived `ScriptEngine` instances on-demand within dedicated worker threads when evaluating handler properties or executing `Game.Play(Context)`.

### Invariant 4: Dynamic Win32 Kernel Mutant Discovery
- **The Rule**: Never hardcode kernel `ObjectTypeIndex` values (e.g. 17).
- **Why**: Modern Windows (10/11 x64) dynamically assigns mutant type indices at system boot (e.g., 38, 41, 44).
- **Implementation**: In `crates/nucleus-platform-windows/src/mutex_manager.rs`, query handles dynamically via `NtQueryObject(ObjectTypeInformation)`, cache the index in `TYPE_INDEX_CACHE`, and terminate handles using `DuplicateHandle(DUPLICATE_CLOSE_SOURCE)`.

### Invariant 5: Filesystem Link Hierarchy (Hardlink Fallback)
- **The Rule**: When staging instance folders (`crates/nucleus-handler/src/instance.rs`), enforce this exact hierarchy:
  1. Attempt symbolic link (`symlink_file`).
  2. If symlinking fails with `ERROR_PRIVILEGE_NOT_HELD` (Windows non-admin without Developer Mode), **ALWAYS** attempt NTFS hardlink (`std::fs::hard_link`).
  3. Fall back to physical copy (`std::fs::copy`) only if hardlinking fails (e.g. cross-volume boundary).
- **Why**: Skipping hardlinks forces gigabytes of physical file copies, turning a 10ms instant launch into a 5-minute stall.

### Invariant 6: STA COM Apartment for Windows Audio Routing
- **The Rule**: All Windows Audio Policy endpoint redirection calls (`IAudioPolicyConfigFactory` & `IAudioPolicyConfigFactoryFor21H2`) must execute inside a dedicated Single-Threaded Apartment (STA) worker thread (`ComThread`). Calling from multi-threaded Tokio pools results in fatal `RPC_E_WRONG_THREAD` failures.

### Invariant 7: GPUI Kit Layout Safety & Vector Icon Policy
- **No Unicode Emojis**: 100% of UI icons must be vector SVG elements or GPUI rendered shapes. No emoji characters (`📂`, `🚀`, `🎮`) in production UI views.
- **Flexbox Safety**: Apply `.flex_shrink_0()` to cards, headers, and option rows in `nucleus-ui` to prevent deep flexbox containers from squishing children down to 0px.
- **Type Unification**: Always unify conditional branches using `.into_any_element()` to prevent GPUI element type mismatches.

---

## 2. Guide for Human Software Engineers

### 2.1 Prerequisites & Toolchain Setup
- **Rust Toolchain**: Rust 2024 edition (Rust 1.85+ recommended).
  ```bash
  rustup default stable
  rustup update
  ```
- **Windows Build Tools**: Visual Studio 2022 C++ Build Tools with Windows 10/11 SDK.
- **Git & GitHub CLI**:
  ```bash
  git clone https://github.com/derwins8a/rusteus-coop.git
  cd rusteus-coop
  ```

### 2.2 Building the Project
Always specify an isolated target directory:
```powershell
# Check workspace compilation
$env:CARGO_TARGET_DIR = "target_dev"
cargo check --workspace -j 4

# Build release executable (outputs to target_dev/release/nucleus-coop.exe)
cargo build --release --bin nucleus-coop
```

### 2.3 Running Automated Tests
> [!WARNING]
> **Anti-Hang Test Rule**: Never run bare `cargo test` or `cargo test -p nucleus-ui --lib` without arguments. GPUI links static wgpu/MinGW window tests that wait indefinitely for window pump messages.

Always execute isolated test suites:
```powershell
$env:CARGO_TARGET_DIR = "target_test"

# 1. Platform-Agnostic Core Tests
cargo test -p nucleus-script --lib -j 4
cargo test -p nucleus-profile --lib -j 4
cargo test -p nucleus-hub --lib -j 4
cargo test -p nucleus-handler --lib -j 4
cargo test -p nucleus-engine --lib -j 4

# 2. Windows Platform & Mutex Killer Tests
cargo test -p nucleus-platform-windows --lib -j 4

# 3. Isolated GPUI UI Integration Tests
cargo test -p nucleus-ui --test components_tests -j 4
cargo test -p nucleus-ui --test carousel_tests -j 4
cargo test -p nucleus-ui --test couch_hub_tests -j 4
cargo test -p nucleus-ui --test couch_setup_tests -j 4
cargo test -p nucleus-ui --test couch_osd_tests -j 4
cargo test -p nucleus-ui --test disconnect_resilience_tests -j 4
cargo test -p nucleus-ui --test gamepad_action_tests -j 4
```

### 2.4 Code Style & Linting
```powershell
# Format code
cargo fmt --all

# Run Clippy checks
cargo clippy --workspace --all-targets -- -D warnings
```

### 2.5 Adding New Features
- **Adding a new handler property**:
  1. Add the field to `HandlerConfig` in `crates/nucleus-script/src/handler_config.rs`.
  2. Expose the getter/setter in Boa context in `crates/nucleus-script/src/engine.rs`.
  3. Implement lifecycle effects in `crates/nucleus-handler/src/game_handler.rs`.
- **Adding a new isolation tool**:
  1. Implement tool under `crates/nucleus-tools/src/<tool_name>.rs`.
  2. Expose tool in `crates/nucleus-tools/src/lib.rs`.
  3. Invoke tool in `GameHandler::prepare_instances()` or `GameHandler::launch_instance()`.

---

## 3. Guide for LLM-Based Autonomous Agents

When delegating tasks or running autonomous LLM coding agents (e.g. Gemini, Claude Code, Goose, Antigravity) on the Rusteus-Coop codebase, adhere to the following operational protocols:

### 3.1 Context & Task Right-Sizing
- **Narrow File Scopes**: Do not ask an agent to "refactor the entire UI layer". Break tasks into disjoint file ownership slices (e.g., Worker 1 owns `components/button.rs` and `components/modal.rs`; Worker 2 owns `settings_view.rs`).
- **Wave-Based Serialization**: If multiple agents run concurrently, arrange them in execution waves where no two parallel agents touch the same file.

### 3.2 Automated Prompt Directives for LLMs
Include the following block in your agent prompts:

```markdown
You are working on the Rusteus-Coop codebase (NucleusCoop Rust Rewrite).
Follow these mandatory engineering invariants:
1. ZERO STUBBING POLICY: No empty functions, todo!(), or dummy returns. Real production logic only.
2. BOA THREAD SAFETY: boa_engine::Context is !Send. Never store ScriptEngine in persistent structs.
3. ISOLATED TARGET DIRS: Always run tests with $env:CARGO_TARGET_DIR = "target_<agent_id>".
4. NO BARE CARGO TEST ON UI: Never execute `cargo test -p nucleus-ui --lib`. Run only isolated integration suites.
5. GPUI KIT SAFETY: Use .flex_shrink_0() on containers, sized radii (rounded_md()), and .into_any_element() on conditional branches. 100% vector icons, ZERO emojis.
6. EVIDENCE BEFORE ASSERTIONS: Run cargo check and isolated tests to confirm zero compiler errors before reporting completion.
```

### 3.3 What Reviewers & Maintainers Reject in Agent Output
Autonomous agents frequently attempt specific shortcuts that maintainers will immediately reject:
1. **Placeholder Stubs**: Writing `// TODO: Implement later` or returning `Ok(())` without executing the underlying Win32 / platform call.
2. **Hardcoded Mutex Types**: Writing `if type_index == 17` instead of using the dynamic `TYPE_INDEX_CACHE`.
3. **Unchecked `unwrap()` on System Calls**: Writing `.unwrap()` on process spawning or handle duplication instead of propagating errors with `NucleusError`.
4. **Emoji Characters in UI Elements**: Adding Unicode icons like 📁 or ⚙️ into GPUI label strings.
5. **Ignoring Hardlink Fallback**: Writing `symlink_file(...)?` directly without catching privilege errors and falling back to `hard_link`.

---

## 4. Verification & Pre-Commit Checklist

Before opening a pull request or merging a feature branch, complete the following verification checklist:

- [ ] Workspace compiles cleanly with `$env:CARGO_TARGET_DIR = "target_verify"; cargo check --workspace -j 4`.
- [ ] All isolated core unit tests pass without errors.
- [ ] All 7 isolated GPUI UI integration suites pass.
- [ ] No Unicode emojis remain in `nucleus-ui/src/` (verify via PowerShell search):
  ```powershell
  Get-ChildItem -Path "nucleus-coop\crates\nucleus-ui\src" -Recurse -Filter "*.rs" |
    Select-String -Pattern "[\u2600-\u27BF]|[\uD800-\uDBFF][\uDC00-\uDFFF]" |
    Where-Object { $_.Line -notmatch "^[ \t]*//" }
  ```
- [ ] `cargo fmt --all -- --check` reports zero formatting deviations.
- [ ] No `todo!()` or `unimplemented!()` stubs exist in non-scaffold crates.
