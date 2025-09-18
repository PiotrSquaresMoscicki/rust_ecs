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

/// An Entity is just a unique identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub usize);

/// The System trait defines the contract for all systems in the ECS.
/// Systems declare their input and output components for change tracking.
pub trait System {
    /// Components that the system will read from without modifying them
    type InputComponents;
    /// Components that the system will read from and write to
    type OutputComponents;

    /// Called once before the first update to initialize system state
    fn initialize(&mut self, world: &mut WorldView<Self::InputComponents, Self::OutputComponents>);
    
    /// Called every frame to update the system
    fn update(&mut self, world: &mut WorldView<Self::InputComponents, Self::OutputComponents>);
    
    /// Called when the system is being removed or the world is shutting down
    fn deinitialize(&mut self, world: &mut WorldView<Self::InputComponents, Self::OutputComponents>);
}

/// A wrapper for mutable component access in queries
pub struct Mut<T>(T);

/// WorldView provides controlled access to world data for systems
pub struct WorldView<InputComponents, OutputComponents> {
    world: *mut World,
    _input_phantom: std::marker::PhantomData<InputComponents>,
    _output_phantom: std::marker::PhantomData<OutputComponents>,
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
}

/// Placeholder for system initialization diff tracking
#[derive(Debug)]
pub struct SystemInitDiff {
    // Will contain details about component changes during initialization
}

impl SystemInitDiff {
    pub fn new() -> Self {
        Self {}
    }
}

/// Placeholder for system update diff tracking
#[derive(Debug)]
pub struct SystemUpdateDiff {
    // Will contain details about component changes during update
}

impl SystemUpdateDiff {
    pub fn new() -> Self {
        Self {}
    }
}

/// Placeholder for system deinitialization diff tracking
#[derive(Debug)]
pub struct SystemDeinitDiff {
    // Will contain details about component changes during deinitialization
}

impl SystemDeinitDiff {
    pub fn new() -> Self {
        Self {}
    }
}

/// Tracks overall world update changes
#[derive(Debug)]
pub struct WorldUpdateDiff {
    system_diffs: Vec<SystemUpdateDiff>,
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
        let mut world_view = WorldView::<S::InputComponents, S::OutputComponents>::new(world);
        self.system.initialize(&mut world_view);
        SystemInitDiff::new()
    }

    fn update(&mut self, world: &mut World) -> SystemUpdateDiff {
        let mut world_view = WorldView::<S::InputComponents, S::OutputComponents>::new(world);
        self.system.update(&mut world_view);
        SystemUpdateDiff::new()
    }

    fn deinitialize(&mut self, world: &mut World) -> SystemDeinitDiff {
        let mut world_view = WorldView::<S::InputComponents, S::OutputComponents>::new(world);
        self.system.deinitialize(&mut world_view);
        SystemDeinitDiff::new()
    }
}

/// The main World struct that manages entities, components, and systems
pub struct World {
    entities: Vec<Entity>,
    components: HashMap<TypeId, Vec<(Entity, Box<dyn Any>)>>,
    systems: Vec<Box<dyn SystemWrapper>>,
    next_entity_id: usize,
    child_worlds: Vec<World>,
    world_update_history: WorldUpdateHistory,
}

impl World {
    /// Creates a new empty world
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            components: HashMap::new(),
            systems: Vec::new(),
            next_entity_id: 0,
            child_worlds: Vec::new(),
            world_update_history: WorldUpdateHistory::new(),
        }
    }

    /// Add a system to the world
    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(ConcreteSystemWrapper::new(system)));
    }

    /// Create a new entity and return its identifier
    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity(self.next_entity_id);
        self.next_entity_id += 1;
        self.entities.push(entity);
        entity
    }

    /// Add a component to an entity
    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        self.components
            .entry(TypeId::of::<T>())
            .or_insert_with(Vec::new)
            .push((entity, Box::new(component)));
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
        assert_eq!(entity1, Entity(0));
        assert_eq!(world.entity_count(), 1);

        let entity2 = world.create_entity();
        assert_eq!(entity2, Entity(1));
        assert_eq!(world.entity_count(), 2);
    }

    // Example components for testing
    #[derive(Debug, PartialEq)]
    struct Position { x: f32, y: f32 }

    #[derive(Debug, PartialEq)]
    struct Velocity { dx: f32, dy: f32 }

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
        type InputComponents = ();
        type OutputComponents = ();

        fn initialize(&mut self, _world: &mut WorldView<Self::InputComponents, Self::OutputComponents>) {
            // Test system initialization
        }

        fn update(&mut self, _world: &mut WorldView<Self::InputComponents, Self::OutputComponents>) {
            // Test system update
        }

        fn deinitialize(&mut self, _world: &mut WorldView<Self::InputComponents, Self::OutputComponents>) {
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
}
