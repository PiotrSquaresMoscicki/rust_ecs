//! A Rust ECS (Entity Component System) framework with high debuggability.
//!
//! This library provides a unique ECS implementation where systems declare their
//! input and output components, enabling comprehensive change tracking and replay
//! functionality for debugging complex system interactions.

use std::any::{Any, TypeId};
use std::collections::HashMap;

/// A dummy function to demonstrate the library.
/// Returns the sum of two numbers.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// An Entity is a unique identifier consisting of world index and entity index.
/// This allows entities to be uniquely identified across multiple worlds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

/// The System trait defines the contract for all systems in the ECS.
/// Systems declare their input and output components for change tracking.
pub trait System {
    /// Components that the system will read from without modifying them
    type InComponents;
    /// Components that the system will read from and write to
    type OutComponents;

    /// Called once before the first update to initialize system state
    fn initialize(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>);

    /// Called every frame to update the system
    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>);

    /// Called when the system is being removed or the world is shutting down
    fn deinitialize(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>);
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

/// Trait for components that can be queried
pub trait QueryComponent<'a> {
    type Item;

    /// Extract the component from the world for a specific entity
    fn get_component(world: &'a World, entity: Entity) -> Option<Self::Item>;
}

/// Implementation for immutable component access
impl<'a, T: 'static> QueryComponent<'a> for T {
    type Item = &'a T;

    fn get_component(world: &'a World, entity: Entity) -> Option<Self::Item> {
        world
            .components
            .get(&TypeId::of::<T>())?
            .iter()
            .find_map(|(e, component)| {
                if *e == entity {
                    component.downcast_ref::<T>()
                } else {
                    None
                }
            })
    }
}

/// Trait for multi-component queries with mixed mutable/immutable access
pub trait MixedMultiQuery<'a> {
    type Item;

    /// Get all entities that have all the required components with mixed access
    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)>;
}

/// Trait for components that can be queried with mixed access patterns
pub trait MixedQueryComponent<'a> {
    type Item;

    /// Extract the component from the world for a specific entity with appropriate access
    fn get_mixed_component(world: &'a mut World, entity: Entity) -> Option<Self::Item>;
}

/// A wrapper to explicitly mark input (immutable) component access
pub struct In<T>(std::marker::PhantomData<T>);

/// Implementation for input (immutable) component access in mixed queries
impl<'a, T: 'static> MixedQueryComponent<'a> for In<T> {
    type Item = &'a T;

    fn get_mixed_component(world: &'a mut World, entity: Entity) -> Option<Self::Item> {
        // For immutable access, we can safely convert the mutable reference
        unsafe {
            let world_ref = &*(world as *const World);
            world_ref
                .components
                .get(&TypeId::of::<T>())?
                .iter()
                .find_map(|(e, component)| {
                    if *e == entity {
                        component.downcast_ref::<T>()
                    } else {
                        None
                    }
                })
        }
    }
}

/// Implementation for output (mutable) component access in mixed queries
impl<'a, T: 'static> MixedQueryComponent<'a> for Out<T> {
    type Item = &'a mut T;

    fn get_mixed_component(world: &'a mut World, entity: Entity) -> Option<Self::Item> {
        world
            .components
            .get_mut(&TypeId::of::<T>())?
            .iter_mut()
            .find_map(|(e, component)| {
                if *e == entity {
                    component.downcast_mut::<T>()
                } else {
                    None
                }
            })
    }
}

// Concrete implementations for 2 components
impl<'a, A, B> MixedMultiQuery<'a> for (A, B)
where
    A: MixedQueryComponent<'a> + 'static,
    B: MixedQueryComponent<'a> + 'static,
{
    type Item = (A::Item, B::Item);

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);
                let b = B::get_mixed_component(&mut *world_ptr, entity);

                if let (Some(a), Some(b)) = (a, b) {
                    results.push((entity, (a, b)));
                }
            }
        }

        results
    }
}

// Concrete implementations for 3 components
impl<'a, A, B, C> MixedMultiQuery<'a> for (A, B, C)
where
    A: MixedQueryComponent<'a> + 'static,
    B: MixedQueryComponent<'a> + 'static,
    C: MixedQueryComponent<'a> + 'static,
{
    type Item = (A::Item, B::Item, C::Item);

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);
                let b = B::get_mixed_component(&mut *world_ptr, entity);
                let c = C::get_mixed_component(&mut *world_ptr, entity);

                if let (Some(a), Some(b), Some(c)) = (a, b, c) {
                    results.push((entity, (a, b, c)));
                }
            }
        }

        results
    }
}

/// WorldView provides controlled access to world data for systems
pub struct WorldView<InComponents, OutComponents> {
    world: *mut World,
    _input_phantom: std::marker::PhantomData<InComponents>,
    _output_phantom: std::marker::PhantomData<OutComponents>,
}

impl<I, O> WorldView<I, O> {
    /// Create a new WorldView with type constraints
    pub fn new(world: &mut World) -> Self {
        Self {
            world: world as *mut World,
            _input_phantom: std::marker::PhantomData,
            _output_phantom: std::marker::PhantomData,
        }
    }

    /// Get a reference to the underlying world (unsafe due to raw pointer)
    unsafe fn world(&self) -> &World {
        &*self.world
    }

    /// Get a mutable reference to the underlying world (unsafe due to raw pointer)
    unsafe fn world_mut(&mut self) -> &mut World {
        &mut *self.world
    }

    /// Create a new entity
    pub fn create_entity(&mut self) -> Entity {
        unsafe { self.world_mut().create_entity() }
    }

    /// Add a component to an entity
    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        unsafe { self.world_mut().add_component(entity, component) }
    }

    /// Get a component for an entity (if it exists)
    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
        unsafe {
            let world = self.world();
            world
                .components
                .get(&TypeId::of::<T>())?
                .iter()
                .find_map(|(e, component)| {
                    if *e == entity {
                        component.downcast_ref::<T>()
                    } else {
                        None
                    }
                })
        }
    }

    /// Get a mutable component for an entity (if it exists)
    pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        unsafe {
            let world = self.world_mut();
            world
                .components
                .get_mut(&TypeId::of::<T>())?
                .iter_mut()
                .find_map(|(e, component)| {
                    if *e == entity {
                        component.downcast_mut::<T>()
                    } else {
                        None
                    }
                })
        }
    }

    /// Query entities that have all specified components
    pub fn query<T: 'static>(&self) -> Vec<(Entity, &T)> {
        unsafe {
            let world = self.world();
            let mut results = Vec::new();

            if let Some(components) = world.components.get(&TypeId::of::<T>()) {
                for (entity, component) in components {
                    if let Some(comp_ref) = component.downcast_ref::<T>() {
                        results.push((*entity, comp_ref));
                    }
                }
            }

            results
        }
    }

    /// Query entities with mutable access to components
    pub fn query_mut<T: 'static>(&mut self) -> Vec<(Entity, &mut T)> {
        unsafe {
            let world = self.world_mut();
            let mut results = Vec::new();

            if let Some(components) = world.components.get_mut(&TypeId::of::<T>()) {
                for (entity, component) in components {
                    if let Some(comp_ref) = component.downcast_mut::<T>() {
                        results.push((*entity, comp_ref));
                    }
                }
            }

            results
        }
    }

    /// Query entities with multiple components, using Out<T> for mutable access and In<T> for immutable access
    /// Example: world_view.query_components::<(In<Position>, Out<Velocity>)>()
    pub fn query_components<Q>(&mut self) -> Vec<(Entity, <Q as MixedMultiQuery<'_>>::Item)>
    where
        for<'a> Q: MixedMultiQuery<'a>,
    {
        unsafe { Q::query_mixed(self.world_mut()) }
    }
}

/// Tracks a specific component change
#[derive(Debug, Clone)]
pub struct ComponentChange {
    pub entity: Entity,
    pub component_type: TypeId,
    pub operation: ComponentOperation,
}

/// Types of component operations
#[derive(Debug, Clone)]
pub enum ComponentOperation {
    Added,
    Modified,
    Removed,
}

/// Placeholder for system initialization diff tracking
#[derive(Debug)]
pub struct SystemInitDiff {
    pub component_changes: Vec<ComponentChange>,
}

impl Default for SystemInitDiff {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemInitDiff {
    pub fn new() -> Self {
        Self {
            component_changes: Vec::new(),
        }
    }

    pub fn record_component_change(&mut self, change: ComponentChange) {
        self.component_changes.push(change);
    }
}

/// Placeholder for system update diff tracking
#[derive(Debug)]
pub struct SystemUpdateDiff {
    pub component_changes: Vec<ComponentChange>,
}

impl Default for SystemUpdateDiff {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemUpdateDiff {
    pub fn new() -> Self {
        Self {
            component_changes: Vec::new(),
        }
    }

    pub fn record_component_change(&mut self, change: ComponentChange) {
        self.component_changes.push(change);
    }
}

/// Placeholder for system deinitialization diff tracking
#[derive(Debug)]
pub struct SystemDeinitDiff {
    pub component_changes: Vec<ComponentChange>,
}

impl Default for SystemDeinitDiff {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemDeinitDiff {
    pub fn new() -> Self {
        Self {
            component_changes: Vec::new(),
        }
    }

    pub fn record_component_change(&mut self, change: ComponentChange) {
        self.component_changes.push(change);
    }
}

/// Tracks overall world update changes
#[derive(Debug)]
pub struct WorldUpdateDiff {
    system_diffs: Vec<SystemUpdateDiff>,
}

impl Default for WorldUpdateDiff {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldUpdateDiff {
    pub fn new() -> Self {
        Self {
            system_diffs: Vec::new(),
        }
    }

    pub fn record(&mut self, diff: SystemUpdateDiff) {
        self.system_diffs.push(diff);
    }
}

/// Maintains history of all world changes for replay functionality
#[derive(Debug)]
pub struct WorldUpdateHistory {
    updates: Vec<WorldUpdateDiff>,
}

impl Default for WorldUpdateHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldUpdateHistory {
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
        }
    }

    pub fn record(&mut self, diff: WorldUpdateDiff) {
        self.updates.push(diff);
    }
}

/// Type-erased system wrapper for storage in World
trait SystemWrapper {
    fn initialize(&mut self, world: &mut World) -> SystemInitDiff;
    fn update(&mut self, world: &mut World) -> SystemUpdateDiff;
    #[allow(dead_code)]
    fn deinitialize(&mut self, world: &mut World) -> SystemDeinitDiff;
}

/// Concrete implementation of SystemWrapper for a specific system type
struct ConcreteSystemWrapper<S: System> {
    system: S,
}

impl<S: System> ConcreteSystemWrapper<S> {
    fn new(system: S) -> Self {
        Self { system }
    }
}

impl<S: System> SystemWrapper for ConcreteSystemWrapper<S> {
    fn initialize(&mut self, world: &mut World) -> SystemInitDiff {
        let mut world_view = WorldView::<S::InComponents, S::OutComponents>::new(world);
        self.system.initialize(&mut world_view);
        SystemInitDiff::new()
    }

    fn update(&mut self, world: &mut World) -> SystemUpdateDiff {
        let mut world_view = WorldView::<S::InComponents, S::OutComponents>::new(world);
        self.system.update(&mut world_view);
        SystemUpdateDiff::new()
    }

    fn deinitialize(&mut self, world: &mut World) -> SystemDeinitDiff {
        let mut world_view = WorldView::<S::InComponents, S::OutComponents>::new(world);
        self.system.deinitialize(&mut world_view);
        SystemDeinitDiff::new()
    }
}

/// Type alias for component storage to reduce complexity
type ComponentStorage = HashMap<TypeId, Vec<(Entity, Box<dyn Any>)>>;

/// The main World struct that manages entities, components, and systems
pub struct World {
    /// Unique index identifying this world
    world_index: usize,
    entities: Vec<Entity>,
    components: ComponentStorage,
    systems: Vec<Box<dyn SystemWrapper>>,
    next_entity_id: usize,
    #[allow(dead_code)]
    child_worlds: Vec<World>,
    world_update_history: WorldUpdateHistory,
    /// Global counter for assigning unique world indices
    next_world_index: usize,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    /// Creates a new empty world with world index 0 (main world)
    pub fn new() -> Self {
        Self::new_with_index(0)
    }

    /// Creates a new world with a specific world index
    pub fn new_with_index(world_index: usize) -> Self {
        Self {
            world_index,
            entities: Vec::new(),
            components: HashMap::new(),
            systems: Vec::new(),
            next_entity_id: 0,
            child_worlds: Vec::new(),
            world_update_history: WorldUpdateHistory::new(),
            next_world_index: world_index + 1,
        }
    }

    /// Get the world index of this world
    pub fn world_index(&self) -> usize {
        self.world_index
    }

    /// Create a child world with a unique world index
    pub fn create_child_world(&mut self) -> usize {
        let child_world_index = self.next_world_index;
        self.next_world_index += 1;
        let child_world = World::new_with_index(child_world_index);
        self.child_worlds.push(child_world);
        child_world_index
    }

    /// Get a reference to a child world by index
    pub fn get_child_world(&self, world_index: usize) -> Option<&World> {
        self.child_worlds
            .iter()
            .find(|world| world.world_index == world_index)
    }

    /// Get a mutable reference to a child world by index
    pub fn get_child_world_mut(&mut self, world_index: usize) -> Option<&mut World> {
        self.child_worlds
            .iter_mut()
            .find(|world| world.world_index == world_index)
    }

    /// Add a system to the world
    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems
            .push(Box::new(ConcreteSystemWrapper::new(system)));
    }

    /// Create a new entity and return its identifier
    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity::new(self.world_index, self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.push(entity);
        entity
    }

    /// Add a component to an entity
    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        self.components
            .entry(TypeId::of::<T>())
            .or_default()
            .push((entity, Box::new(component)));
    }

    /// Get a component for an entity (if it exists)
    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())?
            .iter()
            .find_map(|(e, component)| {
                if *e == entity {
                    component.downcast_ref::<T>()
                } else {
                    None
                }
            })
    }

    /// Initialize all systems (called once before the first update)
    pub fn initialize_systems(&mut self) {
        // We need to work around the borrowing issue by taking ownership temporarily
        let mut systems = std::mem::take(&mut self.systems);

        for system in &mut systems {
            let _diff = system.initialize(self);
            // TODO: Record diff in world update history
        }

        self.systems = systems;
    }

    /// Update all systems for one frame
    pub fn update(&mut self) {
        let mut world_update_diff = WorldUpdateDiff::new();

        // We need to work around the borrowing issue by taking ownership temporarily
        let mut systems = std::mem::take(&mut self.systems);

        for system in &mut systems {
            let system_diff = system.update(self);
            world_update_diff.record(system_diff);
        }

        self.systems = systems;
        self.world_update_history.record(world_update_diff);
    }

    /// Get the number of entities in the world
    pub fn entity_count(&self) -> u32 {
        self.entities.len() as u32
    }

    /// Get the update history for replay functionality
    pub fn get_update_history(&self) -> &WorldUpdateHistory {
        &self.world_update_history
    }

    /// Apply a recorded world update diff for replay
    pub fn apply_update_diff(&mut self, diff: &WorldUpdateDiff) {
        // In a complete implementation, this would replay all component changes
        // For now, we just demonstrate the structure
        println!(
            "Applying world update diff with {} system updates",
            diff.system_diffs.len()
        );
        for (i, system_diff) in diff.system_diffs.iter().enumerate() {
            println!(
                "  System {}: {} component changes",
                i,
                system_diff.component_changes.len()
            );
        }
    }

    /// Replay the entire world history in a new world
    pub fn replay_history(history: &WorldUpdateHistory) -> World {
        let mut new_world = World::new();

        println!(
            "Replaying world history with {} updates",
            history.updates.len()
        );
        for (frame, update) in history.updates.iter().enumerate() {
            println!("Frame {}: Applying update", frame + 1);
            new_world.apply_update_diff(update);
        }

        new_world
    }

    /// Remove an entity and all its components
    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        if let Some(pos) = self.entities.iter().position(|e| *e == entity) {
            self.entities.remove(pos);

            // Remove all components for this entity
            for components in self.components.values_mut() {
                components.retain(|(e, _)| *e != entity);
            }

            true
        } else {
            false
        }
    }

    /// Check if an entity exists
    pub fn entity_exists(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
    }

    /// Get all entities that have a specific component type
    pub fn entities_with_component<T: 'static>(&self) -> Vec<Entity> {
        self.components
            .get(&TypeId::of::<T>())
            .map(|components| components.iter().map(|(entity, _)| *entity).collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_function() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(0, 0), 0);
    }

    #[test]
    fn test_world_creation() {
        let world = World::new();
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_entity_creation() {
        let mut world = World::new();

        let entity1 = world.create_entity();
        assert_eq!(entity1, Entity::new(0, 0)); // world 0, entity 0
        assert_eq!(world.entity_count(), 1);

        let entity2 = world.create_entity();
        assert_eq!(entity2, Entity::new(0, 1)); // world 0, entity 1
        assert_eq!(world.entity_count(), 2);
    }

    // Example components for testing
    #[derive(Debug, PartialEq)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug, PartialEq, Clone)]
    struct Velocity {
        dx: f32,
        dy: f32,
    }

    #[test]
    fn test_component_addition() {
        let mut world = World::new();
        let entity = world.create_entity();

        world.add_component(entity, Position { x: 1.0, y: 2.0 });
        world.add_component(entity, Velocity { dx: 0.5, dy: -0.5 });

        // Components are added successfully if no panic occurs
        assert_eq!(world.entity_count(), 1);
    }

    // Example system for testing
    struct TestSystem;

    impl System for TestSystem {
        type InComponents = ();
        type OutComponents = ();

        fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
            // Test system initialization
        }

        fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
            // Test system update
        }

        fn deinitialize(
            &mut self,
            _world: &mut WorldView<Self::InComponents, Self::OutComponents>,
        ) {
            // Test system deinitialization
        }
    }

    #[test]
    fn test_system_addition() {
        let mut world = World::new();
        world.add_system(TestSystem);

        // System added successfully if no panic occurs
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_system_initialization() {
        let mut world = World::new();
        world.add_system(TestSystem);

        // Should not panic when initializing systems
        world.initialize_systems();
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_world_update() {
        let mut world = World::new();
        world.add_system(TestSystem);
        world.initialize_systems();

        // Should not panic when updating world
        world.update();
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_component_querying() {
        let mut world = World::new();
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        // Add different components to different entities
        world.add_component(entity1, Position { x: 1.0, y: 2.0 });
        world.add_component(entity1, Velocity { dx: 0.5, dy: -0.5 });
        world.add_component(entity2, Position { x: 3.0, y: 4.0 });

        // Test getting component directly
        let pos1 = world.get_component::<Position>(entity1);
        assert!(pos1.is_some());
        assert_eq!(pos1.unwrap().x, 1.0);
        assert_eq!(pos1.unwrap().y, 2.0);

        // Test getting component that doesn't exist
        let vel2 = world.get_component::<Velocity>(entity2);
        assert!(vel2.is_none());
    }

    #[test]
    fn test_worldview_querying() {
        let mut world = World::new();
        let mut world_view = WorldView::<(), ()>::new(&mut world);

        let entity1 = world_view.create_entity();
        let entity2 = world_view.create_entity();

        world_view.add_component(entity1, Position { x: 1.0, y: 2.0 });
        world_view.add_component(entity2, Position { x: 3.0, y: 4.0 });

        // Test querying all positions
        let positions = world_view.query::<Position>();
        assert_eq!(positions.len(), 2);

        // Test mutable querying
        let mut positions_mut = world_view.query_mut::<Position>();
        assert_eq!(positions_mut.len(), 2);

        // Modify a position
        for (entity, position) in &mut positions_mut {
            if *entity == entity1 {
                position.x = 10.0;
            }
        }

        // Verify the change
        let pos1 = world_view.get_component::<Position>(entity1);
        assert_eq!(pos1.unwrap().x, 10.0);
    }

    #[test]
    fn test_entity_removal() {
        let mut world = World::new();
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        world.add_component(entity1, Position { x: 1.0, y: 2.0 });
        world.add_component(entity2, Position { x: 3.0, y: 4.0 });

        assert_eq!(world.entity_count(), 2);
        assert!(world.entity_exists(entity1));
        assert!(world.entity_exists(entity2));

        // Remove entity1
        assert!(world.remove_entity(entity1));
        assert_eq!(world.entity_count(), 1);
        assert!(!world.entity_exists(entity1));
        assert!(world.entity_exists(entity2));

        // Try to remove entity1 again
        assert!(!world.remove_entity(entity1));
        assert_eq!(world.entity_count(), 1);
    }

    #[test]
    fn test_entities_with_component() {
        let mut world = World::new();
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();
        let entity3 = world.create_entity();

        world.add_component(entity1, Position { x: 1.0, y: 2.0 });
        world.add_component(entity1, Velocity { dx: 0.5, dy: -0.5 });
        world.add_component(entity2, Position { x: 3.0, y: 4.0 });
        world.add_component(entity3, Velocity { dx: 1.0, dy: 1.0 });

        let pos_entities = world.entities_with_component::<Position>();
        let vel_entities = world.entities_with_component::<Velocity>();

        assert_eq!(pos_entities.len(), 2);
        assert!(pos_entities.contains(&entity1));
        assert!(pos_entities.contains(&entity2));

        assert_eq!(vel_entities.len(), 2);
        assert!(vel_entities.contains(&entity1));
        assert!(vel_entities.contains(&entity3));
    }

    #[test]
    fn test_update_history() {
        let mut world = World::new();
        world.add_system(TestSystem);
        world.initialize_systems();

        // Run a few updates
        world.update();
        world.update();

        let history = world.get_update_history();
        assert_eq!(history.updates.len(), 2);
    }

    #[test]
    fn test_multi_component_query() {
        let mut world = World::new();
        let mut world_view = WorldView::<(), ()>::new(&mut world);

        let entity1 = world_view.create_entity();
        let entity2 = world_view.create_entity();
        let entity3 = world_view.create_entity();

        // Entity1 has both Position and Velocity
        world_view.add_component(entity1, Position { x: 1.0, y: 2.0 });
        world_view.add_component(entity1, Velocity { dx: 0.5, dy: -0.5 });

        // Entity2 has only Position
        world_view.add_component(entity2, Position { x: 3.0, y: 4.0 });

        // Entity3 has only Velocity
        world_view.add_component(entity3, Velocity { dx: 1.0, dy: 1.0 });

        // Query for entities with both Position and Velocity (both immutable)
        let results = world_view.query_components::<(In<Position>, In<Velocity>)>();

        // Only entity1 should be returned
        assert_eq!(results.len(), 1);
        let (entity, (position, velocity)) = &results[0];
        assert_eq!(*entity, entity1);
        assert_eq!(position.x, 1.0);
        assert_eq!(position.y, 2.0);
        assert_eq!(velocity.dx, 0.5);
        assert_eq!(velocity.dy, -0.5);
    }

    #[test]
    fn test_multi_component_query_mut() {
        let mut world = World::new();
        let mut world_view = WorldView::<(), ()>::new(&mut world);

        let entity1 = world_view.create_entity();
        let entity2 = world_view.create_entity();

        // Both entities have Position and Velocity
        world_view.add_component(entity1, Position { x: 1.0, y: 2.0 });
        world_view.add_component(entity1, Velocity { dx: 0.5, dy: -0.5 });
        world_view.add_component(entity2, Position { x: 3.0, y: 4.0 });
        world_view.add_component(entity2, Velocity { dx: 1.0, dy: 1.0 });

        // Query for entities with Position (immutable) and Velocity (mutable)
        let mut results = world_view.query_components::<(In<Position>, Out<Velocity>)>();

        // Both entities should be returned
        assert_eq!(results.len(), 2);

        // Modify velocities
        for (_entity, (position, velocity)) in &mut results {
            velocity.dx *= 2.0;
            velocity.dy *= 2.0;
            println!(
                "Position: ({}, {}), Modified velocity: ({}, {})",
                position.x, position.y, velocity.dx, velocity.dy
            );
        }

        // Verify changes were applied
        let velocity1 = world_view.get_component::<Velocity>(entity1).unwrap();
        let velocity2 = world_view.get_component::<Velocity>(entity2).unwrap();

        assert_eq!(velocity1.dx, 1.0); // 0.5 * 2.0
        assert_eq!(velocity1.dy, -1.0); // -0.5 * 2.0
        assert_eq!(velocity2.dx, 2.0); // 1.0 * 2.0
        assert_eq!(velocity2.dy, 2.0); // 1.0 * 2.0
    }

    #[test]
    fn test_multi_world_entity_identification() {
        let mut main_world = World::new();

        // Create entities in main world (index 0)
        let main_entity1 = main_world.create_entity();
        let main_entity2 = main_world.create_entity();

        // Create a child world
        let child_world_index = main_world.create_child_world();
        assert_eq!(child_world_index, 1);

        // Verify main world index before borrowing child world
        assert_eq!(main_world.world_index(), 0);

        // Create entities in child world
        let (child_entity1, child_entity2, child_world_idx) = {
            let child_world = main_world.get_child_world_mut(child_world_index).unwrap();
            let entity1 = child_world.create_entity();
            let entity2 = child_world.create_entity();
            let world_idx = child_world.world_index();
            (entity1, entity2, world_idx)
        };

        // Verify entity identification
        assert_eq!(main_entity1, Entity::new(0, 0)); // world 0, entity 0
        assert_eq!(main_entity2, Entity::new(0, 1)); // world 0, entity 1
        assert_eq!(child_entity1, Entity::new(1, 0)); // world 1, entity 0
        assert_eq!(child_entity2, Entity::new(1, 1)); // world 1, entity 1

        // Verify world indices
        assert_eq!(child_world_idx, 1);

        // Entities from different worlds should not be equal even with same entity index
        assert_ne!(main_entity1, child_entity1);
    }
}
