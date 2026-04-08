//! # OpenBee Wallet SDK
//!
//! Secure multi-chain cryptocurrency wallet with local-only key management.
//!
//! ## Security Architecture
//!
//! - **Default OFF** — wallet is disabled until user explicitly enables it after
//!   reading and accepting a security warning
//! - **Pure local** — private keys NEVER leave the device, NEVER transmitted
//! - **AES-256-GCM** — private keys encrypted at rest with authenticated encryption
//! - **Argon2id KDF** — password-based key derivation resistant to GPU/ASIC attacks
//! - **Machine binding** — encryption key incorporates hardware fingerprint;
//!   stolen keystore file is useless on a different computer
//! - **Zeroize** — all secret material scrubbed from memory immediately after use
//! - **Auto-wipe** — keystore destroyed after N consecutive wrong password attempts
//! - **No logging** — private keys and mnemonics are NEVER written to logs
//!
//! ## Supported Chains
//!
//! - Ethereum (ETH) — secp256k1 / EIP-155 transactions
//! - Solana (SOL) — Ed25519 signing
//! - Extensible — implement `ChainBackend` trait for additional chains

pub mod address;
pub mod chains;
pub mod hd;
pub mod keystore;
pub mod machine_binding;
pub mod mnemonic;
pub mod rpc;
pub mod safety;
pub mod security;
pub mod transaction;
pub mod wallet;
