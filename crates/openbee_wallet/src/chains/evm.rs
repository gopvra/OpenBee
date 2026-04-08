//! Generic EVM-compatible chain backend.
//!
//! Many L1s and L2s are EVM-compatible and share the same address format,
//! signing scheme (secp256k1 / Keccak-256 / EIP-155), and BIP-44 coin type
//! (60). This module provides a single configurable struct that can represent
//! any of them: BSC, Polygon, Arbitrum, Optimism, Base, Avalanche C-Chain,
//! and more.

use k256::ecdsa::SigningKey;
use sha3::{Digest, Keccak256};

use crate::chains::ChainBackend;
use crate::security::SecretBytes;
use crate::transaction::{SignedTransaction, TransactionError, TransactionRequest};

// Re-use the RLP encoder from the ethereum module.
use super::ethereum::rlp_helpers;

// ---------------------------------------------------------------------------
// EvmChain
// ---------------------------------------------------------------------------

/// A generic EVM-compatible chain backend.
///
/// All EVM chains share the same address derivation (secp256k1 -> Keccak-256),
/// address validation (`0x` + 40 hex chars), and EIP-155 transaction signing.
/// They differ only in chain ID, native token symbol, and optional RPC URL.
pub struct EvmChain {
    /// Canonical chain identifier used as the registry key, e.g. `"bsc"`,
    /// `"polygon"`, `"arbitrum"`.
    chain_identifier: String,
    /// Native token ticker symbol, e.g. `"BNB"`, `"MATIC"`, `"ETH"`.
    native_symbol: String,
    /// Number of decimal places for the native token (18 for all EVM chains).
    native_decimals: u8,
    /// EIP-155 numeric chain id.
    pub network_chain_id: u64,
    /// Optional JSON-RPC endpoint URL (informational; signing is local-only).
    pub rpc_url: Option<String>,
}

impl EvmChain {
    /// BNB Smart Chain mainnet (chain id 56).
    pub fn bsc() -> Self {
        Self {
            chain_identifier: "bsc".into(),
            native_symbol: "BNB".into(),
            native_decimals: 18,
            network_chain_id: 56,
            rpc_url: None,
        }
    }

    /// BNB Smart Chain testnet (chain id 97).
    pub fn bsc_testnet() -> Self {
        Self {
            chain_identifier: "bsc".into(),
            native_symbol: "BNB".into(),
            native_decimals: 18,
            network_chain_id: 97,
            rpc_url: None,
        }
    }

    /// Polygon PoS mainnet (chain id 137).
    pub fn polygon() -> Self {
        Self {
            chain_identifier: "polygon".into(),
            native_symbol: "MATIC".into(),
            native_decimals: 18,
            network_chain_id: 137,
            rpc_url: None,
        }
    }

    /// Arbitrum One mainnet (chain id 42161).
    pub fn arbitrum() -> Self {
        Self {
            chain_identifier: "arbitrum".into(),
            native_symbol: "ETH".into(),
            native_decimals: 18,
            network_chain_id: 42161,
            rpc_url: None,
        }
    }

    /// Optimism mainnet (chain id 10).
    pub fn optimism() -> Self {
        Self {
            chain_identifier: "optimism".into(),
            native_symbol: "ETH".into(),
            native_decimals: 18,
            network_chain_id: 10,
            rpc_url: None,
        }
    }

    /// Base mainnet (chain id 8453).
    pub fn base() -> Self {
        Self {
            chain_identifier: "base".into(),
            native_symbol: "ETH".into(),
            native_decimals: 18,
            network_chain_id: 8453,
            rpc_url: None,
        }
    }

    /// Avalanche C-Chain mainnet (chain id 43114).
    pub fn avalanche() -> Self {
        Self {
            chain_identifier: "avalanche".into(),
            native_symbol: "AVAX".into(),
            native_decimals: 18,
            network_chain_id: 43114,
            rpc_url: None,
        }
    }
}

impl ChainBackend for EvmChain {
    fn chain_id(&self) -> &str {
        &self.chain_identifier
    }

    fn symbol(&self) -> &str {
        &self.native_symbol
    }

    fn decimals(&self) -> u8 {
        self.native_decimals
    }

    fn coin_type(&self) -> u32 {
        60 // All EVM chains use BIP-44 coin type 60
    }

    /// Derive an address from a raw 32-byte secp256k1 private key.
    ///
    /// Same algorithm as Ethereum: uncompressed pubkey -> Keccak-256 -> last 20 bytes.
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

    /// Validate address: `0x` followed by exactly 40 hex digits.
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

    /// Sign a transaction using EIP-155 encoding with this chain's ID.
    fn sign_transaction(
        &self,
        request: &TransactionRequest,
        private_key: &SecretBytes,
    ) -> Result<SignedTransaction, TransactionError> {
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

        // Build unsigned RLP for EIP-155 signing
        let mut unsigned_items: Vec<rlp_helpers::RlpItem> = vec![
            rlp_helpers::RlpItem::uint(nonce as u128),
            rlp_helpers::RlpItem::uint(gas_price as u128),
            rlp_helpers::RlpItem::uint(gas_limit as u128),
            rlp_helpers::RlpItem::Bytes(to_bytes),
            rlp_helpers::RlpItem::uint(value),
            rlp_helpers::RlpItem::Bytes(data),
            rlp_helpers::RlpItem::uint(self.network_chain_id as u128),
            rlp_helpers::RlpItem::uint(0),
            rlp_helpers::RlpItem::uint(0),
        ];

        let unsigned_rlp = rlp_helpers::rlp_encode_list(&unsigned_items);

        // Hash + sign
        let msg_hash = Keccak256::digest(&unsigned_rlp);

        let (signature, recid) = signing_key
            .sign_prehash_recoverable(&msg_hash)
            .map_err(|e| TransactionError::SigningFailed(e.to_string()))?;

        let (r_bytes, s_bytes) = signature.split_bytes();

        // EIP-155 v = recid + chain_id * 2 + 35
        let v = recid.to_byte() as u128 + self.network_chain_id as u128 * 2 + 35;

        // Build signed RLP
        unsigned_items.truncate(6);
        unsigned_items.push(rlp_helpers::RlpItem::uint(v));
        unsigned_items.push(rlp_helpers::RlpItem::Bytes(
            rlp_helpers::trim_leading_zeros(r_bytes.as_ref()).to_vec(),
        ));
        unsigned_items.push(rlp_helpers::RlpItem::Bytes(
            rlp_helpers::trim_leading_zeros(s_bytes.as_ref()).to_vec(),
        ));

        let signed_rlp = rlp_helpers::rlp_encode_list(&unsigned_items);

        // Transaction hash
        let tx_hash = Keccak256::digest(&signed_rlp);

        Ok(SignedTransaction {
            chain: self.chain_identifier.clone(),
            tx_hash: format!("0x{}", hex::encode(tx_hash)),
            raw_tx: signed_rlp,
            from: request.from.clone(),
            to: request.to.clone(),
            amount: request.amount,
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bsc_address_derivation() {
        let bsc = EvmChain::bsc();
        // Same private key = same address as Ethereum (EVM compatible)
        let key = hex::decode("0000000000000000000000000000000000000000000000000000000000000001")
            .unwrap();
        let addr = bsc.address_from_key(&key).unwrap();
        assert_eq!(
            addr.to_lowercase(),
            "0x7e5f4552091a69125d5dfcb7b8c2659029395bdf"
        );
    }

    #[test]
    fn bsc_chain_properties() {
        let bsc = EvmChain::bsc();
        assert_eq!(bsc.chain_id(), "bsc");
        assert_eq!(bsc.symbol(), "BNB");
        assert_eq!(bsc.decimals(), 18);
        assert_eq!(bsc.coin_type(), 60);
        assert_eq!(bsc.network_chain_id, 56);
    }

    #[test]
    fn polygon_chain_properties() {
        let polygon = EvmChain::polygon();
        assert_eq!(polygon.chain_id(), "polygon");
        assert_eq!(polygon.symbol(), "MATIC");
        assert_eq!(polygon.decimals(), 18);
        assert_eq!(polygon.network_chain_id, 137);
    }

    #[test]
    fn arbitrum_chain_properties() {
        let arb = EvmChain::arbitrum();
        assert_eq!(arb.chain_id(), "arbitrum");
        assert_eq!(arb.symbol(), "ETH");
        assert_eq!(arb.network_chain_id, 42161);
    }

    #[test]
    fn optimism_chain_properties() {
        let op = EvmChain::optimism();
        assert_eq!(op.chain_id(), "optimism");
        assert_eq!(op.symbol(), "ETH");
        assert_eq!(op.network_chain_id, 10);
    }

    #[test]
    fn base_chain_properties() {
        let base = EvmChain::base();
        assert_eq!(base.chain_id(), "base");
        assert_eq!(base.symbol(), "ETH");
        assert_eq!(base.network_chain_id, 8453);
    }

    #[test]
    fn avalanche_chain_properties() {
        let avax = EvmChain::avalanche();
        assert_eq!(avax.chain_id(), "avalanche");
        assert_eq!(avax.symbol(), "AVAX");
        assert_eq!(avax.network_chain_id, 43114);
    }

    #[test]
    fn validate_address() {
        let chain = EvmChain::bsc();
        assert!(chain.validate_address("0x0000000000000000000000000000000000000000"));
        assert!(chain.validate_address("0xABCDEF1234567890abcdef1234567890ABCDEF12"));
        assert!(!chain.validate_address("not-an-address"));
        assert!(!chain.validate_address("0x1234"));
    }

    #[test]
    fn sign_transaction_bsc() {
        let bsc = EvmChain::bsc();
        let key_raw =
            hex::decode("ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80")
                .unwrap();
        let private_key = SecretBytes::new(key_raw.clone());
        let from_addr = bsc.address_from_key(&key_raw).unwrap();

        let request = TransactionRequest {
            chain: "bsc".into(),
            from: from_addr.clone(),
            to: "0x0000000000000000000000000000000000000001".into(),
            amount: 1_000_000_000_000_000_000,
            fee_limit: Some(21_000),
            data: None,
            nonce: Some(0),
            memo: None,
        };

        let signed = bsc.sign_transaction(&request, &private_key).unwrap();
        assert_eq!(signed.chain, "bsc");
        assert!(signed.tx_hash.starts_with("0x"));
        assert!(!signed.raw_tx.is_empty());
        assert_eq!(signed.from, from_addr);
    }

    #[test]
    fn all_chains_coin_type_60() {
        let chains = vec![
            EvmChain::bsc(),
            EvmChain::polygon(),
            EvmChain::arbitrum(),
            EvmChain::optimism(),
            EvmChain::base(),
            EvmChain::avalanche(),
        ];
        for chain in chains {
            assert_eq!(
                chain.coin_type(),
                60,
                "chain {} coin_type",
                chain.chain_id()
            );
        }
    }
}
