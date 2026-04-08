//! Wallet skill — crypto wallet operations.

use crate::skill::{Skill, SkillCommand};
use crate::task::{Task, TaskResult};

/// Handles crypto wallet operations: balance, send, swap, address, etc.
pub struct WalletSkill;

impl WalletSkill {
    pub fn new() -> Self {
        Self
    }

    fn parse_command(instruction: &str) -> &'static str {
        let lower = instruction.to_lowercase();
        if lower.contains("balance") || lower.contains("余额") {
            "balance"
        } else if lower.contains("send")
            || lower.contains("transfer")
            || lower.contains("发送")
            || lower.contains("转账")
        {
            "send"
        } else if lower.contains("swap")
            || lower.contains("exchange")
            || lower.contains("兑换")
            || lower.contains("买")
        {
            "swap"
        } else if lower.contains("address") || lower.contains("地址") {
            "address"
        } else if lower.contains("create") || lower.contains("创建") {
            "create"
        } else if lower.contains("recover") || lower.contains("恢复") {
            "recover"
        } else {
            "help"
        }
    }
}

impl Default for WalletSkill {
    fn default() -> Self {
        Self::new()
    }
}

impl Skill for WalletSkill {
    fn name(&self) -> &str {
        "wallet"
    }

    fn description(&self) -> &str {
        "Crypto wallet: check balance, send tokens, swap, manage addresses"
    }

    fn keywords(&self) -> &[&str] {
        &[
            "wallet", "balance", "send", "transfer", "swap", "buy", "sell", "address", "token",
            "eth", "sol", "bnb", "usdc", "usdt", "钱包", "余额", "转账", "发送", "兑换", "买",
            "地址",
        ]
    }

    fn execute(&self, task: &Task) -> TaskResult {
        let cmd = Self::parse_command(&task.instruction);

        match cmd {
            "balance" => TaskResult::success(
                "Wallet balance check requires RPC connection. \
                 Use `openbee --wallet address` to see your addresses, \
                 then check balances on a block explorer.",
            )
            .with_data(serde_json::json!({
                "action": "balance",
                "note": "RPC not connected — check via block explorer"
            }))
            .with_suggestions(vec![
                "Connect RPC endpoint for live balance queries".to_string(),
                "Use etherscan.io or solscan.io to check manually".to_string(),
            ]),

            "send" => TaskResult::success(
                "To send tokens, use: openbee --wallet send <chain> <to_address> <amount>",
            )
            .with_suggestions(vec![
                "openbee --wallet send ethereum 0x... 0.1".to_string(),
                "openbee --wallet send solana ABC... 1.0".to_string(),
            ]),

            "swap" => TaskResult::success(
                "Token swap available via Uniswap (ETH), PancakeSwap (BSC), \
                 1inch (multi-chain), Jupiter (SOL). \
                 Swap requires RPC connection for live quotes.",
            )
            .with_data(serde_json::json!({
                "available_dex": ["Uniswap V3", "PancakeSwap V3", "1inch", "Jupiter"],
                "supported_chains": ["ethereum", "bsc", "polygon", "arbitrum", "optimism", "base", "avalanche", "solana"]
            })),

            "address" => TaskResult::success(
                "Use `openbee --wallet address` to see all wallet addresses. \
                 Or `openbee --wallet address ethereum` for a specific chain.",
            ),

            "create" => TaskResult::success(
                "Use `openbee --wallet create` to generate a new wallet with a 12-word mnemonic.",
            ),

            "recover" => TaskResult::success(
                "Use `openbee --wallet recover` to restore a wallet from your mnemonic backup.",
            ),

            _ => TaskResult::success("Wallet skill: use 'balance', 'send', 'swap', 'address', 'create', or 'recover'."),
        }
    }

    fn commands(&self) -> Vec<SkillCommand> {
        vec![
            SkillCommand {
                name: "balance".to_string(),
                description: "Check wallet balance".to_string(),
                example: "check my ETH balance".to_string(),
            },
            SkillCommand {
                name: "send".to_string(),
                description: "Send tokens to an address".to_string(),
                example: "send 0.1 ETH to 0xABC...".to_string(),
            },
            SkillCommand {
                name: "swap".to_string(),
                description: "Swap tokens on a DEX".to_string(),
                example: "swap 100 USDC to ETH".to_string(),
            },
            SkillCommand {
                name: "address".to_string(),
                description: "Show wallet addresses".to_string(),
                example: "show my Solana address".to_string(),
            },
        ]
    }
}
