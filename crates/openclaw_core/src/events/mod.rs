//! Event system with type-erased publish/subscribe.

pub mod event_bus;
pub mod game_events;
pub mod input_events;

pub use event_bus::EventBus;
pub use game_events::*;
pub use input_events::*;
