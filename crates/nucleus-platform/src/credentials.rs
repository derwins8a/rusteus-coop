//! Platform-agnostic credential and secret storage trait.

use nucleus_common::error::NucleusResult;
use std::collections::HashMap;
use std::sync::RwLock;

/// Platform-agnostic secure secret store interface.
pub trait CredentialStore: Send + Sync {
    /// Save a secret securely under the given service and account names.
    fn save_secret(&self, service: &str, account: &str, secret: &[u8]) -> NucleusResult<()>;

    /// Load a previously saved secret, or return `None` if not found.
    fn load_secret(&self, service: &str, account: &str) -> NucleusResult<Option<Vec<u8>>>;

    /// Delete a secret entry.
    fn delete_secret(&self, service: &str, account: &str) -> NucleusResult<()>;
}

/// In-memory credential store implementation, primarily for testing or headless execution.
#[derive(Debug, Default)]
pub struct InMemoryCredentialStore {
    secrets: RwLock<HashMap<(String, String), Vec<u8>>>,
}

impl InMemoryCredentialStore {
    /// Creates a new empty `InMemoryCredentialStore`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl CredentialStore for InMemoryCredentialStore {
    fn save_secret(&self, service: &str, account: &str, secret: &[u8]) -> NucleusResult<()> {
        let mut map = self.secrets.write().map_err(|e| {
            nucleus_common::error::NucleusError::Platform(format!("Lock poisoned: {}", e))
        })?;
        map.insert((service.to_string(), account.to_string()), secret.to_vec());
        Ok(())
    }

    fn load_secret(&self, service: &str, account: &str) -> NucleusResult<Option<Vec<u8>>> {
        let map = self.secrets.read().map_err(|e| {
            nucleus_common::error::NucleusError::Platform(format!("Lock poisoned: {}", e))
        })?;
        Ok(map
            .get(&(service.to_string(), account.to_string()))
            .cloned())
    }

    fn delete_secret(&self, service: &str, account: &str) -> NucleusResult<()> {
        let mut map = self.secrets.write().map_err(|e| {
            nucleus_common::error::NucleusError::Platform(format!("Lock poisoned: {}", e))
        })?;
        map.remove(&(service.to_string(), account.to_string()));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_credential_store_roundtrip() {
        let store = InMemoryCredentialStore::new();
        assert_eq!(store.load_secret("hub", "user1").unwrap(), None);

        store
            .save_secret("hub", "user1", b"secret-token-123")
            .unwrap();
        assert_eq!(
            store.load_secret("hub", "user1").unwrap(),
            Some(b"secret-token-123".to_vec())
        );

        store.delete_secret("hub", "user1").unwrap();
        assert_eq!(store.load_secret("hub", "user1").unwrap(), None);
    }
}
