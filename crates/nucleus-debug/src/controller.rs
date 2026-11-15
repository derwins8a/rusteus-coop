//! Thread-safe debug controller managing breakpoints, stepping condvars, and lifecycle stages.

use std::collections::HashMap;
use std::sync::{mpsc, Arc, Condvar, Mutex};

use nucleus_common::error::{NucleusError, NucleusResult};
use crate::protocol::{
    Breakpoint, DebugCommand, DebugEvent, DebugLaunchMode, PauseReason,
    PipelineStage, RuntimeSnapshot, StackFrameInfo,
};

/// Stepping modes supported by the execution controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepMode {
    None,
    StepInto,
    StepOver { target_depth: usize },
    StepOut { target_depth: usize },
}

/// Internal shared mutable state of a debug session.
pub struct DebugSessionState {
    pub is_paused: bool,
    pub pause_reason: Option<PauseReason>,
    pub current_file: Option<String>,
    pub current_line: u32,
    pub current_col: u32,
    pub call_depth: usize,
    pub step_mode: StepMode,
    pub breakpoints: HashMap<(String, u32), Breakpoint>,
    pub active_stage: PipelineStage,
    pub launch_mode: DebugLaunchMode,
    pub pending_eval: Option<(String, usize)>,
    pub eval_result: Option<(String, bool)>,
    pub aborted: bool,
}

/// Thread-safe controller coordinating between the engine/script thread and the debugger UI.
#[derive(Clone)]
pub struct DebugController {
    state: Arc<Mutex<DebugSessionState>>,
    condvar: Arc<Condvar>,
    event_tx: Option<mpsc::Sender<DebugEvent>>,
}

impl DebugController {
    /// Creates a new DebugController with the specified launch mode.
    pub fn new(launch_mode: DebugLaunchMode, event_tx: Option<mpsc::Sender<DebugEvent>>) -> Self {
        let state = DebugSessionState {
            is_paused: false,
            pause_reason: None,
            current_file: None,
            current_line: 0,
            current_col: 0,
            call_depth: 0,
            step_mode: StepMode::None,
            breakpoints: HashMap::new(),
            active_stage: PipelineStage::ScriptEvaluate,
            launch_mode,
            pending_eval: None,
            eval_result: None,
            aborted: false,
        };

        Self {
            state: Arc::new(Mutex::new(state)),
            condvar: Arc::new(Condvar::new()),
            event_tx,
        }
    }

    /// Emits a debug event to the UI listener if configured.
    fn emit_event(&self, event: DebugEvent) {
        if let Some(ref tx) = self.event_tx {
            let _ = tx.send(event);
        }
    }

    /// Returns the active launch mode (Normal, DryRun, SingleInstance, etc.).
    pub fn launch_mode(&self) -> DebugLaunchMode {
        self.state.lock().unwrap().launch_mode
    }

    /// Checks if the debugger is configured for dry-run simulation.
    pub fn is_dry_run(&self) -> bool {
        self.launch_mode() == DebugLaunchMode::DryRun
    }

    /// Sets or updates a breakpoint.
    pub fn set_breakpoint(&self, bp: Breakpoint) {
        let key = (bp.file_id.clone(), bp.line);
        self.state.lock().unwrap().breakpoints.insert(key, bp);
    }

    /// Removes a breakpoint at a specific source line.
    pub fn remove_breakpoint(&self, file_id: &str, line: u32) {
        let key = (file_id.to_string(), line);
        self.state.lock().unwrap().breakpoints.remove(&key);
    }

    /// Clears all installed breakpoints.
    pub fn clear_all_breakpoints(&self) {
        self.state.lock().unwrap().breakpoints.clear();
    }

    /// Updates the active lifecycle stage and emits an event.
    pub fn set_stage(&self, stage: PipelineStage) {
        {
            let mut state = self.state.lock().unwrap();
            state.active_stage = stage;
        }
        self.emit_event(DebugEvent::StageChanged(stage));
    }

    /// Tracks entering a function call, incrementing call depth.
    pub fn enter_function(&self) {
        let mut state = self.state.lock().unwrap();
        state.call_depth += 1;
    }

    /// Tracks exiting a function call, decrementing call depth.
    pub fn exit_function(&self) {
        let mut state = self.state.lock().unwrap();
        if state.call_depth > 0 {
            state.call_depth -= 1;
        }
    }

    /// Statement probe invoked by the instrumented script or stage hook.
    /// Evaluates breakpoints and step modes, parking the caller thread if paused.
    pub fn on_statement_probe<F>(
        &self,
        file_id: &str,
        line: u32,
        col: u32,
        mut eval_fn: Option<&mut F>,
    ) -> NucleusResult<()>
    where
        F: FnMut(&str) -> Result<String, String>,
    {
        let mut state = self.state.lock().unwrap();

        if state.aborted {
            return Err(NucleusError::Other("Debug session aborted by user".to_string()));
        }

        state.current_file = Some(file_id.to_string());
        state.current_line = line;
        state.current_col = col;

        // Check if breakpoint matches
        let bp_key = (file_id.to_string(), line);
        let mut should_pause = false;
        let mut pause_reason = PauseReason::Step;

        if let Some(bp) = state.breakpoints.get_mut(&bp_key) {
            if bp.enabled {
                bp.hit_count += 1;
                should_pause = true;
                pause_reason = PauseReason::Breakpoint;
            }
        }

        // Check stepping conditions
        if !should_pause {
            match state.step_mode {
                StepMode::StepInto => {
                    should_pause = true;
                    pause_reason = PauseReason::Step;
                }
                StepMode::StepOver { target_depth } => {
                    if state.call_depth <= target_depth {
                        should_pause = true;
                        pause_reason = PauseReason::Step;
                    }
                }
                StepMode::StepOut { target_depth } => {
                    if state.call_depth < target_depth {
                        should_pause = true;
                        pause_reason = PauseReason::Step;
                    }
                }
                StepMode::None => {}
            }
        }

        if !should_pause {
            return Ok(());
        }

        // Pause execution and notify listeners
        state.is_paused = true;
        state.pause_reason = Some(pause_reason.clone());
        state.step_mode = StepMode::None;

        let snapshot = RuntimeSnapshot {
            stage: state.active_stage,
            paused_at: Some((file_id.to_string(), line, col)),
            stack_frames: vec![StackFrameInfo {
                frame_id: 0,
                function_name: "anonymous".to_string(),
                file_id: file_id.to_string(),
                line,
                column: col,
            }],
            variables: Vec::new(),
        };

        drop(state); // Drop lock before sending event to prevent deadlock

        self.emit_event(DebugEvent::Paused {
            reason: pause_reason,
            file_id: file_id.to_string(),
            line,
            column: col,
            snapshot,
        });

        // Park execution loop until resumed or aborted
        let mut state = self.state.lock().unwrap();
        while state.is_paused && !state.aborted {
            // Check for pending expression evaluation
            if let Some((expr, _frame_id)) = state.pending_eval.take() {
                let (val, is_err) = if let Some(ref mut eval) = eval_fn {
                    match eval(&expr) {
                        Ok(v) => (v, false),
                        Err(e) => (e, true),
                    }
                } else {
                    ("Scope evaluator unavailable".to_string(), true)
                };

                state.eval_result = Some((val.clone(), is_err));
                self.condvar.notify_all();

                drop(state);
                self.emit_event(DebugEvent::EvalResult {
                    expression: expr,
                    value: val,
                    is_error: is_err,
                });
                state = self.state.lock().unwrap();
            }

            state = self.condvar.wait(state).unwrap();
        }

        if state.aborted {
            return Err(NucleusError::Other("Debug session aborted by user".to_string()));
        }

        Ok(())
    }

    /// Dispatches a debug command received from the UI or client.
    pub fn dispatch_command(&self, cmd: DebugCommand) -> NucleusResult<()> {
        let mut state = self.state.lock().unwrap();

        match cmd {
            DebugCommand::Continue => {
                state.is_paused = false;
                state.step_mode = StepMode::None;
                self.condvar.notify_all();
                drop(state);
                self.emit_event(DebugEvent::Resumed);
            }
            DebugCommand::StepInto => {
                state.is_paused = false;
                state.step_mode = StepMode::StepInto;
                self.condvar.notify_all();
                drop(state);
                self.emit_event(DebugEvent::Resumed);
            }
            DebugCommand::StepOver => {
                let depth = state.call_depth;
                state.is_paused = false;
                state.step_mode = StepMode::StepOver { target_depth: depth };
                self.condvar.notify_all();
                drop(state);
                self.emit_event(DebugEvent::Resumed);
            }
            DebugCommand::StepOut => {
                let depth = state.call_depth;
                state.is_paused = false;
                state.step_mode = StepMode::StepOut { target_depth: depth };
                self.condvar.notify_all();
                drop(state);
                self.emit_event(DebugEvent::Resumed);
            }
            DebugCommand::Pause => {
                state.step_mode = StepMode::StepInto;
            }
            DebugCommand::SetBreakpoint(bp) => {
                let key = (bp.file_id.clone(), bp.line);
                state.breakpoints.insert(key, bp);
            }
            DebugCommand::RemoveBreakpoint { file_id, line } => {
                state.breakpoints.remove(&(file_id, line));
            }
            DebugCommand::ClearAllBreakpoints => {
                state.breakpoints.clear();
            }
            DebugCommand::EvaluateExpression { expression, frame_id } => {
                state.pending_eval = Some((expression, frame_id));
                self.condvar.notify_all();
            }
            DebugCommand::Abort => {
                state.aborted = true;
                state.is_paused = false;
                self.condvar.notify_all();
                drop(state);
                self.emit_event(DebugEvent::Terminated { exit_code: -1 });
            }
        }

        Ok(())
    }

    /// Resumes execution from paused state.
    pub fn resume(&self) {
        let _ = self.dispatch_command(DebugCommand::Continue);
    }

    /// Steps over to next statement at or above current call depth.
    pub fn step_over(&self) {
        let _ = self.dispatch_command(DebugCommand::StepOver);
    }

    /// Steps into the next statement or function call.
    pub fn step_into(&self) {
        let _ = self.dispatch_command(DebugCommand::StepInto);
    }

    /// Steps out of the current function call to parent frame.
    pub fn step_out(&self) {
        let _ = self.dispatch_command(DebugCommand::StepOut);
    }

    /// Pauses execution at next statement probe.
    pub fn pause(&self) {
        let _ = self.dispatch_command(DebugCommand::Pause);
    }

    /// Aborts active debug session.
    pub fn abort(&self) {
        let _ = self.dispatch_command(DebugCommand::Abort);
    }

    /// Submits an expression to be evaluated by the paused script engine.
    pub fn evaluate_expression(&self, expr: &str) -> Result<String, String> {
        let _ = self.dispatch_command(DebugCommand::EvaluateExpression {
            expression: expr.to_string(),
            frame_id: 0,
        });
        Ok(format!("Evaluating '{}'...", expr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_controller_stepping_and_continue() {
        let (tx, rx) = mpsc::channel();
        let controller = DebugController::new(DebugLaunchMode::DryRun, Some(tx));

        let ctrl_clone = controller.clone();
        let worker = thread::spawn(move || {
            // Probe line 1 (no bp)
            ctrl_clone.on_statement_probe::<fn(&str) -> Result<String, String>>("handler.js", 1, 1, None).unwrap();

            // Set breakpoint on line 2
            ctrl_clone.set_breakpoint(Breakpoint::new("handler.js", 2));

            // Probe line 2 (should pause)
            ctrl_clone.on_statement_probe::<fn(&str) -> Result<String, String>>("handler.js", 2, 1, None).unwrap();

            // Probe line 3
            ctrl_clone.on_statement_probe::<fn(&str) -> Result<String, String>>("handler.js", 3, 1, None).unwrap();
        });

        // Receive paused event
        let evt = rx.recv_timeout(Duration::from_secs(2)).unwrap();
        match evt {
            DebugEvent::Paused { file_id, line, .. } => {
                assert_eq!(file_id, "handler.js");
                assert_eq!(line, 2);
            }
            _ => panic!("Expected Paused event"),
        }

        // Resume execution
        controller.dispatch_command(DebugCommand::Continue).unwrap();

        worker.join().unwrap();
    }
}
