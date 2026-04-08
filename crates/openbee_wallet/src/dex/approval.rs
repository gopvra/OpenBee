//! ERC-20 Token Approval Manager — defends against approval-based attacks.
//!
//! Security rules enforced:
//! 1. NEVER approve unlimited (`type(uint256).max`) — always exact amount
//! 2. Approve-to-zero-then-set pattern to prevent race conditions
//! 3. Auto-revoke approval after swap completes
//! 4. User must explicitly confirm every approval
//! 5. Show clear warning about what the approval allows

use serde::{Deserialize, Serialize};

/// Maximum safe approval margin: 1% above swap amount for gas fluctuation.
pub const APPROVAL_SAFETY_MARGIN_BPS: u32 = 100;

/// The 4-byte function selector for `approve(address,uint256)`.
///
/// Computed as the first 4 bytes of `keccak256("approve(address,uint256)")`.
const APPROVE_SELECTOR: [u8; 4] = [0x09, 0x5e, 0xa7, 0xb3];

/// Safety warning shown to user before every approval.
pub const APPROVAL_WARNING: &str = "\
WARNING: TOKEN APPROVAL REQUEST

You are about to allow a smart contract to spend your tokens.
This is a SECURITY-SENSITIVE operation.

* Only the EXACT amount needed will be approved (not unlimited)
* The approval will be AUTOMATICALLY REVOKED after the swap
* If you don't recognize the spender address, DO NOT approve

Approval details:";

// ---------------------------------------------------------------------------
// ApprovalRequest
// ---------------------------------------------------------------------------

/// Approval request — shown to user for confirmation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRequest {
    /// ERC-20 contract address of the token being approved.
    pub token_address: String,
    /// Human-readable symbol, e.g. `"USDC"`.
    pub token_symbol: String,
    /// DEX router address being approved to spend tokens.
    pub spender: String,
    /// Human-readable spender name, e.g. `"Uniswap V3 Router"`.
    pub spender_name: String,
    /// Exact amount to approve (swap amount + safety margin).
    pub amount: u128,
    /// Human-readable amount, e.g. `"1.500000 USDC"`.
    pub amount_display: String,
    /// Chain identifier, e.g. `"ethereum"`, `"bsc"`.
    pub chain: String,
    /// ALWAYS `false` — we never allow unlimited approvals.
    pub is_unlimited: bool,
}

// ---------------------------------------------------------------------------
// ActiveApproval
// ---------------------------------------------------------------------------

/// An approval that has been confirmed and executed on-chain but not yet
/// revoked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveApproval {
    /// ERC-20 contract address.
    pub token_address: String,
    /// DEX router address that was approved.
    pub spender: String,
    /// Approved amount.
    pub amount: u128,
    /// Chain identifier.
    pub chain: String,
    /// Unix timestamp when the approval transaction was confirmed.
    pub approved_at: u64,
    /// Transaction hash of the approval.
    pub tx_hash: String,
}

// ---------------------------------------------------------------------------
// ApprovalManager
// ---------------------------------------------------------------------------

/// The 3-step safe approval flow:
///
/// 1. `approve(spender, 0)`      — reset to zero first (prevent race condition)
/// 2. `approve(spender, amount)` — set exact amount needed
/// 3. after swap: `approve(spender, 0)` — revoke leftovers
pub struct ApprovalManager {
    /// Pending approvals awaiting user confirmation.
    pub pending: Vec<ApprovalRequest>,
    /// Active approvals (approved but not yet revoked).
    pub active: Vec<ActiveApproval>,
    /// Whether to auto-revoke after swap completion (default: `true`).
    pub auto_revoke: bool,
}

impl ApprovalManager {
    /// Create a new approval manager with auto-revoke enabled.
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            active: Vec::new(),
            auto_revoke: true,
        }
    }

    /// Create an approval request for user confirmation.
    ///
    /// The amount is calculated as `swap_amount + safety_margin` (1%).
    /// This function **never** creates unlimited approvals.
    pub fn request_approval(
        &mut self,
        token_address: &str,
        token_symbol: &str,
        spender: &str,
        spender_name: &str,
        swap_amount: u128,
        decimals: u8,
        chain: &str,
    ) -> ApprovalRequest {
        // Add 1% safety margin for gas fluctuation, guarding against overflow.
        let margin = swap_amount / 10_000 * APPROVAL_SAFETY_MARGIN_BPS as u128;
        let amount = swap_amount.saturating_add(margin);

        let amount_display = format!("{} {}", Self::format_amount(amount, decimals), token_symbol);

        let request = ApprovalRequest {
            token_address: token_address.to_string(),
            token_symbol: token_symbol.to_string(),
            spender: spender.to_string(),
            spender_name: spender_name.to_string(),
            amount,
            amount_display,
            chain: chain.to_string(),
            is_unlimited: false, // NEVER unlimited
        };

        self.pending.push(request.clone());
        request
    }

    /// Build the ABI-encoded `approve(spender, 0)` calldata (reset step).
    ///
    /// This is Step 1 of the safe approval flow: reset the allowance to zero
    /// before setting a new value, preventing the well-known ERC-20 approval
    /// race condition.
    ///
    /// Encoding: `0x095ea7b3` + address (32 bytes) + uint256 zero (32 bytes)
    /// Total: 68 bytes
    pub fn build_approve_zero(spender: &str) -> Vec<u8> {
        Self::build_approve(spender, 0)
    }

    /// Build the ABI-encoded `approve(spender, amount)` calldata.
    ///
    /// Function: `approve(address,uint256)`
    /// Selector: `0x095ea7b3`
    ///
    /// Layout: 4 bytes selector + 32 bytes address + 32 bytes uint256 = 68 bytes
    pub fn build_approve(spender: &str, amount: u128) -> Vec<u8> {
        let mut calldata = Vec::with_capacity(68);

        // Function selector: 0x095ea7b3
        calldata.extend_from_slice(&APPROVE_SELECTOR);

        // Address parameter: left-padded to 32 bytes
        calldata.extend_from_slice(&encode_address(spender));

        // Amount parameter: left-padded to 32 bytes (uint256)
        calldata.extend_from_slice(&encode_uint256_from_u128(amount));

        calldata
    }

    /// Build the revoke calldata: `approve(spender, 0)`.
    ///
    /// This is Step 3: after swap completion, revoke any remaining allowance.
    pub fn build_revoke(spender: &str) -> Vec<u8> {
        Self::build_approve_zero(spender)
    }

    /// Record that an approval was confirmed and executed on-chain.
    pub fn record_approval(&mut self, approval: ActiveApproval) {
        // Remove from pending if it matches
        self.pending.retain(|p| {
            !(p.token_address == approval.token_address && p.spender == approval.spender)
        });
        self.active.push(approval);
    }

    /// Revoke all active approvals for a given chain.
    ///
    /// Returns a list of `(token_address, revoke_calldata)` pairs — one
    /// `approve(spender, 0)` transaction per active approval.
    pub fn revoke_all(&mut self, chain: &str) -> Vec<(String, Vec<u8>)> {
        let mut revocations = Vec::new();

        let to_revoke: Vec<ActiveApproval> = self
            .active
            .iter()
            .filter(|a| a.chain == chain)
            .cloned()
            .collect();

        for approval in &to_revoke {
            let calldata = Self::build_revoke(&approval.spender);
            revocations.push((approval.token_address.clone(), calldata));
        }

        self.active.retain(|a| a.chain != chain);

        revocations
    }

    /// Revoke a specific approval by token address and spender.
    ///
    /// Returns the revoke calldata if the approval was found and removed.
    pub fn revoke(&mut self, token_address: &str, spender: &str) -> Option<Vec<u8>> {
        let idx = self
            .active
            .iter()
            .position(|a| a.token_address == token_address && a.spender == spender)?;

        let approval = self.active.remove(idx);
        Some(Self::build_revoke(&approval.spender))
    }

    /// List all active (un-revoked) approvals.
    pub fn active_approvals(&self) -> &[ActiveApproval] {
        &self.active
    }

    /// Format an amount for display given a number of decimals.
    ///
    /// Example: `format_amount(1_500_000, 6)` returns `"1.500000"`.
    fn format_amount(amount: u128, decimals: u8) -> String {
        if decimals == 0 {
            return amount.to_string();
        }

        let divisor = 10u128.pow(decimals as u32);
        let whole = amount / divisor;
        let frac = amount % divisor;

        format!("{whole}.{frac:0>width$}", width = decimals as usize)
    }
}

impl Default for ApprovalManager {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// ABI encoding helpers
// ---------------------------------------------------------------------------

/// ABI-encode an Ethereum address into a 32-byte word (left-padded with zeros).
///
/// The address is 20 bytes and occupies bytes 12..32 of the word.
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

/// ABI-encode a `u128` value into a 32-byte big-endian word (uint256).
///
/// The upper 16 bytes are zero; the lower 16 bytes contain the big-endian
/// representation of the value.
fn encode_uint256_from_u128(value: u128) -> [u8; 32] {
    let mut word = [0u8; 32];
    word[16..32].copy_from_slice(&value.to_be_bytes());
    word
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approve_calldata_length() {
        let calldata =
            ApprovalManager::build_approve("0x0000000000000000000000000000000000000001", 1000);
        // 4 bytes selector + 32 bytes address + 32 bytes uint256 = 68 bytes
        assert_eq!(calldata.len(), 68);
    }

    #[test]
    fn test_approve_selector() {
        let calldata =
            ApprovalManager::build_approve("0x0000000000000000000000000000000000000001", 0);
        assert_eq!(&calldata[..4], &[0x09, 0x5e, 0xa7, 0xb3]);
    }

    #[test]
    fn test_approve_zero_is_revoke() {
        let spender = "0xE592427A0AEce92De3Edee1F18E0157C05861564";
        let zero = ApprovalManager::build_approve_zero(spender);
        let revoke = ApprovalManager::build_revoke(spender);
        assert_eq!(zero, revoke);
        // Amount should be all zeros (bytes 36..68)
        assert!(zero[36..68].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_approve_nonzero_amount() {
        let calldata =
            ApprovalManager::build_approve("0x0000000000000000000000000000000000000001", 42);
        // Last byte of the uint256 should be 42
        assert_eq!(calldata[67], 42);
        // Preceding bytes of the uint256 should be zero
        assert!(calldata[36..67].iter().all(|&b| b == 0));
    }

    #[test]
    fn test_never_unlimited() {
        let mut mgr = ApprovalManager::new();
        let req = mgr.request_approval(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "USDC",
            "0xE592427A0AEce92De3Edee1F18E0157C05861564",
            "Uniswap V3 Router",
            1_000_000, // 1 USDC
            6,
            "ethereum",
        );
        assert!(!req.is_unlimited);
        // Amount should be swap_amount + 1% margin
        assert!(req.amount > 1_000_000);
        assert!(req.amount <= 1_010_000);
    }

    #[test]
    fn test_safety_margin() {
        let mut mgr = ApprovalManager::new();
        let req = mgr.request_approval(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "USDC",
            "0xE592427A0AEce92De3Edee1F18E0157C05861564",
            "Uniswap V3 Router",
            10_000_000, // 10 USDC
            6,
            "ethereum",
        );
        // 1% of 10_000_000 = 100_000
        let expected = 10_000_000 + 100_000;
        assert_eq!(req.amount, expected);
    }

    #[test]
    fn test_auto_revoke_default_true() {
        let mgr = ApprovalManager::new();
        assert!(mgr.auto_revoke);
    }

    #[test]
    fn test_record_and_revoke() {
        let mut mgr = ApprovalManager::new();

        let approval = ActiveApproval {
            token_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".into(),
            spender: "0xE592427A0AEce92De3Edee1F18E0157C05861564".into(),
            amount: 1_000_000,
            chain: "ethereum".into(),
            approved_at: 1_700_000_000,
            tx_hash: "0xabc123".into(),
        };

        mgr.record_approval(approval);
        assert_eq!(mgr.active_approvals().len(), 1);

        let calldata = mgr.revoke(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "0xE592427A0AEce92De3Edee1F18E0157C05861564",
        );
        assert!(calldata.is_some());
        assert_eq!(mgr.active_approvals().len(), 0);
    }

    #[test]
    fn test_revoke_all() {
        let mut mgr = ApprovalManager::new();

        mgr.record_approval(ActiveApproval {
            token_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".into(),
            spender: "0xE592427A0AEce92De3Edee1F18E0157C05861564".into(),
            amount: 1_000_000,
            chain: "ethereum".into(),
            approved_at: 1_700_000_000,
            tx_hash: "0xabc123".into(),
        });
        mgr.record_approval(ActiveApproval {
            token_address: "0xdAC17F958D2ee523a2206206994597C13D831ec7".into(),
            spender: "0xE592427A0AEce92De3Edee1F18E0157C05861564".into(),
            amount: 2_000_000,
            chain: "ethereum".into(),
            approved_at: 1_700_000_001,
            tx_hash: "0xdef456".into(),
        });
        mgr.record_approval(ActiveApproval {
            token_address: "0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56".into(),
            spender: "0x13f4EA83D0bd40E75C8222255bc855a974568Dd4".into(),
            amount: 5_000_000,
            chain: "bsc".into(),
            approved_at: 1_700_000_002,
            tx_hash: "0x789abc".into(),
        });

        // Revoke all Ethereum approvals — should leave BSC untouched
        let revocations = mgr.revoke_all("ethereum");
        assert_eq!(revocations.len(), 2);
        assert_eq!(mgr.active_approvals().len(), 1);
        assert_eq!(mgr.active_approvals()[0].chain, "bsc");
    }

    #[test]
    fn test_revoke_nonexistent() {
        let mut mgr = ApprovalManager::new();
        assert!(mgr.revoke("0xdeadbeef", "0xcafebabe").is_none());
    }

    #[test]
    fn test_format_amount() {
        assert_eq!(ApprovalManager::format_amount(1_500_000, 6), "1.500000");
        assert_eq!(
            ApprovalManager::format_amount(1_000_000_000_000_000_000, 18),
            "1.000000000000000000"
        );
        assert_eq!(ApprovalManager::format_amount(42, 0), "42");
        assert_eq!(ApprovalManager::format_amount(0, 6), "0.000000");
        assert_eq!(ApprovalManager::format_amount(123, 2), "1.23");
    }

    #[test]
    fn test_pending_cleared_on_record() {
        let mut mgr = ApprovalManager::new();

        let _req = mgr.request_approval(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "USDC",
            "0xE592427A0AEce92De3Edee1F18E0157C05861564",
            "Uniswap V3 Router",
            1_000_000,
            6,
            "ethereum",
        );
        assert_eq!(mgr.pending.len(), 1);

        mgr.record_approval(ActiveApproval {
            token_address: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".into(),
            spender: "0xE592427A0AEce92De3Edee1F18E0157C05861564".into(),
            amount: 1_010_000,
            chain: "ethereum".into(),
            approved_at: 1_700_000_000,
            tx_hash: "0xabc".into(),
        });
        assert_eq!(mgr.pending.len(), 0);
        assert_eq!(mgr.active.len(), 1);
    }

    #[test]
    fn test_address_encoding() {
        let word = encode_address("0x0000000000000000000000000000000000000001");
        // First 31 bytes should be zero, last byte should be 1
        assert!(word[..31].iter().all(|&b| b == 0));
        assert_eq!(word[31], 1);
    }

    #[test]
    fn test_amount_display_format() {
        let mut mgr = ApprovalManager::new();
        let req = mgr.request_approval(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "USDC",
            "0xE592427A0AEce92De3Edee1F18E0157C05861564",
            "Uniswap V3 Router",
            1_500_000,
            6,
            "ethereum",
        );
        // Should contain both the formatted amount and the symbol
        assert!(req.amount_display.contains("USDC"));
        assert!(req.amount_display.contains('.'));
    }
}
