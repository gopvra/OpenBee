//! Asset management with caching and pluggable loaders.

pub mod cache;
pub mod loaders;
pub mod manager;

pub use cache::ResourceCache;
pub use manager::ResourceManager;
