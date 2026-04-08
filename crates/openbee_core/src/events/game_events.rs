//! Agent-level events for the OpenBee platform.

use serde::{Deserialize, Serialize};

use crate::events::event_bus::Event;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSubmitted {
    pub task_id: u64,
    pub instruction: String,
}
impl Event for TaskSubmitted {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCompleted {
    pub task_id: u64,
    pub success: bool,
    pub summary: String,
}
impl Event for TaskCompleted {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskFailed {
    pub task_id: u64,
    pub error: String,
}
impl Event for TaskFailed {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSigned {
    pub chain: String,
    pub tx_hash: String,
}
impl Event for TransactionSigned {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletEvent {
    pub action: String,
    pub wallet_name: String,
}
impl Event for WalletEvent {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserEvent {
    pub action: String,
    pub url: Option<String>,
}
impl Event for BrowserEvent {}
