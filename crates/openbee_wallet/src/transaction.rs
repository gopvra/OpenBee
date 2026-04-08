use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Chain-agnostic transaction request
// ---------------------------------------------------------------------------

/// A chain-agnostic transaction request describing an intent to transfer value
/// or invoke a smart contract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    /// Target blockchain identifier (e.g. "ethereum", "solana").
    pub chain: String,
    /// Sender address in the chain's native format.
    pub from: String,
    /// Recipient address.
    pub to: String,
    /// Amount in the chain's smallest unit (wei for ETH, lamports for SOL).
    pub amount: u128,
    /// Maximum fee the sender is willing to pay (gas limit * gas price, etc.).
    pub fee_limit: Option<u64>,
    /// Opaque call-data: contract ABI payload (Ethereum) or instruction data (Solana).
    pub data: Option<Vec<u8>>,
    /// Explicit nonce; when `None` the backend should determine the next nonce.
    pub nonce: Option<u64>,
    /// Human-readable memo attached to the transaction.
    pub memo: Option<String>,
}

// ---------------------------------------------------------------------------
// Signed transaction
// ---------------------------------------------------------------------------

/// A fully signed transaction that is ready for broadcast to the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    /// Chain the transaction belongs to.
    pub chain: String,
    /// Hex-encoded transaction hash.
    pub tx_hash: String,
    /// Serialized, signed transaction bytes (ready for `eth_sendRawTransaction`
    /// or Solana `sendTransaction`).
    pub raw_tx: Vec<u8>,
    /// Sender address.
    pub from: String,
    /// Recipient address.
    pub to: String,
    /// Transfer amount in the chain's smallest unit.
    pub amount: u128,
}

// ---------------------------------------------------------------------------
// Transaction status
// ---------------------------------------------------------------------------

/// Status of a previously broadcast transaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TransactionStatus {
    /// Transaction has been submitted but not yet included in a block.
    Pending,
    /// Transaction is confirmed with the given number of block confirmations.
    Confirmed { confirmations: u64 },
    /// Transaction execution failed.
    Failed { reason: String },
    /// Status could not be determined.
    Unknown,
}

// ---------------------------------------------------------------------------
// Balance
// ---------------------------------------------------------------------------

/// Result of querying the native-token balance of an address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// Chain identifier.
    pub chain: String,
    /// Address that was queried.
    pub address: String,
    /// Balance in the chain's smallest unit.
    pub amount: u128,
    /// Number of decimal places (18 for ETH, 9 for SOL).
    pub decimals: u8,
    /// Ticker symbol (e.g. "ETH", "SOL").
    pub symbol: String,
}

impl Balance {
    /// Format the balance as a human-readable decimal string.
    ///
    /// ```text
    /// Balance { amount: 1_234_567_890, decimals: 9, .. }.display_amount()
    /// // => "1.234567890"
    /// ```
    pub fn display_amount(&self) -> String {
        if self.decimals == 0 {
            return self.amount.to_string();
        }

        let divisor = 10u128.pow(self.decimals as u32);
        let whole = self.amount / divisor;
        let frac = self.amount % divisor;

        // Pad fractional part with leading zeros up to `decimals` width.
        format!("{whole}.{frac:0>width$}", width = self.decimals as usize)
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors that can occur during transaction construction, signing, or broadcast.
#[derive(Debug, thiserror::Error)]
pub enum TransactionError {
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Insufficient balance: have {have}, need {need}")]
    InsufficientBalance { have: u128, need: u128 },

    #[error("Signing failed: {0}")]
    SigningFailed(String),

    #[error("Broadcast failed: {0}")]
    BroadcastFailed(String),

    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Chain not supported: {0}")]
    UnsupportedChain(String),
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_amount_eth() {
        let b = Balance {
            chain: "ethereum".into(),
            address: "0x0000000000000000000000000000000000000000".into(),
            amount: 1_500_000_000_000_000_000, // 1.5 ETH
            decimals: 18,
            symbol: "ETH".into(),
        };
        assert_eq!(b.display_amount(), "1.500000000000000000");
    }

    #[test]
    fn display_amount_sol() {
        let b = Balance {
            chain: "solana".into(),
            address: "So11111111111111111111111111111111111111112".into(),
            amount: 1_234_567_890,
            decimals: 9,
            symbol: "SOL".into(),
        };
        assert_eq!(b.display_amount(), "1.234567890");
    }

    #[test]
    fn display_amount_zero() {
        let b = Balance {
            chain: "ethereum".into(),
            address: String::new(),
            amount: 0,
            decimals: 18,
            symbol: "ETH".into(),
        };
        assert_eq!(b.display_amount(), "0.000000000000000000");
    }

    #[test]
    fn display_amount_no_decimals() {
        let b = Balance {
            chain: "test".into(),
            address: String::new(),
            amount: 42,
            decimals: 0,
            symbol: "TST".into(),
        };
        assert_eq!(b.display_amount(), "42");
    }

    #[test]
    fn transaction_request_serde_roundtrip() {
        let req = TransactionRequest {
            chain: "ethereum".into(),
            from: "0xabc".into(),
            to: "0xdef".into(),
            amount: 1000,
            fee_limit: Some(21000),
            data: None,
            nonce: Some(5),
            memo: Some("test".into()),
        };
        let json = serde_json::to_string(&req).unwrap();
        let deser: TransactionRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(deser.chain, "ethereum");
        assert_eq!(deser.amount, 1000);
    }
}
