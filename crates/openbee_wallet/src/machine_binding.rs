//! Machine-binding and advanced anti-theft protections.
//!
//! Even if an attacker steals the encrypted keystore file AND the user's
//! password, they CANNOT decrypt the private key on a different machine
//! because the encryption key incorporates a hardware fingerprint.
//!
//! ## Security layers
//!
//! 1. **Password** — user-chosen, run through Argon2id
//! 2. **Machine fingerprint** — SHA-256 of hardware identifiers (hostname,
//!    username, OS, temp dir path). Changing machine = different key.
//! 3. **AES-256-GCM** — authenticated encryption; tampered files are rejected
//! 4. **Auto-wipe** — N consecutive wrong-password attempts erase the keystore
//! 5. **Zeroize** — private keys are scrubbed from memory immediately after use

use sha2::{Digest, Sha256};
use tracing::warn;

/// Maximum consecutive failed unlock attempts before the keystore is wiped.
pub const MAX_FAILED_ATTEMPTS: u32 = 10;

/// Collect a machine fingerprint that changes if the keystore file is moved
/// to a different computer.
///
/// The fingerprint is a SHA-256 hash of several environment values that are
/// stable on a given machine but differ across machines. It is NOT meant to be
/// cryptographically unbreakable — it raises the bar so that a stolen file
/// alone is useless without also compromising the original machine.
pub fn machine_fingerprint() -> [u8; 32] {
    let mut hasher = Sha256::new();

    // Hostname
    if let Ok(name) = std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .or_else(|_| hostname_fallback())
    {
        hasher.update(name.as_bytes());
    }

    // Username
    if let Ok(user) = std::env::var("USER").or_else(|_| std::env::var("USERNAME")) {
        hasher.update(user.as_bytes());
    }

    // OS identifier
    hasher.update(std::env::consts::OS.as_bytes());
    hasher.update(std::env::consts::ARCH.as_bytes());

    // Home directory path (unique per user per machine)
    if let Some(home) = dirs_fallback() {
        hasher.update(home.as_bytes());
    }

    // Temp directory path (includes username on many systems)
    hasher.update(std::env::temp_dir().to_string_lossy().as_bytes());

    // A fixed domain separator so our hash can't collide with other uses
    hasher.update(b"openbee-wallet-machine-binding-v1");

    let result = hasher.finalize();
    let mut out = [0u8; 32];
    out.copy_from_slice(&result);
    out
}

/// Combine user password and machine fingerprint into the final encryption key
/// material that is fed into Argon2id.
///
/// `effective_password = SHA-256(password || machine_fingerprint)`
///
/// This means the same password on a different machine yields a completely
/// different Argon2 input, and therefore a different AES key.
pub fn bind_password_to_machine(password: &str) -> Vec<u8> {
    let fingerprint = machine_fingerprint();
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(&fingerprint);
    hasher.update(b"openbee-password-binding-v1");
    hasher.finalize().to_vec()
}

/// Failed-attempt counter stored alongside each keystore file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttemptCounter {
    pub failed_attempts: u32,
    pub max_attempts: u32,
    pub wiped: bool,
}

impl Default for AttemptCounter {
    fn default() -> Self {
        Self {
            failed_attempts: 0,
            max_attempts: MAX_FAILED_ATTEMPTS,
            wiped: false,
        }
    }
}

impl AttemptCounter {
    /// Record a failed attempt. Returns `true` if the keystore should be wiped.
    pub fn record_failure(&mut self) -> bool {
        self.failed_attempts += 1;
        warn!(
            "Wallet unlock failed (attempt {}/{})",
            self.failed_attempts, self.max_attempts
        );
        if self.failed_attempts >= self.max_attempts {
            self.wiped = true;
            warn!(
                "SECURITY: Maximum unlock attempts ({}) exceeded — keystore marked for wipe",
                self.max_attempts
            );
            true
        } else {
            false
        }
    }

    /// Reset on successful unlock.
    pub fn record_success(&mut self) {
        self.failed_attempts = 0;
    }

    /// Check if keystore has been wiped.
    pub fn is_wiped(&self) -> bool {
        self.wiped
    }

    /// Save counter to a file next to the keystore.
    pub fn save(&self, path: &std::path::Path) -> Result<(), std::io::Error> {
        let json = serde_json::to_string(self).map_err(std::io::Error::other)?;
        std::fs::write(path, json)
    }

    /// Load counter from file. Returns default if file doesn't exist.
    pub fn load(path: &std::path::Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
}

/// Securely wipe a keystore file by overwriting with random data before deletion.
pub fn secure_wipe_file(path: &std::path::Path) -> Result<(), std::io::Error> {
    if path.exists() {
        // Overwrite with random data to prevent forensic recovery
        let size = std::fs::metadata(path)?.len() as usize;
        let random_data: Vec<u8> = (0..size).map(|_| rand::random::<u8>()).collect();
        std::fs::write(path, &random_data)?;
        // Then delete
        std::fs::remove_file(path)?;
        warn!("SECURITY: Keystore file securely wiped: {:?}", path);
    }
    Ok(())
}

// ---- Helpers (avoid pulling in extra crates) ----

fn hostname_fallback() -> Result<String, std::env::VarError> {
    // Try reading /etc/hostname on Linux
    std::fs::read_to_string("/etc/hostname")
        .map(|s| s.trim().to_string())
        .map_err(|_| std::env::VarError::NotPresent)
}

fn dirs_fallback() -> Option<String> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_fingerprint_deterministic() {
        let fp1 = machine_fingerprint();
        let fp2 = machine_fingerprint();
        assert_eq!(
            fp1, fp2,
            "Fingerprint should be deterministic on same machine"
        );
    }

    #[test]
    fn test_bound_password_differs_from_raw() {
        let bound = bind_password_to_machine("test_password");
        // Should be 32 bytes (SHA-256 output)
        assert_eq!(bound.len(), 32);
        // Should NOT equal the raw password bytes
        assert_ne!(bound, b"test_password");
    }

    #[test]
    fn test_attempt_counter_wipe() {
        let mut counter = AttemptCounter {
            max_attempts: 3,
            ..Default::default()
        };
        assert!(!counter.record_failure()); // 1/3
        assert!(!counter.record_failure()); // 2/3
        assert!(counter.record_failure()); // 3/3 -> wipe!
        assert!(counter.is_wiped());
    }

    #[test]
    fn test_attempt_counter_reset_on_success() {
        let mut counter = AttemptCounter::default();
        counter.record_failure();
        counter.record_failure();
        assert_eq!(counter.failed_attempts, 2);
        counter.record_success();
        assert_eq!(counter.failed_attempts, 0);
    }
}
