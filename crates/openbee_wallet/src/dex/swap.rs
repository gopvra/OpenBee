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

    // ----- BSC tokens -----

    /// Native BNB on BSC.
    pub fn bnb() -> Self {
        Self {
            symbol: "BNB".into(),
            address: "native".into(),
            decimals: 18,
        }
    }

    /// Wrapped BNB (WBNB) on BSC mainnet.
    pub fn wbnb() -> Self {
        Self {
            symbol: "WBNB".into(),
            address: "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c".into(),
            decimals: 18,
        }
    }

    /// USDC on BSC mainnet.
    pub fn usdc_bsc() -> Self {
        Self {
            symbol: "USDC".into(),
            address: "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d".into(),
            decimals: 18,
        }
    }

    /// USDT on BSC mainnet.
    pub fn usdt_bsc() -> Self {
        Self {
            symbol: "USDT".into(),
            address: "0x55d398326f99059fF775485246999027B3197955".into(),
            decimals: 18,
        }
    }

    /// BUSD on BSC mainnet.
    pub fn busd() -> Self {
        Self {
            symbol: "BUSD".into(),
            address: "0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56".into(),
            decimals: 18,
        }
    }

    /// CAKE (PancakeSwap) token on BSC mainnet.
    pub fn cake() -> Self {
        Self {
            symbol: "CAKE".into(),
            address: "0x0E09FaBB73Bd3Ade0a17ECC321fD13a19e81cE82".into(),
            decimals: 18,
        }
    }

    // ----- Multi-chain tokens -----

    /// Native MATIC on Polygon.
    pub fn matic() -> Self {
        Self {
            symbol: "MATIC".into(),
            address: "native".into(),
            decimals: 18,
        }
    }

    /// Native AVAX on Avalanche C-Chain.
    pub fn avax() -> Self {
        Self {
            symbol: "AVAX".into(),
            address: "native".into(),
            decimals: 18,
        }
    }

    /// ARB (Arbitrum) governance token on Arbitrum One.
    pub fn arb() -> Self {
        Self {
            symbol: "ARB".into(),
            address: "0x912CE59144191C1204E64559FE8253a0e49E6548".into(),
            decimals: 18,
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
