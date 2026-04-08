pub mod ethereum;
pub mod solana;

use crate::security::SecretBytes;
use crate::transaction::{SignedTransaction, TransactionError, TransactionRequest};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// ChainBackend trait
// ---------------------------------------------------------------------------

/// Trait that each blockchain must implement to plug into the wallet.
///
/// All methods are synchronous and operate purely on local data — no network
/// calls are made.  Private keys are accepted as opaque byte slices wrapped
/// in [`SecretBytes`] and are never logged or transmitted.
pub trait ChainBackend: Send + Sync {
    /// Canonical chain identifier (e.g. `"ethereum"`, `"solana"`).
    fn chain_id(&self) -> &str;

    /// Ticker symbol of the chain's native token (e.g. `"ETH"`, `"SOL"`).
    fn symbol(&self) -> &str;

    /// Number of decimal places for the native token (18 for ETH, 9 for SOL).
    fn decimals(&self) -> u8;

    /// Derive the on-chain address that corresponds to `private_key`.
    fn address_from_key(&self, private_key: &[u8]) -> Result<String, TransactionError>;

    /// Return `true` if `address` has a syntactically valid format for this chain.
    fn validate_address(&self, address: &str) -> bool;

    /// Sign a [`TransactionRequest`] with the given private key.
    ///
    /// The key material is borrowed only for the duration of the call and is
    /// **never** copied, cloned, or logged.
    fn sign_transaction(
        &self,
        request: &TransactionRequest,
        private_key: &SecretBytes,
    ) -> Result<SignedTransaction, TransactionError>;

    /// BIP-44 coin type used for HD derivation (e.g. 60 for ETH, 501 for SOL).
    fn coin_type(&self) -> u32;
}

// ---------------------------------------------------------------------------
// ChainRegistry
// ---------------------------------------------------------------------------

/// Registry of supported [`ChainBackend`] implementations, keyed by chain id.
pub struct ChainRegistry {
    chains: HashMap<String, Box<dyn ChainBackend>>,
}

impl ChainRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            chains: HashMap::new(),
        }
    }

    /// Register a chain backend.  If a backend with the same `chain_id` already
    /// exists it is silently replaced.
    pub fn register(&mut self, chain: Box<dyn ChainBackend>) {
        let id = chain.chain_id().to_string();
        tracing::info!(chain = %id, "Registered chain backend");
        self.chains.insert(id, chain);
    }

    /// Look up a chain backend by its identifier.
    pub fn get(&self, chain_id: &str) -> Option<&dyn ChainBackend> {
        self.chains.get(chain_id).map(|b| b.as_ref())
    }

    /// Return the identifiers of all registered chains.
    pub fn supported_chains(&self) -> Vec<&str> {
        self.chains.keys().map(|s| s.as_str()).collect()
    }

    /// Create a registry pre-populated with all built-in chain backends
    /// (Ethereum mainnet, Solana mainnet).
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register(Box::new(ethereum::EthereumChain::mainnet()));
        registry.register(Box::new(solana::SolanaChain::mainnet()));
        registry
    }
}

impl Default for ChainRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_with_defaults() {
        let reg = ChainRegistry::with_defaults();
        assert!(reg.get("ethereum").is_some());
        assert!(reg.get("solana").is_some());
        assert!(reg.get("bitcoin").is_none());

        let mut chains = reg.supported_chains();
        chains.sort();
        assert_eq!(chains, vec!["ethereum", "solana"]);
    }

    #[test]
    fn registry_register_and_get() {
        let mut reg = ChainRegistry::new();
        assert!(reg.supported_chains().is_empty());

        reg.register(Box::new(ethereum::EthereumChain::sepolia()));
        assert_eq!(reg.supported_chains().len(), 1);
        let eth = reg.get("ethereum").unwrap();
        assert_eq!(eth.symbol(), "ETH");
        assert_eq!(eth.decimals(), 18);
    }
}
