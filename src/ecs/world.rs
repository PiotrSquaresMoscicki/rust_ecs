//! World and WorldView implementation
//!
//! This module contains the main World struct that manages entities, components,
//! and systems, as well as the WorldView that provides controlled access for systems.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::marker::PhantomData;

use crate::ecs::core::{Entity, ComponentChange, ComponentOperation, WorldOperation, Out};
use crate::ecs::diff::{Diff, DiffComponent, DiffComponentChange};
use crate::ecs::system::{System, SystemUpdateDiff, SystemInitDiff, SystemDeinitDiff, WorldUpdateDiff, WorldUpdateHistory};
use crate::ecs::replay::{AutoReplayLogger, ReplayLogConfig};
use crate::ecs::query::{QueryComponent, MixedMultiQuery, MixedQueryComponent};

/// WorldView provides controlled access to world data for systems
pub struct WorldView<InComponents, OutComponents> {
    world: *mut World,
    _input_phantom: PhantomData<InComponents>,
    _output_phantom: PhantomData<OutComponents>,
    system_diff: SystemUpdateDiff,
}

impl<I, O> WorldView<I, O> {
    /// Create a new WorldView with type constraints
    pub fn new(world: &mut World) -> Self {
        Self {
            world: world as *mut World,
            _input_phantom: PhantomData,
            _output_phantom: PhantomData,
            system_diff: SystemUpdateDiff::new(),
        }
    }

    /// Get the accumulated system diff from this WorldView session
    pub fn get_system_diff(self) -> SystemUpdateDiff {
        self.system_diff
    }

    /// Get a reference to a component for an entity (immutable access)
    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
        unsafe { (*self.world).get_component::<T>(entity) }
    }

    /// Get a mutable reference to a component for an entity
    pub fn get_component_mut<T: DiffComponent>(&mut self, entity: Entity) -> Option<&mut T> {
        // Record the change for diff tracking
        let change = ComponentChange {
            entity,
            type_id: TypeId::of::<T>(),
            operation: ComponentOperation::Modify,
        };
        self.system_diff.add_component_change(change);

        unsafe { (*self.world).get_component_mut::<T>(entity) }
    }

    /// Add a component to an entity
    pub fn add_component<T: DiffComponent>(&mut self, entity: Entity, component: T) {
        let change = ComponentChange {
            entity,
            type_id: TypeId::of::<T>(),
            operation: ComponentOperation::Add,
        };
        self.system_diff.add_component_change(change);

        unsafe { (*self.world).add_component(entity, component) }
    }

    /// Remove a component from an entity
    pub fn remove_component<T: 'static>(&mut self, entity: Entity) {
        let change = ComponentChange {
            entity,
            type_id: TypeId::of::<T>(),
            operation: ComponentOperation::Remove,
        };
        self.system_diff.add_component_change(change);

        unsafe { (*self.world).remove_component::<T>(entity) }
    }

    /// Create a new entity
    pub fn create_entity(&mut self) -> Entity {
        let entity = unsafe { (*self.world).create_entity() };
        
        let operation = WorldOperation::CreateEntity(entity);
        self.system_diff.add_world_operation(operation);
        
        entity
    }

    /// Query for all entities with a specific component type
    pub fn query<T: 'static>(&self) -> Vec<(Entity, &T)> {
        unsafe { (*self.world).query::<T>() }
    }

    /// Query for all entities with a specific component type (mutable)
    pub fn query_mut<T: DiffComponent>(&mut self) -> Vec<(Entity, &mut T)> {
        // Record that we're modifying components of this type
        // Note: This is a simplified tracking - real implementation would track per-entity
        unsafe { (*self.world).query_mut::<T>() }
    }

    /// Multi-component query with mixed mutability
    pub fn multi_query<Q>(&mut self) -> Vec<(Entity, Q::Item)>
    where
        Q: MixedMultiQuery<'static>,
    {
        // This is a placeholder - real implementation would handle the complex query logic
        Vec::new()
    }
}

// Internal representation of systems
struct SystemWrapper {
    system: Box<dyn Any>,
    initialize_fn: fn(&mut dyn Any, &mut World) -> SystemInitDiff,
    update_fn: fn(&mut dyn Any, &mut World) -> SystemUpdateDiff,
    deinitialize_fn: fn(&mut dyn Any, &mut World) -> SystemDeinitDiff,
    type_name: String,
}

/// Snapshot structures for internal change tracking
#[derive(Debug, Clone)]
struct SystemComponentSnapshot {
    /// Serialized component data specific to this system
    component_data: String,
}

#[derive(Debug, Clone)]
struct SystemStateSnapshot {
    /// System state information
    frame_marker: usize,
}

/// The main ECS World that manages entities, components, and systems
pub struct World {
    /// Map from TypeId to component storage for that type
    components: HashMap<TypeId, HashMap<Entity, Box<dyn Any>>>,
    /// Next entity ID to assign
    next_entity_id: usize,
    /// Current world generation (for entity reuse safety)
    world_generation: usize,
    /// Registered systems
    systems: Vec<SystemWrapper>,
    /// Whether systems have been initialized
    systems_initialized: bool,
    /// History of all world updates for replay functionality
    update_history: WorldUpdateHistory,
    /// Optional replay logger for automatic logging
    replay_logger: Option<AutoReplayLogger>,
    /// Replay mode settings
    replay_mode: bool,
    replay_frame: usize,
}

impl World {
    /// Create a new empty world
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            next_entity_id: 0,
            world_generation: 0,
            systems: Vec::new(),
            systems_initialized: false,
            update_history: WorldUpdateHistory::new(),
            replay_logger: None,
            replay_mode: false,
            replay_frame: 0,
        }
    }

    /// Create a new entity with a unique ID
    pub fn create_entity(&mut self) -> Entity {
        let entity = Entity::new(self.next_entity_id, self.world_generation);
        self.next_entity_id += 1;
        entity
    }

    /// Add a component to an entity
    pub fn add_component<T: 'static>(&mut self, entity: Entity, component: T) {
        let type_id = TypeId::of::<T>();
        self.components
            .entry(type_id)
            .or_insert_with(HashMap::new)
            .insert(entity, Box::new(component));
    }

    /// Get a reference to a component for an entity
    pub fn get_component<T: 'static>(&self, entity: Entity) -> Option<&T> {
        self.components
            .get(&TypeId::of::<T>())
            .and_then(|components| components.get(&entity))
            .and_then(|component| component.downcast_ref::<T>())
    }

    /// Get a mutable reference to a component for an entity
    pub fn get_component_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        self.components
            .get_mut(&TypeId::of::<T>())
            .and_then(|components| components.get_mut(&entity))
            .and_then(|component| component.downcast_mut::<T>())
    }

    /// Remove a component from an entity
    pub fn remove_component<T: 'static>(&mut self, entity: Entity) {
        if let Some(components) = self.components.get_mut(&TypeId::of::<T>()) {
            components.remove(&entity);
        }
    }

    /// Check if an entity has a specific component type
    pub fn has_component<T: 'static>(&self, entity: Entity) -> bool {
        self.components
            .get(&TypeId::of::<T>())
            .map(|components| components.contains_key(&entity))
            .unwrap_or(false)
    }

    /// Remove an entity and all its components
    pub fn remove_entity(&mut self, entity: Entity) {
        for components in self.components.values_mut() {
            components.remove(&entity);
        }
    }

    /// Query for all entities with a specific component type
    pub fn query<T: 'static>(&self) -> Vec<(Entity, &T)> {
        self.components
            .get(&TypeId::of::<T>())
            .map(|components| {
                components
                    .iter()
                    .filter_map(|(entity, component)| {
                        component.downcast_ref::<T>().map(|comp| (*entity, comp))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Query for all entities with a specific component type (mutable)
    pub fn query_mut<T: 'static>(&mut self) -> Vec<(Entity, &mut T)> {
        self.components
            .get_mut(&TypeId::of::<T>())
            .map(|components| {
                components
                    .iter_mut()
                    .filter_map(|(entity, component)| {
                        component.downcast_mut::<T>().map(|comp| (*entity, comp))
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Add a system to the world
    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        let wrapper = SystemWrapper {
            system: Box::new(system),
            initialize_fn: |system_any, world| {
                let system = system_any.downcast_mut::<S>().unwrap();
                let mut world_view = WorldView::<S::InComponents, S::OutComponents>::new(world);
                system.initialize(&mut world_view);
                world_view.get_system_diff().into()
            },
            update_fn: |system_any, world| {
                let system = system_any.downcast_mut::<S>().unwrap();
                let mut world_view = WorldView::<S::InComponents, S::OutComponents>::new(world);
                system.update(&mut world_view);
                world_view.get_system_diff()
            },
            deinitialize_fn: |system_any, world| {
                let system = system_any.downcast_mut::<S>().unwrap();
                let mut world_view = WorldView::<S::InComponents, S::OutComponents>::new(world);
                system.deinitialize(&mut world_view);
                world_view.get_system_diff().into()
            },
            type_name: std::any::type_name::<S>().to_string(),
        };

        self.systems.push(wrapper);
    }

    /// Initialize all systems
    pub fn initialize_systems(&mut self) {
        if self.systems_initialized {
            return;
        }

        let mut world_update_diff = WorldUpdateDiff::new();

        for system in &mut self.systems {
            let init_diff = (system.initialize_fn)(&mut *system.system, self);
            // Convert SystemInitDiff to SystemUpdateDiff for consistency
            let mut update_diff = SystemUpdateDiff::new();
            for change in init_diff.component_changes() {
                update_diff.add_component_change(change.clone());
            }
            for operation in init_diff.world_operations() {
                update_diff.add_world_operation(operation.clone());
            }
            for diff_change in init_diff.diff_changes() {
                update_diff.add_diff_change(diff_change.clone());
            }
            world_update_diff.add_system_diff(update_diff);
        }

        self.update_history.add_update(world_update_diff);
        self.systems_initialized = true;
    }

    /// Update all systems
    pub fn update(&mut self) {
        if !self.systems_initialized {
            self.initialize_systems();
        }

        let mut world_update_diff = WorldUpdateDiff::new();

        for system in &mut self.systems {
            let system_diff = (system.update_fn)(&mut *system.system, self);
            world_update_diff.add_system_diff(system_diff);
        }

        // Log the update if replay logging is enabled
        if let Some(ref mut logger) = self.replay_logger {
            let _ = logger.log_update(&world_update_diff);
        }

        self.update_history.add_update(world_update_diff);
    }

    /// Get the number of entities in the world
    pub fn entity_count(&self) -> usize {
        // Count unique entities across all component types
        let mut entities = std::collections::HashSet::new();
        for components in self.components.values() {
            for entity in components.keys() {
                entities.insert(*entity);
            }
        }
        entities.len()
    }

    /// Get all entities that have a specific component type
    pub fn entities_with_component<T: 'static>(&self) -> Vec<Entity> {
        self.components
            .get(&TypeId::of::<T>())
            .map(|components| components.keys().copied().collect())
            .unwrap_or_default()
    }

    /// Get the update history for replay functionality
    pub fn get_update_history(&self) -> &WorldUpdateHistory {
        &self.update_history
    }

    /// Enable replay mode
    pub fn enable_replay_mode(&mut self) {
        self.replay_mode = true;
        self.replay_frame = 0;
    }

    /// Disable replay mode
    pub fn disable_replay_mode(&mut self) {
        self.replay_mode = false;
    }

    /// Check if replay mode is enabled
    pub fn is_replay_mode_enabled(&self) -> bool {
        self.replay_mode
    }

    /// Get the current replay frame number
    pub fn get_replay_frame(&self) -> usize {
        self.replay_frame
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

    /// Replay a world history to create a new world with the same state
    pub fn replay_history(history: &WorldUpdateHistory) -> World {
        let mut world = World::new();
        
        // Apply each update in the history
        for _update in history.updates() {
            // This would need to be implemented to actually apply the changes
            // For now, just create an empty world
        }
        
        world
    }

    /// Parse a replay log file and return the parsed history
    pub fn parse_and_replay_log(file_path: &str) -> Result<World, Box<dyn std::error::Error>> {
        let _history = crate::ecs::replay::analysis::parse_replay_log(file_path)?;
        
        // For now, return a new empty world
        // A full implementation would replay the parsed history
        Ok(World::new())
    }
}

impl SystemInitDiff {
    /// Convert SystemInitDiff to SystemUpdateDiff for consistency
    pub fn into(self) -> SystemUpdateDiff {
        let mut update_diff = SystemUpdateDiff::new();
        for change in self.component_changes() {
            update_diff.add_component_change(change.clone());
        }
        for operation in self.world_operations() {
            update_diff.add_world_operation(operation.clone());
        }
        for diff_change in self.diff_changes() {
            update_diff.add_diff_change(diff_change.clone());
        }
        update_diff
    }
}

impl SystemDeinitDiff {
    /// Convert SystemDeinitDiff to SystemUpdateDiff for consistency
    pub fn into(self) -> SystemUpdateDiff {
        let mut update_diff = SystemUpdateDiff::new();
        for change in self.component_changes() {
            update_diff.add_component_change(change.clone());
        }
        for operation in self.world_operations() {
            update_diff.add_world_operation(operation.clone());
        }
        for diff_change in self.diff_changes() {
            update_diff.add_diff_change(diff_change.clone());
        }
        update_diff
    }
}