//! Hazard components for environmental dangers.

pub mod area_damage;
pub mod destructible_terrain;
pub mod floor_spike;
pub mod gravity_zone;
pub mod saw_blade;
pub mod spring_board;
pub mod stepping_ground;
pub mod water_zone;

pub use area_damage::AreaDamageComponent;
pub use floor_spike::{FloorSpikeComponent, SpikeState};
pub use saw_blade::SawBladeComponent;
pub use spring_board::SpringBoardComponent;
pub use stepping_ground::SteppingGroundComponent;
