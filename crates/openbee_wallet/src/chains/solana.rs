//! Solana chain backend -- Ed25519 / Base58.

use ed25519_dalek::{Signer, SigningKey as Ed25519SigningKey};
use sha2::{Digest, Sha256};

use crate::chains::ChainBackend;
use crate::security::SecretBytes;
use crate::transaction::{SignedTransaction, TransactionError, TransactionRequest};

// ---------------------------------------------------------------------------
// Cluster enum
// ---------------------------------------------------------------------------

/// Well-known Solana clusters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SolanaCluster {
    Mainnet,
    Devnet,
    Testnet,
    Custom(String),
}

impl SolanaCluster {
    /// Default RPC endpoint for the cluster.
    pub fn rpc_url(&self) -> &str {
        match self {
            Self::Mainnet => "https://api.mainnet-beta.solana.com",
            Self::Devnet => "https://api.devnet.solana.com",
            Self::Testnet => "https://api.testnet.solana.com",
            Self::Custom(url) => url.as_str(),
        }
    }
}

// ---------------------------------------------------------------------------
// SolanaChain
// ---------------------------------------------------------------------------

/// Solana chain backend.
pub struct SolanaChain {
    /// Which cluster this instance targets.
    pub cluster: SolanaCluster,
    /// Optional override RPC URL.
    pub rpc_url: Option<String>,
}

impl SolanaChain {
    /// Mainnet-beta.
    pub fn mainnet() -> Self {
        Self {
            cluster: SolanaCluster::Mainnet,
            rpc_url: None,
        }
    }

    /// Devnet.
    pub fn devnet() -> Self {
        Self {
            cluster: SolanaCluster::Devnet,
            rpc_url: None,
        }
    }

    /// Testnet.
    pub fn testnet() -> Self {
        Self {
            cluster: SolanaCluster::Testnet,
            rpc_url: None,
        }
    }

    /// Custom cluster with a user-specified RPC endpoint.
    pub fn custom(rpc_url: &str) -> Self {
        Self {
            cluster: SolanaCluster::Custom(rpc_url.to_string()),
            rpc_url: Some(rpc_url.to_string()),
        }
    }

    /// Resolve the effective RPC URL.
    pub fn effective_rpc_url(&self) -> &str {
        self.rpc_url
            .as_deref()
            .unwrap_or_else(|| self.cluster.rpc_url())
    }
}

impl ChainBackend for SolanaChain {
    fn chain_id(&self) -> &str {
        "solana"
    }

    fn symbol(&self) -> &str {
        "SOL"
    }

    fn decimals(&self) -> u8 {
        9
    }

    fn coin_type(&self) -> u32 {
        501
    }

    /// Derive a Solana address from a 32-byte Ed25519 private key (seed).
    ///
    /// The address is the base58-encoded 32-byte public key.
    fn address_from_key(&self, private_key: &[u8]) -> Result<String, TransactionError> {
        if private_key.len() != 32 {
            return Err(TransactionError::SigningFailed(format!(
                "Ed25519 seed must be 32 bytes, got {}",
                private_key.len()
            )));
        }
        let seed: [u8; 32] = private_key.try_into().unwrap();
        let signing_key = Ed25519SigningKey::from_bytes(&seed);
        let pubkey = signing_key.verifying_key();
        Ok(bs58::encode(pubkey.as_bytes()).into_string())
    }

    /// Validate a Solana address: base58-encoded string that decodes to 32 bytes.
    fn validate_address(&self, address: &str) -> bool {
        // Solana addresses are 32-44 characters of base58
        if address.is_empty() || address.len() > 44 {
            return false;
        }
        match bs58::decode(address).into_vec() {
            Ok(bytes) => bytes.len() == 32,
            Err(_) => false,
        }
    }

    /// Sign a Solana transaction.
    ///
    /// Builds a simplified Solana transaction message containing a single
    /// system-program transfer instruction, then signs with Ed25519.
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

        if private_key.as_bytes().len() != 32 {
            return Err(TransactionError::SigningFailed(
                "Ed25519 seed must be 32 bytes".into(),
            ));
        }

        let seed: [u8; 32] = private_key.as_bytes().try_into().unwrap();
        let signing_key = Ed25519SigningKey::from_bytes(&seed);

        // Verify the key matches the from address
        let derived_addr = self.address_from_key(private_key.as_bytes())?;
        if derived_addr != request.from {
            return Err(TransactionError::SigningFailed(
                "Private key does not match sender address".into(),
            ));
        }

        let from_pubkey = bs58::decode(&request.from)
            .into_vec()
            .map_err(|e| TransactionError::InvalidAddress(e.to_string()))?;
        let to_pubkey = bs58::decode(&request.to)
            .into_vec()
            .map_err(|e| TransactionError::InvalidAddress(e.to_string()))?;

        // Build a simplified Solana transaction message for a system-program
        // transfer.  This is NOT a complete Solana wire format but captures
        // the essential structure for offline signing.
        let message = build_transfer_message(
            &from_pubkey,
            &to_pubkey,
            request.amount,
            request.nonce.unwrap_or(0),
        );

        // Sign the message
        let signature = signing_key.sign(&message);
        let sig_bytes = signature.to_bytes();

        // Transaction hash is the base58-encoded signature
        let tx_hash = bs58::encode(&sig_bytes).into_string();

        // Build the full wire transaction: [signature_count, signature, message]
        let mut raw_tx = Vec::new();
        // Compact-u16 for signature count = 1
        raw_tx.push(1u8);
        raw_tx.extend_from_slice(&sig_bytes);
        raw_tx.extend_from_slice(&message);

        Ok(SignedTransaction {
            chain: "solana".into(),
            tx_hash,
            raw_tx,
            from: request.from.clone(),
            to: request.to.clone(),
            amount: request.amount,
        })
    }
}

// ---------------------------------------------------------------------------
// Simplified Solana transfer message builder
// ---------------------------------------------------------------------------

/// System program public key (all zeros).
const SYSTEM_PROGRAM: [u8; 32] = [0u8; 32];

/// Build a simplified Solana transaction message for a SOL transfer.
///
/// Layout (V0 legacy message):
///   - header: [num_required_sigs, num_readonly_signed, num_readonly_unsigned]
///   - num_accounts (compact-u16)
///   - accounts: [from_pubkey(32), to_pubkey(32), system_program(32)]
///   - recent_blockhash (32 bytes) -- we use a hash of the nonce as placeholder
///   - num_instructions (compact-u16)
///   - instruction: program_id_index, accounts, data
fn build_transfer_message(
    from_pubkey: &[u8],
    to_pubkey: &[u8],
    lamports: u128,
    nonce: u64,
) -> Vec<u8> {
    let mut msg = Vec::new();

    // ---- Header ----
    msg.push(1); // num_required_signatures
    msg.push(0); // num_readonly_signed_accounts
    msg.push(1); // num_readonly_unsigned_accounts (system program)

    // ---- Account addresses (compact-u16 count = 3) ----
    msg.push(3);
    msg.extend_from_slice(from_pubkey); // index 0: signer + writable
    msg.extend_from_slice(to_pubkey); // index 1: writable
    msg.extend_from_slice(&SYSTEM_PROGRAM); // index 2: readonly

    // ---- Recent blockhash ----
    // In production this would come from the RPC; we derive a deterministic
    // placeholder from the nonce so that signing is fully offline-testable.
    let blockhash_placeholder = Sha256::digest(nonce.to_le_bytes());
    msg.extend_from_slice(&blockhash_placeholder);

    // ---- Instructions (compact-u16 count = 1) ----
    msg.push(1);

    // -- Instruction: System Program Transfer --
    // program_id_index
    msg.push(2); // system program at index 2

    // account indices (compact-u16 length = 2)
    msg.push(2);
    msg.push(0); // from
    msg.push(1); // to

    // instruction data: Transfer instruction
    // Solana system transfer instruction = [2, 0, 0, 0] (u32 LE) + lamports (u64 LE)
    // The transfer instruction index is 2.
    let lamports_u64 = lamports as u64; // Solana uses u64 for lamports
    let mut ix_data = Vec::with_capacity(12);
    ix_data.extend_from_slice(&2u32.to_le_bytes()); // instruction index = Transfer
    ix_data.extend_from_slice(&lamports_u64.to_le_bytes());

    // data length (compact-u16)
    msg.push(ix_data.len() as u8);
    msg.extend_from_slice(&ix_data);

    msg
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_derivation() {
        let sol = SolanaChain::mainnet();
        let key = [1u8; 32];
        let addr = sol.address_from_key(&key).unwrap();

        // Ed25519 public key for seed=[1;32] should decode to 32 bytes
        let decoded = bs58::decode(&addr).into_vec().unwrap();
        assert_eq!(decoded.len(), 32);
        assert!(sol.validate_address(&addr));
    }

    #[test]
    fn address_deterministic() {
        let sol = SolanaChain::mainnet();
        let key = [42u8; 32];
        let a1 = sol.address_from_key(&key).unwrap();
        let a2 = sol.address_from_key(&key).unwrap();
        assert_eq!(a1, a2);
    }

    #[test]
    fn address_from_key_wrong_length() {
        let sol = SolanaChain::mainnet();
        assert!(sol.address_from_key(&[1u8; 16]).is_err());
    }

    #[test]
    fn validate_address_valid() {
        let sol = SolanaChain::mainnet();
        // The system program address (all zeros) in base58
        assert!(sol.validate_address("11111111111111111111111111111111"));
    }

    #[test]
    fn validate_address_invalid() {
        let sol = SolanaChain::mainnet();
        assert!(!sol.validate_address("")); // empty
        assert!(!sol.validate_address("0x1234")); // Ethereum-style
        assert!(!sol.validate_address("too_short!!")); // invalid base58 chars
    }

    #[test]
    fn sign_transaction_basic() {
        let sol = SolanaChain::mainnet();
        let key = [7u8; 32];
        let private_key = SecretBytes::new(key.to_vec());
        let from_addr = sol.address_from_key(&key).unwrap();

        // Use a second key for the recipient
        let to_key = [8u8; 32];
        let to_addr = sol.address_from_key(&to_key).unwrap();

        let request = TransactionRequest {
            chain: "solana".into(),
            from: from_addr.clone(),
            to: to_addr.clone(),
            amount: 1_000_000_000, // 1 SOL
            fee_limit: None,
            data: None,
            nonce: Some(0),
            memo: None,
        };

        let signed = sol.sign_transaction(&request, &private_key).unwrap();
        assert_eq!(signed.chain, "solana");
        assert!(!signed.tx_hash.is_empty());
        assert!(!signed.raw_tx.is_empty());
        assert_eq!(signed.from, from_addr);
        assert_eq!(signed.to, to_addr);
        assert_eq!(signed.amount, 1_000_000_000);

        // Verify the signature: first byte is sig count (1), then 64 bytes sig,
        // then the message.
        assert_eq!(signed.raw_tx[0], 1u8);
        let sig_bytes: [u8; 64] = signed.raw_tx[1..65].try_into().unwrap();
        let message = &signed.raw_tx[65..];

        let signing_key = Ed25519SigningKey::from_bytes(&key);
        let verifying_key = signing_key.verifying_key();
        let sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
        assert!(verifying_key.verify_strict(message, &sig).is_ok());
    }

    #[test]
    fn sign_transaction_wrong_key() {
        let sol = SolanaChain::mainnet();
        let key = [7u8; 32];
        let private_key = SecretBytes::new(key.to_vec());

        // Use the wrong from address
        let wrong_key = [8u8; 32];
        let wrong_addr = sol.address_from_key(&wrong_key).unwrap();

        let request = TransactionRequest {
            chain: "solana".into(),
            from: wrong_addr,
            to: sol.address_from_key(&[9u8; 32]).unwrap(),
            amount: 100,
            fee_limit: None,
            data: None,
            nonce: None,
            memo: None,
        };

        let result = sol.sign_transaction(&request, &private_key);
        assert!(result.is_err());
    }

    #[test]
    fn cluster_rpc_urls() {
        assert_eq!(
            SolanaCluster::Mainnet.rpc_url(),
            "https://api.mainnet-beta.solana.com"
        );
        assert_eq!(
            SolanaCluster::Devnet.rpc_url(),
            "https://api.devnet.solana.com"
        );
    }

    #[test]
    fn coin_type_is_501() {
        let sol = SolanaChain::mainnet();
        assert_eq!(sol.coin_type(), 501);
    }
}
