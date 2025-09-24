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
    /// Index of the world this entity belongs to
    pub world_index: usize,
    /// Index of the entity within its world
    pub entity_index: usize,
}

impl Entity {
    /// Create a new entity with world and entity indices
    pub fn new(world_index: usize, entity_index: usize) -> Self {
        Self {
            world_index,
            entity_index,
        }
    }

    /// Get the world index of this entity
    pub fn world_index(&self) -> usize {
        self.world_index
    }

    /// Get the entity index within its world
    pub fn entity_index(&self) -> usize {
        self.entity_index
    }
}

/// A wrapper for output (mutable) component access in queries
pub struct Out<T>(pub T);

impl<T> Out<T> {
    /// Create a new Out wrapper
    pub fn new(value: T) -> Self {
        Out(value)
    }

    /// Get the inner value
    pub fn get(&self) -> &T {
        &self.0
    }

    /// Get a mutable reference to the inner value
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> std::ops::Deref for Out<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Out<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A wrapper to explicitly mark input (immutable) component access
pub struct In<T>(std::marker::PhantomData<T>);

/// A wrapper to mark that entities should NOT have a specific component
pub struct Not<T>(std::marker::PhantomData<T>);

/// A trait to mark components as events that should be automatically cleaned up
pub trait EventMarker {
    /// Returns true to indicate this is an event component
    fn is_event() -> bool where Self: Sized { true }
}

/// A wrapper for event components that are automatically cleaned up each frame
/// Events are short-lived components that systems can dispatch and consume
/// They are automatically removed at the end of each frame after all systems have run
#[derive(Debug, Clone)]
pub struct Event<T> {
    /// The actual event data
    pub data: T,
}

impl<T> EventMarker for Event<T> {}

impl<T> Event<T> {
    /// Create a new event with the provided data
    pub fn new(data: T) -> Self {
        Event { data }
    }

    /// Get a reference to the event data
    pub fn get(&self) -> &T {
        &self.data
    }

    /// Get a mutable reference to the event data
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Consume the event and return the inner data
    pub fn into_inner(self) -> T {
        self.data
    }
}

impl<T> std::ops::Deref for Event<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> std::ops::DerefMut for Event<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

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
    /// A world was created
    CreateWorld(usize),
    /// A world was removed
    RemoveWorld(usize),
    /// A system was added to the world
    AddSystem(String),
}

/// Record of a component change during system execution
#[derive(Debug, Clone)]
pub struct ComponentChange {
    pub entity: Entity,
    pub component_type: TypeId,
    pub operation: ComponentOperation,
}