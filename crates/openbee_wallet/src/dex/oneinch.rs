//! 1inch DEX aggregator — the largest multi-chain DEX aggregator.
//!
//! Supports: Ethereum, BSC, Polygon, Arbitrum, Optimism, Base, Avalanche.
//!
//! This is a placeholder implementation. A production version would call
//! the 1inch Aggregation Router API to find the best swap route across
//! multiple DEXes and liquidity sources.

use super::swap::{DexBackend, DexError, SwapQuote, SwapRequest};

/// 1inch DEX aggregator backend.
///
/// 1inch splits a single swap across multiple DEXes to find the best price.
/// Each instance is configured for a specific chain.
pub struct OneInchAggregator {
    /// EIP-155 chain ID.
    pub chain_id: u64,
    /// Human-readable chain name, e.g. `"ethereum"`, `"bsc"`.
    pub chain_name: String,
}

impl OneInchAggregator {
    /// 1inch on Ethereum mainnet (chain id 1).
    pub fn ethereum() -> Self {
        Self {
            chain_id: 1,
            chain_name: "ethereum".into(),
        }
    }

    /// 1inch on BSC mainnet (chain id 56).
    pub fn bsc() -> Self {
        Self {
            chain_id: 56,
            chain_name: "bsc".into(),
        }
    }

    /// 1inch on Polygon mainnet (chain id 137).
    pub fn polygon() -> Self {
        Self {
            chain_id: 137,
            chain_name: "polygon".into(),
        }
    }

    /// 1inch on Arbitrum One (chain id 42161).
    pub fn arbitrum() -> Self {
        Self {
            chain_id: 42161,
            chain_name: "arbitrum".into(),
        }
    }

    /// 1inch on Optimism (chain id 10).
    pub fn optimism() -> Self {
        Self {
            chain_id: 10,
            chain_name: "optimism".into(),
        }
    }

    /// 1inch on Base (chain id 8453).
    pub fn base() -> Self {
        Self {
            chain_id: 8453,
            chain_name: "base".into(),
        }
    }

    /// 1inch on Avalanche C-Chain (chain id 43114).
    pub fn avalanche() -> Self {
        Self {
            chain_id: 43114,
            chain_name: "avalanche".into(),
        }
    }
}

/// 1inch Aggregation Router V6 `swap` function selector.
///
/// Function: `swap(address,(address,address,address,address,uint256,uint256,uint256),bytes,bytes)`
/// Selector: `0x12aa3caf`
const SWAP_SELECTOR: [u8; 4] = [0x12, 0xaa, 0x3c, 0xaf];

// ---------------------------------------------------------------------------
// ABI encoding helpers
// ---------------------------------------------------------------------------

/// ABI-encode an Ethereum address into a 32-byte word (left-padded with zeros).
fn encode_address(addr: &str) -> [u8; 32] {
    let mut word = [0u8; 32];
    let hex_str = addr.strip_prefix("0x").unwrap_or(addr);
    if let Ok(bytes) = hex::decode(hex_str) {
        let len = bytes.len().min(20);
        let start = 32 - len;
        word[start..start + len].copy_from_slice(&bytes[bytes.len() - len..]);
    }
    word
}

/// ABI-encode a `u128` value into a 32-byte big-endian word.
fn encode_uint256_from_u128(value: u128) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[16..32].copy_from_slice(&value.to_be_bytes());
    word
}

/// Build simplified 1inch swap calldata.
///
/// In production this would come from the 1inch API response. Here we build
/// a minimal placeholder that encodes the essential swap parameters.
fn build_swap_calldata(
    token_in: &str,
    token_out: &str,
    recipient: &str,
    amount_in: u128,
    minimum_out: u128,
) -> Vec<u8> {
    // Simplified encoding: selector + 5 ABI words
    let mut calldata = Vec::with_capacity(4 + 5 * 32);

    calldata.extend_from_slice(&SWAP_SELECTOR);
    calldata.extend_from_slice(&encode_address(token_in));
    calldata.extend_from_slice(&encode_address(token_out));
    calldata.extend_from_slice(&encode_address(recipient));
    calldata.extend_from_slice(&encode_uint256_from_u128(amount_in));
    calldata.extend_from_slice(&encode_uint256_from_u128(minimum_out));

    calldata
}

impl DexBackend for OneInchAggregator {
    fn name(&self) -> &str {
        "1inch"
    }

    fn supported_chain(&self) -> &str {
        &self.chain_name
    }

    fn get_quote(&self, request: &SwapRequest) -> Result<SwapQuote, DexError> {
        if request.chain != self.chain_name {
            return Err(DexError::UnsupportedChain(request.chain.clone()));
        }

        // Placeholder: a real implementation would call the 1inch Quote API:
        //   GET https://api.1inch.dev/swap/v6.0/{chainId}/quote
        //     ?src={tokenIn}&dst={tokenOut}&amount={amount}
        // 1inch typically finds better rates by splitting across DEXes.

        // Simulated 0.1% aggregator fee (lower than single-DEX due to
        // multi-path optimization).
        let fee_amount = request.amount_in / 1000; // 0.1%
        let after_fee = request.amount_in - fee_amount;
        let estimated_out = after_fee;

        let slippage_factor = request.slippage_bps as u128;
        let minimum_out = estimated_out * (10_000 - slippage_factor) / 10_000;

        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Ok(SwapQuote {
            amount_in: request.amount_in,
            amount_out: estimated_out,
            minimum_out,
            price_impact: 0.1,
            fee: fee_amount,
            route: vec![
                request.from_token.symbol.clone(),
                "1inch".into(),
                request.to_token.symbol.clone(),
            ],
            expires_at: now_secs + 60, // 1-minute quote validity
        })
    }

    fn build_swap_tx(&self, request: &SwapRequest, quote: &SwapQuote) -> Result<Vec<u8>, DexError> {
        if request.chain != self.chain_name {
            return Err(DexError::UnsupportedChain(request.chain.clone()));
        }

        let calldata = build_swap_calldata(
            &request.from_token.address,
            &request.to_token.address,
            &request.sender,
            quote.amount_in,
            quote.minimum_out,
        );

        Ok(calldata)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dex::swap::TokenInfo;

    #[test]
    fn test_swap_calldata_length() {
        let data = build_swap_calldata(
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "0x0000000000000000000000000000000000000001",
            1_000_000_000_000_000_000,
            950_000,
        );
        // 4 bytes selector + 5 * 32 bytes = 164 bytes
        assert_eq!(data.len(), 164);
        assert_eq!(&data[..4], &[0x12, 0xaa, 0x3c, 0xaf]);
    }

    #[test]
    fn test_oneinch_ethereum() {
        let inch = OneInchAggregator::ethereum();
        assert_eq!(inch.chain_id, 1);
        assert_eq!(inch.supported_chain(), "ethereum");
        assert_eq!(inch.name(), "1inch");
    }

    #[test]
    fn test_oneinch_bsc() {
        let inch = OneInchAggregator::bsc();
        assert_eq!(inch.chain_id, 56);
        assert_eq!(inch.supported_chain(), "bsc");
    }

    #[test]
    fn test_oneinch_all_chains() {
        let chains = vec![
            (OneInchAggregator::ethereum(), 1u64, "ethereum"),
            (OneInchAggregator::bsc(), 56, "bsc"),
            (OneInchAggregator::polygon(), 137, "polygon"),
            (OneInchAggregator::arbitrum(), 42161, "arbitrum"),
            (OneInchAggregator::optimism(), 10, "optimism"),
            (OneInchAggregator::base(), 8453, "base"),
            (OneInchAggregator::avalanche(), 43114, "avalanche"),
        ];
        for (agg, expected_id, expected_name) in chains {
            assert_eq!(agg.chain_id, expected_id);
            assert_eq!(agg.chain_name, expected_name);
        }
    }

    #[test]
    fn test_oneinch_get_quote() {
        let inch = OneInchAggregator::ethereum();
        let req = SwapRequest {
            chain: "ethereum".into(),
            from_token: TokenInfo::eth(),
            to_token: TokenInfo::usdc_eth(),
            amount_in: 1_000_000_000_000_000_000,
            slippage_bps: 50,
            sender: "0x0000000000000000000000000000000000000001".into(),
            deadline_secs: 300,
        };
        let quote = inch.get_quote(&req).unwrap();
        assert!(quote.amount_out > 0);
        assert!(quote.minimum_out <= quote.amount_out);
        // 1inch route includes the aggregator name
        assert_eq!(quote.route, vec!["ETH", "1inch", "USDC"]);
    }

    #[test]
    fn test_oneinch_wrong_chain() {
        let inch = OneInchAggregator::ethereum();
        let req = SwapRequest {
            chain: "bsc".into(),
            from_token: TokenInfo::bnb(),
            to_token: TokenInfo::busd(),
            amount_in: 1_000_000_000_000_000_000,
            slippage_bps: 50,
            sender: "0x0000000000000000000000000000000000000001".into(),
            deadline_secs: 300,
        };
        assert!(inch.get_quote(&req).is_err());
    }

    #[test]
    fn test_oneinch_build_swap_tx() {
        let inch = OneInchAggregator::polygon();
        let req = SwapRequest {
            chain: "polygon".into(),
            from_token: TokenInfo::matic(),
            to_token: TokenInfo::usdc_eth(), // placeholder
            amount_in: 1_000_000_000_000_000_000,
            slippage_bps: 100,
            sender: "0x0000000000000000000000000000000000000001".into(),
            deadline_secs: 300,
        };
        let quote = inch.get_quote(&req).unwrap();
        let tx_data = inch.build_swap_tx(&req, &quote).unwrap();
        assert_eq!(tx_data.len(), 164);
    }
}
