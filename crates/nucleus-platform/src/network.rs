//! Platform-agnostic network management operations.

use nucleus_common::error::NucleusResult;
use nucleus_common::types::NetworkInterface;

/// Platform-agnostic network management operations (virtual adapters, IP assignment).
pub trait NetworkManager: Send + Sync {
    /// Retrieve all network interfaces on the system.
    fn get_interfaces(&self) -> NucleusResult<Vec<NetworkInterface>>;

    /// Create a virtual network adapter for multi-instance network isolation.
    fn create_virtual_adapter(&self) -> NucleusResult<NetworkInterface>;

    /// Configure the IP address and subnet mask for a network adapter.
    fn set_adapter_ip(&self, adapter: &str, ip: &str, subnet: &str) -> NucleusResult<()>;
}
