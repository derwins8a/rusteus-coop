//! Debug protocol messages, pipeline stages, breakpoints, and runtime inspection models.

use serde::{Deserialize, Serialize};

/// High-level lifecycle pipeline stages in Nucleus Co-op.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PipelineStage {
    /// Parsing and executing top-level handler JavaScript.
    ScriptEvaluate,
    /// Populating and collecting custom UI options (`Game.AddOption`).
    OptionsUi,
    /// Preparing instance folders, symlinks, Goldberg emulator files.
    PrepareInstances,
    /// Pre-launch mutex discovery and termination sweep.
    MutexKillSweep,
    /// Launching player child processes.
    ProcessSpawn,
    /// Polling for game main window HWNDs.
    HwndPoll,
    /// Applying borderless window styles and computed viewport boundaries.
    ApplyBounds,
    /// Runtime session heartbeat loop (repositioning, focus management).
    HeartbeatTick,
    /// Terminating processes, restoring registry and display settings.
    Teardown,
}

impl PipelineStage {
    /// Human-readable label for the UI step timeline.
    pub fn display_label(self) -> &'static str {
        match self {
            Self::ScriptEvaluate => "1. Script Evaluation",
            Self::OptionsUi => "2. User Options",
            Self::PrepareInstances => "3. Instance Preparation",
            Self::MutexKillSweep => "4. Mutex Sweep",
            Self::ProcessSpawn => "5. Process Launch",
            Self::HwndPoll => "6. HWND Discovery",
            Self::ApplyBounds => "7. Viewport Bounds",
            Self::HeartbeatTick => "8. Session Heartbeat",
            Self::Teardown => "9. Session Teardown",
        }
    }
}

/// Execution mode when launching a session through the debugger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DebugLaunchMode {
    /// Standard execution launching real processes and applying real hooks.
    #[default]
    Normal,
    /// Dry-run simulation: intercepts child process spawning and HWND polling, simulating success.
    DryRun,
    /// Forces launch of only a single instance (player 0) regardless of selected layout.
    SingleInstance,
    /// Mocks controller inputs using synthetic virtual gamepads.
    VirtualGamepads,
}

/// Source breakpoint definition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Breakpoint {
    pub file_id: String,
    pub line: u32,
    #[serde(default)]
    pub column: Option<u32>,
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub hit_count: u32,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl Breakpoint {
    pub fn new(file_id: impl Into<String>, line: u32) -> Self {
        Self {
            file_id: file_id.into(),
            line,
            column: None,
            condition: None,
            hit_count: 0,
            enabled: true,
        }
    }
}

/// Reason the debugger paused execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PauseReason {
    Breakpoint,
    Step,
    PipelineStageBreak(PipelineStage),
    Exception(String),
    UserPause,
}

/// Categorization of log or console output events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputCategory {
    Stdout,
    Stderr,
    DebugConsole,
    LifecycleStage,
}

/// A captured call stack frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackFrameInfo {
    pub frame_id: usize,
    pub function_name: String,
    pub file_id: String,
    pub line: u32,
    pub column: u32,
}

/// Hierarchical representation of an inspected variable or object property.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InspectValue {
    pub name: String,
    pub type_name: String,
    pub summary: String,
    #[serde(default)]
    pub children: Vec<InspectValue>,
}

/// Snapshot of the runtime state at a pause point.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeSnapshot {
    pub stage: PipelineStage,
    pub paused_at: Option<(String, u32, u32)>,
    pub stack_frames: Vec<StackFrameInfo>,
    pub variables: Vec<InspectValue>,
}

/// Commands sent from the UI or client to control the debug engine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugCommand {
    /// Resumes execution until the next breakpoint or pause condition.
    Continue,
    /// Steps over the next statement at the current call depth.
    StepOver,
    /// Steps into the next statement or function call.
    StepInto,
    /// Steps out of the current function call to the caller frame.
    StepOut,
    /// Pauses execution immediately at the next executed statement.
    Pause,
    /// Adds or updates a breakpoint.
    SetBreakpoint(Breakpoint),
    /// Removes a breakpoint at the given location.
    RemoveBreakpoint { file_id: String, line: u32 },
    /// Clears all breakpoints.
    ClearAllBreakpoints,
    /// Evaluates a JavaScript expression within the current pause scope.
    EvaluateExpression { expression: String, frame_id: usize },
    /// Aborts and terminates the debug session.
    Abort,
}

/// Events dispatched by the debug engine to the UI or client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugEvent {
    /// Execution is paused.
    Paused {
        reason: PauseReason,
        file_id: String,
        line: u32,
        column: u32,
        snapshot: RuntimeSnapshot,
    },
    /// Execution has resumed.
    Resumed,
    /// Active lifecycle stage changed.
    StageChanged(PipelineStage),
    /// Log or console output emitted.
    Output {
        category: OutputCategory,
        message: String,
    },
    /// Result of an interactive expression evaluation.
    EvalResult {
        expression: String,
        value: String,
        is_error: bool,
    },
    /// Debug session terminated.
    Terminated { exit_code: i32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_serde_roundtrip() {
        let cmd = DebugCommand::SetBreakpoint(Breakpoint::new("handler.js", 42));
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: DebugCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(cmd, decoded);

        let evt = DebugEvent::StageChanged(PipelineStage::PrepareInstances);
        let json_evt = serde_json::to_string(&evt).unwrap();
        let decoded_evt: DebugEvent = serde_json::from_str(&json_evt).unwrap();
        assert_eq!(evt, decoded_evt);
    }
}
