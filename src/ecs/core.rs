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

/// A wrapper for event components that are automatically cleaned up at the end of each frame
/// 
/// Events are short-lived components used for communication between systems.
/// When you add Event<T> to an entity, it will be automatically removed at the end of the frame
/// after all systems have been updated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Event<T> {
    pub data: T,
}

impl<T> Event<T> {
    /// Create a new event with the given data
    pub fn new(data: T) -> Self {
        Self { data }
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

/// A marker component that indicates a component of type T was just added to an entity
/// 
/// This component is automatically generated when a component is added to an entity
/// and is automatically cleaned up at the end of the frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentAdded<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> ComponentAdded<T> {
    /// Create a new ComponentAdded marker
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> Default for ComponentAdded<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// A wrapper component that contains the data of a component that was just removed from an entity
/// 
/// This component is automatically generated when a component is removed from an entity
/// and contains the data of the removed component. It is automatically cleaned up at the end of the frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentRemoved<T> {
    pub data: T,
}

impl<T> ComponentRemoved<T> {
    /// Create a new ComponentRemoved with the removed component's data
    pub fn new(data: T) -> Self {
        Self { data }
    }
    
    /// Get a reference to the removed component's data
    pub fn get(&self) -> &T {
        &self.data
    }
    
    /// Consume this wrapper and return the removed component's data
    pub fn into_inner(self) -> T {
        self.data
    }
}

impl<T> std::ops::Deref for ComponentRemoved<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> std::ops::DerefMut for ComponentRemoved<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

/// Trait for components that should be automatically cleaned up at the end of each frame
pub trait TemporaryComponent {
    /// Returns true if this component type should be automatically cleaned up
    fn is_temporary() -> bool {
        true
    }
}

// Implement TemporaryComponent for Event<T>
impl<T> TemporaryComponent for Event<T> {}

// Implement TemporaryComponent for ComponentAdded<T>
impl<T> TemporaryComponent for ComponentAdded<T> {}

// Implement TemporaryComponent for ComponentRemoved<T>
impl<T> TemporaryComponent for ComponentRemoved<T> {}