use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::{Algorithm, Argon2, Params, Version};
use rand::rngs::OsRng;
use rand::RngCore;
use zeroize::Zeroize;

// ---------------------------------------------------------------------------
// SecretBytes — zeroed-on-drop secure buffer
// ---------------------------------------------------------------------------

/// Securely zeroed byte buffer that clears memory on drop.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    /// Wrap raw bytes in a secure container.
    pub fn new(data: Vec<u8>) -> Self {
        Self(data)
    }

    /// Borrow the inner bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Length in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum WalletSecurityError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: wrong password or corrupted data")]
    DecryptionFailed,
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },
}

// ---------------------------------------------------------------------------
// Key derivation (Argon2id)
// ---------------------------------------------------------------------------

/// Derive a 32-byte encryption key from a password using Argon2id.
///
/// Parameters: 64 MiB memory, 3 iterations, parallelism 4.
pub fn derive_key_from_password(
    password: &str,
    salt: &[u8],
) -> Result<[u8; 32], WalletSecurityError> {
    let params = Params::new(
        65536, // 64 MiB
        3,     // iterations
        4,     // parallelism
        Some(32),
    )
    .map_err(|e| WalletSecurityError::KeyDerivationFailed(e.to_string()))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut output = [0u8; 32];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output)
        .map_err(|e| WalletSecurityError::KeyDerivationFailed(e.to_string()))?;

    Ok(output)
}

// ---------------------------------------------------------------------------
// AES-256-GCM encryption / decryption
// ---------------------------------------------------------------------------

/// AES-256-GCM nonce size in bytes (96 bits).
const NONCE_LEN: usize = 12;

/// Encrypt `plaintext` with AES-256-GCM.
///
/// Returns `nonce (12 bytes) || ciphertext+tag`.
pub fn encrypt_aes256gcm(key: &[u8; 32], plaintext: &[u8]) -> Result<Vec<u8>, WalletSecurityError> {
    let cipher_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(cipher_key);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| WalletSecurityError::EncryptionFailed(e.to_string()))?;

    let mut result = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

/// Decrypt AES-256-GCM data produced by [`encrypt_aes256gcm`].
///
/// Expected input: `nonce (12 bytes) || ciphertext+tag`.
pub fn decrypt_aes256gcm(
    key: &[u8; 32],
    encrypted: &[u8],
) -> Result<SecretBytes, WalletSecurityError> {
    if encrypted.len() < NONCE_LEN {
        return Err(WalletSecurityError::DecryptionFailed);
    }

    let (nonce_bytes, ciphertext) = encrypted.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let cipher_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(cipher_key);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| WalletSecurityError::DecryptionFailed)?;

    Ok(SecretBytes::new(plaintext))
}

// ---------------------------------------------------------------------------
// Utility helpers
// ---------------------------------------------------------------------------

/// Constant-time comparison to prevent timing attacks.
///
/// Returns `false` immediately if lengths differ (this leaks length, which
/// is acceptable for our use-cases where lengths are always equal).
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Generate `len` cryptographically secure random bytes.
pub fn random_bytes(len: usize) -> Vec<u8> {
    let mut buf = vec![0u8; len];
    OsRng.fill_bytes(&mut buf);
    buf
}

/// Generate a random 16-byte salt.
pub fn generate_salt() -> [u8; 16] {
    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);
    salt
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = {
            let mut k = [0u8; 32];
            OsRng.fill_bytes(&mut k);
            k
        };
        let plaintext = b"super secret private key material";

        let encrypted = encrypt_aes256gcm(&key, plaintext).unwrap();
        let decrypted = decrypt_aes256gcm(&key, &encrypted).unwrap();

        assert_eq!(decrypted.as_bytes(), plaintext);
    }

    #[test]
    fn decrypt_wrong_key_fails() {
        let key1 = {
            let mut k = [0u8; 32];
            OsRng.fill_bytes(&mut k);
            k
        };
        let key2 = {
            let mut k = [0u8; 32];
            OsRng.fill_bytes(&mut k);
            k
        };

        let encrypted = encrypt_aes256gcm(&key1, b"data").unwrap();
        assert!(decrypt_aes256gcm(&key2, &encrypted).is_err());
    }

    #[test]
    fn constant_time_eq_works() {
        assert!(constant_time_eq(b"hello", b"hello"));
        assert!(!constant_time_eq(b"hello", b"world"));
        assert!(!constant_time_eq(b"hi", b"hello"));
    }

    #[test]
    fn key_derivation_deterministic() {
        let salt = generate_salt();
        let k1 = derive_key_from_password("password123", &salt).unwrap();
        let k2 = derive_key_from_password("password123", &salt).unwrap();
        assert_eq!(k1, k2);
    }

    #[test]
    fn secret_bytes_len() {
        let s = SecretBytes::new(vec![1, 2, 3]);
        assert_eq!(s.len(), 3);
        assert!(!s.is_empty());
        assert!(SecretBytes::new(vec![]).is_empty());
    }
}
