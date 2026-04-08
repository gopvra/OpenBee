//! Ethereum chain backend -- secp256k1 / Keccak-256 / EIP-155.

use k256::ecdsa::SigningKey;
use sha3::{Digest, Keccak256};

use crate::chains::ChainBackend;
use crate::security::SecretBytes;
use crate::transaction::{SignedTransaction, TransactionError, TransactionRequest};

// ---------------------------------------------------------------------------
// EthereumChain
// ---------------------------------------------------------------------------

/// Ethereum-compatible chain backend.
///
/// Supports mainnet (chain id 1), Sepolia testnet (11155111), and arbitrary
/// EVM-compatible networks via [`EthereumChain::custom`].
pub struct EthereumChain {
    /// EIP-155 numeric chain id.
    pub network_chain_id: u64,
    /// Optional JSON-RPC endpoint URL (informational; signing is local-only).
    pub rpc_url: Option<String>,
}

impl EthereumChain {
    /// Ethereum mainnet (chain id 1).
    pub fn mainnet() -> Self {
        Self {
            network_chain_id: 1,
            rpc_url: None,
        }
    }

    /// Sepolia testnet (chain id 11155111).
    pub fn sepolia() -> Self {
        Self {
            network_chain_id: 11155111,
            rpc_url: None,
        }
    }

    /// A custom EVM-compatible network.
    pub fn custom(chain_id: u64, rpc_url: &str) -> Self {
        Self {
            network_chain_id: chain_id,
            rpc_url: Some(rpc_url.to_string()),
        }
    }
}

impl ChainBackend for EthereumChain {
    fn chain_id(&self) -> &str {
        "ethereum"
    }

    fn symbol(&self) -> &str {
        "ETH"
    }

    fn decimals(&self) -> u8 {
        18
    }

    fn coin_type(&self) -> u32 {
        60
    }

    /// Derive an Ethereum address from a raw 32-byte secp256k1 private key.
    ///
    /// 1. Recover the uncompressed public key (65 bytes, 0x04 prefix).
    /// 2. Keccak-256 of the 64-byte public key (sans 0x04 prefix).
    /// 3. Take the last 20 bytes, hex-encode, prepend "0x".
    fn address_from_key(&self, private_key: &[u8]) -> Result<String, TransactionError> {
        let signing_key = SigningKey::from_slice(private_key)
            .map_err(|e| TransactionError::SigningFailed(format!("Invalid private key: {e}")))?;

        let verifying_key = signing_key.verifying_key();
        let pubkey_uncompressed = verifying_key.to_encoded_point(false);
        let pubkey_bytes = &pubkey_uncompressed.as_bytes()[1..]; // drop 0x04

        let hash = Keccak256::digest(pubkey_bytes);
        let addr_bytes = &hash[12..]; // last 20 bytes

        Ok(format!("0x{}", hex::encode(addr_bytes)))
    }

    /// Validate an Ethereum address: must be `0x` followed by exactly 40 hex digits.
    fn validate_address(&self, address: &str) -> bool {
        if address.len() != 42 {
            return false;
        }
        let prefix = &address[..2];
        if prefix != "0x" && prefix != "0X" {
            return false;
        }
        address[2..].chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Sign a transaction using simplified EIP-155 encoding.
    ///
    /// 1. RLP-encode unsigned tx: `[nonce, gas_price, gas_limit, to, value, data, chain_id, 0, 0]`
    /// 2. Keccak-256 hash the encoded bytes.
    /// 3. ECDSA-sign with secp256k1.
    /// 4. Build signed tx: `[nonce, gas_price, gas_limit, to, value, data, v, r, s]`
    fn sign_transaction(
        &self,
        request: &TransactionRequest,
        private_key: &SecretBytes,
    ) -> Result<SignedTransaction, TransactionError> {
        // Validate addresses
        if !self.validate_address(&request.from) {
            return Err(TransactionError::InvalidAddress(request.from.clone()));
        }
        if !self.validate_address(&request.to) {
            return Err(TransactionError::InvalidAddress(request.to.clone()));
        }

        let signing_key = SigningKey::from_slice(private_key.as_bytes())
            .map_err(|e| TransactionError::SigningFailed(format!("Invalid private key: {e}")))?;

        // Verify the key matches the `from` address
        let derived_addr = self.address_from_key(private_key.as_bytes())?;
        if derived_addr.to_lowercase() != request.from.to_lowercase() {
            return Err(TransactionError::SigningFailed(
                "Private key does not match sender address".into(),
            ));
        }

        let nonce = request.nonce.unwrap_or(0);
        let gas_price: u64 = 20_000_000_000; // 20 Gwei default
        let gas_limit: u64 = request.fee_limit.unwrap_or(21_000);
        let to_bytes = hex::decode(&request.to[2..])
            .map_err(|e| TransactionError::InvalidAddress(e.to_string()))?;
        let value = request.amount;
        let data = request.data.clone().unwrap_or_default();

        // ---- Build unsigned RLP for EIP-155 signing ----
        let mut unsigned_items: Vec<RlpItem> = vec![
            RlpItem::uint(nonce as u128),
            RlpItem::uint(gas_price as u128),
            RlpItem::uint(gas_limit as u128),
            RlpItem::Bytes(to_bytes.clone()),
            RlpItem::uint(value),
            RlpItem::Bytes(data.clone()),
            RlpItem::uint(self.network_chain_id as u128),
            RlpItem::uint(0),
            RlpItem::uint(0),
        ];

        let unsigned_rlp = rlp_encode_list(&unsigned_items);

        // ---- Hash + sign ----
        let msg_hash = Keccak256::digest(&unsigned_rlp);

        let (signature, recid) = signing_key
            .sign_prehash_recoverable(&msg_hash)
            .map_err(|e| TransactionError::SigningFailed(e.to_string()))?;

        let (r_bytes, s_bytes) = signature.split_bytes();

        // EIP-155 v = recid + chain_id * 2 + 35
        let v = recid.to_byte() as u128 + self.network_chain_id as u128 * 2 + 35;

        // ---- Build signed RLP ----
        unsigned_items.truncate(6); // remove chain_id, 0, 0
        unsigned_items.push(RlpItem::uint(v));
        unsigned_items.push(RlpItem::Bytes(
            trim_leading_zeros(r_bytes.as_ref()).to_vec(),
        ));
        unsigned_items.push(RlpItem::Bytes(
            trim_leading_zeros(s_bytes.as_ref()).to_vec(),
        ));

        let signed_rlp = rlp_encode_list(&unsigned_items);

        // ---- Transaction hash ----
        let tx_hash = Keccak256::digest(&signed_rlp);

        Ok(SignedTransaction {
            chain: "ethereum".into(),
            tx_hash: format!("0x{}", hex::encode(tx_hash)),
            raw_tx: signed_rlp,
            from: request.from.clone(),
            to: request.to.clone(),
            amount: request.amount,
        })
    }
}

// ===========================================================================
// Minimal RLP encoder (just enough for legacy Ethereum transactions)
// ===========================================================================

/// An item in an RLP structure: either raw bytes or a nested list.
enum RlpItem {
    Bytes(Vec<u8>),
    #[allow(dead_code)]
    List(Vec<RlpItem>),
}

impl RlpItem {
    /// Encode an unsigned integer as the shortest big-endian byte representation.
    /// Zero encodes as an empty byte string (RLP convention).
    fn uint(value: u128) -> Self {
        if value == 0 {
            RlpItem::Bytes(vec![])
        } else {
            let bytes = value.to_be_bytes();
            let start = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
            RlpItem::Bytes(bytes[start..].to_vec())
        }
    }
}

/// RLP-encode a single item.
fn rlp_encode(item: &RlpItem) -> Vec<u8> {
    match item {
        RlpItem::Bytes(b) => {
            if b.is_empty() {
                // Empty byte string
                vec![0x80]
            } else if b.len() == 1 && b[0] < 0x80 {
                // Single byte in [0x00, 0x7f] is its own RLP encoding.
                vec![b[0]]
            } else if b.len() <= 55 {
                let mut out = vec![0x80 + b.len() as u8];
                out.extend_from_slice(b);
                out
            } else {
                let len_bytes = encode_length(b.len());
                let mut out = vec![0xb7 + len_bytes.len() as u8];
                out.extend_from_slice(&len_bytes);
                out.extend_from_slice(b);
                out
            }
        }
        RlpItem::List(items) => {
            let payload: Vec<u8> = items.iter().flat_map(rlp_encode).collect();
            rlp_encode_list_payload(&payload)
        }
    }
}

/// Wrap already-encoded payload bytes in an RLP list header.
fn rlp_encode_list_payload(payload: &[u8]) -> Vec<u8> {
    if payload.len() <= 55 {
        let mut out = vec![0xc0 + payload.len() as u8];
        out.extend_from_slice(payload);
        out
    } else {
        let len_bytes = encode_length(payload.len());
        let mut out = vec![0xf7 + len_bytes.len() as u8];
        out.extend_from_slice(&len_bytes);
        out.extend_from_slice(payload);
        out
    }
}

/// Encode a flat list of items.
fn rlp_encode_list(items: &[RlpItem]) -> Vec<u8> {
    let payload: Vec<u8> = items.iter().flat_map(rlp_encode).collect();
    rlp_encode_list_payload(&payload)
}

/// Big-endian encoding of `len` without leading zeros.
fn encode_length(len: usize) -> Vec<u8> {
    let bytes = (len as u64).to_be_bytes();
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    bytes[start..].to_vec()
}

/// Strip leading zero bytes from a byte slice.
fn trim_leading_zeros(data: &[u8]) -> &[u8] {
    let start = data.iter().position(|&b| b != 0).unwrap_or(data.len());
    if start == data.len() {
        &[]
    } else {
        &data[start..]
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_derivation() {
        let eth = EthereumChain::mainnet();
        // Well-known test vector: private key = 1
        let key = hex::decode("0000000000000000000000000000000000000000000000000000000000000001")
            .unwrap();
        let addr = eth.address_from_key(&key).unwrap();
        // Known address for private key = 1:
        // 0x7e5f4552091a69125d5dfcb7b8c2659029395bdf
        assert!(addr.starts_with("0x"));
        assert_eq!(addr.len(), 42);
        assert_eq!(
            addr.to_lowercase(),
            "0x7e5f4552091a69125d5dfcb7b8c2659029395bdf"
        );
    }

    #[test]
    fn validate_address_good() {
        let eth = EthereumChain::mainnet();
        assert!(eth.validate_address("0x0000000000000000000000000000000000000000"));
        assert!(eth.validate_address("0xABCDEF1234567890abcdef1234567890ABCDEF12"));
    }

    #[test]
    fn validate_address_bad() {
        let eth = EthereumChain::mainnet();
        assert!(!eth.validate_address("not-an-address"));
        assert!(!eth.validate_address("0x1234")); // too short
        assert!(!eth.validate_address("0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG"));
    }

    #[test]
    fn rlp_single_byte() {
        let encoded = rlp_encode(&RlpItem::Bytes(vec![0x42]));
        assert_eq!(encoded, vec![0x42]);
    }

    #[test]
    fn rlp_short_string() {
        // "dog" = [0x83, 'd', 'o', 'g']
        let encoded = rlp_encode(&RlpItem::Bytes(b"dog".to_vec()));
        assert_eq!(encoded, vec![0x83, b'd', b'o', b'g']);
    }

    #[test]
    fn rlp_empty_string() {
        let encoded = rlp_encode(&RlpItem::Bytes(vec![]));
        assert_eq!(encoded, vec![0x80]);
    }

    #[test]
    fn rlp_list() {
        // ["cat", "dog"]
        let items = vec![
            RlpItem::Bytes(b"cat".to_vec()),
            RlpItem::Bytes(b"dog".to_vec()),
        ];
        let encoded = rlp_encode_list(&items);
        assert_eq!(
            encoded,
            vec![0xc8, 0x83, b'c', b'a', b't', 0x83, b'd', b'o', b'g']
        );
    }

    #[test]
    fn rlp_uint_zero() {
        let item = RlpItem::uint(0);
        let encoded = rlp_encode(&item);
        assert_eq!(encoded, vec![0x80]);
    }

    #[test]
    fn rlp_uint_small() {
        let item = RlpItem::uint(127);
        let encoded = rlp_encode(&item);
        assert_eq!(encoded, vec![127]);
    }

    #[test]
    fn sign_transaction_basic() {
        let eth = EthereumChain::mainnet();
        // Hardhat account #0 private key
        let key_raw =
            hex::decode("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .unwrap();
        let private_key = SecretBytes::new(key_raw.clone());
        let from_addr = eth.address_from_key(&key_raw).unwrap();

        let request = TransactionRequest {
            chain: "ethereum".into(),
            from: from_addr.clone(),
            to: "0x0000000000000000000000000000000000000001".into(),
            amount: 1_000_000_000_000_000_000, // 1 ETH in wei
            fee_limit: Some(21_000),
            data: None,
            nonce: Some(0),
            memo: None,
        };

        let signed = eth.sign_transaction(&request, &private_key).unwrap();
        assert_eq!(signed.chain, "ethereum");
        assert!(signed.tx_hash.starts_with("0x"));
        assert!(!signed.raw_tx.is_empty());
        assert_eq!(signed.from, from_addr);
        assert_eq!(signed.amount, 1_000_000_000_000_000_000);
    }

    #[test]
    fn sign_transaction_wrong_key() {
        let eth = EthereumChain::mainnet();
        let key_raw =
            hex::decode("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .unwrap();
        let private_key = SecretBytes::new(key_raw);

        let request = TransactionRequest {
            chain: "ethereum".into(),
            from: "0x0000000000000000000000000000000000000099".into(),
            to: "0x0000000000000000000000000000000000000001".into(),
            amount: 100,
            fee_limit: None,
            data: None,
            nonce: None,
            memo: None,
        };

        let result = eth.sign_transaction(&request, &private_key);
        assert!(result.is_err());
    }

    #[test]
    fn coin_type_is_60() {
        let eth = EthereumChain::mainnet();
        assert_eq!(eth.coin_type(), 60);
    }

    #[test]
    fn custom_chain() {
        let chain = EthereumChain::custom(137, "https://polygon-rpc.com");
        assert_eq!(chain.network_chain_id, 137);
        assert_eq!(chain.rpc_url.as_deref(), Some("https://polygon-rpc.com"));
        assert_eq!(chain.chain_id(), "ethereum");
    }
}
