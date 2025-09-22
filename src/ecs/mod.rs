//! Rust ECS Framework
//!
//! A high-debuggability ECS implementation with comprehensive change tracking
//! and replay functionality for debugging complex system interactions.

pub mod core;
pub mod diff;
pub mod system;
pub mod query;
pub mod replay;
pub mod world;

// Re-export the most commonly used types
pub use core::{Entity, Out, In, ComponentChange, ComponentOperation, WorldOperation};
pub use diff::{Diff, DiffComponent, DiffComponentChange};
pub use system::{System, SystemInitDiff, SystemUpdateDiff, SystemDeinitDiff, WorldUpdateDiff, WorldUpdateHistory};
pub use query::{QueryComponent, MixedMultiQuery, MixedQueryComponent};
pub use replay::{ReplayLogConfig, AutoReplayLogger};
pub use world::{World, WorldView};

// Re-export the derive macro
pub use rust_ecs_derive::Diff;