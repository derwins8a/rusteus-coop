//! Low-level process hooking, DLL injection, and input interception for Nucleus Coop.
//!
//! Provides the core primitives to inject DLLs into game instances, configure ProtoInput
//! hooks for multiple input devices, and maintain simulated window focus.

pub mod focus_hook;
pub mod injector;
pub mod proto_input;

pub use focus_hook::FocusHook;
pub use injector::DllInjector;
pub use proto_input::{
    guid_bytes_to_dinput_fields, guid_to_dinput_fields, DrawFakeCursor, InjectMethod,
    MoveWindowHook, PlayerProtoInputConfig, ProcessBitness, ProtoHookIDs, ProtoInput,
    ProtoInputConfig, ProtoInputLoader, ProtoInputMessageFilterIDs, ProtoInputRuntime,
    ProtoInputUtil, ProtoMessageFilterIDs, PutMouseInsideWindow, SetRemoveBorderHook,
    SetWindowPosHook,
};
