use super::swap::{DexBackend, DexError, SwapQuote, SwapRequest};

/// Jupiter aggregator DEX backend for Solana.
///
/// This is a placeholder implementation. A production version would call
/// the Jupiter REST API (`https://quote-api.jup.ag/v6/quote`) or directly
/// build Solana instructions against the Jupiter program.
pub struct Jupiter {
    /// The Jupiter program ID on Solana.
    pub program_id: String,
}

impl Jupiter {
    /// Jupiter on Solana mainnet-beta.
    pub fn mainnet() -> Self {
        Self {
            program_id: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".into(),
        }
    }

    /// Jupiter on Solana devnet.
    pub fn devnet() -> Self {
        Self {
            program_id: "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4".into(),
        }
    }
}

impl DexBackend for Jupiter {
    fn name(&self) -> &str {
        "Jupiter"
    }

    fn supported_chain(&self) -> &str {
        "solana"
    }

    fn get_quote(&self, request: &SwapRequest) -> Result<SwapQuote, DexError> {
        if request.chain != "solana" {
            return Err(DexError::UnsupportedChain(request.chain.clone()));
        }

        // Placeholder: a real implementation would call the Jupiter Quote API:
        //   GET https://quote-api.jup.ag/v6/quote?inputMint=...&outputMint=...&amount=...
        // and parse the JSON response for route, output amount, and price impact.

        let fee_amount = request.amount_in * 3 / 1000; // 0.3% placeholder fee
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
                request.to_token.symbol.clone(),
            ],
            expires_at: now_secs + 30, // Jupiter quotes are short-lived
        })
    }

    fn build_swap_tx(&self, request: &SwapRequest, quote: &SwapQuote) -> Result<Vec<u8>, DexError> {
        if request.chain != "solana" {
            return Err(DexError::UnsupportedChain(request.chain.clone()));
        }

        // Placeholder: a real implementation would call the Jupiter Swap API:
        //   POST https://quote-api.jup.ag/v6/swap
        // which returns a base64-encoded Solana transaction that the wallet
        // can sign and submit to an RPC node.
        //
        // Here we encode minimal placeholder instruction data:
        //   [8-byte discriminator][amount_in as le u128][minimum_out as le u128]
        let mut data = Vec::with_capacity(40);

        // Anchor-style 8-byte instruction discriminator (placeholder)
        data.extend_from_slice(&[0xe5, 0x17, 0xcb, 0x97, 0x7a, 0xe3, 0xad, 0x2a]);

        // amount_in as little-endian u128 (16 bytes)
        data.extend_from_slice(&quote.amount_in.to_le_bytes());

        // minimum_out as little-endian u128 (16 bytes)
        data.extend_from_slice(&quote.minimum_out.to_le_bytes());

        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dex::swap::TokenInfo;

    #[test]
    fn test_jupiter_get_quote() {
        let jup = Jupiter::mainnet();
        let req = SwapRequest {
            chain: "solana".into(),
            from_token: TokenInfo::sol(),
            to_token: TokenInfo::usdc_sol(),
            amount_in: 1_000_000_000, // 1 SOL in lamports
            slippage_bps: 100,        // 1%
            sender: "SomeSOLAddress1111111111111111111111111111111".into(),
            deadline_secs: 60,
        };
        let quote = jup.get_quote(&req).unwrap();
        assert!(quote.amount_out > 0);
        assert!(quote.minimum_out <= quote.amount_out);
        assert_eq!(quote.route, vec!["SOL", "USDC"]);
    }

    #[test]
    fn test_jupiter_wrong_chain() {
        let jup = Jupiter::mainnet();
        let req = SwapRequest {
            chain: "ethereum".into(),
            from_token: TokenInfo::eth(),
            to_token: TokenInfo::usdc_eth(),
            amount_in: 1_000_000_000_000_000_000,
            slippage_bps: 50,
            sender: "0x0000000000000000000000000000000000000001".into(),
            deadline_secs: 300,
        };
        assert!(jup.get_quote(&req).is_err());
    }

    #[test]
    fn test_jupiter_build_swap_tx() {
        let jup = Jupiter::mainnet();
        let req = SwapRequest {
            chain: "solana".into(),
            from_token: TokenInfo::sol(),
            to_token: TokenInfo::usdc_sol(),
            amount_in: 1_000_000_000,
            slippage_bps: 100,
            sender: "SomeSOLAddress1111111111111111111111111111111".into(),
            deadline_secs: 60,
        };
        let quote = jup.get_quote(&req).unwrap();
        let tx_data = jup.build_swap_tx(&req, &quote).unwrap();
        // 8 discriminator + 16 amount_in + 16 minimum_out = 40 bytes
        assert_eq!(tx_data.len(), 40);
    }
}
