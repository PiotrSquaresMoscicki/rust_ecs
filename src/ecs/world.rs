//! World and WorldView implementations
//!
//! This module contains the main World struct that manages entities, components,
//! and systems, along with the WorldView that provides controlled access for systems.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::ecs::core::{Entity, WorldOperation};
use crate::ecs::diff::{Diff, DiffComponentChange};
use crate::ecs::system::{System, SystemDependencies, SystemInitDiff, SystemUpdateDiff, SystemDeinitDiff, WorldUpdateDiff, WorldUpdateHistory};
use crate::ecs::query::{MixedMultiQuery};
use crate::ecs::replay::{ReplayLogConfig, AutoReplayLogger};

/// Main ECS world container
pub struct World {
    /// Index of this world (for multi-world support)
    world_index: usize,
    /// All entities in this world
    pub(crate) entities: Vec<Entity>,
    /// Component storage, organized by type
    pub(crate) components: HashMap<TypeId, Vec<(Entity, Box<dyn Any>)>>,
    /// Systems registered to this world
    systems: Vec<Box<dyn SystemWrapper>>,
    /// Next entity ID to assign
    next_entity_id: usize,
    /// Child worlds for hierarchical organization
    child_worlds: Vec<World>,
    /// Complete history of world updates for replay functionality
    world_update_history: WorldUpdateHistory,
    /// Next world index for child worlds
    next_world_index: usize,
    /// Automatic replay logger for debugging and analysis
    replay_logger: Option<AutoReplayLogger>,
    /// Replay mode tracking for system-level snapshot/restore
    replay_mode: bool,
    /// Current frame number in replay mode
    replay_frame: usize,
    /// Replay data for use during replay mode
    replay_data: Option<WorldUpdateHistory>,
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
            replay_logger: None,
            replay_mode: false,
            replay_frame: 0,
            replay_data: None,
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

        // Record world creation operation
        let mut world_diff = WorldUpdateDiff::new();
        let mut system_diff = SystemUpdateDiff::new();
        system_diff.record_world_operation(WorldOperation::CreateWorld(child_world_index));
        world_diff.record(system_diff);
        self.world_update_history.record(world_diff);

        self.child_worlds.push(child_world);
        child_world_index
    }

    /// Remove a child world by index
    pub fn remove_child_world(&mut self, world_index: usize) -> Option<World> {
        if let Some(pos) = self
            .child_worlds
            .iter()
            .position(|w| w.world_index == world_index)
        {
            let removed_world = self.child_worlds.remove(pos);

            // Record world removal operation
            let mut world_diff = WorldUpdateDiff::new();
            let mut system_diff = SystemUpdateDiff::new();
            system_diff.record_world_operation(WorldOperation::RemoveWorld(world_index));
            world_diff.record(system_diff);
            self.world_update_history.record(world_diff);

            Some(removed_world)
        } else {
            None
        }
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
        let system_type_name = std::any::type_name::<S>().to_string();
        
        // Record the system addition operation in world update history
        let mut world_diff = WorldUpdateDiff::new();
        let mut system_diff = SystemUpdateDiff::new();
        system_diff.record_world_operation(WorldOperation::AddSystem(system_type_name));
        world_diff.record(system_diff);
        self.world_update_history.record(world_diff);
        
        // Add the system to the world
        self.add_system_internal(system);
    }

    /// Internal method to add a system without recording (for replay)
    pub fn add_system_internal<S: System + 'static>(&mut self, system: S) {
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

    /// Remove a component from an entity
    pub fn remove_component<T: 'static>(&mut self, entity: Entity) -> Option<T> {
        if let Some(components) = self.components.get_mut(&TypeId::of::<T>()) {
            if let Some(pos) = components.iter().position(|(e, _)| *e == entity) {
                let (_, component_box) = components.remove(pos);
                return component_box.downcast::<T>().ok().map(|boxed| *boxed);
            }
        }
        None
    }

    /// Remove an entity and all its components
    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        let initial_count = self.entities.len();

        // Remove from entities list
        self.entities.retain(|e| *e != entity);

        // Remove all components belonging to this entity
        for components in self.components.values_mut() {
            components.retain(|(e, _)| *e != entity);
        }

        // Return whether entity was actually removed
        self.entities.len() < initial_count
    }

    /// Check if an entity exists
    pub fn entity_exists(&self, entity: Entity) -> bool {
        self.entities.contains(&entity)
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

    /// Sort systems according to their dependencies using topological sort
    /// Returns the indices of systems in dependency order, or Err if there are circular dependencies
    fn sort_systems_by_dependencies(&self) -> Result<Vec<usize>, String> {
        let n = self.systems.len();
        let mut in_degree = vec![0; n];
        let mut adj_list: Vec<Vec<usize>> = vec![Vec::new(); n];
        
        // Build adjacency list and calculate in-degrees
        for i in 0..n {
            let dependencies = self.systems[i].dependency_type_ids();
            for dep_type_id in dependencies {
                // Find the system with the matching type
                if let Some(dep_index) = self.systems.iter().position(|s| s.system_type_id() == dep_type_id) {
                    adj_list[dep_index].push(i);
                    in_degree[i] += 1;
                } else {
                    return Err(format!("Dependency not found: {:?}", dep_type_id));
                }
            }
        }
        
        // Topological sort using Kahn's algorithm
        let mut queue = Vec::new();
        let mut result = Vec::new();
        
        // Add all nodes with no incoming edges
        for i in 0..n {
            if in_degree[i] == 0 {
                queue.push(i);
            }
        }
        
        while let Some(current) = queue.pop() {
            result.push(current);
            
            // For each neighbor of current
            for neighbor in &adj_list[current] {
                in_degree[*neighbor] -= 1;
                if in_degree[*neighbor] == 0 {
                    queue.push(*neighbor);
                }
            }
        }
        
        // Check for circular dependencies
        if result.len() != n {
            return Err("Circular dependency detected".to_string());
        }
        
        Ok(result)
    }

    /// Initialize all systems (called once before the first update)
    pub fn initialize_systems(&mut self) {
        // Sort systems by dependencies
        let sorted_indices = match self.sort_systems_by_dependencies() {
            Ok(indices) => indices,
            Err(err) => {
                eprintln!("Warning: Failed to resolve system dependencies: {}. Initializing in registration order.", err);
                (0..self.systems.len()).collect()
            }
        };

        // We need to work around the borrowing issue by taking ownership temporarily
        let mut systems = std::mem::take(&mut self.systems);

        // Initialize systems in dependency order
        for &index in &sorted_indices {
            let _diff = systems[index].initialize(self);
            // TODO: Record diff in world update history
        }

        self.systems = systems;
    }

    /// Update all systems for one frame
    pub fn update(&mut self) {
        let mut world_update_diff = WorldUpdateDiff::new();

        // Sort systems by dependencies
        let sorted_indices = match self.sort_systems_by_dependencies() {
            Ok(indices) => indices,
            Err(err) => {
                eprintln!("Warning: Failed to resolve system dependencies: {}. Updating in registration order.", err);
                (0..self.systems.len()).collect()
            }
        };

        // We need to work around the borrowing issue by taking ownership temporarily
        let mut systems = std::mem::take(&mut self.systems);

        // Update systems in dependency order
        for &index in &sorted_indices {
            let system_diff = if self.replay_mode {
                // In replay mode, use system-level snapshot/restore
                systems[index].update_with_replay(self, self.replay_frame)
            } else {
                // In normal mode, just update normally
                systems[index].update(self)
            };
            world_update_diff.record(system_diff);
        }

        self.systems = systems;
        
        // Increment replay frame if in replay mode
        if self.replay_mode {
            self.replay_frame += 1;
        }
        
        // Record the update in history
        self.world_update_history.record(world_update_diff.clone());
        
        // Log the update if replay logging is enabled
        if let Some(ref mut logger) = self.replay_logger {
            if let Err(e) = logger.log_update(&world_update_diff) {
                eprintln!("Failed to log replay data: {}", e);
            }
        }
    }
    
    /// Deinitialize all systems (called when shutting down)
    /// Systems are deinitialized in reverse dependency order
    pub fn deinitialize_systems(&mut self) {
        // Sort systems by dependencies, then reverse for deinitialization
        let sorted_indices = match self.sort_systems_by_dependencies() {
            Ok(mut indices) => {
                indices.reverse(); // Deinitialize in reverse order
                indices
            },
            Err(err) => {
                eprintln!("Warning: Failed to resolve system dependencies: {}. Deinitializing in reverse registration order.", err);
                (0..self.systems.len()).rev().collect()
            }
        };

        // We need to work around the borrowing issue by taking ownership temporarily
        let mut systems = std::mem::take(&mut self.systems);

        // Deinitialize systems in reverse dependency order
        for &index in &sorted_indices {
            let _diff = systems[index].deinitialize(self);
            // TODO: Record diff in world update history if needed
        }

        self.systems = systems;
    }

    /// Enable replay mode for this world
    pub fn enable_replay_mode(&mut self) {
        self.replay_mode = true;
        self.replay_frame = 0;
        // Replay mode enabled - systems will use snapshot/restore pattern for deterministic replay
    }

    /// Set replay data for this world and enable replay mode
    pub fn set_replay_data(&mut self, replay_data: WorldUpdateHistory) {
        self.replay_data = Some(replay_data);
        self.enable_replay_mode();
    }

    /// Disable replay mode for this world
    pub fn disable_replay_mode(&mut self) {
        self.replay_mode = false;
        self.replay_frame = 0;
        self.replay_data = None;
        // Replay mode disabled - systems will run normally
    }

    /// Check if replay mode is enabled
    pub fn is_replay_mode_enabled(&self) -> bool {
        self.replay_mode
    }

    /// Get the current replay frame number
    pub fn get_replay_frame(&self) -> usize {
        self.replay_frame
    }

    /// Get the number of entities in the world
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Replay a world history to create a new world with the same state
    pub fn replay_history(history: &WorldUpdateHistory) -> World {
        let world = World::new();

        // For now, return an empty world - full replay implementation would require
        // more sophisticated state tracking and component serialization
        println!(
            "Replaying world history with {} updates",
            history.updates().len()
        );
        for (i, _update) in history.updates().iter().enumerate() {
            println!("Frame {}: Applying update", i + 1);
            // Would apply each update to reconstruct the world state
        }

        world
    }

    /// Get the update history for replay functionality
    pub fn get_update_history(&self) -> &WorldUpdateHistory {
        &self.world_update_history
    }

    /// Enable replay logging with the given configuration
    pub fn enable_replay_logging(&mut self, config: ReplayLogConfig) -> Result<(), std::io::Error> {
        let mut logger = AutoReplayLogger::new(config);
        logger.initialize()?;
        self.replay_logger = Some(logger);
        Ok(())
    }

    /// Enable replay logging with basic parameters (convenience method)
    pub fn enable_replay_logging_simple(
        &mut self, 
        log_directory: &str, 
        file_prefix: &str, 
        flush_interval: usize
    ) -> Result<(), std::io::Error> {
        let config = ReplayLogConfig {
            enabled: true,
            log_directory: log_directory.to_string(),
            file_prefix: file_prefix.to_string(),
            flush_interval,
            include_component_details: true,
        };
        self.enable_replay_logging(config)
    }

    /// Disable replay logging and finalize the current log file
    pub fn disable_replay_logging(&mut self) -> Result<(), std::io::Error> {
        if let Some(mut logger) = self.replay_logger.take() {
            logger.finalize()?;
        }
        Ok(())
    }

    /// Check if replay logging is enabled
    pub fn is_replay_logging_enabled(&self) -> bool {
        self.replay_logger.is_some()
    }

    /// Get the current replay logger session ID (if logging is enabled)
    pub fn replay_session_id(&self) -> Option<&str> {
        self.replay_logger.as_ref().map(|logger| logger.session_id())
    }

    /// Get the current replay logger update count (if logging is enabled)
    pub fn replay_update_count(&self) -> Option<usize> {
        self.replay_logger.as_ref().map(|logger| logger.update_count())
    }

    /// Parse a replay log file and return the parsed history
    pub fn parse_replay_log_file(file_path: &str) -> Result<WorldUpdateHistory, Box<dyn std::error::Error>> {
        crate::ecs::replay::parse_replay_log(file_path)
    }

    /// Get all entities that have a specific component type
    pub fn entities_with_component<T: 'static>(&self) -> Vec<Entity> {
        self.components
            .get(&TypeId::of::<T>())
            .map(|components| components.iter().map(|(entity, _)| *entity).collect())
            .unwrap_or_default()
    }
}

/// WorldView provides controlled access to world data for systems
pub struct WorldView<InComponents, OutComponents> {
    world: *mut World,
    _input_phantom: std::marker::PhantomData<InComponents>,
    _output_phantom: std::marker::PhantomData<OutComponents>,
    system_diff: SystemUpdateDiff,
}

impl<I, O> WorldView<I, O> {
    /// Create a new WorldView with type constraints
    pub fn new(world: &mut World) -> Self {
        Self {
            world: world as *mut World,
            _input_phantom: std::marker::PhantomData,
            _output_phantom: std::marker::PhantomData,
            system_diff: SystemUpdateDiff::new(),
        }
    }

    /// Get the accumulated system diff from this WorldView session
    pub fn get_system_diff(self) -> SystemUpdateDiff {
        self.system_diff
    }

    /// Record a component modification (call this when you modify a component)
    pub fn record_component_modification<T: Diff + Clone + std::fmt::Debug + 'static>(
        &mut self, 
        entity: Entity, 
        old_value: &T, 
        new_value: &T
    ) {
        if let Some(diff) = old_value.diff(new_value) {
            let diff_str = T::diff_to_string(&diff);
            let type_name = std::any::type_name::<T>().split("::").last().unwrap_or(std::any::type_name::<T>());
            
            let change = DiffComponentChange::Modified {
                entity,
                type_name: type_name.to_string(),
                diff: diff_str,
            };
            
            self.system_diff.record_component_change(change);
        }
    }

    /// Record a component addition
    pub fn record_component_addition<T: std::fmt::Debug + 'static>(
        &mut self, 
        entity: Entity, 
        component: &T
    ) {
        let type_name = std::any::type_name::<T>().split("::").last().unwrap_or(std::any::type_name::<T>());
        let data = format!("{:?}", component);
        
        let change = DiffComponentChange::Added {
            entity,
            type_name: type_name.to_string(),
            data,
        };
        
        self.system_diff.record_component_change(change);
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

    /// Remove a component from an entity
    pub fn remove_component<T: 'static>(&mut self, entity: Entity) -> Option<T> {
        unsafe { self.world_mut().remove_component(entity) }
    }

    /// Remove an entity and all its components
    pub fn remove_entity(&mut self, entity: Entity) -> bool {
        unsafe { self.world_mut().remove_entity(entity) }
    }

    /// Query entities with multiple components, using Out<T> for mutable access and In<T> for immutable access
    /// Example: world_view.query_components::<(In<Position>, Out<Velocity>)>()
    pub fn query_components<Q>(&mut self) -> Vec<(Entity, <Q as MixedMultiQuery<'_>>::Item)>
    where
        for<'a> Q: MixedMultiQuery<'a>,
    {
        // Get the query results
        let results = unsafe { Q::query_mixed(self.world_mut()) };
        
        // For now, return results directly without tracking
        // TODO: Implement automatic change tracking
        results
    }
}

/// Type-erased system wrapper for storage in World
trait SystemWrapper {
    fn initialize(&mut self, world: &mut World) -> SystemInitDiff;
    fn update(&mut self, world: &mut World) -> SystemUpdateDiff;
    fn update_with_replay(&mut self, world: &mut World, frame_number: usize) -> SystemUpdateDiff;
    #[allow(dead_code)]
    fn deinitialize(&mut self, world: &mut World) -> SystemDeinitDiff;
    /// Get the TypeId of this system
    fn system_type_id(&self) -> TypeId;
    /// Get the TypeIds of systems this system depends on
    fn dependency_type_ids(&self) -> Vec<TypeId>;
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

impl<S: System + 'static> SystemWrapper for ConcreteSystemWrapper<S> {
    fn initialize(&mut self, world: &mut World) -> SystemInitDiff {
        let mut world_view = WorldView::new(world);
        self.system.initialize(&mut world_view);
        
        // Convert SystemUpdateDiff to SystemInitDiff
        let update_diff = world_view.get_system_diff();
        let mut init_diff = SystemInitDiff::new();
        for change in update_diff.diff_changes() {
            init_diff.record_component_change(change.clone());
        }
        for operation in update_diff.world_operations() {
            init_diff.record_world_operation(operation.clone());
        }
        
        init_diff
    }

    fn update(&mut self, world: &mut World) -> SystemUpdateDiff {
        let mut world_view = WorldView::new(world);
        self.system.update(&mut world_view);
        world_view.get_system_diff()
    }

    fn update_with_replay(&mut self, world: &mut World, _frame_number: usize) -> SystemUpdateDiff {
        // For now, just call normal update
        // In a full implementation, this would apply replay diffs
        self.update(world)
    }

    fn deinitialize(&mut self, world: &mut World) -> SystemDeinitDiff {
        let mut world_view = WorldView::new(world);
        self.system.deinitialize(&mut world_view);
        
        // Convert SystemUpdateDiff to SystemDeinitDiff
        let update_diff = world_view.get_system_diff();
        let mut deinit_diff = SystemDeinitDiff::new();
        for change in update_diff.diff_changes() {
            deinit_diff.record_component_change(change.clone());
        }
        for operation in update_diff.world_operations() {
            deinit_diff.record_world_operation(operation.clone());
        }
        
        deinit_diff
    }

    fn system_type_id(&self) -> TypeId {
        TypeId::of::<S>()
    }

    fn dependency_type_ids(&self) -> Vec<TypeId> {
        S::Dependencies::dependency_type_ids()
    }
}