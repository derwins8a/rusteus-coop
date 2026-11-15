//! Windows Data Protection API (DPAPI) credential and secret storage.

use nucleus_common::error::{NucleusError, NucleusResult};
use nucleus_platform::CredentialStore;
use std::fs;
use std::path::PathBuf;

#[cfg(windows)]
use windows::Win32::Foundation::LocalFree;
#[cfg(windows)]
use windows::Win32::Security::Cryptography::{
    CRYPT_INTEGER_BLOB, CRYPTPROTECT_UI_FORBIDDEN, CryptProtectData, CryptUnprotectData,
};

/// RAII wrapper for LocalFree on Win32 memory pointers allocated by DPAPI.
#[cfg(windows)]
struct LocalAllocGuard(*mut u8);

#[cfg(windows)]
impl Drop for LocalAllocGuard {
    fn drop(&mut self) {
        if !self.0.is_null() {
            // SAFETY: self.0 was allocated by CryptProtectData / CryptUnprotectData and is freed once.
            unsafe {
                let _ = LocalFree(windows::Win32::Foundation::HLOCAL(self.0 as *mut _));
            }
        }
    }
}

/// Windows DPAPI-backed credential store persisting encrypted secrets under `%APPDATA%\NucleusCoop\secrets`.
pub struct WindowsDpapiCredentialStore {
    storage_dir: PathBuf,
}

impl WindowsDpapiCredentialStore {
    /// Creates a new `WindowsDpapiCredentialStore` using the default application data directory.
    pub fn new() -> Self {
        let storage_dir = if let Some(globals) = nucleus_common::Globals::try_get() {
            globals.app_data_path.join("secrets")
        } else if let Some(dirs) = dirs::data_dir() {
            dirs.join("NucleusCoop").join("secrets")
        } else {
            PathBuf::from("content").join("secrets")
        };
        Self::with_dir(storage_dir)
    }

    /// Creates a `WindowsDpapiCredentialStore` with a custom storage directory (useful for testing).
    pub fn with_dir(storage_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&storage_dir);
        Self { storage_dir }
    }

    /// Derives a deterministic, safe filesystem entry name from `(service, account)` using a SHA-256 hash.
    fn entry_path(&self, service: &str, account: &str) -> PathBuf {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(service.as_bytes());
        hasher.update(b"::");
        hasher.update(account.as_bytes());
        let hex_hash = hex::encode(hasher.finalize());
        self.storage_dir.join(format!("{}.dpapi", hex_hash))
    }
}

impl Default for WindowsDpapiCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CredentialStore for WindowsDpapiCredentialStore {
    #[cfg(windows)]
    fn save_secret(&self, service: &str, account: &str, secret: &[u8]) -> NucleusResult<()> {
        let _ = fs::create_dir_all(&self.storage_dir);
        let path = self.entry_path(service, account);

        let mut data_in = CRYPT_INTEGER_BLOB {
            cbData: secret.len() as u32,
            pbData: secret.as_ptr() as *mut u8,
        };
        let mut data_out = CRYPT_INTEGER_BLOB::default();

        // SAFETY: data_in points to a valid slice for secret.len() bytes; data_out receives allocated ciphertext.
        unsafe {
            CryptProtectData(
                &mut data_in,
                windows::core::PCWSTR::null(),
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut data_out,
            )
            .map_err(|e| NucleusError::Platform(format!("CryptProtectData failed: {}", e)))?;
        }

        let _guard = LocalAllocGuard(data_out.pbData);
        let cipher_slice =
            unsafe { std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize) };

        fs::write(&path, cipher_slice)?;
        Ok(())
    }

    #[cfg(not(windows))]
    fn save_secret(&self, service: &str, account: &str, secret: &[u8]) -> NucleusResult<()> {
        let _ = fs::create_dir_all(&self.storage_dir);
        let path = self.entry_path(service, account);
        fs::write(&path, secret)?;
        Ok(())
    }

    #[cfg(windows)]
    fn load_secret(&self, service: &str, account: &str) -> NucleusResult<Option<Vec<u8>>> {
        let path = self.entry_path(service, account);
        if !path.exists() {
            return Ok(None);
        }

        let cipher_bytes = fs::read(&path)?;
        if cipher_bytes.is_empty() {
            return Ok(None);
        }

        let mut data_in = CRYPT_INTEGER_BLOB {
            cbData: cipher_bytes.len() as u32,
            pbData: cipher_bytes.as_ptr() as *mut u8,
        };
        let mut data_out = CRYPT_INTEGER_BLOB::default();

        // SAFETY: data_in points to read cipher bytes; data_out receives decrypted plaintext.
        unsafe {
            CryptUnprotectData(
                &mut data_in,
                None,
                None,
                None,
                None,
                CRYPTPROTECT_UI_FORBIDDEN,
                &mut data_out,
            )
            .map_err(|e| NucleusError::Platform(format!("CryptUnprotectData failed: {}", e)))?;
        }

        let _guard = LocalAllocGuard(data_out.pbData);
        let plain_slice =
            unsafe { std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize) };

        Ok(Some(plain_slice.to_vec()))
    }

    #[cfg(not(windows))]
    fn load_secret(&self, service: &str, account: &str) -> NucleusResult<Option<Vec<u8>>> {
        let path = self.entry_path(service, account);
        if !path.exists() {
            return Ok(None);
        }
        let bytes = fs::read(&path)?;
        Ok(Some(bytes))
    }

    fn delete_secret(&self, service: &str, account: &str) -> NucleusResult<()> {
        let path = self.entry_path(service, account);
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dpapi_credential_store_roundtrip() {
        let temp_dir =
            std::env::temp_dir().join(format!("nucleus_dpapi_test_{}", std::process::id()));
        let store = WindowsDpapiCredentialStore::with_dir(temp_dir.clone());

        let service = "hub.splitscreen.me";
        let account = "test_user";
        let secret = b"my_super_secret_resume_token_456";

        assert_eq!(store.load_secret(service, account).unwrap(), None);

        store.save_secret(service, account, secret).unwrap();

        // Verify on disk that raw plaintext is NOT present when running on Windows (encrypted by DPAPI)
        let path = store.entry_path(service, account);
        assert!(path.exists());
        #[cfg(windows)]
        {
            let raw_disk = fs::read(&path).unwrap();
            assert_ne!(&raw_disk[..], &secret[..]);
        }

        let loaded = store.load_secret(service, account).unwrap();
        assert_eq!(loaded, Some(secret.to_vec()));

        store.delete_secret(service, account).unwrap();
        assert_eq!(store.load_secret(service, account).unwrap(), None);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
