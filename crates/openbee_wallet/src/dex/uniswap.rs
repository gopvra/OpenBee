use super::swap::{DexBackend, DexError, SwapQuote, SwapRequest};

/// Uniswap V3 DEX backend for Ethereum.
///
/// This is a placeholder implementation that demonstrates ABI encoding for
/// the `exactInputSingle` function. A production implementation would query
/// on-chain pool state via an Ethereum JSON-RPC node.
pub struct UniswapV3 {
    /// Address of the Uniswap V3 SwapRouter contract.
    pub router_address: String,
    /// EIP-155 chain ID (1 = mainnet, 11155111 = Sepolia).
    pub chain_id: u64,
}

impl UniswapV3 {
    /// Uniswap V3 on Ethereum mainnet.
    pub fn mainnet() -> Self {
        Self {
            router_address: "0xE592427A0AEce92De3Edee1F18E0157C05861564".into(),
            chain_id: 1,
        }
    }

    /// Uniswap V3 on Sepolia testnet.
    pub fn sepolia() -> Self {
        Self {
            router_address: "0xE592427A0AEce92De3Edee1F18E0157C05861564".into(),
            chain_id: 11155111,
        }
    }
}

/// ABI-encode an Ethereum address into a 32-byte word (left-padded with zeros).
fn encode_address(addr: &str) -> [u8; 32] {
    let mut word = [0u8; 32];
    let hex_str = addr.strip_prefix("0x").unwrap_or(addr);
    // An Ethereum address is 20 bytes; it goes into the last 20 bytes of a 32-byte word.
    if let Ok(bytes) = hex::decode(hex_str) {
        let start = 32 - bytes.len().min(20);
        let len = bytes.len().min(20);
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

/// ABI-encode a `u32` value into a 32-byte big-endian word (used for `uint24 fee`).
fn encode_uint24(value: u32) -> [u8; 32] {
    let mut word = [0u8; 32];
    // uint24 fits in the last 3 bytes, but ABI encodes as uint256
    word[28..32].copy_from_slice(&value.to_be_bytes());
    word
}

/// Build the ABI-encoded calldata for Uniswap V3 `exactInputSingle`.
///
/// Function signature:
/// ```text
/// exactInputSingle((address,address,uint24,address,uint256,uint256,uint256,uint160))
/// ```
/// Selector: `0x414bf389`
///
/// The parameters are:
/// - `tokenIn`: address of the input token
/// - `tokenOut`: address of the output token
/// - `fee`: pool fee tier (e.g. 3000 = 0.3%)
/// - `recipient`: address that receives the output tokens
/// - `deadline`: unix timestamp after which the TX reverts
/// - `amountIn`: exact input amount
/// - `amountOutMinimum`: minimum acceptable output (slippage protection)
/// - `sqrtPriceLimitX96`: price limit (0 = no limit)
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
    // 4 bytes selector + 8 * 32 bytes params = 260 bytes
    let mut calldata = Vec::with_capacity(4 + 8 * 32);

    // Function selector: 0x414bf389
    calldata.extend_from_slice(&[0x41, 0x4b, 0xf3, 0x89]);

    // Encode the 8 tuple fields as consecutive ABI words
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

/// The default Uniswap V3 pool fee tier: 0.3% (3000 bps).
const DEFAULT_FEE_TIER: u32 = 3000;

impl DexBackend for UniswapV3 {
    fn name(&self) -> &str {
        "Uniswap V3"
    }

    fn supported_chain(&self) -> &str {
        "ethereum"
    }

    fn get_quote(&self, request: &SwapRequest) -> Result<SwapQuote, DexError> {
        if request.chain != "ethereum" {
            return Err(DexError::UnsupportedChain(request.chain.clone()));
        }

        // Placeholder: a real implementation would call the Uniswap V3 Quoter
        // contract via `eth_call` JSON-RPC to get the exact output amount based
        // on current pool state and tick liquidity. Here we return a synthetic
        // estimate with a 0.3% fee.
        let fee_amount = request.amount_in * 3 / 1000; // 0.3%
        let after_fee = request.amount_in - fee_amount;

        // Synthetic price ratio (placeholder — real price comes from the pool).
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
            price_impact: 0.3,
            fee: fee_amount,
            route: vec![
                request.from_token.symbol.clone(),
                request.to_token.symbol.clone(),
            ],
            expires_at: now_secs + 120, // 2-minute quote validity
        })
    }

    fn build_swap_tx(&self, request: &SwapRequest, quote: &SwapQuote) -> Result<Vec<u8>, DexError> {
        if request.chain != "ethereum" {
            return Err(DexError::UnsupportedChain(request.chain.clone()));
        }

        // Resolve token addresses — native ETH swaps go through WETH.
        let token_in = if request.from_token.address == "native" {
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2" // WETH
        } else {
            &request.from_token.address
        };

        let token_out = if request.to_token.address == "native" {
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2" // WETH
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dex::swap::TokenInfo;

    #[test]
    fn test_calldata_length() {
        let data = build_exact_input_single_calldata(
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            3000,
            "0x0000000000000000000000000000000000000001",
            1_700_000_000,
            1_000_000_000_000_000_000, // 1 ETH
            950_000,                   // ~0.95 USDC
            0,
        );
        // 4 bytes selector + 8 * 32 bytes = 260 bytes
        assert_eq!(data.len(), 260);
        // Check function selector
        assert_eq!(&data[..4], &[0x41, 0x4b, 0xf3, 0x89]);
    }

    #[test]
    fn test_uniswap_get_quote() {
        let uni = UniswapV3::mainnet();
        let req = SwapRequest {
            chain: "ethereum".into(),
            from_token: TokenInfo::eth(),
            to_token: TokenInfo::usdc_eth(),
            amount_in: 1_000_000_000_000_000_000, // 1 ETH in wei
            slippage_bps: 50,
            sender: "0x0000000000000000000000000000000000000001".into(),
            deadline_secs: 300,
        };
        let quote = uni.get_quote(&req).unwrap();
        assert!(quote.amount_out > 0);
        assert!(quote.minimum_out <= quote.amount_out);
        assert_eq!(quote.route, vec!["ETH", "USDC"]);
    }

    #[test]
    fn test_uniswap_wrong_chain() {
        let uni = UniswapV3::mainnet();
        let req = SwapRequest {
            chain: "solana".into(),
            from_token: TokenInfo::sol(),
            to_token: TokenInfo::usdc_sol(),
            amount_in: 1_000_000_000,
            slippage_bps: 50,
            sender: "some_address".into(),
            deadline_secs: 300,
        };
        assert!(uni.get_quote(&req).is_err());
    }
}
