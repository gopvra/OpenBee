//! # OpenBee Agent — AI Task Execution System
//!
//! Assign tasks to the bee agent, it executes them autonomously and reports back.
//!
//! ## Example
//!
//! ```ignore
//! let mut agent = BeeAgent::new();
//! agent.register_skill(Box::new(WalletSkill::new()));
//! agent.register_skill(Box::new(BrowserSkill::new()));
//! agent.register_skill(Box::new(SystemSkill::new()));
//!
//! let task_id = agent.submit("Check my ETH balance").await;
//! let result = agent.wait_for(task_id).await;
//! println!("{}", result.summary);
//! ```

pub mod agent;
pub mod executor;
pub mod skill;
pub mod skills;
pub mod task;

pub use agent::BeeAgent;
pub use skill::Skill;
pub use task::{Task, TaskId, TaskResult, TaskStatus};
