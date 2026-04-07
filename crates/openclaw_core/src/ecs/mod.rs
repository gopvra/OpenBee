//! Custom lightweight Entity-Component-System framework.

pub mod component;
pub mod entity;
pub mod system;
pub mod world;

pub use component::{Component, ComponentStorage};
pub use entity::{Entity, EntityAllocator};
pub use system::{System, SystemScheduler};
pub use world::World;
