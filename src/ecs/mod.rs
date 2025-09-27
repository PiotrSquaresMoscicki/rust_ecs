//! Rust ECS Framework
//!
//! A high-debuggability ECS implementation with comprehensive change tracking
//! and replay functionality for debugging complex system interactions.

pub mod core;
pub mod diff;
pub mod query;
pub mod replay;
pub mod system;
pub mod world;

// Re-export the most commonly used types
pub use core::{
    ComponentAdded, ComponentChange, ComponentOperation, ComponentRemoved, Entity, Event, In, Not,
    Out, WorldOperation,
};
pub use diff::{Diff, DiffComponent, DiffComponentChange};
pub use query::{MixedMultiQuery, MixedQueryComponent, QueryComponent};
pub use replay::{AutoReplayLogger, ReplayLogConfig};
pub use system::{
    System, SystemDeinitDiff, SystemInitDiff, SystemUpdateDiff, WorldUpdateDiff, WorldUpdateHistory,
};
pub use world::{World, WorldView};

// Re-export the derive macro
pub use rust_ecs_derive::Diff;
