// Stub — full implementation provided by another agent.

use crate::security::SecretBytes;

/// BIP-39 mnemonic phrase wrapper.
#[derive(Clone)]
pub struct Mnemonic {
    phrase: String,
}

impl Mnemonic {
    /// Generate a new random 12-word mnemonic.
    pub fn generate_12() -> Result<Self, MnemonicError> {
        Self::generate(16) // 128 bits = 12 words
    }

    /// Generate a new random 24-word mnemonic.
    pub fn generate_24() -> Result<Self, MnemonicError> {
        Self::generate(32) // 256 bits = 24 words
    }

    fn generate(entropy_bytes: usize) -> Result<Self, MnemonicError> {
        use rand::RngCore;
        use sha2::{Digest, Sha256};

        // BIP-39 mini wordlist (first 2048 common English words)
        // In production, this should be the full BIP-39 English wordlist.
        let words = bip39_wordlist();

        let mut entropy = vec![0u8; entropy_bytes];
        rand::rngs::OsRng.fill_bytes(&mut entropy);

        // Checksum: first (entropy_bits / 32) bits of SHA-256
        let hash = Sha256::digest(&entropy);
        let checksum_bits = entropy_bytes * 8 / 32;

        // Combine entropy + checksum bits
        let mut bits = Vec::with_capacity(entropy_bytes * 8 + checksum_bits);
        for byte in &entropy {
            for j in (0..8).rev() {
                bits.push((byte >> j) & 1);
            }
        }
        for j in (0..checksum_bits).rev() {
            let byte_idx = (8 - 1 - j) / 8; // always 0 for 12/24 words
            let bit_idx = 7 - ((8 - 1 - j) % 8);
            bits.push((hash[byte_idx] >> bit_idx) & 1);
        }

        // Convert to 11-bit indices
        let word_count = bits.len() / 11;
        let mut phrase_words = Vec::with_capacity(word_count);
        for i in 0..word_count {
            let mut index: usize = 0;
            for j in 0..11 {
                index = (index << 1) | bits[i * 11 + j] as usize;
            }
            phrase_words.push(words[index % words.len()].to_string());
        }

        let phrase = phrase_words.join(" ");
        // Zeroize entropy
        for b in entropy.iter_mut() {
            *b = 0;
        }

        Ok(Self { phrase })
    }

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

/// Minimal BIP-39 English wordlist (2048 words).
/// A production deployment should embed the canonical list from
/// https://github.com/bitcoin/bips/blob/master/bip-0039/english.txt
fn bip39_wordlist() -> Vec<&'static str> {
    // First 2048 common English words for BIP-39 compatibility.
    // This uses a compact representation — split a long string.
    let raw = include_str!("bip39_english.txt");
    raw.lines().filter(|l: &&str| !l.is_empty()).collect()
}

#[derive(Debug, thiserror::Error)]
pub enum MnemonicError {
    #[error("Invalid word count: {0} (expected 12 or 24)")]
    InvalidWordCount(usize),
    #[error("Invalid mnemonic word: {0}")]
    InvalidWord(String),
}
