//! Core ECS types: Entity, Component basics
//!
//! This module contains the fundamental building blocks of the ECS system.

use serde::{Deserialize, Serialize};
use std::any::TypeId;

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

/// Trait for components that can specify their required components
///
/// Components implementing this trait can specify other components that must be present
/// on an entity before this component can be added.
///
/// Example usage:
/// ```rust
/// use rust_ecs::*;
///
/// #[derive(Debug)]
/// struct Position { x: i32, y: i32 }
///
/// #[derive(Debug)]
/// struct Velocity { dx: f32, dy: f32 }
///
/// impl Component for Velocity {
///     type RequiredComponents = (Position,);
/// }
/// ```
pub trait Component: 'static {
    /// Tuple of component types that must be present on an entity before this component can be added
    type RequiredComponents: RequiredComponentsCheck;
}

/// Trait for types that can validate required components exist on an entity
pub trait RequiredComponentsCheck {
    /// Check if all required components exist on the given entity in the world
    fn check_requirements(world: &crate::ecs::world::World, entity: Entity) -> bool;
}

// Implement RequiredComponentsCheck for empty requirements
impl RequiredComponentsCheck for () {
    fn check_requirements(_world: &crate::ecs::world::World, _entity: Entity) -> bool {
        true // No requirements, always satisfied
    }
}

// Implement RequiredComponentsCheck for single component requirement
impl<T: 'static> RequiredComponentsCheck for (T,) {
    fn check_requirements(world: &crate::ecs::world::World, entity: Entity) -> bool {
        world.has_component::<T>(entity)
    }
}

// Implement RequiredComponentsCheck for two component requirements
impl<T1: 'static, T2: 'static> RequiredComponentsCheck for (T1, T2) {
    fn check_requirements(world: &crate::ecs::world::World, entity: Entity) -> bool {
        world.has_component::<T1>(entity) && world.has_component::<T2>(entity)
    }
}

// Implement RequiredComponentsCheck for three component requirements
impl<T1: 'static, T2: 'static, T3: 'static> RequiredComponentsCheck for (T1, T2, T3) {
    fn check_requirements(world: &crate::ecs::world::World, entity: Entity) -> bool {
        world.has_component::<T1>(entity)
            && world.has_component::<T2>(entity)
            && world.has_component::<T3>(entity)
    }
}

// Implement RequiredComponentsCheck for four component requirements
impl<T1: 'static, T2: 'static, T3: 'static, T4: 'static> RequiredComponentsCheck
    for (T1, T2, T3, T4)
{
    fn check_requirements(world: &crate::ecs::world::World, entity: Entity) -> bool {
        world.has_component::<T1>(entity)
            && world.has_component::<T2>(entity)
            && world.has_component::<T3>(entity)
            && world.has_component::<T4>(entity)
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

/// A generic event wrapper component that gets automatically cleaned up at the end of each frame
///
/// Events are short-lived components that systems can dispatch to communicate with other systems.
/// They are automatically removed after all systems have been updated.
///
/// Example usage:
/// ```
/// use rust_ecs::*;
///
/// // Define an event type
/// #[derive(Debug, Clone)]
/// struct ShotsFired {
///     damage: i32,
///     target_id: u32,
/// }
///
/// // Create world and entity
/// let mut world = World::new();
/// let entity = world.create_entity();
///
/// // Dispatch the event by adding it to an entity
/// world.add_event(entity, ShotsFired { damage: 10, target_id: 5 });
///
/// // The event is now available for querying
/// let event = world.get_component::<Event<ShotsFired>>(entity);
/// assert!(event.is_some());
/// ```
#[derive(Debug, Clone)]
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

    /// Take ownership of the event data
    pub fn into_inner(self) -> T {
        self.data
    }
}

/// Automatically generated component that indicates a component was added to an entity
///
/// This component is automatically created when any component is added to an entity.
/// It gets cleaned up at the end of each frame.
///
/// Example usage:
/// ```
/// use rust_ecs::*;
///
/// // Create world and entity
/// let mut world = World::new();
/// let entity = world.create_entity();
///
/// // Add a component (automatically creates ComponentAdded notification)
/// world.add_component(entity, game::components::Position { x: 10, y: 20 });
///
/// // The ComponentAdded notification is now available for querying
/// let added = world.get_component::<ComponentAdded<game::components::Position>>(entity);
/// assert!(added.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct ComponentAdded<T> {
    /// Phantom data to carry the component type information
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

/// Automatically generated component that indicates a component was removed from an entity
///
/// This component is automatically created when any component is removed from an entity.
/// It contains the data of the removed component (moved, not copied).
/// It gets cleaned up at the end of each frame.
///
/// Example usage:
/// ```
/// use rust_ecs::*;
///
/// // Create world and entity
/// let mut world = World::new();
/// let entity = world.create_entity();
///
/// // Add a component first
/// world.add_component(entity, game::components::Position { x: 10, y: 20 });
///
/// // Remove the component with notification
/// let was_removed = world.remove_component_with_notification::<game::components::Position>(entity);
/// assert!(was_removed);
///
/// // The ComponentRemoved notification is now available for querying
/// let removed = world.get_component::<ComponentRemoved<game::components::Position>>(entity);
/// assert!(removed.is_some());
/// ```
#[derive(Debug, Clone)]
pub struct ComponentRemoved<T> {
    /// The data from the removed component
    pub data: T,
}

impl<T> ComponentRemoved<T> {
    /// Create a new ComponentRemoved with the given data
    pub fn new(data: T) -> Self {
        Self { data }
    }

    /// Get a reference to the removed component's data
    pub fn get_data(&self) -> &T {
        &self.data
    }

    /// Get a mutable reference to the removed component's data
    pub fn get_data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Take ownership of the removed component's data
    pub fn into_data(self) -> T {
        self.data
    }
}
