//! Safety warnings and feature gate for the wallet module.
//!
//! The wallet is **disabled by default**. Users must explicitly enable it
//! and acknowledge the security risks before any crypto operations can proceed.

use tracing::{error, warn};

/// Safety acknowledgement state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletSafetyState {
    /// Wallet feature is completely disabled (default).
    Disabled,
    /// User has been shown the warning but hasn't accepted yet.
    WarningShown,
    /// User has accepted the risks and enabled the wallet.
    Accepted,
}

impl Default for WalletSafetyState {
    fn default() -> Self {
        Self::Disabled
    }
}

/// The full safety warning text shown to the user before enabling the wallet.
pub const SAFETY_WARNING: &str = r#"
╔══════════════════════════════════════════════════════════════╗
║                    ⚠️  SECURITY WARNING  ⚠️                  ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  You are about to enable the CRYPTO WALLET feature.          ║
║  This feature involves REAL CRYPTOCURRENCY and carries       ║
║  significant financial risk.                                 ║
║                                                              ║
║  Before proceeding, you MUST ensure:                         ║
║                                                              ║
║  1. Your computer is NOT compromised by hackers or malware   ║
║  2. You have up-to-date antivirus / security software        ║
║  3. You are NOT on a public or shared computer               ║
║  4. No one is watching your screen or recording it           ║
║  5. You understand that cryptocurrency transactions are      ║
║     IRREVERSIBLE — sent funds cannot be recovered            ║
║                                                              ║
║  ⚠️  DO NOT store large amounts of funds in this wallet.     ║
║  This is a software wallet integrated into a game engine.    ║
║  For large holdings, use a hardware wallet (Ledger/Trezor).  ║
║                                                              ║
║  ⚠️  OpenBee is NOT responsible for any loss of funds.       ║
║  Use this feature entirely at your own risk.                 ║
║                                                              ║
║  ⚠️  BACK UP YOUR MNEMONIC PHRASE. If you lose it,          ║
║  your funds are PERMANENTLY LOST. No one can recover them.   ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
"#;

/// Chinese translation of the safety warning.
pub const SAFETY_WARNING_ZH: &str = r#"
╔══════════════════════════════════════════════════════════════╗
║                    ⚠️  安全警告  ⚠️                           ║
╠══════════════════════════════════════════════════════════════╣
║                                                              ║
║  你即将启用 加密货币钱包 功能。                                ║
║  此功能涉及 真实加密货币，存在重大财务风险。                    ║
║                                                              ║
║  在继续之前，你必须确保：                                      ║
║                                                              ║
║  1. 你的电脑 没有被黑客入侵 或感染恶意软件                      ║
║  2. 你已安装最新的杀毒软件/安全软件                             ║
║  3. 你不在公共或共享电脑上操作                                  ║
║  4. 没有人在偷看你的屏幕或录屏                                  ║
║  5. 你理解加密货币交易是 不可逆的 —— 发出的资金无法追回         ║
║                                                              ║
║  ⚠️  不要在此钱包中存入大量资金。                              ║
║  这是集成在游戏引擎中的软件钱包。                              ║
║  大额资金请使用硬件钱包（Ledger/Trezor）。                     ║
║                                                              ║
║  ⚠️  OpenBee 不对任何资金损失负责。                            ║
║  使用此功能的风险完全由你自行承担。                              ║
║                                                              ║
║  ⚠️  务必备份你的助记词。丢失助记词 = 永久丢失资金。           ║
║  没有任何人能帮你找回。                                        ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
"#;

/// Feature gate that must be passed before any wallet operation.
pub struct WalletSafetyGate {
    state: WalletSafetyState,
    user_typed_confirmation: bool,
}

impl Default for WalletSafetyGate {
    fn default() -> Self {
        Self::new()
    }
}

impl WalletSafetyGate {
    /// Create a new gate (disabled by default).
    pub fn new() -> Self {
        Self {
            state: WalletSafetyState::Disabled,
            user_typed_confirmation: false,
        }
    }

    /// Get the current state.
    pub fn state(&self) -> WalletSafetyState {
        self.state
    }

    /// Show the safety warning (must be called before accept).
    pub fn show_warning(&mut self) -> &'static str {
        self.state = WalletSafetyState::WarningShown;
        warn!("Wallet safety warning displayed to user");
        SAFETY_WARNING
    }

    /// Show the Chinese safety warning.
    pub fn show_warning_zh(&mut self) -> &'static str {
        self.state = WalletSafetyState::WarningShown;
        warn!("钱包安全警告已向用户展示");
        SAFETY_WARNING_ZH
    }

    /// User accepts the risks. The `confirmation` string must be exactly
    /// `"I understand the risks"` (English) or `"我了解风险"` (Chinese).
    /// This prevents accidental acceptance.
    pub fn accept(&mut self, confirmation: &str) -> bool {
        if self.state != WalletSafetyState::WarningShown {
            error!("Cannot accept wallet risks: warning has not been shown yet");
            return false;
        }

        let valid_confirmations = [
            "I understand the risks",
            "I UNDERSTAND THE RISKS",
            "我了解风险",
            "我了解风险并接受",
        ];

        if valid_confirmations
            .iter()
            .any(|c| confirmation.trim() == *c)
        {
            self.state = WalletSafetyState::Accepted;
            self.user_typed_confirmation = true;
            warn!("User accepted wallet security risks");
            true
        } else {
            error!(
                "Invalid confirmation string: '{}'. Must type exact phrase.",
                confirmation
            );
            false
        }
    }

    /// Disable the wallet (re-lock the gate).
    pub fn disable(&mut self) {
        self.state = WalletSafetyState::Disabled;
        self.user_typed_confirmation = false;
        warn!("Wallet feature disabled by user");
    }

    /// Check if wallet operations are allowed.
    pub fn is_enabled(&self) -> bool {
        self.state == WalletSafetyState::Accepted && self.user_typed_confirmation
    }

    /// Guard function — returns Ok(()) if enabled, Err if not.
    /// Call this at the start of every wallet operation.
    pub fn require_enabled(&self) -> Result<(), WalletGateError> {
        match self.state {
            WalletSafetyState::Disabled => Err(WalletGateError::FeatureDisabled),
            WalletSafetyState::WarningShown => Err(WalletGateError::NotAccepted),
            WalletSafetyState::Accepted => {
                if self.user_typed_confirmation {
                    Ok(())
                } else {
                    Err(WalletGateError::NotAccepted)
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WalletGateError {
    #[error("Wallet feature is disabled. Call show_warning() and accept() to enable.")]
    FeatureDisabled,
    #[error("Safety warning must be accepted before using wallet features. Type the confirmation phrase.")]
    NotAccepted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_default_disabled() {
        let gate = WalletSafetyGate::new();
        assert!(!gate.is_enabled());
        assert!(gate.require_enabled().is_err());
    }

    #[test]
    fn test_gate_cannot_skip_warning() {
        let mut gate = WalletSafetyGate::new();
        // Try accepting without showing warning first
        assert!(!gate.accept("I understand the risks"));
        assert!(!gate.is_enabled());
    }

    #[test]
    fn test_gate_wrong_confirmation() {
        let mut gate = WalletSafetyGate::new();
        gate.show_warning();
        assert!(!gate.accept("yes"));
        assert!(!gate.accept("ok"));
        assert!(!gate.accept("I accept"));
        assert!(!gate.is_enabled());
    }

    #[test]
    fn test_gate_correct_flow() {
        let mut gate = WalletSafetyGate::new();
        gate.show_warning();
        assert!(gate.accept("I understand the risks"));
        assert!(gate.is_enabled());
        assert!(gate.require_enabled().is_ok());
    }

    #[test]
    fn test_gate_chinese_confirmation() {
        let mut gate = WalletSafetyGate::new();
        gate.show_warning_zh();
        assert!(gate.accept("我了解风险"));
        assert!(gate.is_enabled());
    }

    #[test]
    fn test_gate_disable() {
        let mut gate = WalletSafetyGate::new();
        gate.show_warning();
        gate.accept("I understand the risks");
        assert!(gate.is_enabled());
        gate.disable();
        assert!(!gate.is_enabled());
    }
}
