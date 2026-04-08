// Stub — full implementation provided by another agent.

use crate::security::SecretBytes;

/// BIP-39 mnemonic phrase wrapper.
#[derive(Clone)]
pub struct Mnemonic {
    phrase: String,
}

impl Mnemonic {
    /// Create from an existing phrase (validates word count only).
    pub fn from_phrase(phrase: &str) -> Result<Self, MnemonicError> {
        let word_count = phrase.split_whitespace().count();
        if word_count != 12 && word_count != 24 {
            return Err(MnemonicError::InvalidWordCount(word_count));
        }
        Ok(Self {
            phrase: phrase.to_string(),
        })
    }

    /// Return the phrase as a string slice.
    pub fn phrase(&self) -> &str {
        &self.phrase
    }

    /// Derive a 64-byte seed from the mnemonic + optional passphrase.
    pub fn to_seed(&self, passphrase: &str) -> SecretBytes {
        use hmac::Hmac;
        use sha2::Sha512;

        let salt = format!("mnemonic{}", passphrase);
        let mut seed = vec![0u8; 64];
        pbkdf2::pbkdf2::<Hmac<Sha512>>(self.phrase.as_bytes(), salt.as_bytes(), 2048, &mut seed)
            .expect("PBKDF2 should not fail with valid params");
        SecretBytes::new(seed)
    }
}

impl Drop for Mnemonic {
    fn drop(&mut self) {
        // Zero out the phrase
        unsafe {
            let bytes = self.phrase.as_bytes_mut();
            for b in bytes.iter_mut() {
                *b = 0;
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MnemonicError {
    #[error("Invalid word count: {0} (expected 12 or 24)")]
    InvalidWordCount(usize),
    #[error("Invalid mnemonic word: {0}")]
    InvalidWord(String),
}
