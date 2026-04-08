//! Placeholder RPC client for blockchain interaction.
//!
//! In production this module would use `reqwest` or `hyper` to issue JSON-RPC
//! calls to Ethereum / Solana nodes.  Because the SDK intentionally avoids
//! heavy HTTP dependencies for now, every method returns a descriptive error
//! indicating that real network access is not yet wired up.

use crate::transaction::{TransactionError, TransactionStatus};

// ---------------------------------------------------------------------------
// RpcClient
// ---------------------------------------------------------------------------

/// Blockchain RPC client (currently a non-functional placeholder).
///
/// Instantiate with an endpoint URL and optional tuning knobs.  All async
/// methods immediately return `TransactionError::RpcError` until a real HTTP
/// transport is plugged in.
#[derive(Debug, Clone)]
pub struct RpcClient {
    /// JSON-RPC endpoint URL (e.g. `https://mainnet.infura.io/v3/KEY`).
    pub endpoint: String,
    /// Per-request timeout in milliseconds.
    pub timeout_ms: u64,
    /// Number of automatic retries on transient failures.
    pub max_retries: u32,
}

impl RpcClient {
    /// Create a new RPC client pointing at `endpoint`.
    pub fn new(endpoint: &str) -> Self {
        Self {
            endpoint: endpoint.to_string(),
            timeout_ms: 30_000,
            max_retries: 3,
        }
    }

    /// Override the request timeout.
    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Override the retry count.
    pub fn with_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    // -----------------------------------------------------------------------
    // Placeholder methods — all return errors until a real transport exists
    // -----------------------------------------------------------------------

    /// Fetch the native-token balance for `address`.
    pub async fn get_balance(&self, address: &str) -> Result<u128, TransactionError> {
        tracing::debug!(
            endpoint = %self.endpoint,
            address = %address,
            "RPC get_balance (placeholder)"
        );
        Err(TransactionError::RpcError(format!(
            "RPC not implemented: get_balance for {} at {}",
            address, self.endpoint
        )))
    }

    /// Fetch the current nonce / sequence number for `address`.
    pub async fn get_nonce(&self, address: &str) -> Result<u64, TransactionError> {
        tracing::debug!(
            endpoint = %self.endpoint,
            address = %address,
            "RPC get_nonce (placeholder)"
        );
        Err(TransactionError::RpcError(format!(
            "RPC not implemented: get_nonce for {} at {}",
            address, self.endpoint
        )))
    }

    /// Broadcast a signed transaction and return its hash.
    pub async fn broadcast_transaction(&self, raw_tx: &[u8]) -> Result<String, TransactionError> {
        tracing::debug!(
            endpoint = %self.endpoint,
            tx_len = raw_tx.len(),
            "RPC broadcast_transaction (placeholder)"
        );
        Err(TransactionError::BroadcastFailed(format!(
            "RPC not implemented: broadcast_transaction ({} bytes) at {}",
            raw_tx.len(),
            self.endpoint
        )))
    }

    /// Query the status of a previously broadcast transaction.
    pub async fn get_transaction_status(
        &self,
        tx_hash: &str,
    ) -> Result<TransactionStatus, TransactionError> {
        tracing::debug!(
            endpoint = %self.endpoint,
            tx_hash = %tx_hash,
            "RPC get_transaction_status (placeholder)"
        );
        Err(TransactionError::RpcError(format!(
            "RPC not implemented: get_transaction_status for {} at {}",
            tx_hash, self.endpoint
        )))
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rpc_client_builder() {
        let client = RpcClient::new("https://example.com")
            .with_timeout(5_000)
            .with_retries(1);
        assert_eq!(client.endpoint, "https://example.com");
        assert_eq!(client.timeout_ms, 5_000);
        assert_eq!(client.max_retries, 1);
    }

    #[tokio::test]
    async fn get_balance_returns_error() {
        let client = RpcClient::new("https://localhost:8545");
        let result = client.get_balance("0xdeadbeef").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn broadcast_returns_error() {
        let client = RpcClient::new("https://localhost:8545");
        let result = client.broadcast_transaction(&[0u8; 100]).await;
        assert!(result.is_err());
    }
}
