//! Change tracking and diff functionality
//!
//! This module provides the core change tracking capabilities that enable
//! the ECS framework's replay and debugging features.

use std::collections::HashMap;
use std::hash::Hash;
use serde::{Serialize, Deserialize};

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

/// Extended trait for types that support binary diff serialization
/// This is a simplified version that works with the existing diff system
pub trait BinaryDiff: Diff {
    /// Convert the diff to binary format for efficient storage
    /// Default implementation tries to serialize the diff if it supports serde
    fn diff_to_binary(diff: &Self::Diff) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Err("Binary serialization not implemented for this type".into())
    }

    /// Create a diff from binary format
    /// Default implementation returns an error
    fn diff_from_binary(_data: &[u8]) -> Result<Self::Diff, Box<dyn std::error::Error>> {
        Err("Binary deserialization not implemented for this type".into())
    }
}

/// Marker trait for component types that can be diffed
pub trait DiffComponent: Diff + std::fmt::Debug + 'static {
    /// Get the type name as a string for serialization/debugging
    fn type_name() -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Enumeration of component changes that can be recorded in replay logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiffComponentChange {
    /// A component was added to an entity
    Added {
        entity: Entity,
        type_name: String,
        diff_string: String,
    },
    /// A component was modified on an entity
    Modified {
        entity: Entity,
        type_name: String,
        diff_string: String,
    },
    /// A component was removed from an entity
    Removed {
        entity: Entity,
        type_name: String,
    },
}

/// Binary-optimized enumeration of component changes for high-performance recording
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryDiffComponentChange {
    /// A component was added to an entity
    Added {
        entity: Entity,
        type_name: String,
        diff_data: Vec<u8>,
    },
    /// A component was modified on an entity
    Modified {
        entity: Entity,
        type_name: String,
        diff_data: Vec<u8>,
    },
    /// A component was removed from an entity
    Removed {
        entity: Entity,
        type_name: String,
    },
}

impl BinaryDiffComponentChange {
    /// Convert from regular DiffComponentChange to binary format with raw diff data
    pub fn from_diff_change_raw(
        entity: Entity,
        type_name: String,
        diff_data: Vec<u8>,
        change_type: DiffChangeType,
    ) -> Self {
        match change_type {
            DiffChangeType::Added => Self::Added { entity, type_name, diff_data },
            DiffChangeType::Modified => Self::Modified { entity, type_name, diff_data },
            DiffChangeType::Removed => Self::Removed { entity, type_name },
        }
    }

    /// Convert to regular DiffComponentChange for text-based logging
    pub fn to_diff_change(&self) -> DiffComponentChange {
        match self {
            Self::Added { entity, type_name, diff_data } => {
                DiffComponentChange::Added {
                    entity: *entity,
                    type_name: type_name.clone(),
                    diff_string: format!("Binary({} bytes)", diff_data.len()),
                }
            }
            Self::Modified { entity, type_name, diff_data } => {
                DiffComponentChange::Modified {
                    entity: *entity,
                    type_name: type_name.clone(),
                    diff_string: format!("Binary({} bytes)", diff_data.len()),
                }
            }
            Self::Removed { entity, type_name } => {
                DiffComponentChange::Removed {
                    entity: *entity,
                    type_name: type_name.clone(),
                }
            }
        }
    }
}

/// Type of diff change for helper functions
#[derive(Debug, Clone, Copy)]
pub enum DiffChangeType {
    Added,
    Modified,
    Removed,
}

/// Macro to automatically implement Diff for structs
/// Generates diff functions for all fields
#[macro_export]
macro_rules! impl_diff {
    ($type:ident { $($field:ident: $field_type:ty),* $(,)? }) => {
        paste::paste! {
            #[derive(Clone, Debug)]
            pub struct [<$type Diff>] {
                $(
                    pub $field: Option<<$field_type as Diff>::Diff>,
                )*
            }

            impl Diff for $type {
                type Diff = [<$type Diff>];

                fn diff(&self, other: &Self) -> Option<Self::Diff> {
                    let mut has_changes = false;
                    let diff = Self::Diff {
                        $(
                            $field: {
                                let field_diff = self.$field.diff(&other.$field);
                                if field_diff.is_some() {
                                    has_changes = true;
                                }
                                field_diff
                            },
                        )*
                    };

                    if has_changes {
                        Some(diff)
                    } else {
                        None
                    }
                }

                fn apply_diff(&mut self, diff: &Self::Diff) {
                    $(
                        if let Some(ref field_diff) = diff.$field {
                            self.$field.apply_diff(field_diff);
                        }
                    )*
                }
            }

            impl DiffComponent for $type {}
        }
    };
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

impl BinaryDiff for i32 {
    fn diff_to_binary(diff: &Self::Diff) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(diff)?)
    }

    fn diff_from_binary(data: &[u8]) -> Result<Self::Diff, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(data)?)
    }
}

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

impl BinaryDiff for f32 {
    fn diff_to_binary(diff: &Self::Diff) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(diff)?)
    }

    fn diff_from_binary(data: &[u8]) -> Result<Self::Diff, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(data)?)
    }
}

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

impl BinaryDiff for usize {
    fn diff_to_binary(diff: &Self::Diff) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(diff)?)
    }

    fn diff_from_binary(data: &[u8]) -> Result<Self::Diff, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(data)?)
    }
}

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

impl BinaryDiff for u32 {
    fn diff_to_binary(diff: &Self::Diff) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(diff)?)
    }

    fn diff_from_binary(data: &[u8]) -> Result<Self::Diff, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(data)?)
    }
}

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

impl BinaryDiff for String {
    fn diff_to_binary(diff: &Self::Diff) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        Ok(bincode::serialize(diff)?)
    }

    fn diff_from_binary(data: &[u8]) -> Result<Self::Diff, Box<dyn std::error::Error>> {
        Ok(bincode::deserialize(data)?)
    }
}

impl<T: Diff + Clone + std::fmt::Debug> Diff for Vec<T> {
    type Diff = VecDiff<T>;

    fn diff(&self, other: &Self) -> Option<Self::Diff> {
        let mut changes = Vec::new();
        let mut has_changes = false;

        // Find additions, modifications, and removals
        let max_len = self.len().max(other.len());

        for i in 0..max_len {
            match (self.get(i), other.get(i)) {
                (Some(old), Some(new)) => {
                    if let Some(item_diff) = old.diff(new) {
                        changes.push(VecChange::Modified { index: i, diff: item_diff });
                        has_changes = true;
                    }
                }
                (Some(_), None) => {
                    changes.push(VecChange::Removed { index: i });
                    has_changes = true;
                }
                (None, Some(new)) => {
                    changes.push(VecChange::Added { index: i, value: new.clone() });
                    has_changes = true;
                }
                (None, None) => unreachable!("Both can't be None in max_len range"),
            }
        }

        if has_changes {
            Some(VecDiff { changes })
        } else {
            None
        }
    }

    fn apply_diff(&mut self, diff: &Self::Diff) {
        for change in &diff.changes {
            match change {
                VecChange::Added { index, value } => {
                    if *index >= self.len() {
                        self.resize_with(*index + 1, || value.clone());
                    }
                    self[*index] = value.clone();
                }
                VecChange::Modified { index, diff } => {
                    if let Some(item) = self.get_mut(*index) {
                        item.apply_diff(diff);
                    }
                }
                VecChange::Removed { index } => {
                    if *index < self.len() {
                        self.remove(*index);
                    }
                }
            }
        }
    }
}

/// Diff type for Vec<T>
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VecDiff<T: Diff + std::fmt::Debug> {
    pub changes: Vec<VecChange<T>>,
}

/// Individual change in a Vec
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VecChange<T: Diff + std::fmt::Debug> {
    Added { index: usize, value: T },
    Modified { index: usize, diff: T::Diff },
    Removed { index: usize },
}

impl<T: Diff + std::fmt::Debug> VecChange<T> {
    pub fn index(&self) -> usize {
        match self {
            VecChange::Added { index, .. } => *index,
            VecChange::Modified { index, .. } => *index,
            VecChange::Removed { index } => *index,
        }
    }
}

impl<
    K: Clone + Eq + Hash + std::fmt::Debug,
    V: Diff + Clone + std::fmt::Debug,
> Diff for HashMap<K, V> {
    type Diff = HashMapDiff<K, V>;

    fn diff(&self, other: &Self) -> Option<Self::Diff> {
        let mut changes = Vec::new();
        let mut has_changes = false;

        // Check for modifications and removals
        for (key, old_value) in self {
            match other.get(key) {
                Some(new_value) => {
                    if let Some(value_diff) = old_value.diff(new_value) {
                        changes.push(HashMapChange::Modified { key: key.clone(), diff: value_diff });
                        has_changes = true;
                    }
                }
                None => {
                    changes.push(HashMapChange::Removed { key: key.clone() });
                    has_changes = true;
                }
            }
        }

        // Check for additions
        for (key, new_value) in other {
            if !self.contains_key(key) {
                changes.push(HashMapChange::Added { key: key.clone(), value: new_value.clone() });
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
        for change in &diff.changes {
            match change {
                HashMapChange::Added { key, value } => {
                    self.insert(key.clone(), value.clone());
                }
                HashMapChange::Modified { key, diff } => {
                    if let Some(existing_value) = self.get_mut(key) {
                        existing_value.apply_diff(diff);
                    }
                }
                HashMapChange::Removed { key } => {
                    self.remove(key);
                }
            }
        }
    }
}

/// Diff type for HashMap<K, V>
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HashMapDiff<K: std::fmt::Debug, V: Diff + std::fmt::Debug> {
    pub changes: Vec<HashMapChange<K, V>>,
}

/// Individual change in a HashMap
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum HashMapChange<K: std::fmt::Debug, V: Diff + std::fmt::Debug> {
    Added { key: K, value: V },
    Modified { key: K, diff: V::Diff },
    Removed { key: K },
}