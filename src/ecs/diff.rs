//! Diff trait and related functionality for change tracking
//!
//! This module provides the diffing system that enables the ECS to track
//! changes between component states for replay functionality.

use std::collections::HashMap;
use crate::ecs::core::Entity;

/// Trait for types that can be diffed to track changes
pub trait Diff {
    /// The type representing the diff between two instances
    type Diff: Clone + std::fmt::Debug;

    /// Compute the difference between self and other
    /// Returns None if there are no differences
    fn diff(&self, other: &Self) -> Option<Self::Diff>;

    /// Apply a diff to self to get the new state
    fn apply_diff(&mut self, diff: &Self::Diff);

    /// Convert the diff to a human-readable string representation
    fn diff_to_string(diff: &Self::Diff) -> String {
        format!("{:?}", diff)
    }
}

// Implement Diff for primitive types
impl Diff for i32 {
    type Diff = i32;

    fn diff(&self, other: &Self) -> Option<Self::Diff> {
        if self != other {
            Some(*other)
        } else {
            None
        }
    }

    fn apply_diff(&mut self, diff: &Self::Diff) {
        *self = *diff;
    }
}

impl DiffComponent for i32 {}

impl Diff for f32 {
    type Diff = f32;

    fn diff(&self, other: &Self) -> Option<Self::Diff> {
        if (self - other).abs() > f32::EPSILON {
            Some(*other)
        } else {
            None
        }
    }

    fn apply_diff(&mut self, diff: &Self::Diff) {
        *self = *diff;
    }
}

impl DiffComponent for f32 {}

impl Diff for usize {
    type Diff = usize;

    fn diff(&self, other: &Self) -> Option<Self::Diff> {
        if self != other {
            Some(*other)
        } else {
            None
        }
    }

    fn apply_diff(&mut self, diff: &Self::Diff) {
        *self = *diff;
    }
}

impl DiffComponent for usize {}

impl Diff for u32 {
    type Diff = u32;

    fn diff(&self, other: &Self) -> Option<Self::Diff> {
        if self != other {
            Some(*other)
        } else {
            None
        }
    }

    fn apply_diff(&mut self, diff: &Self::Diff) {
        *self = *diff;
    }
}

impl DiffComponent for u32 {}

impl Diff for bool {
    type Diff = bool;

    fn diff(&self, other: &Self) -> Option<Self::Diff> {
        if self != other {
            Some(*other)
        } else {
            None
        }
    }

    fn apply_diff(&mut self, diff: &Self::Diff) {
        *self = *diff;
    }
}

impl DiffComponent for bool {}

impl Diff for (i32, i32) {
    type Diff = (Option<i32>, Option<i32>);

    fn diff(&self, other: &Self) -> Option<Self::Diff> {
        let mut has_changes = false;
        let diff = (
            if self.0 != other.0 {
                has_changes = true;
                Some(other.0)
            } else {
                None
            },
            if self.1 != other.1 {
                has_changes = true;
                Some(other.1)
            } else {
                None
            },
        );

        if has_changes {
            Some(diff)
        } else {
            None
        }
    }

    fn apply_diff(&mut self, diff: &Self::Diff) {
        if let Some(x) = diff.0 {
            self.0 = x;
        }
        if let Some(y) = diff.1 {
            self.1 = y;
        }
    }
}

impl DiffComponent for (i32, i32) {}

impl Diff for String {
    type Diff = String;

    fn diff(&self, other: &Self) -> Option<Self::Diff> {
        if self != other {
            Some(other.clone())
        } else {
            None
        }
    }

    fn apply_diff(&mut self, diff: &Self::Diff) {
        *self = diff.clone();
    }
}

impl DiffComponent for String {}

impl<T: Diff + Clone + std::fmt::Debug> Diff for Vec<T> {
    type Diff = VecDiff<T>;

    fn diff(&self, other: &Self) -> Option<Self::Diff> {
        let mut changes = Vec::new();
        let max_len = self.len().max(other.len());
        let mut has_changes = false;

        for i in 0..max_len {
            match (self.get(i), other.get(i)) {
                (Some(a), Some(b)) => {
                    if let Some(item_diff) = a.diff(b) {
                        changes.push(VecChange::Modified {
                            index: i,
                            diff: item_diff,
                        });
                        has_changes = true;
                    }
                }
                (Some(_), None) => {
                    changes.push(VecChange::Removed { index: i });
                    has_changes = true;
                }
                (None, Some(b)) => {
                    changes.push(VecChange::Added {
                        index: i,
                        value: b.clone(),
                    });
                    has_changes = true;
                }
                (None, None) => unreachable!(),
            }
        }

        if has_changes {
            Some(VecDiff { changes })
        } else {
            None
        }
    }

    fn apply_diff(&mut self, diff: &Self::Diff) {
        // Sort changes by index in reverse order to handle removals correctly
        let mut sorted_changes = diff.changes.clone();
        sorted_changes.sort_by_key(|b| std::cmp::Reverse(b.index()));

        for change in sorted_changes {
            match change {
                VecChange::Added { index, value } => {
                    if index <= self.len() {
                        self.insert(index, value);
                    } else {
                        self.push(value);
                    }
                }
                VecChange::Removed { index } => {
                    if index < self.len() {
                        self.remove(index);
                    }
                }
                VecChange::Modified { index, diff } => {
                    if let Some(item) = self.get_mut(index) {
                        item.apply_diff(&diff);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct VecDiff<T: Diff + std::fmt::Debug> {
    pub changes: Vec<VecChange<T>>,
}

#[derive(Clone, Debug)]
pub enum VecChange<T: Diff + std::fmt::Debug> {
    Added { index: usize, value: T },
    Removed { index: usize },
    Modified { index: usize, diff: T::Diff },
}

impl<T: Diff + std::fmt::Debug> VecChange<T> {
    fn index(&self) -> usize {
        match self {
            VecChange::Added { index, .. } => *index,
            VecChange::Removed { index } => *index,
            VecChange::Modified { index, .. } => *index,
        }
    }
}

impl<
        K: Clone + std::cmp::Eq + std::hash::Hash + std::fmt::Debug,
        V: Diff + Clone + std::fmt::Debug,
    > Diff for HashMap<K, V>
{
    type Diff = HashMapDiff<K, V>;

    fn diff(&self, other: &Self) -> Option<Self::Diff> {
        let mut changes = HashMap::new();
        let mut has_changes = false;

        // Check for added and modified entries
        for (key, other_value) in other {
            match self.get(key) {
                Some(self_value) => {
                    if let Some(value_diff) = self_value.diff(other_value) {
                        changes.insert(key.clone(), HashMapChange::Modified(value_diff));
                        has_changes = true;
                    }
                }
                None => {
                    changes.insert(key.clone(), HashMapChange::Added(other_value.clone()));
                    has_changes = true;
                }
            }
        }

        // Check for removed entries
        for key in self.keys() {
            if !other.contains_key(key) {
                changes.insert(key.clone(), HashMapChange::Removed);
                has_changes = true;
            }
        }

        if has_changes {
            Some(HashMapDiff { changes })
        } else {
            None
        }
    }

    fn apply_diff(&mut self, diff: &Self::Diff) {
        for (key, change) in &diff.changes {
            match change {
                HashMapChange::Added(value) => {
                    self.insert(key.clone(), value.clone());
                }
                HashMapChange::Removed => {
                    self.remove(key);
                }
                HashMapChange::Modified(value_diff) => {
                    if let Some(existing_value) = self.get_mut(key) {
                        existing_value.apply_diff(value_diff);
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct HashMapDiff<K: std::fmt::Debug, V: Diff + std::fmt::Debug> {
    pub changes: HashMap<K, HashMapChange<V>>,
}

#[derive(Clone, Debug)]
pub enum HashMapChange<V: Diff + std::fmt::Debug> {
    Added(V),
    Removed,
    Modified(V::Diff),
}

/// Enhanced component change operations for better tracking
#[derive(Debug, Clone)]
pub enum DiffComponentChange {
    Added {
        entity: Entity,
        type_name: String,
        data: String,
    },
    Modified {
        entity: Entity,
        type_name: String,
        diff: String,
    },
    Removed {
        entity: Entity,
        type_name: String,
    },
}

/// Trait for components that can be tracked in the diff change system
pub trait DiffComponent: Diff + std::fmt::Debug + 'static {
    /// Serialize the component to a string representation
    fn serialize(&self) -> String {
        format!("{:?}", self)
    }

    /// Get the type name for this component
    fn type_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

// Implement Diff for Entity (defined in core but implemented here)
impl Diff for Entity {
    type Diff = EntityDiff;

    fn diff(&self, other: &Self) -> Option<Self::Diff> {
        let mut has_changes = false;
        let diff = EntityDiff {
            world_index: if self.world_index != other.world_index {
                has_changes = true;
                Some(other.world_index)
            } else {
                None
            },
            entity_index: if self.entity_index != other.entity_index {
                has_changes = true;
                Some(other.entity_index)
            } else {
                None
            },
        };

        if has_changes {
            Some(diff)
        } else {
            None
        }
    }

    fn apply_diff(&mut self, diff: &Self::Diff) {
        if let Some(world_index) = diff.world_index {
            self.world_index = world_index;
        }
        if let Some(entity_index) = diff.entity_index {
            self.entity_index = entity_index;
        }
    }
}

#[derive(Clone, Debug)]
pub struct EntityDiff {
    pub world_index: Option<usize>,
    pub entity_index: Option<usize>,
}

impl DiffComponent for Entity {}