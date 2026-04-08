use serde::{Deserialize, Serialize};

/// A chain-agnostic token swap request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapRequest {
    /// Target blockchain: `"ethereum"` or `"solana"`.
    pub chain: String,
    /// Token to sell.
    pub from_token: TokenInfo,
    /// Token to buy.
    pub to_token: TokenInfo,
    /// Amount of `from_token` in its smallest unit (e.g. wei for ETH).
    pub amount_in: u128,
    /// Maximum acceptable slippage in basis points (50 = 0.5%).
    pub slippage_bps: u32,
    /// Sender wallet address.
    pub sender: String,
    /// Transaction deadline in seconds from now.
    pub deadline_secs: u64,
}

/// Metadata for a token on a specific chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    /// Ticker symbol, e.g. `"ETH"`, `"USDC"`.
    pub symbol: String,
    /// On-chain contract/mint address, or `"native"` for the chain's native coin.
    pub address: String,
    /// Number of decimal places (18 for ETH, 6 for USDC, 9 for SOL, etc.).
    pub decimals: u8,
}

impl TokenInfo {
    /// Native Ether on Ethereum.
    pub fn eth() -> Self {
        Self {
            symbol: "ETH".into(),
            address: "native".into(),
            decimals: 18,
        }
    }

    /// Native SOL on Solana.
    pub fn sol() -> Self {
        Self {
            symbol: "SOL".into(),
            address: "native".into(),
            decimals: 9,
        }
    }

    /// USDC on Ethereum mainnet.
    pub fn usdc_eth() -> Self {
        Self {
            symbol: "USDC".into(),
            address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".into(),
            decimals: 6,
        }
    }

    /// USDT on Ethereum mainnet.
    pub fn usdt_eth() -> Self {
        Self {
            symbol: "USDT".into(),
            address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".into(),
            decimals: 6,
        }
    }

    /// USDC on Solana mainnet.
    pub fn usdc_sol() -> Self {
        Self {
            symbol: "USDC".into(),
            address: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".into(),
            decimals: 6,
        }
    }

    /// Wrapped ETH (WETH) on Ethereum mainnet.
    pub fn weth() -> Self {
        Self {
            symbol: "WETH".into(),
            address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".into(),
            decimals: 18,
        }
    }

    /// Wrapped SOL on Solana mainnet.
    pub fn wsol() -> Self {
        Self {
            symbol: "WSOL".into(),
            address: "So11111111111111111111111111111111111111112".into(),
            decimals: 9,
        }
    }
}

/// A price quote returned by a DEX backend before executing a swap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapQuote {
    /// Amount of input token.
    pub amount_in: u128,
    /// Estimated output amount.
    pub amount_out: u128,
    /// Minimum output after accounting for slippage.
    pub minimum_out: u128,
    /// Estimated price impact as a percentage (e.g. `0.3` means 0.3%).
    pub price_impact: f64,
    /// DEX fee in the input token's smallest unit.
    pub fee: u128,
    /// The swap route, e.g. `["ETH", "USDC"]`.
    pub route: Vec<String>,
    /// Unix timestamp when this quote expires.
    pub expires_at: u64,
}

/// The result of an executed swap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapResult {
    /// Transaction hash.
    pub tx_hash: String,
    /// Actual input amount.
    pub amount_in: u128,
    /// Actual output amount.
    pub amount_out: u128,
    /// Current status of the swap transaction.
    pub status: SwapStatus,
}

/// Status of a swap transaction.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum SwapStatus {
    Pending,
    Confirmed,
    Failed,
}

/// Trait implemented by each DEX backend (Uniswap, Jupiter, etc.).
pub trait DexBackend: Send + Sync {
    /// Human-readable name of this DEX, e.g. `"Uniswap V3"`.
    fn name(&self) -> &str;

    /// The chain this backend operates on, e.g. `"ethereum"` or `"solana"`.
    fn supported_chain(&self) -> &str;

    /// Fetch a price quote for the given swap request.
    fn get_quote(&self, request: &SwapRequest) -> Result<SwapQuote, DexError>;

    /// Build the swap transaction data (e.g. EVM calldata or Solana instruction data).
    fn build_swap_tx(&self, request: &SwapRequest, quote: &SwapQuote) -> Result<Vec<u8>, DexError>;
}

/// Errors that can occur during DEX operations.
#[derive(Debug, thiserror::Error)]
pub enum DexError {
    #[error("Token not supported: {0}")]
    UnsupportedToken(String),
    #[error("Insufficient liquidity for swap")]
    InsufficientLiquidity,
    #[error("Slippage too high: {0}%")]
    SlippageTooHigh(f64),
    #[error("Quote expired")]
    QuoteExpired,
    #[error("Chain not supported: {0}")]
    UnsupportedChain(String),
    #[error("RPC error: {0}")]
    RpcError(String),
}
