//! Real-time Inter-Process Communication (Named Pipes and Shared Memory).

pub mod named_pipe;
pub mod shared_memory;

pub use named_pipe::{send_pipe_message_sync, IpcMessage, NamedPipeConnection, NamedPipeIpcServer};
pub use shared_memory::SharedMemorySegment;
