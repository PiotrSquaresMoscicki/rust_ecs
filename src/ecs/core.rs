//! Core ECS types: Entity, Component basics
//!
//! This module contains the fundamental building blocks of the ECS system.

use std::any::TypeId;
use serde::{Serialize, Deserialize};

/// A unique identifier for entities in the ECS world.
/// 
/// Each entity has an ID and generation to handle entity reuse safely.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Entity {
    pub(crate) id: usize,
    pub(crate) generation: usize,
}

impl Entity {
    /// Create a new entity with the given ID and generation
    pub fn new(id: usize, generation: usize) -> Self {
        Self { id, generation }
    }

    /// Get the entity's ID
    pub fn id(&self) -> usize {
        self.id
    }

    /// Get the entity's generation
    pub fn generation(&self) -> usize {
        self.generation
    }
}

/// A wrapper for output (mutable) component access in queries
pub struct Out<T>(pub T);

/// A wrapper for input (immutable) component access in queries  
pub struct In<T>(std::marker::PhantomData<T>);

/// Enumeration of component operations that can occur during system execution
#[derive(Debug, Clone)]
pub enum ComponentOperation {
    /// A component was added to an entity
    Add,
    /// A component was modified on an entity
    Modify,
    /// A component was removed from an entity
    Remove,
}

/// Enumeration of world-level operations that can occur during system execution
#[derive(Debug, Clone)]
pub enum WorldOperation {
    /// An entity was created
    CreateEntity(Entity),
    /// An entity was removed
    RemoveEntity(Entity),
    /// A system was added to the world
    AddSystem(String),
}

/// Record of a component change during system execution
#[derive(Debug, Clone)]
pub struct ComponentChange {
    pub entity: Entity,
    pub type_id: TypeId,
    pub operation: ComponentOperation,
}