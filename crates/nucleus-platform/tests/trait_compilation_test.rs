use nucleus_common::dll_injection::DllInjectionConfig;
use nucleus_common::error::NucleusResult;
use nucleus_platform::{DllInjectionManager, ProcessMemoryManager};

struct MockDllManager;

impl DllInjectionManager for MockDllManager {
    fn inject_dll(&self, _pid: u32, _config: &DllInjectionConfig) -> NucleusResult<usize> {
        Ok(0x10000000)
    }

    fn call_export(
        &self,
        _pid: u32,
        _module_handle: usize,
        _export: &str,
        _arg: Option<&str>,
    ) -> NucleusResult<u32> {
        Ok(0)
    }

    fn eject_dll(&self, _pid: u32, _module_handle: usize) -> NucleusResult<()> {
        Ok(())
    }
}

struct MockMemoryManager;

impl ProcessMemoryManager for MockMemoryManager {
    fn read_memory(&self, _pid: u32, _address: usize, buffer: &mut [u8]) -> NucleusResult<usize> {
        buffer.fill(0x90);
        Ok(buffer.len())
    }

    fn write_memory(&self, _pid: u32, _address: usize, data: &[u8]) -> NucleusResult<usize> {
        Ok(data.len())
    }

    fn virtual_protect(
        &self,
        _pid: u32,
        _address: usize,
        _size: usize,
        _new_protect: u32,
    ) -> NucleusResult<u32> {
        Ok(0x40) // PAGE_EXECUTE_READWRITE
    }
}

#[test]
fn test_mock_dll_and_memory_managers_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MockDllManager>();
    assert_send_sync::<MockMemoryManager>();

    let dll_mgr: Box<dyn DllInjectionManager> = Box::new(MockDllManager);
    let cfg = DllInjectionConfig::new("mod.dll");
    let base = dll_mgr.inject_dll(1234, &cfg).unwrap();
    assert_eq!(base, 0x10000000);
    assert_eq!(dll_mgr.call_export(1234, base, "Init", None).unwrap(), 0);
    assert!(dll_mgr.eject_dll(1234, base).is_ok());

    let mem_mgr: Box<dyn ProcessMemoryManager> = Box::new(MockMemoryManager);
    let mut buf = [0u8; 4];
    assert_eq!(mem_mgr.read_memory(1234, 0x2000, &mut buf).unwrap(), 4);
    assert_eq!(buf, [0x90, 0x90, 0x90, 0x90]);
}
