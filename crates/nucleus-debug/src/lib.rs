//! Nucleus Co-op Interactive Handler Runtime Debugger.
//!
//! Provides AST statement instrumentation, breakpoint management, thread-safe stepping
//! controllers, variable diffing, dry-run simulation, and lifecycle pipeline hooks.

pub mod controller;
pub mod inspector;
pub mod instrument;
pub mod protocol;

pub use controller::{DebugController, StepMode};
pub use inspector::{DiffKind, StateInspector, VariableDiff};
pub use instrument::{InstrumentedScript, JsInstrumentor};
pub use protocol::{
    Breakpoint, DebugCommand, DebugEvent, DebugLaunchMode, InspectValue, OutputCategory,
    PauseReason, PipelineStage, RuntimeSnapshot, StackFrameInfo,
};
