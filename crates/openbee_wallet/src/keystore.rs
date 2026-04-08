use crate::security::{
    decrypt_aes256gcm, derive_key_from_password, encrypt_aes256gcm, generate_salt, SecretBytes,
    WalletSecurityError,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// On-disk encrypted keystore format (JSON-serializable)
// ---------------------------------------------------------------------------

/// Encrypted keystore that can be safely persisted to disk.
///
/// Private key material is encrypted with AES-256-GCM using a key derived
/// from the user's password via Argon2id.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedKeystore {
    /// Format version (currently 1).
    pub version: u32,
    /// Encryption parameters and ciphertext.
    pub crypto: KeystoreCrypto,
    /// Public address — used for identification only, never secret.
    pub address: String,
    /// Chain identifier, e.g. "ethereum", "solana".
    pub chain: String,
    /// ISO-8601 creation timestamp.
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystoreCrypto {
    /// Cipher algorithm identifier.
    pub cipher: String,
    /// Hex-encoded `nonce || ciphertext || tag`.
    pub ciphertext: String,
    /// Key derivation function identifier.
    pub kdf: String,
    /// KDF tuning parameters.
    pub kdf_params: KdfParams,
    /// Hex-encoded salt used by the KDF.
    pub salt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    /// Argon2 memory cost in KiB (65536 = 64 MiB).
    pub memory_cost: u32,
    /// Argon2 time cost (iterations).
    pub time_cost: u32,
    /// Argon2 parallelism degree.
    pub parallelism: u32,
}

impl EncryptedKeystore {
    /// Encrypt a raw private key and package it into the keystore format.
    pub fn encrypt(
        private_key: &[u8],
        password: &str,
        address: &str,
        chain: &str,
    ) -> Result<Self, WalletSecurityError> {
        let salt = generate_salt();
        let enc_key = derive_key_from_password(password, &salt)?;
        let ciphertext = encrypt_aes256gcm(&enc_key, private_key)?;

        let now = chrono_lite_now();

        Ok(Self {
            version: 1,
            crypto: KeystoreCrypto {
                cipher: "aes-256-gcm".to_string(),
                ciphertext: hex::encode(&ciphertext),
                kdf: "argon2id".to_string(),
                kdf_params: KdfParams {
                    memory_cost: 65536,
                    time_cost: 3,
                    parallelism: 4,
                },
                salt: hex::encode(salt),
            },
            address: address.to_string(),
            chain: chain.to_string(),
            created_at: now,
        })
    }

    /// Decrypt the keystore and return the private key.
    ///
    /// The returned [`SecretBytes`] is automatically zeroized on drop.
    pub fn decrypt(&self, password: &str) -> Result<SecretBytes, WalletSecurityError> {
        let salt =
            hex::decode(&self.crypto.salt).map_err(|_| WalletSecurityError::DecryptionFailed)?;
        let ciphertext = hex::decode(&self.crypto.ciphertext)
            .map_err(|_| WalletSecurityError::DecryptionFailed)?;

        let enc_key = derive_key_from_password(password, &salt)?;
        decrypt_aes256gcm(&enc_key, &ciphertext)
    }

    /// Serialize the keystore to a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize a keystore from a JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Persist the keystore to `path` as JSON.
    pub fn save_to_file(&self, path: &Path) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json)
    }

    /// Load a keystore from a JSON file.
    pub fn load_from_file(path: &Path) -> Result<Self, anyhow::Error> {
        let data = std::fs::read_to_string(path)?;
        let ks: Self = serde_json::from_str(&data)?;
        Ok(ks)
    }
}

// ---------------------------------------------------------------------------
// Minimal ISO-8601 timestamp without pulling in chrono
// ---------------------------------------------------------------------------

fn chrono_lite_now() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();

    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    let (year, month, day) = days_to_ymd(days);

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    // Algorithm from Howard Hinnant's date library.
    days += 719468;
    let era = days / 146097;
    let doe = days - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let private_key = b"0123456789abcdef0123456789abcdef";
        let password = "strong-password!";
        let address = "0xdeadbeef";
        let chain = "ethereum";

        let ks = EncryptedKeystore::encrypt(private_key, password, address, chain).unwrap();

        assert_eq!(ks.version, 1);
        assert_eq!(ks.chain, "ethereum");
        assert_eq!(ks.crypto.cipher, "aes-256-gcm");
        assert_eq!(ks.crypto.kdf, "argon2id");

        let decrypted = ks.decrypt(password).unwrap();
        assert_eq!(decrypted.as_bytes(), private_key);
    }

    #[test]
    fn wrong_password_fails() {
        let ks =
            EncryptedKeystore::encrypt(b"secret-key-data!", "correct-password", "0x1", "ethereum")
                .unwrap();
        assert!(ks.decrypt("wrong-password").is_err());
    }

    #[test]
    fn json_roundtrip() {
        let ks = EncryptedKeystore::encrypt(b"key-data-here!!!", "pass", "addr", "solana").unwrap();
        let json = ks.to_json().unwrap();
        let ks2 = EncryptedKeystore::from_json(&json).unwrap();
        assert_eq!(ks2.address, "addr");
        assert_eq!(ks2.chain, "solana");
        assert_eq!(ks2.version, 1);
    }

    #[test]
    fn file_roundtrip() {
        let dir = std::env::temp_dir().join("openbee_keystore_test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test.json");

        let ks =
            EncryptedKeystore::encrypt(b"file-key-data!!!", "pw", "0xabc", "ethereum").unwrap();
        ks.save_to_file(&path).unwrap();

        let loaded = EncryptedKeystore::load_from_file(&path).unwrap();
        let decrypted = loaded.decrypt("pw").unwrap();
        assert_eq!(decrypted.as_bytes(), b"file-key-data!!!");

        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&dir).ok();
    }
}
