//! System trait and system wrapper functionality
//!
//! This module defines the System trait and associated types for implementing
//! game logic in the ECS framework.

use crate::ecs::core::{ComponentChange, WorldOperation};
use crate::ecs::diff::DiffComponentChange;

/// The System trait defines the contract for all systems in the ECS.
/// Systems declare their input and output components for change tracking.
pub trait System {
    /// Components that the system will read from without modifying them
    type InComponents;
    /// Components that the system will read from and write to
    type OutComponents;

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
    /// Create a new empty system initialization diff
    pub fn new() -> Self {
        Self {
            component_changes: Vec::new(),
            world_operations: Vec::new(),
            diff_changes: Vec::new(),
        }
    }

    /// Add a component change to this diff
    pub fn add_component_change(&mut self, change: ComponentChange) {
        self.component_changes.push(change);
    }

    /// Add a world operation to this diff
    pub fn add_world_operation(&mut self, operation: WorldOperation) {
        self.world_operations.push(operation);
    }

    /// Add a diff component change to this diff
    pub fn add_diff_change(&mut self, change: DiffComponentChange) {
        self.diff_changes.push(change);
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
    /// Create a new empty system update diff
    pub fn new() -> Self {
        Self {
            component_changes: Vec::new(),
            world_operations: Vec::new(),
            diff_changes: Vec::new(),
        }
    }

    /// Add a component change to this diff
    pub fn add_component_change(&mut self, change: ComponentChange) {
        self.component_changes.push(change);
    }

    /// Add a world operation to this diff
    pub fn add_world_operation(&mut self, operation: WorldOperation) {
        self.world_operations.push(operation);
    }

    /// Add a diff component change to this diff
    pub fn add_diff_change(&mut self, change: DiffComponentChange) {
        self.diff_changes.push(change);
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
    /// Create a new empty system deinitialization diff
    pub fn new() -> Self {
        Self {
            component_changes: Vec::new(),
            world_operations: Vec::new(),
            diff_changes: Vec::new(),
        }
    }

    /// Add a component change to this diff
    pub fn add_component_change(&mut self, change: ComponentChange) {
        self.component_changes.push(change);
    }

    /// Add a world operation to this diff
    pub fn add_world_operation(&mut self, operation: WorldOperation) {
        self.world_operations.push(operation);
    }

    /// Add a diff component change to this diff
    pub fn add_diff_change(&mut self, change: DiffComponentChange) {
        self.diff_changes.push(change);
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

/// Records all changes made during a world update
#[derive(Debug, Clone)]
pub struct WorldUpdateDiff {
    /// System-specific changes that occurred during this update
    system_diffs: Vec<SystemUpdateDiff>,
}

impl WorldUpdateDiff {
    /// Create a new empty world update diff
    pub fn new() -> Self {
        Self {
            system_diffs: Vec::new(),
        }
    }

    /// Add a system update diff to this world update diff
    pub fn add_system_diff(&mut self, diff: SystemUpdateDiff) {
        self.system_diffs.push(diff);
    }

    /// Get the system diffs
    pub fn system_diffs(&self) -> &[SystemUpdateDiff] {
        &self.system_diffs
    }
}

/// Complete history of all world updates for replay functionality
#[derive(Debug, Clone)]
pub struct WorldUpdateHistory {
    /// All update diffs recorded
    updates: Vec<WorldUpdateDiff>,
}

impl WorldUpdateHistory {
    /// Create a new empty update history
    pub fn new() -> Self {
        Self {
            updates: Vec::new(),
        }
    }

    /// Add an update diff to the history
    pub fn add_update(&mut self, diff: WorldUpdateDiff) {
        self.updates.push(diff);
    }

    /// Get all updates
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