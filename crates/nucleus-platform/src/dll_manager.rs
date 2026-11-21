//! Pure abstract traits for dynamic library injection and remote process memory manipulation.

use nucleus_common::dll_injection::DllInjectionConfig;
use nucleus_common::error::NucleusResult;

/// Trait defining platform operations for injecting DLLs and invoking remote exports.
pub trait DllInjectionManager: Send + Sync {
    /// Injects a dynamic library into the remote target process specified by `pid`.
    ///
    /// Returns the remote module base address (`HMODULE` represented as `usize`) upon success.
    fn inject_dll(&self, pid: u32, config: &DllInjectionConfig) -> NucleusResult<usize>;

    /// Executes an exported function in a loaded module inside the remote target process.
    ///
    /// # Parameters
    /// - `pid`: Target process ID.
    /// - `module_handle`: Base address of the loaded module.
    /// - `export`: Name of the exported function symbol.
    /// - `arg`: Optional string argument or serialized payload passed to the exported function.
    ///
    /// Returns the remote thread exit code / function return value.
    fn call_export(
        &self,
        pid: u32,
        module_handle: usize,
        export: &str,
        arg: Option<&str>,
    ) -> NucleusResult<u32>;

    /// Ejects a previously injected dynamic library from the target process.
    fn eject_dll(&self, pid: u32, module_handle: usize) -> NucleusResult<()>;
}

/// Trait defining platform operations for reading, writing, and scanning virtual memory of a target process.
pub trait ProcessMemoryManager: Send + Sync {
    /// Reads a slice of bytes from a remote process's virtual memory space.
    fn read_memory(&self, pid: u32, address: usize, buffer: &mut [u8]) -> NucleusResult<usize>;

    /// Writes a slice of bytes into a remote process's virtual memory space.
    fn write_memory(&self, pid: u32, address: usize, data: &[u8]) -> NucleusResult<usize>;

    /// Changes the memory protection attributes of a region in the remote process.
    ///
    /// Returns the previous memory protection value.
    fn virtual_protect(
        &self,
        pid: u32,
        address: usize,
        size: usize,
        new_protect: u32,
    ) -> NucleusResult<u32>;
}
