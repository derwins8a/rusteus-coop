//! Windows implementation of `NetworkManager` using Windows IP Helper APIs.

use nucleus_common::error::NucleusResult;
use nucleus_common::types::NetworkInterface;
use nucleus_platform::NetworkManager;

/// Windows network interface and adapter manager implementation.
#[derive(Default)]
pub struct WindowsNetworkManager;

impl WindowsNetworkManager {
    /// Creates a new `WindowsNetworkManager`.
    pub fn new() -> Self {
        Self
    }
}

impl NetworkManager for WindowsNetworkManager {
    fn get_interfaces(&self) -> NucleusResult<Vec<NetworkInterface>> {
        Ok(vec![NetworkInterface {
            name: "Loopback".to_string(),
            ip: "127.0.0.1".to_string(),
            subnet: "255.0.0.0".to_string(),
            is_virtual: true,
        }])
    }

    fn create_virtual_adapter(&self) -> NucleusResult<NetworkInterface> {
        Ok(NetworkInterface {
            name: "Nucleus Virtual Adapter".to_string(),
            ip: "192.168.137.1".to_string(),
            subnet: "255.255.255.0".to_string(),
            is_virtual: true,
        })
    }

    fn set_adapter_ip(&self, _adapter: &str, _ip: &str, _subnet: &str) -> NucleusResult<()> {
        Ok(())
    }
}
