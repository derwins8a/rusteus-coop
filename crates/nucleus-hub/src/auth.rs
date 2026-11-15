//! Authentication credentials, session tokens, and password hashing for Splitscreen.Me Hub.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zeroize::{Zeroize, Zeroizing};

/// Represents an active authenticated Hub session token.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HubSessionToken {
    /// Remote user ID on the Hub.
    pub user_id: String,
    /// Public username.
    pub username: String,
    /// Authenticated session/resume token string.
    pub token: String,
    /// Expiration timestamp in UTC milliseconds (if known).
    pub token_expires_unix_ms: Option<i64>,
}

/// Login credentials for authentication against the Hub.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct HubAuthCredentials {
    pub identifier: String,
    pub password: Zeroizing<String>,
}

impl HubAuthCredentials {
    /// Creates a new credentials struct.
    pub fn new(identifier: impl Into<String>, password: Zeroizing<String>) -> Self {
        Self {
            identifier: identifier.into(),
            password,
        }
    }
}

/// Formatted SHA-256 password digest object required by Meteor `accounts-password`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeteorPasswordDigest {
    pub digest: String,
    pub algorithm: String,
}

impl MeteorPasswordDigest {
    /// Computes the SHA-256 hex digest of the raw password string.
    pub fn compute(password: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let hex_hash = hex::encode(hasher.finalize());
        Self {
            digest: hex_hash,
            algorithm: "sha-256".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meteor_password_digest_computation() {
        // SHA-256 of "password123":
        // ef92b778bafe771e89245b89ecbc08a44a4e166c06659911881f383d4473e94f
        let digest = MeteorPasswordDigest::compute("password123");
        assert_eq!(digest.algorithm, "sha-256");
        assert_eq!(
            digest.digest,
            "ef92b778bafe771e89245b89ecbc08a44a4e166c06659911881f383d4473e94f"
        );
    }
}
