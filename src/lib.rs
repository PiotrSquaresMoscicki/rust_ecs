//! A Rust ECS (Entity Component System) framework with high debuggability.
//!
//! This library provides a unique ECS implementation where systems declare their
//! input and output components, enabling comprehensive change tracking and replay
//! functionality for debugging complex system interactions.

// Re-export the main ECS framework
pub mod ecs;

// Game-specific implementation (example usage)
pub mod game;

// Re-export the most commonly used types for convenience
pub use ecs::{
    Entity, World, WorldView, System, DiffComponent, Out, In,
    ReplayLogConfig, AutoReplayLogger, WorldUpdateHistory,
    QueryComponent, MixedMultiQuery, MixedQueryComponent,
    ComponentChange, ComponentOperation, WorldOperation,
    SystemInitDiff, SystemUpdateDiff, SystemDeinitDiff, WorldUpdateDiff,
    DiffComponentChange
};

// Re-export the derive macro from the derive crate and the trait
pub use rust_ecs_derive::Diff;
pub use ecs::Diff;