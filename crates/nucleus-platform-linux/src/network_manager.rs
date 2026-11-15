//! Linux stub implementation of `NetworkManager`.

#![cfg(target_os = "linux")]

use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_common::types::NetworkInterface;
use nucleus_platform::NetworkManager;

/// Linux network management stub.
#[derive(Default)]
pub struct LinuxNetworkManager;

impl LinuxNetworkManager {
    /// Creates a new `LinuxNetworkManager`.
    pub fn new() -> Self {
        Self
    }
}

impl NetworkManager for LinuxNetworkManager {
    fn get_interfaces(&self) -> NucleusResult<Vec<NetworkInterface>> {
        Err(NucleusError::Unsupported("Network interfaces query not yet implemented on Linux".into()))
    }

    fn create_virtual_adapter(&self) -> NucleusResult<NetworkInterface> {
        Err(NucleusError::Unsupported("Virtual adapter creation not yet implemented on Linux".into()))
    }

    fn set_adapter_ip(&self, _adapter: &str, _ip: &str, _subnet: &str) -> NucleusResult<()> {
        Err(NucleusError::Unsupported("Network adapter configuration not yet implemented on Linux".into()))
    }
}
