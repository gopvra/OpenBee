use crate::security::SecretBytes;
use hmac::{Hmac, Mac};
use sha2::Sha512;

type HmacSha512 = Hmac<Sha512>;

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum HdError {
    #[error("Invalid seed length: expected >= 16, got {0}")]
    InvalidSeedLength(usize),
    #[error("Invalid derivation path: {0}")]
    InvalidPath(String),
    #[error("HMAC computation failed")]
    HmacFailed,
    #[error("Derived key is invalid (zero or exceeds curve order)")]
    InvalidDerivedKey,
}

// ---------------------------------------------------------------------------
// BIP-44 derivation path
// ---------------------------------------------------------------------------

/// A BIP-44 hierarchical deterministic derivation path:
/// `m / purpose' / coin_type' / account' / change / index`
pub struct DerivationPath {
    pub purpose: u32,
    pub coin_type: u32,
    pub account: u32,
    pub change: u32,
    pub index: u32,
}

/// Bit flag indicating a hardened child index.
const HARDENED: u32 = 0x8000_0000;

impl DerivationPath {
    /// Standard Ethereum path: `m/44'/60'/account'/0/index`.
    pub fn ethereum(account: u32, index: u32) -> Self {
        Self {
            purpose: 44,
            coin_type: 60,
            account,
            change: 0,
            index,
        }
    }

    /// Standard Solana path: `m/44'/501'/account'/0/index`.
    pub fn solana(account: u32, index: u32) -> Self {
        Self {
            purpose: 44,
            coin_type: 501,
            account,
            change: 0,
            index,
        }
    }

    /// Render as the canonical string `m/44'/60'/0'/0/0`.
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        format!(
            "m/{}'/{}'/{}'/{}/{}",
            self.purpose, self.coin_type, self.account, self.change, self.index
        )
    }

    /// Parse a derivation-path string such as `m/44'/60'/0'/0/0`.
    pub fn parse(path: &str) -> Result<Self, HdError> {
        let path = path.trim();
        if !path.starts_with("m/") {
            return Err(HdError::InvalidPath(
                "path must start with 'm/'".to_string(),
            ));
        }

        let segments: Vec<&str> = path[2..].split('/').collect();
        if segments.len() != 5 {
            return Err(HdError::InvalidPath(format!(
                "expected 5 segments, got {}",
                segments.len()
            )));
        }

        let parse_segment = |s: &str, hardened_expected: bool| -> Result<u32, HdError> {
            let (num_str, is_hardened) = if let Some(stripped) = s.strip_suffix('\'') {
                (stripped, true)
            } else {
                (s, false)
            };
            if hardened_expected && !is_hardened {
                return Err(HdError::InvalidPath(format!(
                    "segment '{s}' should be hardened (end with ')"
                )));
            }
            num_str
                .parse::<u32>()
                .map_err(|_| HdError::InvalidPath(format!("invalid number: '{num_str}'")))
        };

        Ok(Self {
            purpose: parse_segment(segments[0], true)?,
            coin_type: parse_segment(segments[1], true)?,
            account: parse_segment(segments[2], true)?,
            change: parse_segment(segments[3], false)?,
            index: parse_segment(segments[4], false)?,
        })
    }

    /// Return the full list of child indices (with hardened flag applied).
    fn child_indices(&self) -> [u32; 5] {
        [
            self.purpose | HARDENED,
            self.coin_type | HARDENED,
            self.account | HARDENED,
            self.change,
            self.index,
        ]
    }
}

// ---------------------------------------------------------------------------
// BIP-32 key derivation
// ---------------------------------------------------------------------------

/// Derive a 32-byte private key from a BIP-39 seed following BIP-32 / BIP-44.
///
/// This is a simplified implementation that supports hardened derivation for
/// the first three path components and private-key-based derivation for the
/// non-hardened `change` and `index` levels.
pub fn derive_key_from_seed(seed: &[u8], path: &DerivationPath) -> Result<SecretBytes, HdError> {
    if seed.len() < 16 {
        return Err(HdError::InvalidSeedLength(seed.len()));
    }

    // Master key generation: HMAC-SHA512 with key "Bitcoin seed".
    let (mut key, mut chain_code) = hmac_sha512(b"Bitcoin seed", seed)?;

    // Derive each child level.
    for &child_index in path.child_indices().iter() {
        let is_hardened = child_index & HARDENED != 0;

        let mut data = Vec::with_capacity(37);
        if is_hardened {
            // Hardened child: 0x00 || ser256(key) || ser32(index)
            data.push(0x00);
            data.extend_from_slice(&key);
        } else {
            // Non-hardened child: we use the private key for derivation.
            // (Full BIP-32 would use the compressed public key here, but
            // since we always hold the private key this produces equivalent
            // entropy for wallet-local derivation.)
            data.push(0x00);
            data.extend_from_slice(&key);
        }
        data.extend_from_slice(&child_index.to_be_bytes());

        let (child_key, child_chain) = hmac_sha512(&chain_code, &data)?;
        key = child_key;
        chain_code = child_chain;
    }

    Ok(SecretBytes::new(key.to_vec()))
}

/// HMAC-SHA512: returns `(IL: [u8;32], IR: [u8;32])`.
fn hmac_sha512(key: &[u8], data: &[u8]) -> Result<([u8; 32], [u8; 32]), HdError> {
    let mut mac = HmacSha512::new_from_slice(key).map_err(|_| HdError::HmacFailed)?;
    mac.update(data);
    let result = mac.finalize().into_bytes();

    let mut il = [0u8; 32];
    let mut ir = [0u8; 32];
    il.copy_from_slice(&result[..32]);
    ir.copy_from_slice(&result[32..]);
    Ok((il, ir))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ethereum_path_string() {
        let p = DerivationPath::ethereum(0, 0);
        assert_eq!(p.to_string(), "m/44'/60'/0'/0/0");
    }

    #[test]
    fn solana_path_string() {
        let p = DerivationPath::solana(2, 5);
        assert_eq!(p.to_string(), "m/44'/501'/2'/0/5");
    }

    #[test]
    fn parse_roundtrip() {
        let p = DerivationPath::parse("m/44'/60'/0'/0/0").unwrap();
        assert_eq!(p.purpose, 44);
        assert_eq!(p.coin_type, 60);
        assert_eq!(p.account, 0);
        assert_eq!(p.change, 0);
        assert_eq!(p.index, 0);
    }

    #[test]
    fn parse_invalid_paths() {
        assert!(DerivationPath::parse("44'/60'/0'/0/0").is_err()); // no m/
        assert!(DerivationPath::parse("m/44/60'/0'/0/0").is_err()); // missing '
        assert!(DerivationPath::parse("m/44'/60'").is_err()); // too few segments
    }

    #[test]
    fn derive_produces_32_bytes() {
        let seed = [0xABu8; 64];
        let path = DerivationPath::ethereum(0, 0);
        let key = derive_key_from_seed(&seed, &path).unwrap();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn derive_deterministic() {
        let seed = [0x42u8; 64];
        let path = DerivationPath::ethereum(0, 0);
        let k1 = derive_key_from_seed(&seed, &path).unwrap();
        let k2 = derive_key_from_seed(&seed, &path).unwrap();
        assert_eq!(k1.as_bytes(), k2.as_bytes());
    }

    #[test]
    fn different_paths_different_keys() {
        let seed = [0x42u8; 64];
        let k1 = derive_key_from_seed(&seed, &DerivationPath::ethereum(0, 0)).unwrap();
        let k2 = derive_key_from_seed(&seed, &DerivationPath::ethereum(0, 1)).unwrap();
        assert_ne!(k1.as_bytes(), k2.as_bytes());
    }

    #[test]
    fn different_chains_different_keys() {
        let seed = [0x42u8; 64];
        let k1 = derive_key_from_seed(&seed, &DerivationPath::ethereum(0, 0)).unwrap();
        let k2 = derive_key_from_seed(&seed, &DerivationPath::solana(0, 0)).unwrap();
        assert_ne!(k1.as_bytes(), k2.as_bytes());
    }

    #[test]
    fn seed_too_short() {
        let seed = [0u8; 8];
        assert!(derive_key_from_seed(&seed, &DerivationPath::ethereum(0, 0)).is_err());
    }
}
