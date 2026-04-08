//! Multi-chain wallet manager.
//!
//! Orchestrates mnemonic-based key derivation, per-chain account management,
//! encrypted keystore persistence, and offline transaction signing.  Private
//! key material is kept in [`SecretBytes`] containers that are zeroized on
//! drop, and the wallet exposes explicit `lock`/`unlock` lifecycle methods.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::chains::ChainRegistry;
use crate::hd::{self, DerivationPath};
use crate::keystore::EncryptedKeystore;
use crate::mnemonic::Mnemonic;
use crate::security::{constant_time_eq, derive_key_from_password, generate_salt, SecretBytes};
use crate::transaction::{SignedTransaction, TransactionError, TransactionRequest};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// WalletAccount
// ---------------------------------------------------------------------------

/// A single account within the wallet, identified by chain + derivation path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAccount {
    /// Chain identifier (e.g. "ethereum", "solana").
    pub chain: String,
    /// On-chain address in the chain's native format.
    pub address: String,
    /// BIP-44 derivation path used to generate this account's key.
    pub derivation_path: String,
    /// Optional human-readable label.
    pub label: Option<String>,
    /// Path to the encrypted keystore file on disk (relative to wallet dir).
    pub keystore_path: Option<String>,
}

// ---------------------------------------------------------------------------
// Persisted wallet metadata (written to wallet.json)
// ---------------------------------------------------------------------------

/// Serializable wallet metadata (persisted to `wallet.json`).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WalletMetadata {
    name: String,
    accounts: Vec<WalletAccount>,
    /// Hex-encoded salt used for password verification.
    password_salt: String,
    /// Hex-encoded hash used to verify the password without decrypting keys.
    password_hash: String,
    /// Hex-encoded encrypted mnemonic (AES-256-GCM).
    encrypted_mnemonic: String,
    /// Hex-encoded salt for the mnemonic encryption key.
    mnemonic_salt: String,
}

// ---------------------------------------------------------------------------
// Wallet
// ---------------------------------------------------------------------------

/// The multi-chain wallet.
///
/// # Security invariants
///
/// - Private keys are stored encrypted on disk; decrypted keys only live in
///   `cached_keys` while the wallet is unlocked.
/// - [`Wallet::lock`] zeroizes every cached key and is automatically called
///   by the [`Drop`] implementation.
/// - Transaction signing requires the wallet to be unlocked.
/// - The mnemonic phrase is encrypted separately and can only be exported with
///   the correct password.
pub struct Wallet {
    /// Human-readable wallet name.
    pub name: String,
    /// All accounts across all chains.
    pub accounts: Vec<WalletAccount>,
    /// Registry of supported chain backends.
    pub chains: ChainRegistry,
    /// Directory containing wallet.json and keystore files.
    pub wallet_dir: PathBuf,
    /// Whether the wallet is currently locked.
    locked: bool,
    /// Cached decrypted private keys, keyed by address.
    /// Cleared (zeroized) on lock.
    cached_keys: HashMap<String, SecretBytes>,
    /// Password verification salt.
    password_salt: Vec<u8>,
    /// Password verification hash (Argon2id output).
    password_hash: Vec<u8>,
    /// Encrypted mnemonic blob.
    encrypted_mnemonic: Vec<u8>,
    /// Salt for mnemonic encryption key derivation.
    mnemonic_salt: Vec<u8>,
}

impl Wallet {
    // -------------------------------------------------------------------
    // Construction
    // -------------------------------------------------------------------

    /// Create a brand-new wallet from a mnemonic phrase.
    ///
    /// This derives initial accounts for every chain in the default registry,
    /// encrypts the mnemonic and per-account keys to disk, and returns an
    /// **unlocked** wallet.
    pub fn create(
        name: &str,
        mnemonic: &Mnemonic,
        password: &str,
        wallet_dir: &Path,
    ) -> Result<Self, WalletError> {
        std::fs::create_dir_all(wallet_dir)?;

        let chains = ChainRegistry::with_defaults();

        // -- Password verification material --
        let pw_salt = generate_salt();
        let pw_hash = derive_key_from_password(password, &pw_salt)?;

        // -- Encrypt mnemonic --
        let mn_salt = generate_salt();
        let mn_key = derive_key_from_password(password, &mn_salt)?;
        let encrypted_mnemonic =
            crate::security::encrypt_aes256gcm(&mn_key, mnemonic.phrase().as_bytes())?;

        // -- Derive initial accounts (index 0) for each chain --
        let seed = mnemonic.to_seed("");
        let mut accounts = Vec::new();
        let mut cached_keys = HashMap::new();

        let chain_ids: Vec<String> = chains
            .supported_chains()
            .iter()
            .map(|s| s.to_string())
            .collect();

        for chain_id in &chain_ids {
            let backend = chains.get(chain_id).unwrap();
            let path = derivation_path_for_chain(chain_id, 0, 0)?;
            let derived_key =
                hd::derive_key_from_seed(seed.as_bytes(), &path).map_err(|e: hd::HdError| {
                    WalletError::Security(
                        crate::security::WalletSecurityError::KeyDerivationFailed(e.to_string()),
                    )
                })?;

            let address = backend.address_from_key(derived_key.as_bytes())?;

            // Encrypt the key to a keystore file
            let keystore_name = format!("{}_{}.json", chain_id, 0);
            let keystore =
                EncryptedKeystore::encrypt(derived_key.as_bytes(), password, &address, chain_id)?;
            keystore.save_to_file(&wallet_dir.join(&keystore_name))?;

            // Cache the decrypted key
            cached_keys.insert(address.clone(), derived_key);

            accounts.push(WalletAccount {
                chain: chain_id.clone(),
                address,
                derivation_path: path.to_string(),
                label: Some(format!("{} Account 0", backend.symbol())),
                keystore_path: Some(keystore_name),
            });
        }

        let wallet = Wallet {
            name: name.to_string(),
            accounts,
            chains,
            wallet_dir: wallet_dir.to_path_buf(),
            locked: false,
            cached_keys,
            password_salt: pw_salt.to_vec(),
            password_hash: pw_hash.to_vec(),
            encrypted_mnemonic,
            mnemonic_salt: mn_salt.to_vec(),
        };

        wallet.save_metadata()?;

        tracing::info!(name = %name, "Wallet created");
        Ok(wallet)
    }

    /// Open an existing wallet from its directory.
    ///
    /// The wallet is returned in a **locked** state; call [`Wallet::unlock`]
    /// to decrypt keys into memory before signing.
    pub fn open(wallet_dir: &Path) -> Result<Self, WalletError> {
        let meta_path = wallet_dir.join("wallet.json");
        let json = std::fs::read_to_string(&meta_path)?;
        let meta: WalletMetadata =
            serde_json::from_str(&json).map_err(|e| WalletError::Serialization(e.to_string()))?;

        let wallet = Wallet {
            name: meta.name,
            accounts: meta.accounts,
            chains: ChainRegistry::with_defaults(),
            wallet_dir: wallet_dir.to_path_buf(),
            locked: true,
            cached_keys: HashMap::new(),
            password_salt: hex::decode(&meta.password_salt)
                .map_err(|e| WalletError::Serialization(e.to_string()))?,
            password_hash: hex::decode(&meta.password_hash)
                .map_err(|e| WalletError::Serialization(e.to_string()))?,
            encrypted_mnemonic: hex::decode(&meta.encrypted_mnemonic)
                .map_err(|e| WalletError::Serialization(e.to_string()))?,
            mnemonic_salt: hex::decode(&meta.mnemonic_salt)
                .map_err(|e| WalletError::Serialization(e.to_string()))?,
        };

        tracing::info!(name = %wallet.name, "Wallet opened (locked)");
        Ok(wallet)
    }

    // -------------------------------------------------------------------
    // Lock / Unlock
    // -------------------------------------------------------------------

    /// Unlock the wallet: verify the password, then decrypt all keystore files
    /// and cache the private keys in memory.
    pub fn unlock(&mut self, password: &str) -> Result<(), WalletError> {
        if !self.locked {
            return Ok(());
        }

        // Verify password
        if !self.verify_password(password) {
            return Err(WalletError::WrongPassword);
        }

        // Decrypt each account's keystore
        for account in &self.accounts {
            if let Some(ks_name) = &account.keystore_path {
                let ks_path = self.wallet_dir.join(ks_name);
                if ks_path.exists() {
                    let keystore = EncryptedKeystore::load_from_file(&ks_path)
                        .map_err(|e| WalletError::Serialization(e.to_string()))?;
                    let key = keystore.decrypt(password)?;
                    self.cached_keys.insert(account.address.clone(), key);
                }
            }
        }

        self.locked = false;
        tracing::info!(name = %self.name, "Wallet unlocked");
        Ok(())
    }

    /// Lock the wallet: zeroize all cached private keys.
    pub fn lock(&mut self) {
        // Zeroize each cached key explicitly, then clear the map.
        // SecretBytes implements Zeroize on Drop, but we also clear the map
        // entries so no references remain.
        self.cached_keys.clear();
        self.locked = true;
        tracing::debug!(name = %self.name, "Wallet locked — all cached keys zeroized");
    }

    /// Returns `true` if the wallet is locked (no keys in memory).
    pub fn is_locked(&self) -> bool {
        self.locked
    }

    // -------------------------------------------------------------------
    // Account management
    // -------------------------------------------------------------------

    /// Add a new account for the given chain at the next derivation index.
    ///
    /// Requires the password to decrypt the mnemonic and derive a new key.
    pub fn add_account(
        &mut self,
        chain: &str,
        password: &str,
        label: Option<&str>,
    ) -> Result<WalletAccount, WalletError> {
        // Verify password
        if !self.verify_password(password) {
            return Err(WalletError::WrongPassword);
        }

        let backend = self
            .chains
            .get(chain)
            .ok_or_else(|| WalletError::UnsupportedChain(chain.to_string()))?;

        // Determine next account index for this chain
        let existing_count = self.accounts.iter().filter(|a| a.chain == chain).count() as u32;

        // Decrypt the mnemonic
        let mnemonic = self.decrypt_mnemonic(password)?;
        let seed = mnemonic.to_seed("");

        let path = derivation_path_for_chain(chain, 0, existing_count)?;
        let derived_key =
            hd::derive_key_from_seed(seed.as_bytes(), &path).map_err(|e: hd::HdError| {
                WalletError::Security(crate::security::WalletSecurityError::KeyDerivationFailed(
                    e.to_string(),
                ))
            })?;

        let address = backend.address_from_key(derived_key.as_bytes())?;

        // Encrypt and persist keystore
        let keystore_name = format!("{}_{}.json", chain, existing_count);
        let keystore =
            EncryptedKeystore::encrypt(derived_key.as_bytes(), password, &address, chain)?;
        keystore.save_to_file(&self.wallet_dir.join(&keystore_name))?;

        // Cache key if wallet is unlocked
        if !self.locked {
            self.cached_keys.insert(address.clone(), derived_key);
        }

        let account = WalletAccount {
            chain: chain.to_string(),
            address,
            derivation_path: path.to_string(),
            label: label.map(String::from),
            keystore_path: Some(keystore_name),
        };

        self.accounts.push(account.clone());
        self.save_metadata()?;

        tracing::info!(
            chain = %chain,
            address = %account.address,
            "Account added"
        );
        Ok(account)
    }

    /// Get all accounts.
    pub fn accounts(&self) -> &[WalletAccount] {
        &self.accounts
    }

    /// Get accounts belonging to a specific chain.
    pub fn accounts_for_chain(&self, chain: &str) -> Vec<&WalletAccount> {
        self.accounts.iter().filter(|a| a.chain == chain).collect()
    }

    // -------------------------------------------------------------------
    // Transaction signing
    // -------------------------------------------------------------------

    /// Sign a transaction. The wallet must be unlocked.
    ///
    /// The `from` field of the request is used to look up the cached private
    /// key.  The request's `chain` field selects the appropriate
    /// [`ChainBackend`](crate::chains::ChainBackend).
    pub fn sign_transaction(
        &self,
        request: &TransactionRequest,
    ) -> Result<SignedTransaction, WalletError> {
        if self.locked {
            return Err(WalletError::WalletLocked);
        }

        let backend = self
            .chains
            .get(&request.chain)
            .ok_or_else(|| WalletError::UnsupportedChain(request.chain.clone()))?;

        let private_key = self
            .cached_keys
            .get(&request.from)
            .ok_or_else(|| WalletError::AccountNotFound(request.from.clone()))?;

        let signed = backend.sign_transaction(request, private_key)?;
        Ok(signed)
    }

    // -------------------------------------------------------------------
    // Address helpers
    // -------------------------------------------------------------------

    /// Get the address for a specific chain and account index.
    pub fn get_address(&self, chain: &str, index: u32) -> Result<String, WalletError> {
        let chain_accounts = self.accounts_for_chain(chain);
        chain_accounts
            .get(index as usize)
            .map(|a| a.address.clone())
            .ok_or_else(|| WalletError::AccountNotFound(format!("{}:{}", chain, index)))
    }

    // -------------------------------------------------------------------
    // Mnemonic management
    // -------------------------------------------------------------------

    /// Export the mnemonic phrase.  Requires password verification.
    ///
    /// The caller is responsible for securely displaying / storing the phrase.
    pub fn export_mnemonic(&self, password: &str) -> Result<String, WalletError> {
        if !self.verify_password(password) {
            return Err(WalletError::WrongPassword);
        }
        let mnemonic = self.decrypt_mnemonic(password)?;
        Ok(mnemonic.phrase().to_string())
    }

    /// Verify a password against the stored password hash using constant-time
    /// comparison.
    pub fn verify_password(&self, password: &str) -> bool {
        match derive_key_from_password(password, &self.password_salt) {
            Ok(derived) => constant_time_eq(&derived, &self.password_hash),
            Err(_) => false,
        }
    }

    // -------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------

    /// Persist wallet metadata to `wallet.json`.
    fn save_metadata(&self) -> Result<(), WalletError> {
        let meta = WalletMetadata {
            name: self.name.clone(),
            accounts: self.accounts.clone(),
            password_salt: hex::encode(&self.password_salt),
            password_hash: hex::encode(&self.password_hash),
            encrypted_mnemonic: hex::encode(&self.encrypted_mnemonic),
            mnemonic_salt: hex::encode(&self.mnemonic_salt),
        };
        let json = serde_json::to_string_pretty(&meta)
            .map_err(|e| WalletError::Serialization(e.to_string()))?;
        std::fs::write(self.wallet_dir.join("wallet.json"), json)?;
        Ok(())
    }

    /// Decrypt the stored mnemonic phrase.
    fn decrypt_mnemonic(&self, password: &str) -> Result<Mnemonic, WalletError> {
        let mn_key = derive_key_from_password(password, &self.mnemonic_salt)?;
        let decrypted = crate::security::decrypt_aes256gcm(&mn_key, &self.encrypted_mnemonic)?;
        let phrase = String::from_utf8(decrypted.as_bytes().to_vec())
            .map_err(|e| WalletError::Serialization(e.to_string()))?;
        let mnemonic = Mnemonic::from_phrase(&phrase)
            .map_err(|e| WalletError::Serialization(e.to_string()))?;
        Ok(mnemonic)
    }
}

// ---------------------------------------------------------------------------
// Helper: build a DerivationPath for a known chain
// ---------------------------------------------------------------------------

/// Build the appropriate BIP-44 derivation path for a chain identifier.
fn derivation_path_for_chain(
    chain: &str,
    account: u32,
    index: u32,
) -> Result<DerivationPath, WalletError> {
    match chain {
        // Ethereum and all EVM-compatible chains use BIP-44 coin type 60.
        "ethereum" | "bsc" | "polygon" | "arbitrum" | "optimism" | "base" | "avalanche" => {
            Ok(DerivationPath::ethereum(account, index))
        }
        "solana" => Ok(DerivationPath::solana(account, index)),
        other => Err(WalletError::UnsupportedChain(other.to_string())),
    }
}

// ---------------------------------------------------------------------------
// Drop — always zeroize cached keys
// ---------------------------------------------------------------------------

impl Drop for Wallet {
    fn drop(&mut self) {
        self.lock();
    }
}

// ---------------------------------------------------------------------------
// WalletError
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum WalletError {
    #[error("Wallet is locked -- call unlock() first")]
    WalletLocked,

    #[error("Wrong password")]
    WrongPassword,

    #[error("Account not found: {0}")]
    AccountNotFound(String),

    #[error("Chain not supported: {0}")]
    UnsupportedChain(String),

    #[error("Security error: {0}")]
    Security(#[from] crate::security::WalletSecurityError),

    #[error("Transaction error: {0}")]
    Transaction(#[from] TransactionError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Generate a test mnemonic (12 words, not cryptographically generated).
    fn test_mnemonic() -> Mnemonic {
        Mnemonic::from_phrase(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        ).unwrap()
    }

    #[test]
    fn create_and_open_wallet() {
        let dir = std::env::temp_dir().join("openbee_wallet_test_create");
        let _ = std::fs::remove_dir_all(&dir);

        let mnemonic = test_mnemonic();
        let password = "test-password-123";

        // Create
        let wallet = Wallet::create("test", &mnemonic, password, &dir).unwrap();
        assert!(!wallet.is_locked());
        assert_eq!(wallet.name, "test");
        assert!(!wallet.accounts.is_empty());

        // Should have accounts for both chains
        let eth_accounts = wallet.accounts_for_chain("ethereum");
        let sol_accounts = wallet.accounts_for_chain("solana");
        assert_eq!(eth_accounts.len(), 1);
        assert_eq!(sol_accounts.len(), 1);

        // Addresses should be valid
        let eth_addr = &eth_accounts[0].address;
        assert!(eth_addr.starts_with("0x"));
        assert_eq!(eth_addr.len(), 42);

        drop(wallet);

        // Open
        let wallet2 = Wallet::open(&dir).unwrap();
        assert!(wallet2.is_locked());
        assert_eq!(wallet2.name, "test");
        assert_eq!(wallet2.accounts.len(), 2);

        // Clean up
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unlock_and_lock() {
        let dir = std::env::temp_dir().join("openbee_wallet_test_unlock");
        let _ = std::fs::remove_dir_all(&dir);

        let mnemonic = test_mnemonic();
        let password = "unlock-test";

        let _wallet = Wallet::create("test", &mnemonic, password, &dir).unwrap();
        drop(_wallet);

        let mut wallet = Wallet::open(&dir).unwrap();
        assert!(wallet.is_locked());

        // Wrong password
        assert!(wallet.unlock("wrong").is_err());
        assert!(wallet.is_locked());

        // Correct password
        wallet.unlock(password).unwrap();
        assert!(!wallet.is_locked());

        // Lock
        wallet.lock();
        assert!(wallet.is_locked());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn sign_transaction_requires_unlock() {
        let dir = std::env::temp_dir().join("openbee_wallet_test_sign");
        let _ = std::fs::remove_dir_all(&dir);

        let mnemonic = test_mnemonic();
        let password = "sign-test";

        let wallet = Wallet::create("test", &mnemonic, password, &dir).unwrap();
        let eth_addr = wallet.accounts_for_chain("ethereum")[0].address.clone();
        drop(wallet);

        let mut wallet = Wallet::open(&dir).unwrap();

        // Try signing while locked
        let request = TransactionRequest {
            chain: "ethereum".into(),
            from: eth_addr.clone(),
            to: "0x0000000000000000000000000000000000000001".into(),
            amount: 1000,
            fee_limit: Some(21_000),
            data: None,
            nonce: Some(0),
            memo: None,
        };

        let result = wallet.sign_transaction(&request);
        assert!(matches!(result, Err(WalletError::WalletLocked)));

        // Unlock and sign
        wallet.unlock(password).unwrap();
        let signed = wallet.sign_transaction(&request).unwrap();
        assert_eq!(signed.chain, "ethereum");
        assert!(signed.tx_hash.starts_with("0x"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn add_account() {
        let dir = std::env::temp_dir().join("openbee_wallet_test_add_account");
        let _ = std::fs::remove_dir_all(&dir);

        let mnemonic = test_mnemonic();
        let password = "add-acct";

        let mut wallet = Wallet::create("test", &mnemonic, password, &dir).unwrap();
        assert_eq!(wallet.accounts_for_chain("ethereum").len(), 1);

        let account = wallet
            .add_account("ethereum", password, Some("Savings"))
            .unwrap();
        assert_eq!(account.chain, "ethereum");
        assert_eq!(account.label.as_deref(), Some("Savings"));
        assert_eq!(wallet.accounts_for_chain("ethereum").len(), 2);

        // The two accounts should have different addresses
        let addrs: Vec<&str> = wallet
            .accounts_for_chain("ethereum")
            .iter()
            .map(|a| a.address.as_str())
            .collect();
        assert_ne!(addrs[0], addrs[1]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_mnemonic() {
        let dir = std::env::temp_dir().join("openbee_wallet_test_export_mn");
        let _ = std::fs::remove_dir_all(&dir);

        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic = Mnemonic::from_phrase(phrase).unwrap();
        let password = "export-test";

        let wallet = Wallet::create("test", &mnemonic, password, &dir).unwrap();
        let exported = wallet.export_mnemonic(password).unwrap();
        assert_eq!(exported, phrase);

        // Wrong password should fail
        assert!(wallet.export_mnemonic("wrong").is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn verify_password_works() {
        let dir = std::env::temp_dir().join("openbee_wallet_test_verify_pw");
        let _ = std::fs::remove_dir_all(&dir);

        let mnemonic = test_mnemonic();
        let password = "correct-horse-battery-staple";

        let wallet = Wallet::create("test", &mnemonic, password, &dir).unwrap();
        assert!(wallet.verify_password(password));
        assert!(!wallet.verify_password("wrong-password"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn get_address() {
        let dir = std::env::temp_dir().join("openbee_wallet_test_get_addr");
        let _ = std::fs::remove_dir_all(&dir);

        let mnemonic = test_mnemonic();
        let password = "get-addr";

        let wallet = Wallet::create("test", &mnemonic, password, &dir).unwrap();

        let addr = wallet.get_address("ethereum", 0).unwrap();
        assert!(addr.starts_with("0x"));

        let addr = wallet.get_address("solana", 0).unwrap();
        assert!(!addr.is_empty());

        // Out of bounds
        assert!(wallet.get_address("ethereum", 99).is_err());

        // Unsupported chain
        assert!(wallet.get_address("bitcoin", 0).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
