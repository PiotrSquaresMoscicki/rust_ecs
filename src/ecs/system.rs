//! System trait and system wrapper functionality
//!
//! This module defines the System trait and associated types for implementing
//! game logic in the ECS framework.

use crate::ecs::core::{ComponentChange, WorldOperation};
use crate::ecs::diff::DiffComponentChange;
use std::any::TypeId;

/// Trait to represent system dependencies
/// This is implemented for tuples of system types
pub trait SystemDependencies {
    /// Get the TypeIds of all dependent system types
    fn dependency_type_ids() -> Vec<TypeId>;
}

/// Base case: no dependencies
impl SystemDependencies for () {
    fn dependency_type_ids() -> Vec<TypeId> {
        Vec::new()
    }
}

/// Single dependency - tuple with one element
impl<A: 'static> SystemDependencies for (A,) {
    fn dependency_type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<A>()]
    }
}

/// Two dependencies
impl<A: 'static, B: 'static> SystemDependencies for (A, B) {
    fn dependency_type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>()]
    }
}

/// Three dependencies
impl<A: 'static, B: 'static, C: 'static> SystemDependencies for (A, B, C) {
    fn dependency_type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>(), TypeId::of::<C>()]
    }
}

/// Four dependencies
impl<A: 'static, B: 'static, C: 'static, D: 'static> SystemDependencies for (A, B, C, D) {
    fn dependency_type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>(), TypeId::of::<C>(), TypeId::of::<D>()]
    }
}

/// Five dependencies
impl<A: 'static, B: 'static, C: 'static, D: 'static, E: 'static> SystemDependencies for (A, B, C, D, E) {
    fn dependency_type_ids() -> Vec<TypeId> {
        vec![TypeId::of::<A>(), TypeId::of::<B>(), TypeId::of::<C>(), TypeId::of::<D>(), TypeId::of::<E>()]
    }
}

/// The System trait defines the contract for all systems in the ECS.
/// Systems declare their input and output components for change tracking.
pub trait System {
    /// Components that the system will read from without modifying them
    type InComponents;
    /// Components that the system will read from and write to
    type OutComponents;
    /// Systems that this system depends on - they will be initialized, updated, and deinitialized first
    type Dependencies: SystemDependencies;

    /// Called once before the first update to initialize system state
    fn initialize(&mut self, world: &mut crate::ecs::WorldView<Self::InComponents, Self::OutComponents>);

    /// Called every frame to update the system
    fn update(&mut self, world: &mut crate::ecs::WorldView<Self::InComponents, Self::OutComponents>);

    /// Called when the system is being removed or the world is shutting down
    fn deinitialize(&mut self, world: &mut crate::ecs::WorldView<Self::InComponents, Self::OutComponents>);
}

/// Records changes made during system initialization
#[derive(Debug, Clone)]
pub struct SystemInitDiff {
    /// Component changes made during initialization
    component_changes: Vec<ComponentChange>,
    /// World-level operations performed during initialization
    world_operations: Vec<WorldOperation>,
    /// Diff representation of component changes for replay
    diff_changes: Vec<DiffComponentChange>,
}

impl SystemInitDiff {
    pub fn new() -> Self {
        Self {
            component_changes: Vec::new(),
            world_operations: Vec::new(),
            diff_changes: Vec::new(),
        }
    }

    pub fn record_component_change(&mut self, change: DiffComponentChange) {
        self.diff_changes.push(change);
    }

    pub fn record_world_operation(&mut self, operation: WorldOperation) {
        self.world_operations.push(operation);
    }

    /// Get the component changes
    pub fn component_changes(&self) -> &[ComponentChange] {
        &self.component_changes
    }

    /// Get the world operations
    pub fn world_operations(&self) -> &[WorldOperation] {
        &self.world_operations
    }

    /// Get the diff changes
    pub fn diff_changes(&self) -> &[DiffComponentChange] {
        &self.diff_changes
    }
}

impl Default for SystemInitDiff {
    fn default() -> Self {
        Self::new()
    }
}

/// Records changes made during system update
#[derive(Debug, Clone)]
pub struct SystemUpdateDiff {
    /// Component changes made during update
    component_changes: Vec<ComponentChange>,
    /// World-level operations performed during update
    world_operations: Vec<WorldOperation>,
    /// Diff representation of component changes for replay
    diff_changes: Vec<DiffComponentChange>,
}

impl SystemUpdateDiff {
    pub fn new() -> Self {
        Self {
            component_changes: Vec::new(),
            world_operations: Vec::new(),
            diff_changes: Vec::new(),
        }
    }

    pub fn record_component_change(&mut self, change: DiffComponentChange) {
        self.diff_changes.push(change);
    }

    pub fn record_world_operation(&mut self, operation: WorldOperation) {
        self.world_operations.push(operation);
    }

    /// Get the component changes
    pub fn component_changes(&self) -> &[ComponentChange] {
        &self.component_changes
    }

    /// Get the world operations
    pub fn world_operations(&self) -> &[WorldOperation] {
        &self.world_operations
    }

    /// Get the diff changes
    pub fn diff_changes(&self) -> &[DiffComponentChange] {
        &self.diff_changes
    }
}

impl Default for SystemUpdateDiff {
    fn default() -> Self {
        Self::new()
    }
}

/// Records changes made during system deinitialization
#[derive(Debug, Clone)]
pub struct SystemDeinitDiff {
    /// Component changes made during deinitialization
    component_changes: Vec<ComponentChange>,
    /// World-level operations performed during deinitialization
    world_operations: Vec<WorldOperation>,
    /// Diff representation of component changes for replay
    diff_changes: Vec<DiffComponentChange>,
}

impl SystemDeinitDiff {
    pub fn new() -> Self {
        Self {
            component_changes: Vec::new(),
            world_operations: Vec::new(),
            diff_changes: Vec::new(),
        }
    }

    pub fn record_component_change(&mut self, change: DiffComponentChange) {
        self.diff_changes.push(change);
    }

    pub fn record_world_operation(&mut self, operation: WorldOperation) {
        self.world_operations.push(operation);
    }

    /// Get the component changes
    pub fn component_changes(&self) -> &[ComponentChange] {
        &self.component_changes
    }

    /// Get the world operations
    pub fn world_operations(&self) -> &[WorldOperation] {
        &self.world_operations
    }

    /// Get the diff changes
    pub fn diff_changes(&self) -> &[DiffComponentChange] {
        &self.diff_changes
    }
}

impl Default for SystemDeinitDiff {
    fn default() -> Self {
        Self::new()
    }
}

/// Records all changes made during a world update
#[derive(Debug, Clone)]
pub struct WorldUpdateDiff {
    /// The changes from each system that ran during this update
    system_diffs: Vec<SystemUpdateDiff>,
}

impl WorldUpdateDiff {
    pub fn new() -> Self {
        Self {
            system_diffs: Vec::new(),
        }
    }

    /// Record a system's diff
    pub fn record(&mut self, system_diff: SystemUpdateDiff) {
        self.system_diffs.push(system_diff);
    }

    /// Get all system diffs from this update
    pub fn system_diffs(&self) -> &[SystemUpdateDiff] {
        &self.system_diffs
    }
}

impl Default for WorldUpdateDiff {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete history of all world updates for replay functionality
#[derive(Debug, Clone)]
pub struct WorldUpdateHistory {
    /// All updates that have been recorded
    updates: Vec<WorldUpdateDiff>,
}

impl WorldUpdateHistory {
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
        }
    }

    /// Record a world update
    pub fn record(&mut self, update: WorldUpdateDiff) {
        self.updates.push(update);
    }

    /// Get all recorded updates
    pub fn updates(&self) -> &[WorldUpdateDiff] {
        &self.updates
    }

    /// Get the number of updates recorded
    pub fn len(&self) -> usize {
        self.updates.len()
    }

    /// Check if the history is empty
    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
    }
}

impl Default for WorldUpdateHistory {
    fn default() -> Self {
        Self::new()
    }
}