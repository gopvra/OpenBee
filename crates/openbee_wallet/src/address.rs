use k256::ecdsa::SigningKey;
use sha3::{Digest, Keccak256};

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum AddressError {
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),
}

// ---------------------------------------------------------------------------
// Ethereum
// ---------------------------------------------------------------------------

/// Derive an Ethereum address from a 32-byte secp256k1 private key.
///
/// Process: uncompressed public key (64 bytes, without the 0x04 prefix) ->
/// Keccak-256 -> take last 20 bytes -> hex-encode with `0x` prefix.
pub fn ethereum_address_from_private_key(private_key: &[u8]) -> Result<String, AddressError> {
    let signing_key = SigningKey::from_bytes(private_key.into())
        .map_err(|e| AddressError::InvalidPrivateKey(e.to_string()))?;

    let verifying_key = signing_key.verifying_key();
    let public_key_point = verifying_key.to_encoded_point(false); // uncompressed
    let public_key_bytes = public_key_point.as_bytes();

    // Skip the 0x04 prefix byte -- hash only the 64-byte X||Y.
    let hash = Keccak256::digest(&public_key_bytes[1..]);

    // Last 20 bytes of the hash.
    let address_bytes = &hash[12..];
    Ok(format!("0x{}", hex::encode(address_bytes)))
}

/// Validate an Ethereum address: `0x`-prefixed, 40 hex characters.
pub fn validate_ethereum_address(address: &str) -> bool {
    if !address.starts_with("0x") && !address.starts_with("0X") {
        return false;
    }
    let hex_part = &address[2..];
    if hex_part.len() != 40 {
        return false;
    }
    hex_part.chars().all(|c| c.is_ascii_hexdigit())
}

// ---------------------------------------------------------------------------
// Solana
// ---------------------------------------------------------------------------

/// Derive a Solana address from a 32-byte Ed25519 private key (seed).
///
/// The address is the base58-encoded 32-byte public key.
pub fn solana_address_from_private_key(private_key: &[u8]) -> Result<String, AddressError> {
    if private_key.len() != 32 {
        return Err(AddressError::InvalidPrivateKey(format!(
            "expected 32 bytes, got {}",
            private_key.len()
        )));
    }

    let secret_key: [u8; 32] = private_key.try_into().unwrap();
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&secret_key);
    let public_key = signing_key.verifying_key();

    Ok(bs58::encode(public_key.as_bytes()).into_string())
}

/// Validate a Solana address: base58-encoded, decodes to exactly 32 bytes.
pub fn validate_solana_address(address: &str) -> bool {
    if address.len() < 32 || address.len() > 44 {
        return false;
    }
    match bs58::decode(address).into_vec() {
        Ok(bytes) => bytes.len() == 32,
        Err(_) => false,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eth_address_valid_format() {
        let pk = [1u8; 32];
        let addr = ethereum_address_from_private_key(&pk).unwrap();
        assert!(addr.starts_with("0x"));
        assert_eq!(addr.len(), 42); // 0x + 40 hex chars
        assert!(validate_ethereum_address(&addr));
    }

    #[test]
    fn eth_address_deterministic() {
        let pk = [42u8; 32];
        let a1 = ethereum_address_from_private_key(&pk).unwrap();
        let a2 = ethereum_address_from_private_key(&pk).unwrap();
        assert_eq!(a1, a2);
    }

    #[test]
    fn eth_validate() {
        assert!(validate_ethereum_address(
            "0x1234567890abcdef1234567890abcdef12345678"
        ));
        assert!(!validate_ethereum_address("1234567890abcdef1234")); // no 0x
        assert!(!validate_ethereum_address("0x1234")); // too short
        assert!(!validate_ethereum_address(
            "0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG"
        )); // invalid hex
    }

    #[test]
    fn sol_address_valid_format() {
        let pk = [2u8; 32];
        let addr = solana_address_from_private_key(&pk).unwrap();
        assert!(validate_solana_address(&addr));
    }

    #[test]
    fn sol_address_deterministic() {
        let pk = [7u8; 32];
        let a1 = solana_address_from_private_key(&pk).unwrap();
        let a2 = solana_address_from_private_key(&pk).unwrap();
        assert_eq!(a1, a2);
    }

    #[test]
    fn sol_validate() {
        assert!(validate_solana_address("11111111111111111111111111111111"));
        assert!(!validate_solana_address("too_short"));
    }

    #[test]
    fn sol_invalid_key_length() {
        assert!(solana_address_from_private_key(&[1u8; 16]).is_err());
    }
}
