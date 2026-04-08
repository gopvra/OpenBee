//! PancakeSwap V3 DEX backend for BNB Smart Chain (BSC).
//!
//! PancakeSwap V3 is a fork of Uniswap V3 deployed on BSC and other chains.
//! It uses the same `exactInputSingle` interface as Uniswap V3.

use super::swap::{DexBackend, DexError, SwapQuote, SwapRequest};

/// PancakeSwap V3 DEX backend.
///
/// This is a placeholder implementation that demonstrates ABI encoding for
/// the `exactInputSingle` function on BSC. A production implementation would
/// query on-chain pool state via a BSC JSON-RPC node.
pub struct PancakeSwapV3 {
    /// Address of the PancakeSwap V3 SmartRouter contract.
    pub router_address: String,
    /// EIP-155 chain ID (56 = BSC mainnet, 97 = BSC testnet).
    pub chain_id: u64,
}

impl PancakeSwapV3 {
    /// PancakeSwap V3 on BSC mainnet (chain id 56).
    pub fn bsc_mainnet() -> Self {
        Self {
            router_address: "0x13f4EA83D0bd40E75C8222255bc855a974568Dd4".into(),
            chain_id: 56,
        }
    }

    /// PancakeSwap V3 on BSC testnet (chain id 97).
    pub fn bsc_testnet() -> Self {
        Self {
            router_address: "0x13f4EA83D0bd40E75C8222255bc855a974568Dd4".into(),
            chain_id: 97,
        }
    }
}

/// Wrapped BNB address on BSC mainnet.
const WBNB_ADDRESS: &str = "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c";

/// Default PancakeSwap V3 pool fee tier: 0.25% (2500 bps).
const DEFAULT_FEE_TIER: u32 = 2500;

// ---------------------------------------------------------------------------
// ABI encoding helpers (same layout as Uniswap V3)
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

/// ABI-encode a `u64` value into a 32-byte big-endian word.
fn encode_uint256_from_u64(value: u64) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[24..32].copy_from_slice(&value.to_be_bytes());
    word
}

/// ABI-encode a `u32` fee value into a 32-byte big-endian word (for `uint24`).
fn encode_uint24(value: u32) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[28..32].copy_from_slice(&value.to_be_bytes());
    word
}

/// Build the ABI-encoded calldata for PancakeSwap V3 `exactInputSingle`.
///
/// Uses the same function signature as Uniswap V3:
/// ```text
/// exactInputSingle((address,address,uint24,address,uint256,uint256,uint256,uint160))
/// ```
/// Selector: `0x414bf389`
fn build_exact_input_single_calldata(
    token_in: &str,
    token_out: &str,
    fee: u32,
    recipient: &str,
    deadline: u64,
    amount_in: u128,
    amount_out_minimum: u128,
    sqrt_price_limit_x96: u128,
) -> Vec<u8> {
    let mut calldata = Vec::with_capacity(4 + 8 * 32);

    // Function selector: 0x414bf389
    calldata.extend_from_slice(&[0x41, 0x4b, 0xf3, 0x89]);

    calldata.extend_from_slice(&encode_address(token_in));
    calldata.extend_from_slice(&encode_address(token_out));
    calldata.extend_from_slice(&encode_uint24(fee));
    calldata.extend_from_slice(&encode_address(recipient));
    calldata.extend_from_slice(&encode_uint256_from_u64(deadline));
    calldata.extend_from_slice(&encode_uint256_from_u128(amount_in));
    calldata.extend_from_slice(&encode_uint256_from_u128(amount_out_minimum));
    calldata.extend_from_slice(&encode_uint256_from_u128(sqrt_price_limit_x96));

    calldata
}

impl DexBackend for PancakeSwapV3 {
    fn name(&self) -> &str {
        "PancakeSwap V3"
    }

    fn supported_chain(&self) -> &str {
        "bsc"
    }

    fn get_quote(&self, request: &SwapRequest) -> Result<SwapQuote, DexError> {
        if request.chain != "bsc" {
            return Err(DexError::UnsupportedChain(request.chain.clone()));
        }

        // Placeholder: a real implementation would call the PancakeSwap V3
        // Quoter contract via `eth_call` on a BSC RPC node.
        let fee_amount = request.amount_in * 25 / 10_000; // 0.25%
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
            price_impact: 0.25,
            fee: fee_amount,
            route: vec![
                request.from_token.symbol.clone(),
                request.to_token.symbol.clone(),
            ],
            expires_at: now_secs + 120,
        })
    }

    fn build_swap_tx(&self, request: &SwapRequest, quote: &SwapQuote) -> Result<Vec<u8>, DexError> {
        if request.chain != "bsc" {
            return Err(DexError::UnsupportedChain(request.chain.clone()));
        }

        // Resolve token addresses — native BNB swaps go through WBNB.
        let token_in = if request.from_token.address == "native" {
            WBNB_ADDRESS
        } else {
            &request.from_token.address
        };

        let token_out = if request.to_token.address == "native" {
            WBNB_ADDRESS
        } else {
            &request.to_token.address
        };

        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let deadline = now_secs + request.deadline_secs;

        let calldata = build_exact_input_single_calldata(
            token_in,
            token_out,
            DEFAULT_FEE_TIER,
            &request.sender,
            deadline,
            quote.amount_in,
            quote.minimum_out,
            0, // sqrtPriceLimitX96 = 0 (no price limit)
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
    fn test_calldata_length() {
        let data = build_exact_input_single_calldata(
            WBNB_ADDRESS,
            "0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56",
            2500,
            "0x0000000000000000000000000000000000000001",
            1_700_000_000,
            1_000_000_000_000_000_000, // 1 BNB
            950_000_000_000_000_000,   // ~0.95 BUSD
            0,
        );
        assert_eq!(data.len(), 260);
        assert_eq!(&data[..4], &[0x41, 0x4b, 0xf3, 0x89]);
    }

    #[test]
    fn test_pancakeswap_get_quote() {
        let pcs = PancakeSwapV3::bsc_mainnet();
        let req = SwapRequest {
            chain: "bsc".into(),
            from_token: TokenInfo::bnb(),
            to_token: TokenInfo::busd(),
            amount_in: 1_000_000_000_000_000_000, // 1 BNB in wei
            slippage_bps: 50,
            sender: "0x0000000000000000000000000000000000000001".into(),
            deadline_secs: 300,
        };
        let quote = pcs.get_quote(&req).unwrap();
        assert!(quote.amount_out > 0);
        assert!(quote.minimum_out <= quote.amount_out);
        assert_eq!(quote.route, vec!["BNB", "BUSD"]);
    }

    #[test]
    fn test_pancakeswap_wrong_chain() {
        let pcs = PancakeSwapV3::bsc_mainnet();
        let req = SwapRequest {
            chain: "ethereum".into(),
            from_token: TokenInfo::eth(),
            to_token: TokenInfo::usdc_eth(),
            amount_in: 1_000_000_000_000_000_000,
            slippage_bps: 50,
            sender: "0x0000000000000000000000000000000000000001".into(),
            deadline_secs: 300,
        };
        assert!(pcs.get_quote(&req).is_err());
    }

    #[test]
    fn test_pancakeswap_build_swap_tx() {
        let pcs = PancakeSwapV3::bsc_mainnet();
        let req = SwapRequest {
            chain: "bsc".into(),
            from_token: TokenInfo::bnb(),
            to_token: TokenInfo::busd(),
            amount_in: 1_000_000_000_000_000_000,
            slippage_bps: 50,
            sender: "0x0000000000000000000000000000000000000001".into(),
            deadline_secs: 300,
        };
        let quote = pcs.get_quote(&req).unwrap();
        let tx_data = pcs.build_swap_tx(&req, &quote).unwrap();
        assert_eq!(tx_data.len(), 260);
    }

    #[test]
    fn test_bsc_mainnet_chain_id() {
        let pcs = PancakeSwapV3::bsc_mainnet();
        assert_eq!(pcs.chain_id, 56);
    }

    #[test]
    fn test_bsc_testnet_chain_id() {
        let pcs = PancakeSwapV3::bsc_testnet();
        assert_eq!(pcs.chain_id, 97);
    }
}
