//! # OpenClaw Mod
//!
//! Mod support system for OpenClaw — discovery, loading, registry, dependency
//! resolution, and asset override management.

pub mod asset_override;
pub mod mod_loader;
pub mod mod_manifest;
pub mod mod_registry;

pub use asset_override::AssetOverrideManager;
pub use mod_loader::{LoadedMod, ModLoader};
pub use mod_manifest::{ModDependency, ModManifest, OverrideMode};
pub use mod_registry::ModRegistry;
