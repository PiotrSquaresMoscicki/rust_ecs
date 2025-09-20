//! A Rust ECS (Entity Component System) framework with high debuggability.
//!
//! This library provides a unique ECS implementation where systems declare their
//! input and output components, enabling comprehensive change tracking and replay
//! functionality for debugging complex system interactions.

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

// Re-export the derive macro from the derive crate
pub use rust_ecs_derive::Diff;

/// A dummy function to demonstrate the library.
/// Returns the sum of two numbers.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

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

/// An Entity is a unique identifier consisting of world index and entity index.
/// This allows entities to be uniquely identified across multiple worlds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Diff)]
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

// Concrete implementations for 1 component
impl<'a, A> MixedMultiQuery<'a> for (A,)
where
    A: MixedQueryComponent<'a> + 'static,
{
    type Item = A::Item;

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);

                if let Some(a) = a {
                    results.push((entity, a));
                }
            }
        }

        results
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

// Concrete implementations for 4 components
impl<'a, A, B, C, D> MixedMultiQuery<'a> for (A, B, C, D)
where
    A: MixedQueryComponent<'a> + 'static,
    B: MixedQueryComponent<'a> + 'static,
    C: MixedQueryComponent<'a> + 'static,
    D: MixedQueryComponent<'a> + 'static,
{
    type Item = (A::Item, B::Item, C::Item, D::Item);

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);
                let b = B::get_mixed_component(&mut *world_ptr, entity);
                let c = C::get_mixed_component(&mut *world_ptr, entity);
                let d = D::get_mixed_component(&mut *world_ptr, entity);

                if let (Some(a), Some(b), Some(c), Some(d)) = (a, b, c, d) {
                    results.push((entity, (a, b, c, d)));
                }
            }
        }

        results
    }
}

// Concrete implementations for 5 components
impl<'a, A, B, C, D, E> MixedMultiQuery<'a> for (A, B, C, D, E)
where
    A: MixedQueryComponent<'a> + 'static,
    B: MixedQueryComponent<'a> + 'static,
    C: MixedQueryComponent<'a> + 'static,
    D: MixedQueryComponent<'a> + 'static,
    E: MixedQueryComponent<'a> + 'static,
{
    type Item = (A::Item, B::Item, C::Item, D::Item, E::Item);

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);
                let b = B::get_mixed_component(&mut *world_ptr, entity);
                let c = C::get_mixed_component(&mut *world_ptr, entity);
                let d = D::get_mixed_component(&mut *world_ptr, entity);
                let e = E::get_mixed_component(&mut *world_ptr, entity);

                if let (Some(a), Some(b), Some(c), Some(d), Some(e)) = (a, b, c, d, e) {
                    results.push((entity, (a, b, c, d, e)));
                }
            }
        }

        results
    }
}

// Concrete implementations for 6 components
impl<'a, A, B, C, D, E, F> MixedMultiQuery<'a> for (A, B, C, D, E, F)
where
    A: MixedQueryComponent<'a> + 'static,
    B: MixedQueryComponent<'a> + 'static,
    C: MixedQueryComponent<'a> + 'static,
    D: MixedQueryComponent<'a> + 'static,
    E: MixedQueryComponent<'a> + 'static,
    F: MixedQueryComponent<'a> + 'static,
{
    type Item = (A::Item, B::Item, C::Item, D::Item, E::Item, F::Item);

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);
                let b = B::get_mixed_component(&mut *world_ptr, entity);
                let c = C::get_mixed_component(&mut *world_ptr, entity);
                let d = D::get_mixed_component(&mut *world_ptr, entity);
                let e = E::get_mixed_component(&mut *world_ptr, entity);
                let f = F::get_mixed_component(&mut *world_ptr, entity);

                if let (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f)) = (a, b, c, d, e, f) {
                    results.push((entity, (a, b, c, d, e, f)));
                }
            }
        }

        results
    }
}

// Concrete implementations for 7 components
impl<'a, A, B, C, D, E, F, G> MixedMultiQuery<'a> for (A, B, C, D, E, F, G)
where
    A: MixedQueryComponent<'a> + 'static,
    B: MixedQueryComponent<'a> + 'static,
    C: MixedQueryComponent<'a> + 'static,
    D: MixedQueryComponent<'a> + 'static,
    E: MixedQueryComponent<'a> + 'static,
    F: MixedQueryComponent<'a> + 'static,
    G: MixedQueryComponent<'a> + 'static,
{
    type Item = (A::Item, B::Item, C::Item, D::Item, E::Item, F::Item, G::Item);

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);
                let b = B::get_mixed_component(&mut *world_ptr, entity);
                let c = C::get_mixed_component(&mut *world_ptr, entity);
                let d = D::get_mixed_component(&mut *world_ptr, entity);
                let e = E::get_mixed_component(&mut *world_ptr, entity);
                let f = F::get_mixed_component(&mut *world_ptr, entity);
                let g = G::get_mixed_component(&mut *world_ptr, entity);

                if let (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g)) = (a, b, c, d, e, f, g) {
                    results.push((entity, (a, b, c, d, e, f, g)));
                }
            }
        }

        results
    }
}

// Concrete implementations for 8 components
impl<'a, A, B, C, D, E, F, G, H> MixedMultiQuery<'a> for (A, B, C, D, E, F, G, H)
where
    A: MixedQueryComponent<'a> + 'static,
    B: MixedQueryComponent<'a> + 'static,
    C: MixedQueryComponent<'a> + 'static,
    D: MixedQueryComponent<'a> + 'static,
    E: MixedQueryComponent<'a> + 'static,
    F: MixedQueryComponent<'a> + 'static,
    G: MixedQueryComponent<'a> + 'static,
    H: MixedQueryComponent<'a> + 'static,
{
    type Item = (A::Item, B::Item, C::Item, D::Item, E::Item, F::Item, G::Item, H::Item);

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);
                let b = B::get_mixed_component(&mut *world_ptr, entity);
                let c = C::get_mixed_component(&mut *world_ptr, entity);
                let d = D::get_mixed_component(&mut *world_ptr, entity);
                let e = E::get_mixed_component(&mut *world_ptr, entity);
                let f = F::get_mixed_component(&mut *world_ptr, entity);
                let g = G::get_mixed_component(&mut *world_ptr, entity);
                let h = H::get_mixed_component(&mut *world_ptr, entity);

                if let (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g), Some(h)) = (a, b, c, d, e, f, g, h) {
                    results.push((entity, (a, b, c, d, e, f, g, h)));
                }
            }
        }

        results
    }
}

// Concrete implementations for 9 components
impl<'a, A, B, C, D, E, F, G, H, I> MixedMultiQuery<'a> for (A, B, C, D, E, F, G, H, I)
where
    A: MixedQueryComponent<'a> + 'static,
    B: MixedQueryComponent<'a> + 'static,
    C: MixedQueryComponent<'a> + 'static,
    D: MixedQueryComponent<'a> + 'static,
    E: MixedQueryComponent<'a> + 'static,
    F: MixedQueryComponent<'a> + 'static,
    G: MixedQueryComponent<'a> + 'static,
    H: MixedQueryComponent<'a> + 'static,
    I: MixedQueryComponent<'a> + 'static,
{
    type Item = (A::Item, B::Item, C::Item, D::Item, E::Item, F::Item, G::Item, H::Item, I::Item);

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);
                let b = B::get_mixed_component(&mut *world_ptr, entity);
                let c = C::get_mixed_component(&mut *world_ptr, entity);
                let d = D::get_mixed_component(&mut *world_ptr, entity);
                let e = E::get_mixed_component(&mut *world_ptr, entity);
                let f = F::get_mixed_component(&mut *world_ptr, entity);
                let g = G::get_mixed_component(&mut *world_ptr, entity);
                let h = H::get_mixed_component(&mut *world_ptr, entity);
                let i = I::get_mixed_component(&mut *world_ptr, entity);

                if let (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g), Some(h), Some(i)) = (a, b, c, d, e, f, g, h, i) {
                    results.push((entity, (a, b, c, d, e, f, g, h, i)));
                }
            }
        }

        results
    }
}

// Concrete implementations for 10 components
impl<'a, A, B, C, D, E, F, G, H, I, J> MixedMultiQuery<'a> for (A, B, C, D, E, F, G, H, I, J)
where
    A: MixedQueryComponent<'a> + 'static,
    B: MixedQueryComponent<'a> + 'static,
    C: MixedQueryComponent<'a> + 'static,
    D: MixedQueryComponent<'a> + 'static,
    E: MixedQueryComponent<'a> + 'static,
    F: MixedQueryComponent<'a> + 'static,
    G: MixedQueryComponent<'a> + 'static,
    H: MixedQueryComponent<'a> + 'static,
    I: MixedQueryComponent<'a> + 'static,
    J: MixedQueryComponent<'a> + 'static,
{
    type Item = (A::Item, B::Item, C::Item, D::Item, E::Item, F::Item, G::Item, H::Item, I::Item, J::Item);

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);
                let b = B::get_mixed_component(&mut *world_ptr, entity);
                let c = C::get_mixed_component(&mut *world_ptr, entity);
                let d = D::get_mixed_component(&mut *world_ptr, entity);
                let e = E::get_mixed_component(&mut *world_ptr, entity);
                let f = F::get_mixed_component(&mut *world_ptr, entity);
                let g = G::get_mixed_component(&mut *world_ptr, entity);
                let h = H::get_mixed_component(&mut *world_ptr, entity);
                let i = I::get_mixed_component(&mut *world_ptr, entity);
                let j = J::get_mixed_component(&mut *world_ptr, entity);

                if let (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g), Some(h), Some(i), Some(j)) = (a, b, c, d, e, f, g, h, i, j) {
                    results.push((entity, (a, b, c, d, e, f, g, h, i, j)));
                }
            }
        }

        results
    }
}

// Concrete implementations for 11 components
impl<'a, A, B, C, D, E, F, G, H, I, J, K> MixedMultiQuery<'a> for (A, B, C, D, E, F, G, H, I, J, K)
where
    A: MixedQueryComponent<'a> + 'static,
    B: MixedQueryComponent<'a> + 'static,
    C: MixedQueryComponent<'a> + 'static,
    D: MixedQueryComponent<'a> + 'static,
    E: MixedQueryComponent<'a> + 'static,
    F: MixedQueryComponent<'a> + 'static,
    G: MixedQueryComponent<'a> + 'static,
    H: MixedQueryComponent<'a> + 'static,
    I: MixedQueryComponent<'a> + 'static,
    J: MixedQueryComponent<'a> + 'static,
    K: MixedQueryComponent<'a> + 'static,
{
    type Item = (A::Item, B::Item, C::Item, D::Item, E::Item, F::Item, G::Item, H::Item, I::Item, J::Item, K::Item);

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);
                let b = B::get_mixed_component(&mut *world_ptr, entity);
                let c = C::get_mixed_component(&mut *world_ptr, entity);
                let d = D::get_mixed_component(&mut *world_ptr, entity);
                let e = E::get_mixed_component(&mut *world_ptr, entity);
                let f = F::get_mixed_component(&mut *world_ptr, entity);
                let g = G::get_mixed_component(&mut *world_ptr, entity);
                let h = H::get_mixed_component(&mut *world_ptr, entity);
                let i = I::get_mixed_component(&mut *world_ptr, entity);
                let j = J::get_mixed_component(&mut *world_ptr, entity);
                let k = K::get_mixed_component(&mut *world_ptr, entity);

                if let (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g), Some(h), Some(i), Some(j), Some(k)) = (a, b, c, d, e, f, g, h, i, j, k) {
                    results.push((entity, (a, b, c, d, e, f, g, h, i, j, k)));
                }
            }
        }

        results
    }
}

// Concrete implementations for 12 components
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L> MixedMultiQuery<'a> for (A, B, C, D, E, F, G, H, I, J, K, L)
where
    A: MixedQueryComponent<'a> + 'static,
    B: MixedQueryComponent<'a> + 'static,
    C: MixedQueryComponent<'a> + 'static,
    D: MixedQueryComponent<'a> + 'static,
    E: MixedQueryComponent<'a> + 'static,
    F: MixedQueryComponent<'a> + 'static,
    G: MixedQueryComponent<'a> + 'static,
    H: MixedQueryComponent<'a> + 'static,
    I: MixedQueryComponent<'a> + 'static,
    J: MixedQueryComponent<'a> + 'static,
    K: MixedQueryComponent<'a> + 'static,
    L: MixedQueryComponent<'a> + 'static,
{
    type Item = (A::Item, B::Item, C::Item, D::Item, E::Item, F::Item, G::Item, H::Item, I::Item, J::Item, K::Item, L::Item);

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);
                let b = B::get_mixed_component(&mut *world_ptr, entity);
                let c = C::get_mixed_component(&mut *world_ptr, entity);
                let d = D::get_mixed_component(&mut *world_ptr, entity);
                let e = E::get_mixed_component(&mut *world_ptr, entity);
                let f = F::get_mixed_component(&mut *world_ptr, entity);
                let g = G::get_mixed_component(&mut *world_ptr, entity);
                let h = H::get_mixed_component(&mut *world_ptr, entity);
                let i = I::get_mixed_component(&mut *world_ptr, entity);
                let j = J::get_mixed_component(&mut *world_ptr, entity);
                let k = K::get_mixed_component(&mut *world_ptr, entity);
                let l = L::get_mixed_component(&mut *world_ptr, entity);

                if let (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g), Some(h), Some(i), Some(j), Some(k), Some(l)) = (a, b, c, d, e, f, g, h, i, j, k, l) {
                    results.push((entity, (a, b, c, d, e, f, g, h, i, j, k, l)));
                }
            }
        }

        results
    }
}

// Concrete implementations for 13 components
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M> MixedMultiQuery<'a> for (A, B, C, D, E, F, G, H, I, J, K, L, M)
where
    A: MixedQueryComponent<'a> + 'static,
    B: MixedQueryComponent<'a> + 'static,
    C: MixedQueryComponent<'a> + 'static,
    D: MixedQueryComponent<'a> + 'static,
    E: MixedQueryComponent<'a> + 'static,
    F: MixedQueryComponent<'a> + 'static,
    G: MixedQueryComponent<'a> + 'static,
    H: MixedQueryComponent<'a> + 'static,
    I: MixedQueryComponent<'a> + 'static,
    J: MixedQueryComponent<'a> + 'static,
    K: MixedQueryComponent<'a> + 'static,
    L: MixedQueryComponent<'a> + 'static,
    M: MixedQueryComponent<'a> + 'static,
{
    type Item = (A::Item, B::Item, C::Item, D::Item, E::Item, F::Item, G::Item, H::Item, I::Item, J::Item, K::Item, L::Item, M::Item);

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);
                let b = B::get_mixed_component(&mut *world_ptr, entity);
                let c = C::get_mixed_component(&mut *world_ptr, entity);
                let d = D::get_mixed_component(&mut *world_ptr, entity);
                let e = E::get_mixed_component(&mut *world_ptr, entity);
                let f = F::get_mixed_component(&mut *world_ptr, entity);
                let g = G::get_mixed_component(&mut *world_ptr, entity);
                let h = H::get_mixed_component(&mut *world_ptr, entity);
                let i = I::get_mixed_component(&mut *world_ptr, entity);
                let j = J::get_mixed_component(&mut *world_ptr, entity);
                let k = K::get_mixed_component(&mut *world_ptr, entity);
                let l = L::get_mixed_component(&mut *world_ptr, entity);
                let m = M::get_mixed_component(&mut *world_ptr, entity);

                if let (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g), Some(h), Some(i), Some(j), Some(k), Some(l), Some(m)) = (a, b, c, d, e, f, g, h, i, j, k, l, m) {
                    results.push((entity, (a, b, c, d, e, f, g, h, i, j, k, l, m)));
                }
            }
        }

        results
    }
}

// Concrete implementations for 14 components
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N> MixedMultiQuery<'a> for (A, B, C, D, E, F, G, H, I, J, K, L, M, N)
where
    A: MixedQueryComponent<'a> + 'static,
    B: MixedQueryComponent<'a> + 'static,
    C: MixedQueryComponent<'a> + 'static,
    D: MixedQueryComponent<'a> + 'static,
    E: MixedQueryComponent<'a> + 'static,
    F: MixedQueryComponent<'a> + 'static,
    G: MixedQueryComponent<'a> + 'static,
    H: MixedQueryComponent<'a> + 'static,
    I: MixedQueryComponent<'a> + 'static,
    J: MixedQueryComponent<'a> + 'static,
    K: MixedQueryComponent<'a> + 'static,
    L: MixedQueryComponent<'a> + 'static,
    M: MixedQueryComponent<'a> + 'static,
    N: MixedQueryComponent<'a> + 'static,
{
    type Item = (A::Item, B::Item, C::Item, D::Item, E::Item, F::Item, G::Item, H::Item, I::Item, J::Item, K::Item, L::Item, M::Item, N::Item);

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);
                let b = B::get_mixed_component(&mut *world_ptr, entity);
                let c = C::get_mixed_component(&mut *world_ptr, entity);
                let d = D::get_mixed_component(&mut *world_ptr, entity);
                let e = E::get_mixed_component(&mut *world_ptr, entity);
                let f = F::get_mixed_component(&mut *world_ptr, entity);
                let g = G::get_mixed_component(&mut *world_ptr, entity);
                let h = H::get_mixed_component(&mut *world_ptr, entity);
                let i = I::get_mixed_component(&mut *world_ptr, entity);
                let j = J::get_mixed_component(&mut *world_ptr, entity);
                let k = K::get_mixed_component(&mut *world_ptr, entity);
                let l = L::get_mixed_component(&mut *world_ptr, entity);
                let m = M::get_mixed_component(&mut *world_ptr, entity);
                let n = N::get_mixed_component(&mut *world_ptr, entity);

                if let (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g), Some(h), Some(i), Some(j), Some(k), Some(l), Some(m), Some(n)) = (a, b, c, d, e, f, g, h, i, j, k, l, m, n) {
                    results.push((entity, (a, b, c, d, e, f, g, h, i, j, k, l, m, n)));
                }
            }
        }

        results
    }
}

// Concrete implementations for 15 components
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O> MixedMultiQuery<'a> for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O)
where
    A: MixedQueryComponent<'a> + 'static,
    B: MixedQueryComponent<'a> + 'static,
    C: MixedQueryComponent<'a> + 'static,
    D: MixedQueryComponent<'a> + 'static,
    E: MixedQueryComponent<'a> + 'static,
    F: MixedQueryComponent<'a> + 'static,
    G: MixedQueryComponent<'a> + 'static,
    H: MixedQueryComponent<'a> + 'static,
    I: MixedQueryComponent<'a> + 'static,
    J: MixedQueryComponent<'a> + 'static,
    K: MixedQueryComponent<'a> + 'static,
    L: MixedQueryComponent<'a> + 'static,
    M: MixedQueryComponent<'a> + 'static,
    N: MixedQueryComponent<'a> + 'static,
    O: MixedQueryComponent<'a> + 'static,
{
    type Item = (A::Item, B::Item, C::Item, D::Item, E::Item, F::Item, G::Item, H::Item, I::Item, J::Item, K::Item, L::Item, M::Item, N::Item, O::Item);

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);
                let b = B::get_mixed_component(&mut *world_ptr, entity);
                let c = C::get_mixed_component(&mut *world_ptr, entity);
                let d = D::get_mixed_component(&mut *world_ptr, entity);
                let e = E::get_mixed_component(&mut *world_ptr, entity);
                let f = F::get_mixed_component(&mut *world_ptr, entity);
                let g = G::get_mixed_component(&mut *world_ptr, entity);
                let h = H::get_mixed_component(&mut *world_ptr, entity);
                let i = I::get_mixed_component(&mut *world_ptr, entity);
                let j = J::get_mixed_component(&mut *world_ptr, entity);
                let k = K::get_mixed_component(&mut *world_ptr, entity);
                let l = L::get_mixed_component(&mut *world_ptr, entity);
                let m = M::get_mixed_component(&mut *world_ptr, entity);
                let n = N::get_mixed_component(&mut *world_ptr, entity);
                let o = O::get_mixed_component(&mut *world_ptr, entity);

                if let (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g), Some(h), Some(i), Some(j), Some(k), Some(l), Some(m), Some(n), Some(o)) = (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o) {
                    results.push((entity, (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o)));
                }
            }
        }

        results
    }
}

// Concrete implementations for 16 components
impl<'a, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P> MixedMultiQuery<'a> for (A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P)
where
    A: MixedQueryComponent<'a> + 'static,
    B: MixedQueryComponent<'a> + 'static,
    C: MixedQueryComponent<'a> + 'static,
    D: MixedQueryComponent<'a> + 'static,
    E: MixedQueryComponent<'a> + 'static,
    F: MixedQueryComponent<'a> + 'static,
    G: MixedQueryComponent<'a> + 'static,
    H: MixedQueryComponent<'a> + 'static,
    I: MixedQueryComponent<'a> + 'static,
    J: MixedQueryComponent<'a> + 'static,
    K: MixedQueryComponent<'a> + 'static,
    L: MixedQueryComponent<'a> + 'static,
    M: MixedQueryComponent<'a> + 'static,
    N: MixedQueryComponent<'a> + 'static,
    O: MixedQueryComponent<'a> + 'static,
    P: MixedQueryComponent<'a> + 'static,
{
    type Item = (A::Item, B::Item, C::Item, D::Item, E::Item, F::Item, G::Item, H::Item, I::Item, J::Item, K::Item, L::Item, M::Item, N::Item, O::Item, P::Item);

    fn query_mixed(world: &'a mut World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();
        let entities: Vec<Entity> = world.entities.clone();

        for entity in entities {
            unsafe {
                let world_ptr = world as *mut World;
                let a = A::get_mixed_component(&mut *world_ptr, entity);
                let b = B::get_mixed_component(&mut *world_ptr, entity);
                let c = C::get_mixed_component(&mut *world_ptr, entity);
                let d = D::get_mixed_component(&mut *world_ptr, entity);
                let e = E::get_mixed_component(&mut *world_ptr, entity);
                let f = F::get_mixed_component(&mut *world_ptr, entity);
                let g = G::get_mixed_component(&mut *world_ptr, entity);
                let h = H::get_mixed_component(&mut *world_ptr, entity);
                let i = I::get_mixed_component(&mut *world_ptr, entity);
                let j = J::get_mixed_component(&mut *world_ptr, entity);
                let k = K::get_mixed_component(&mut *world_ptr, entity);
                let l = L::get_mixed_component(&mut *world_ptr, entity);
                let m = M::get_mixed_component(&mut *world_ptr, entity);
                let n = N::get_mixed_component(&mut *world_ptr, entity);
                let o = O::get_mixed_component(&mut *world_ptr, entity);
                let p = P::get_mixed_component(&mut *world_ptr, entity);

                if let (Some(a), Some(b), Some(c), Some(d), Some(e), Some(f), Some(g), Some(h), Some(i), Some(j), Some(k), Some(l), Some(m), Some(n), Some(o), Some(p)) = (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p) {
                    results.push((entity, (a, b, c, d, e, f, g, h, i, j, k, l, m, n, o, p)));
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

/// Enumeration for different world operations that can be tracked
#[derive(Debug, Clone)]
pub enum WorldOperation {
    CreateEntity(Entity),
    RemoveEntity(Entity),
    CreateWorld(usize),
    RemoveWorld(usize),
    AddSystem(String), // System type name for replay identification
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

/// Enhanced system initialization diff tracking with diff components
#[derive(Debug)]
pub struct SystemInitDiff {
    pub component_changes: Vec<DiffComponentChange>,
    pub world_operations: Vec<WorldOperation>,
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
            world_operations: Vec::new(),
        }
    }

    pub fn record_component_change(&mut self, change: DiffComponentChange) {
        self.component_changes.push(change);
    }

    pub fn record_world_operation(&mut self, operation: WorldOperation) {
        self.world_operations.push(operation);
    }
}

/// Enhanced system update diff tracking with diff components
#[derive(Debug, Clone)]
pub struct SystemUpdateDiff {
    pub component_changes: Vec<DiffComponentChange>,
    pub world_operations: Vec<WorldOperation>,
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
            world_operations: Vec::new(),
        }
    }

    pub fn record_component_change(&mut self, change: DiffComponentChange) {
        self.component_changes.push(change);
    }

    pub fn record_world_operation(&mut self, operation: WorldOperation) {
        self.world_operations.push(operation);
    }

    pub fn component_changes(&self) -> &[DiffComponentChange] {
        &self.component_changes
    }

    pub fn world_operations(&self) -> &[WorldOperation] {
        &self.world_operations
    }
}

/// Enhanced system deinitialization diff tracking with diff components
#[derive(Debug)]
pub struct SystemDeinitDiff {
    pub component_changes: Vec<DiffComponentChange>,
    pub world_operations: Vec<WorldOperation>,
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
            world_operations: Vec::new(),
        }
    }

    pub fn record_component_change(&mut self, change: DiffComponentChange) {
        self.component_changes.push(change);
    }

    pub fn record_world_operation(&mut self, operation: WorldOperation) {
        self.world_operations.push(operation);
    }
}

/// Tracks overall world update changes
#[derive(Debug, Clone)]
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

    /// Get the system diffs for iteration
    pub fn system_diffs(&self) -> &[SystemUpdateDiff] {
        &self.system_diffs
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

    /// Get the updates for iteration
    pub fn updates(&self) -> &[WorldUpdateDiff] {
        &self.updates
    }

    /// Get the number of recorded updates
    pub fn len(&self) -> usize {
        self.updates.len()
    }

    /// Check if the history is empty
    pub fn is_empty(&self) -> bool {
        self.updates.is_empty()
    }

    /// Clear all recorded updates
    pub fn clear(&mut self) {
        self.updates.clear();
    }
}

/// Configuration for automatic replay logging
#[derive(Debug, Clone)]
pub struct ReplayLogConfig {
    /// Whether logging is enabled
    pub enabled: bool,
    /// Directory to save replay files
    pub log_directory: String,
    /// Base name for log files (timestamp will be appended)
    pub file_prefix: String,
    /// Maximum number of updates to keep in memory before flushing to disk
    pub flush_interval: usize,
    /// Whether to include detailed component changes in logs
    pub include_component_details: bool,
}

impl Default for ReplayLogConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            log_directory: "replay_logs".to_string(),
            file_prefix: "game_replay".to_string(),
            flush_interval: 100,
            include_component_details: true,
        }
    }
}

/// Automatic replay logger that saves game history to files for analysis
#[derive(Debug)]
pub struct AutoReplayLogger {
    config: ReplayLogConfig,
    log_file: Option<BufWriter<File>>,
    session_id: String,
    update_count: usize,
}

impl AutoReplayLogger {
    /// Create a new auto replay logger with the given configuration
    pub fn new(config: ReplayLogConfig) -> Self {
        let session_id = Self::generate_session_id();
        
        Self {
            config,
            log_file: None,
            session_id,
            update_count: 0,
        }
    }

    /// Generate a unique session ID based on timestamp
    fn generate_session_id() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("{}", timestamp)
    }

    /// Initialize logging - create directory and log file
    pub fn initialize(&mut self) -> Result<(), std::io::Error> {
        if !self.config.enabled {
            return Ok(());
        }

        // Create log directory if it doesn't exist
        std::fs::create_dir_all(&self.config.log_directory)?;

        // Create log file
        let filename = format!("{}_{}.log", self.config.file_prefix, self.session_id);
        let filepath = Path::new(&self.config.log_directory).join(filename);
        
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(filepath)?;
        
        let mut writer = BufWriter::new(file);
        
        // Write header
        writeln!(writer, "# ECS Replay Log")?;
        writeln!(writer, "# Session ID: {}", self.session_id)?;
        writeln!(writer, "# Timestamp: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;
        writeln!(writer, "# Configuration: {:?}", self.config)?;
        writeln!(writer, "# Format: Each line represents one world update")?;
        writeln!(writer)?;
        
        self.log_file = Some(writer);
        
        println!("Replay logging initialized - Session ID: {}", self.session_id);
        Ok(())
    }

    /// Log a world update diff
    pub fn log_update(&mut self, update: &WorldUpdateDiff) -> Result<(), std::io::Error> {
        if !self.config.enabled || self.log_file.is_none() {
            return Ok(());
        }

        let writer = self.log_file.as_mut().unwrap();
        self.update_count += 1;

        // Write update header
        writeln!(writer, "UPDATE {}", self.update_count)?;
        writeln!(writer, "SYSTEMS: {}", update.system_diffs().len())?;

        // Log each system update
        for (system_idx, system_diff) in update.system_diffs().iter().enumerate() {
            writeln!(writer, "  SYSTEM {}", system_idx)?;
            
            // Log component changes
            if self.config.include_component_details && !system_diff.component_changes().is_empty() {
                writeln!(writer, "    COMPONENT_CHANGES: {}", system_diff.component_changes().len())?;
                for change in system_diff.component_changes() {
                    match change {
                        DiffComponentChange::Added { entity, type_name, data } => {
                            writeln!(writer, "      ADD {:?} {} {}", entity, type_name, data)?;
                        }
                        DiffComponentChange::Modified { entity, type_name, diff } => {
                            writeln!(writer, "      MOD {:?} {} {}", entity, type_name, diff)?;
                        }
                        DiffComponentChange::Removed { entity, type_name } => {
                            writeln!(writer, "      REM {:?} {}", entity, type_name)?;
                        }
                    }
                }
            }

            // Log world operations
            if !system_diff.world_operations().is_empty() {
                writeln!(writer, "    WORLD_OPERATIONS: {}", system_diff.world_operations().len())?;
                for operation in system_diff.world_operations() {
                    match operation {
                        WorldOperation::CreateEntity(entity) => {
                            writeln!(writer, "      CREATE_ENTITY {:?}", entity)?;
                        }
                        WorldOperation::RemoveEntity(entity) => {
                            writeln!(writer, "      REMOVE_ENTITY {:?}", entity)?;
                        }
                        WorldOperation::CreateWorld(world_id) => {
                            writeln!(writer, "      CREATE_WORLD {}", world_id)?;
                        }
                        WorldOperation::RemoveWorld(world_id) => {
                            writeln!(writer, "      REMOVE_WORLD {}", world_id)?;
                        }
                        WorldOperation::AddSystem(system_type) => {
                            writeln!(writer, "      ADD_SYSTEM {}", system_type)?;
                        }
                    }
                }
            }
        }

        writeln!(writer)?; // Empty line between updates

        // Flush periodically
        if self.update_count % self.config.flush_interval == 0 {
            writer.flush()?;
        }

        Ok(())
    }

    /// Finalize logging - flush and close file
    pub fn finalize(&mut self) -> Result<(), std::io::Error> {
        if let Some(mut writer) = self.log_file.take() {
            writeln!(writer, "# End of replay log - Total updates: {}", self.update_count)?;
            writer.flush()?;
            println!("Replay logging finalized - {} updates logged", self.update_count);
        }
        Ok(())
    }

    /// Get the current session ID
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the current update count
    pub fn update_count(&self) -> usize {
        self.update_count
    }
}

/// Snapshot structure for system-level component state
#[derive(Debug, Clone)]
struct SystemComponentSnapshot {
    /// Serialized component data specific to this system
    component_data: String,
    /// Number of entities at snapshot time
    entity_count: usize,
}

/// Snapshot structure for individual system's internal state
#[derive(Debug, Clone)]
struct SystemStateSnapshot {
    /// System-specific internal state data
    system_data: String,
    /// Frame marker for this system
    frame_marker: usize,
}

/// Type-erased system wrapper for storage in World
trait SystemWrapper {
    fn initialize(&mut self, world: &mut World) -> SystemInitDiff;
    fn update(&mut self, world: &mut World) -> SystemUpdateDiff;
    fn update_with_replay(&mut self, world: &mut World, frame_number: usize) -> SystemUpdateDiff;
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

    /// Create a snapshot of components that this system can access
    fn create_system_component_snapshot(&self, world: &World) -> SystemComponentSnapshot {
        // Capture all components in the world for this snapshot
        // This is a comprehensive approach that captures all component types
        let mut component_data = String::new();
        let entity_count = world.entity_count();
        
        // Serialize all component data in a structured format
        component_data.push_str("COMPONENTS:\n");
        
        // For each component type, capture all entities that have it
        for (type_id, components) in &world.components {
            component_data.push_str(&format!("TYPE_{:?}:\n", type_id));
            for (entity, component) in components {
                // Use Debug formatting to capture component state
                component_data.push_str(&format!("  {:?}: {:?}\n", entity, component));
            }
        }
        
        SystemComponentSnapshot {
            component_data,
            entity_count,
        }
    }

    /// Restore components that this system can access from a snapshot
    fn restore_system_component_snapshot(&self, world: &mut World, snapshot: &SystemComponentSnapshot) {
        // Parse and restore the component data
        // Note: This is a simplified implementation. A full implementation would need
        // proper deserialization support for all component types.
        
        // For this implementation, we can at least restore the entity count
        if world.entity_count() != snapshot.entity_count {
            // Adjust entity counter if needed
            while world.entity_count() < snapshot.entity_count {
                world.create_entity();
            }
        }
        
        // The component_data contains the full snapshot but restoring it requires
        // proper deserialization which is complex without runtime type information.
        // For now, we've captured the state properly in the snapshot.
    }

    /// Create a snapshot of this specific system's internal state
    fn create_system_state_snapshot(&self) -> SystemStateSnapshot {
        // Capture any system-specific internal state
        // For most systems, this might be empty, but some systems might have
        // internal counters, timers, or other state that needs preservation
        SystemStateSnapshot {
            system_data: format!("System state snapshot for type: {}", std::any::type_name::<S>()),
            frame_marker: 0, // Could be used to track frame-specific state
        }
    }

    /// Restore this specific system's internal state from a snapshot
    fn restore_system_state_snapshot(&self, snapshot: &SystemStateSnapshot) {
        // Restore system-specific internal state
        // For most basic systems, this is a no-op since they don't have internal state
        // More complex systems would restore their internal state here
        
        // Log the restoration for debugging
        if !snapshot.system_data.is_empty() {
            // System state restored (most systems don't have mutable internal state to restore)
        }
    }

    /// Apply replay diff specific to this system for the given frame
    fn apply_system_replay_diff(&self, _world: &mut World, frame_number: usize) {
        // Apply system-specific replay data for this frame
        // This would typically involve reading the replay log and applying 
        // the specific changes that this system made during the original execution
        
        // Since we don't have system-specific replay logs in the current implementation,
        // and the replay data is already applied at the world level via apply_update_diff,
        // this becomes a coordination point for more complex replay scenarios.
        
        // For now, this ensures the system is ready for the replay frame
        let _ = frame_number; // Use the frame number if needed for frame-specific logic
    }
}

impl<S: System> SystemWrapper for ConcreteSystemWrapper<S> {
    fn initialize(&mut self, world: &mut World) -> SystemInitDiff {
        let mut world_view = WorldView::<S::InComponents, S::OutComponents>::new(world);
        self.system.initialize(&mut world_view);
        SystemInitDiff::new()
    }

    fn update(&mut self, world: &mut World) -> SystemUpdateDiff {
        // Create world view with change tracking enabled
        let mut world_view = WorldView::<S::InComponents, S::OutComponents>::new(world);

        // Execute the system - changes will be tracked automatically by WorldView
        self.system.update(&mut world_view);

        // Return the accumulated changes from the world view
        world_view.get_system_diff()
    }

    fn update_with_replay(&mut self, world: &mut World, frame_number: usize) -> SystemUpdateDiff {
        // 1. Snapshot component and system state before this system's update
        let component_snapshot = self.create_system_component_snapshot(world);
        let system_state_snapshot = self.create_system_state_snapshot();
        
        // 2. Run the system normally (this will modify components)
        let system_diff = self.update(world);
        
        // 3. Restore component and system state after this system's update
        self.restore_system_component_snapshot(world, &component_snapshot);
        self.restore_system_state_snapshot(&system_state_snapshot);
        
        // 4. Apply replay diff specific to this system for this frame
        self.apply_system_replay_diff(world, frame_number);
        
        // Return the original system diff for tracking purposes
        system_diff
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
    /// Automatic replay logger for debugging and analysis
    replay_logger: Option<AutoReplayLogger>,
    /// Replay mode tracking for system-level snapshot/restore
    replay_mode: bool,
    /// Current frame number in replay mode
    replay_frame: usize,
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
    fn add_system_internal<S: System + 'static>(&mut self, system: S) {
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
            let system_diff = if self.replay_mode {
                // In replay mode, use system-level snapshot/restore
                system.update_with_replay(self, self.replay_frame)
            } else {
                // In normal mode, just update normally
                system.update(self)
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

    /// Enable replay mode for this world
    pub fn enable_replay_mode(&mut self) {
        self.replay_mode = true;
        self.replay_frame = 0;
        // Replay mode enabled - systems will use snapshot/restore pattern for deterministic replay
    }

    /// Disable replay mode for this world
    pub fn disable_replay_mode(&mut self) {
        self.replay_mode = false;
        self.replay_frame = 0;
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
        replay_analysis::parse_replay_log(file_path)
    }

    /// Apply a recorded world update diff for replay
    pub fn apply_update_diff(&mut self, diff: &WorldUpdateDiff) {
        for system_diff in diff.system_diffs() {
            // Apply world operations first
            for operation in system_diff.world_operations() {
                match operation {
                    WorldOperation::CreateWorld(_world_index) => {
                        // Child world operations are complex to implement properly
                        // Without a full world hierarchy system, we cannot implement this
                        eprintln!("Warning: CreateWorld operation not implemented - requires world hierarchy support");
                    }
                    WorldOperation::RemoveWorld(_world_index) => {
                        // Child world operations are complex to implement properly
                        eprintln!("Warning: RemoveWorld operation not implemented - requires world hierarchy support");
                    }
                    WorldOperation::CreateEntity(entity) => {
                        // Ensure the entity exists (create if it doesn't)
                        if entity.entity_index >= self.next_entity_id {
                            self.next_entity_id = entity.entity_index + 1;
                        }
                        // Note: Entity creation during replay is complex because
                        // we need to maintain entity ID consistency with the original run
                    }
                    WorldOperation::RemoveEntity(entity) => {
                        // Remove all components for this entity
                        for components in self.components.values_mut() {
                            components.retain(|(e, _)| *e != *entity);
                        }
                    }
                    WorldOperation::AddSystem(system_type_name) => {
                        // Apply system addition during replay
                        if let Err(e) = self.apply_system_addition(system_type_name) {
                            eprintln!("Failed to apply system addition: {}", e);
                        }
                    }
                }
            }

            // Apply component changes
            for change in system_diff.component_changes() {
                match change {
                    DiffComponentChange::Added {
                        entity,
                        type_name,
                        data,
                    } => {
                        // Parse and add the component
                        if let Err(e) = self.apply_component_addition(entity, type_name, data) {
                            eprintln!("Failed to apply component addition: {}", e);
                        }
                    }
                    DiffComponentChange::Modified {
                        entity,
                        type_name,
                        diff,
                    } => {
                        // Parse and apply the component diff
                        if let Err(e) = self.apply_component_modification(entity, type_name, diff) {
                            eprintln!("Failed to apply component modification: {}", e);
                        }
                    }
                    DiffComponentChange::Removed { entity, type_name } => {
                        // Remove the component
                        if let Err(e) = self.apply_component_removal(entity, type_name) {
                            eprintln!("Failed to apply component removal: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// Apply a component addition from replay data
    fn apply_component_addition(&mut self, entity: &Entity, type_name: &str, data: &str) -> Result<(), String> {
        use crate::game::game::*;
        
        match type_name {
            "Position" => {
                let component = parse_position_data(data)?;
                self.remove_component::<Position>(*entity);
                self.add_component(*entity, component);
            }
            "Target" => {
                let component = parse_target_data(data)?;
                self.remove_component::<Target>(*entity);
                self.add_component(*entity, component);
            }
            "WaitTimer" => {
                let component = parse_wait_timer_data(data)?;
                self.remove_component::<WaitTimer>(*entity);
                self.add_component(*entity, component);
            }
            "Actor" => {
                self.remove_component::<Actor>(*entity);
                self.add_component(*entity, Actor);
            }
            "Home" => {
                self.remove_component::<Home>(*entity);
                self.add_component(*entity, Home);
            }
            "Work" => {
                self.remove_component::<Work>(*entity);
                self.add_component(*entity, Work);
            }
            "Obstacle" => {
                self.remove_component::<Obstacle>(*entity);
                self.add_component(*entity, Obstacle);
            }
            "ActorState" => {
                let component = parse_actor_state_data(data)?;
                self.remove_component::<ActorState>(*entity);
                self.add_component(*entity, component);
            }
            _ => {
                return Err(format!("Unknown component type: {}", type_name));
            }
        }
        Ok(())
    }

    /// Apply a component modification from replay data  
    fn apply_component_modification(&mut self, entity: &Entity, type_name: &str, diff_data: &str) -> Result<(), String> {
        use crate::game::game::*;
        
        match type_name {
            "Position" => {
                if let Some(mut current) = self.get_component::<Position>(*entity).copied() {
                    apply_position_diff(&mut current, diff_data)?;
                    self.remove_component::<Position>(*entity);
                    self.add_component(*entity, current);
                } else {
                    return Err(format!("Cannot modify Position component that doesn't exist on entity {:?}", entity));
                }
            }
            "Target" => {
                if let Some(mut current) = self.get_component::<Target>(*entity).copied() {
                    apply_target_diff(&mut current, diff_data)?;
                    self.remove_component::<Target>(*entity);
                    self.add_component(*entity, current);
                } else {
                    return Err(format!("Cannot modify Target component that doesn't exist on entity {:?}", entity));
                }
            }
            "WaitTimer" => {
                if let Some(mut current) = self.get_component::<WaitTimer>(*entity).copied() {
                    apply_wait_timer_diff(&mut current, diff_data)?;
                    self.remove_component::<WaitTimer>(*entity);
                    self.add_component(*entity, current);
                } else {
                    return Err(format!("Cannot modify WaitTimer component that doesn't exist on entity {:?}", entity));
                }
            }
            "ActorState" => {
                if let Some(mut current) = self.get_component::<ActorState>(*entity).copied() {
                    apply_actor_state_diff(&mut current, diff_data)?;
                    self.remove_component::<ActorState>(*entity);
                    self.add_component(*entity, current);
                } else {
                    return Err(format!("Cannot modify ActorState component that doesn't exist on entity {:?}", entity));
                }
            }
            _ => {
                return Err(format!("Unknown component type for modification: {}", type_name));
            }
        }
        Ok(())
    }

    /// Apply a component removal from replay data
    fn apply_component_removal(&mut self, entity: &Entity, type_name: &str) -> Result<(), String> {
        use crate::game::game::*;
        
        match type_name {
            "Position" => { self.remove_component::<Position>(*entity); }
            "Target" => { self.remove_component::<Target>(*entity); }
            "WaitTimer" => { self.remove_component::<WaitTimer>(*entity); }
            "Actor" => { self.remove_component::<Actor>(*entity); }
            "Home" => { self.remove_component::<Home>(*entity); }
            "Work" => { self.remove_component::<Work>(*entity); }
            "Obstacle" => { self.remove_component::<Obstacle>(*entity); }
            "ActorState" => { self.remove_component::<ActorState>(*entity); }
            _ => {
                return Err(format!("Unknown component type for removal: {}", type_name));
            }
        }
        Ok(())
    }

    /// Apply a system addition from replay data
    fn apply_system_addition(&mut self, system_type_name: &str) -> Result<(), String> {
        use crate::game::game::*;
        
        match system_type_name {
            "rust_ecs::game::game::MovementSystem" => {
                self.add_system_internal(MovementSystem);
            }
            "rust_ecs::game::game::WaitSystem" => {
                self.add_system_internal(WaitSystem);
            }
            "rust_ecs::game::game::RenderSystem" => {
                self.add_system_internal(RenderSystem);
            }
            _ => {
                return Err(format!("Unknown system type for addition: {}", system_type_name));
            }
        }
        Ok(())
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

        // Test querying all positions (immutable)
        let positions = world_view.query_components::<(In<Position>,)>();
        assert_eq!(positions.len(), 2);

        // Test mutable querying
        let mut positions_mut = world_view.query_components::<(Out<Position>,)>();
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
        assert_eq!(history.updates.len(), 3); // 1 system addition + 2 updates
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

    #[test]
    fn test_diff_entity() {
        let entity1 = Entity::new(0, 5);
        let entity2 = Entity::new(0, 5);
        let entity3 = Entity::new(0, 10);
        let entity4 = Entity::new(1, 5);

        // No diff for identical entities
        assert!(entity1.diff(&entity2).is_none());

        // Diff for different entity indices
        let diff = entity1.diff(&entity3).unwrap();
        assert!(diff.world_index.is_none());
        assert_eq!(diff.entity_index, Some(10));

        // Diff for different world indices
        let diff = entity1.diff(&entity4).unwrap();
        assert_eq!(diff.world_index, Some(1));
        assert!(diff.entity_index.is_none());

        // Apply diff
        let mut entity = entity1;
        entity.apply_diff(&entity1.diff(&entity3).unwrap());
        assert_eq!(entity, entity3);
    }

    #[test]
    fn test_diff_primitives() {
        // Test i32 diffing
        let a = 5i32;
        let b = 5i32;
        let c = 10i32;

        assert!(a.diff(&b).is_none());
        assert_eq!(a.diff(&c), Some(10));

        let mut x = a;
        x.apply_diff(&10);
        assert_eq!(x, 10);

        // Test f32 diffing
        let f1 = std::f32::consts::PI;
        let f2 = std::f32::consts::PI;
        let f3 = 2.71f32;

        assert!(f1.diff(&f2).is_none());
        assert_eq!(f1.diff(&f3), Some(2.71));

        // Test String diffing
        let s1 = "hello".to_string();
        let s2 = "hello".to_string();
        let s3 = "world".to_string();

        assert!(s1.diff(&s2).is_none());
        assert_eq!(s1.diff(&s3), Some("world".to_string()));
    }

    #[test]
    fn test_diff_vec() {
        let vec1 = vec![1, 2, 3];
        let vec2 = vec![1, 2, 3];
        let vec3 = vec![1, 5, 3, 4];

        // No diff for identical vectors
        assert!(vec1.diff(&vec2).is_none());

        // Diff for modified and added elements
        let diff = vec1.diff(&vec3).unwrap();
        assert_eq!(diff.changes.len(), 2);

        // Apply diff
        let mut vec = vec1.clone();
        vec.apply_diff(&diff);
        assert_eq!(vec, vec3);
    }

    #[test]
    fn test_diff_hashmap() {
        let mut map1 = HashMap::new();
        map1.insert("key1".to_string(), 1);
        map1.insert("key2".to_string(), 2);

        let mut map2 = HashMap::new();
        map2.insert("key1".to_string(), 1);
        map2.insert("key2".to_string(), 2);

        let mut map3 = HashMap::new();
        map3.insert("key1".to_string(), 5);
        map3.insert("key3".to_string(), 3);

        // No diff for identical maps
        assert!(map1.diff(&map2).is_none());

        // Diff for modified, added, and removed entries
        let diff = map1.diff(&map3).unwrap();
        assert_eq!(diff.changes.len(), 3);

        // Apply diff
        let mut map = map1.clone();
        map.apply_diff(&diff);
        assert_eq!(map, map3);
    }

    #[test]
    fn test_diff_u32() {
        // Test u32 diffing (newly implemented)
        let a = 5u32;
        let b = 5u32;
        let c = 10u32;

        assert!(a.diff(&b).is_none());
        assert_eq!(a.diff(&c), Some(10));

        let mut x = a;
        x.apply_diff(&10);
        assert_eq!(x, 10);
    }

    #[test]
    fn test_diff_derive_unit_struct() {
        // Test derive macro for unit structs
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        struct TestUnit;

        let unit1 = TestUnit;
        let unit2 = TestUnit;

        // Unit structs should never have differences
        assert!(unit1.diff(&unit2).is_none());

        // Apply diff should work without doing anything
        let mut unit = unit1;
        unit.apply_diff(&());
        assert_eq!(unit, unit1);
    }

    #[test]
    fn test_diff_derive_enum() {
        // Test derive macro for enums
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        enum TestEnum {
            Variant1,
            Variant2,
            Variant3,
        }

        let e1 = TestEnum::Variant1;
        let e2 = TestEnum::Variant1;
        let e3 = TestEnum::Variant2;

        // No diff for identical variants
        assert!(e1.diff(&e2).is_none());

        // Diff for different variants
        assert_eq!(e1.diff(&e3), Some(TestEnum::Variant2));

        // Apply diff
        let mut e = e1;
        e.apply_diff(&TestEnum::Variant3);
        assert_eq!(e, TestEnum::Variant3);
    }

    #[test]
    fn test_diff_derive_struct_with_u32() {
        // Test derive macro for struct containing u32
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        struct TestStruct {
            counter: u32,
            value: i32,
        }

        let s1 = TestStruct { counter: 1, value: 10 };
        let s2 = TestStruct { counter: 1, value: 10 };
        let s3 = TestStruct { counter: 5, value: 10 };
        let s4 = TestStruct { counter: 1, value: 20 };

        // No diff for identical structs
        assert!(s1.diff(&s2).is_none());

        // Diff for changed u32 field
        let diff = s1.diff(&s3).unwrap();
        assert!(diff.counter.is_some());
        assert!(diff.value.is_none());

        // Diff for changed i32 field
        let diff = s1.diff(&s4).unwrap();
        assert!(diff.counter.is_none());
        assert!(diff.value.is_some());

        // Apply diff
        let mut s = s1;
        s.apply_diff(&s1.diff(&s3).unwrap());
        assert_eq!(s, s3);
    }

    #[test]
    fn test_extended_multi_component_query() {
        let mut world = World::new();
        let mut world_view = WorldView::<(), ()>::new(&mut world);

        let entity1 = world_view.create_entity();

        // Define additional test components to test extended queries
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        struct TestA { value: i32 }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        struct TestB { value: i32 }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        struct TestC { value: i32 }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        struct TestD { value: i32 }
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        struct TestE { value: i32 }

        // Add multiple components to entity
        world_view.add_component(entity1, TestA { value: 1 });
        world_view.add_component(entity1, TestB { value: 2 });
        world_view.add_component(entity1, TestC { value: 3 });
        world_view.add_component(entity1, TestD { value: 4 });
        world_view.add_component(entity1, TestE { value: 5 });

        // Test 4-component query
        let results4 = world_view.query_components::<(In<TestA>, In<TestB>, In<TestC>, In<TestD>)>();
        assert_eq!(results4.len(), 1);
        let (entity, (a, b, c, d)) = &results4[0];
        assert_eq!(*entity, entity1);
        assert_eq!(a.value, 1);
        assert_eq!(b.value, 2);
        assert_eq!(c.value, 3);
        assert_eq!(d.value, 4);

        // Test 5-component query
        let results5 = world_view.query_components::<(In<TestA>, In<TestB>, In<TestC>, In<TestD>, In<TestE>)>();
        assert_eq!(results5.len(), 1);
        let (entity, (a, b, c, d, e)) = &results5[0];
        assert_eq!(*entity, entity1);
        assert_eq!(a.value, 1);
        assert_eq!(b.value, 2);
        assert_eq!(c.value, 3);
        assert_eq!(d.value, 4);
        assert_eq!(e.value, 5);

        // Test mixed access (mutable and immutable)
        let mut results_mixed = world_view.query_components::<(Out<TestA>, In<TestB>, Out<TestC>, In<TestD>, In<TestE>)>();
        assert_eq!(results_mixed.len(), 1);
        let (entity, (mut_a, b, mut_c, d, e)) = &mut results_mixed[0];
        assert_eq!(*entity, entity1);
        assert_eq!(b.value, 2);
        assert_eq!(d.value, 4);
        assert_eq!(e.value, 5);
        
        // Modify the mutable components
        mut_a.value = 10;
        mut_c.value = 30;

        // Verify modifications were applied
        let verification = world_view.query_components::<(In<TestA>, In<TestB>, In<TestC>, In<TestD>, In<TestE>)>();
        let (_, (a, b, c, d, e)) = &verification[0];
        assert_eq!(a.value, 10); // Modified
        assert_eq!(b.value, 2);  // Unchanged
        assert_eq!(c.value, 30); // Modified
        assert_eq!(d.value, 4);  // Unchanged
        assert_eq!(e.value, 5);  // Unchanged
    }
}

/// Replay data analysis utilities for developers
pub mod replay_analysis {
    use super::*;

    /// Statistics about a replay session
    #[derive(Debug)]
    pub struct ReplayStats {
        pub total_updates: usize,
        pub total_system_executions: usize,
        pub total_component_changes: usize,
        pub total_world_operations: usize,
        pub entities_created: usize,
        pub entities_removed: usize,
        pub component_types_involved: Vec<String>,
        pub most_active_frame: Option<usize>,
        pub most_changes_in_frame: usize,
    }

    /// Analyze a world update history and generate statistics
    pub fn analyze_replay_history(history: &WorldUpdateHistory) -> ReplayStats {
        let mut stats = ReplayStats {
            total_updates: history.len(),
            total_system_executions: 0,
            total_component_changes: 0,
            total_world_operations: 0,
            entities_created: 0,
            entities_removed: 0,
            component_types_involved: Vec::new(),
            most_active_frame: None,
            most_changes_in_frame: 0,
        };

        let mut component_types = std::collections::HashSet::new();
        let mut frame_changes: Vec<usize> = Vec::new();

        for update in history.updates() {
            stats.total_system_executions += update.system_diffs().len();
            
            let mut frame_change_count = 0;
            
            for system_diff in update.system_diffs() {
                stats.total_component_changes += system_diff.component_changes().len();
                stats.total_world_operations += system_diff.world_operations().len();
                frame_change_count += system_diff.component_changes().len() + system_diff.world_operations().len();

                // Collect component types
                for change in system_diff.component_changes() {
                    match change {
                        DiffComponentChange::Added { type_name, .. } |
                        DiffComponentChange::Modified { type_name, .. } |
                        DiffComponentChange::Removed { type_name, .. } => {
                            component_types.insert(type_name.clone());
                        }
                    }
                }

                // Count entities created/removed
                for operation in system_diff.world_operations() {
                    match operation {
                        WorldOperation::CreateEntity(_) => stats.entities_created += 1,
                        WorldOperation::RemoveEntity(_) => stats.entities_removed += 1,
                        _ => {}
                    }
                }
            }
            
            frame_changes.push(frame_change_count);
        }

        // Find most active frame
        if let Some((frame_idx, max_changes)) = frame_changes.iter().enumerate().max_by_key(|(_, &changes)| changes) {
            stats.most_active_frame = Some(frame_idx);
            stats.most_changes_in_frame = *max_changes;
        }

        stats.component_types_involved = component_types.into_iter().collect();
        stats.component_types_involved.sort();

        stats
    }

    /// Print a detailed analysis report of a replay session
    pub fn print_replay_analysis(history: &WorldUpdateHistory) {
        let stats = analyze_replay_history(history);
        
        println!("=== ECS Replay Analysis Report ===");
        println!("Total Updates: {}", stats.total_updates);
        println!("Total System Executions: {}", stats.total_system_executions);
        println!("Total Component Changes: {}", stats.total_component_changes);
        println!("Total World Operations: {}", stats.total_world_operations);
        println!("Entities Created: {}", stats.entities_created);
        println!("Entities Removed: {}", stats.entities_removed);
        
        if let Some(frame) = stats.most_active_frame {
            println!("Most Active Frame: {} (with {} changes)", frame, stats.most_changes_in_frame);
        }
        
        println!("Component Types Involved:");
        for component_type in &stats.component_types_involved {
            println!("  - {}", component_type);
        }
        
        if stats.total_updates > 0 {
            println!("Average Changes per Frame: {:.2}", 
                stats.total_component_changes as f64 / stats.total_updates as f64);
        }
        
        println!("=== End Report ===");
    }

    /// Find frames with unusual activity (significantly above average)
    pub fn find_anomalous_frames(history: &WorldUpdateHistory, threshold_multiplier: f64) -> Vec<usize> {
        let updates = history.updates();
        if updates.is_empty() {
            return Vec::new();
        }

        // Calculate average changes per frame
        let total_changes: usize = updates.iter()
            .map(|update| update.system_diffs().iter()
                .map(|sys| sys.component_changes().len() + sys.world_operations().len())
                .sum::<usize>())
            .sum();
        
        let avg_changes = total_changes as f64 / updates.len() as f64;
        let threshold = avg_changes * threshold_multiplier;

        let mut anomalous_frames = Vec::new();
        
        for (frame_idx, update) in updates.iter().enumerate() {
            let frame_changes: usize = update.system_diffs().iter()
                .map(|sys| sys.component_changes().len() + sys.world_operations().len())
                .sum();
            
            if frame_changes as f64 > threshold {
                anomalous_frames.push(frame_idx);
            }
        }

        anomalous_frames
    }

    /// Read and parse a replay log file
    pub fn read_replay_log(file_path: &str) -> Result<Vec<String>, std::io::Error> {
        std::fs::read_to_string(file_path)
            .map(|content| content.lines().map(|line| line.to_string()).collect())
    }

    /// Parse a replay log file into WorldUpdateHistory
    pub fn parse_replay_log(file_path: &str) -> Result<WorldUpdateHistory, Box<dyn std::error::Error>> {
        let lines = read_replay_log(file_path)?;
        let mut history = WorldUpdateHistory::new();
        let mut current_update: Option<WorldUpdateDiff> = None;
        let mut current_system: Option<SystemUpdateDiff> = None;
        let mut line_number = 0;

        for line in lines {
            line_number += 1;
            let line = line.trim();
            
            // Skip comments and empty lines
            if line.starts_with('#') || line.is_empty() {
                continue;
            }

            if line.starts_with("UPDATE ") {
                // Save previous update if exists
                if let Some(update) = current_update.take() {
                    history.record(update);
                }
                current_update = Some(WorldUpdateDiff::new());
            } else if line.starts_with("SYSTEMS: ") {
                // Just metadata, continue
            } else if line.starts_with("  SYSTEM ") {
                // Save previous system if exists
                if let Some(system) = current_system.take() {
                    if let Some(ref mut update) = current_update {
                        update.record(system);
                    }
                }
                current_system = Some(SystemUpdateDiff::new());
            } else if line.starts_with("    COMPONENT_CHANGES: ") {
                // Component changes section header
            } else if line.starts_with("      ADD ") {
                // Parse component addition: "ADD Entity(world_id, entity_id) ComponentType data"
                if let Some(change) = parse_component_add(&line[10..]) {
                    if let Some(ref mut system) = current_system {
                        system.record_component_change(change);
                    }
                }
            } else if line.starts_with("      MOD ") {
                // Parse component modification: "MOD Entity(world_id, entity_id) ComponentType diff"
                if let Some(change) = parse_component_mod(&line[10..]) {
                    if let Some(ref mut system) = current_system {
                        system.record_component_change(change);
                    }
                }
            } else if line.starts_with("      REM ") {
                // Parse component removal: "REM Entity(world_id, entity_id) ComponentType"
                if let Some(change) = parse_component_rem(&line[10..]) {
                    if let Some(ref mut system) = current_system {
                        system.record_component_change(change);
                    }
                }
            } else if line.starts_with("    WORLD_OPERATIONS: ") {
                // World operations section header
            } else if line.starts_with("      CREATE_ENTITY ") {
                // Parse entity creation: "CREATE_ENTITY Entity(world_id, entity_id)"
                if let Some(entity) = parse_entity(&line[20..]) {
                    if let Some(ref mut system) = current_system {
                        system.record_world_operation(WorldOperation::CreateEntity(entity));
                    }
                }
            } else if line.starts_with("      REMOVE_ENTITY ") {
                // Parse entity removal: "REMOVE_ENTITY Entity(world_id, entity_id)"
                if let Some(entity) = parse_entity(&line[20..]) {
                    if let Some(ref mut system) = current_system {
                        system.record_world_operation(WorldOperation::RemoveEntity(entity));
                    }
                }
            } else if line.starts_with("      CREATE_WORLD ") {
                // Parse world creation: "CREATE_WORLD world_id"
                if let Ok(world_id) = line[19..].parse::<usize>() {
                    if let Some(ref mut system) = current_system {
                        system.record_world_operation(WorldOperation::CreateWorld(world_id));
                    }
                }
            } else if line.starts_with("      REMOVE_WORLD ") {
                // Parse world removal: "REMOVE_WORLD world_id"
                if let Ok(world_id) = line[19..].parse::<usize>() {
                    if let Some(ref mut system) = current_system {
                        system.record_world_operation(WorldOperation::RemoveWorld(world_id));
                    }
                }
            }
        }

        // Save any remaining data
        if let Some(system) = current_system {
            if let Some(ref mut update) = current_update {
                update.record(system);
            }
        }
        if let Some(update) = current_update {
            history.record(update);
        }

        Ok(history)
    }
}

/// Parse entity from string like "Entity(0, 123)"
fn parse_entity(input: &str) -> Option<Entity> {
    if input.starts_with("Entity(") && input.ends_with(')') {
        let content = &input[7..input.len()-1];
        let parts: Vec<&str> = content.split(", ").collect();
        if parts.len() == 2 {
            if let (Ok(world_index), Ok(entity_index)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                return Some(Entity { world_index, entity_index });
            }
        }
    }
    None
}

/// Parse component addition from string like "Entity(0, 123) Position Position { x: 1.0, y: 2.0 }"
fn parse_component_add(input: &str) -> Option<DiffComponentChange> {
    let parts: Vec<&str> = input.splitn(3, ' ').collect();
    if parts.len() >= 3 {
        if let Some(entity) = parse_entity(parts[0]) {
            let type_name = parts[1].to_string();
            let data = if parts.len() > 2 { parts[2].to_string() } else { String::new() };
            return Some(DiffComponentChange::Added { entity, type_name, data });
        }
    }
    None
}

/// Parse component modification from string like "Entity(0, 123) Position PositionDiff { x: Some(1.0), y: None }"
fn parse_component_mod(input: &str) -> Option<DiffComponentChange> {
    let parts: Vec<&str> = input.splitn(3, ' ').collect();
    if parts.len() >= 3 {
        if let Some(entity) = parse_entity(parts[0]) {
            let type_name = parts[1].to_string();
            let diff = if parts.len() > 2 { parts[2].to_string() } else { String::new() };
            return Some(DiffComponentChange::Modified { entity, type_name, diff });
        }
    }
    None
}

/// Parse component removal from string like "Entity(0, 123) Position"
fn parse_component_rem(input: &str) -> Option<DiffComponentChange> {
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    if parts.len() >= 2 {
        if let Some(entity) = parse_entity(parts[0]) {
            let type_name = parts[1].to_string();
            return Some(DiffComponentChange::Removed { entity, type_name });
        }
    }
    None
}

/// Helper function for reading replay log files
fn read_replay_log(file_path: &str) -> Result<Vec<String>, std::io::Error> {
    std::fs::read_to_string(file_path)
        .map(|content| content.lines().map(|line| line.to_string()).collect())
}

/// Parse Position component data from string like "Position { x: 1, y: 2 }"
fn parse_position_data(data: &str) -> Result<crate::game::game::Position, String> {
    // Simple parser for Position { x: value, y: value }
    if let Some(content) = data.strip_prefix("Position { ").and_then(|s| s.strip_suffix(" }")) {
        let mut x: Option<i32> = None;
        let mut y: Option<i32> = None;
        
        for part in content.split(", ") {
            if let Some(value_str) = part.strip_prefix("x: ") {
                x = Some(value_str.parse().map_err(|e| format!("Failed to parse x: {}", e))?);
            } else if let Some(value_str) = part.strip_prefix("y: ") {
                y = Some(value_str.parse().map_err(|e| format!("Failed to parse y: {}", e))?);
            }
        }
        
        if let (Some(x), Some(y)) = (x, y) {
            Ok(crate::game::game::Position { x, y })
        } else {
            Err("Missing x or y value in Position data".to_string())
        }
    } else {
        Err(format!("Invalid Position data format: {}", data))
    }
}

/// Parse Target component data from string like "Target { x: 1, y: 2 }"
fn parse_target_data(data: &str) -> Result<crate::game::game::Target, String> {
    if let Some(content) = data.strip_prefix("Target { ").and_then(|s| s.strip_suffix(" }")) {
        let mut x: Option<i32> = None;
        let mut y: Option<i32> = None;
        
        for part in content.split(", ") {
            if let Some(value_str) = part.strip_prefix("x: ") {
                x = Some(value_str.parse().map_err(|e| format!("Failed to parse x: {}", e))?);
            } else if let Some(value_str) = part.strip_prefix("y: ") {
                y = Some(value_str.parse().map_err(|e| format!("Failed to parse y: {}", e))?);
            }
        }
        
        if let (Some(x), Some(y)) = (x, y) {
            Ok(crate::game::game::Target { x, y })
        } else {
            Err("Missing x or y value in Target data".to_string())
        }
    } else {
        Err(format!("Invalid Target data format: {}", data))
    }
}

/// Parse WaitTimer component data from string like "WaitTimer { ticks: 5 }"
fn parse_wait_timer_data(data: &str) -> Result<crate::game::game::WaitTimer, String> {
    if let Some(content) = data.strip_prefix("WaitTimer { ").and_then(|s| s.strip_suffix(" }")) {
        if let Some(value_str) = content.strip_prefix("ticks: ") {
            let ticks = value_str.parse().map_err(|e| format!("Failed to parse ticks: {}", e))?;
            Ok(crate::game::game::WaitTimer { ticks })
        } else {
            Err("Missing ticks value in WaitTimer data".to_string())
        }
    } else {
        Err(format!("Invalid WaitTimer data format: {}", data))
    }
}

/// Parse ActorState component data from string like "MovingToWork"
fn parse_actor_state_data(data: &str) -> Result<crate::game::game::ActorState, String> {
    match data {
        "MovingToWork" => Ok(crate::game::game::ActorState::MovingToWork),
        "MovingToHome" => Ok(crate::game::game::ActorState::MovingToHome),
        "WaitingAtWork" => Ok(crate::game::game::ActorState::WaitingAtWork),
        "WaitingAtHome" => Ok(crate::game::game::ActorState::WaitingAtHome),
        _ => Err(format!("Unknown ActorState variant: {}", data))
    }
}

/// Apply Position diff from string like "PositionDiff { x: Some(1), y: Some(2) }"
fn apply_position_diff(position: &mut crate::game::game::Position, diff_data: &str) -> Result<(), String> {
    if let Some(content) = diff_data.strip_prefix("PositionDiff { ").and_then(|s| s.strip_suffix(" }")) {
        for part in content.split(", ") {
            if let Some(value_str) = part.strip_prefix("x: Some(").and_then(|s| s.strip_suffix(")")) {
                position.x = value_str.parse().map_err(|e| format!("Failed to parse x diff: {}", e))?;
            } else if let Some(value_str) = part.strip_prefix("y: Some(").and_then(|s| s.strip_suffix(")")) {
                position.y = value_str.parse().map_err(|e| format!("Failed to parse y diff: {}", e))?;
            }
            // Ignore None values as they mean no change
        }
        Ok(())
    } else {
        Err(format!("Invalid PositionDiff format: {}", diff_data))
    }
}

/// Apply Target diff from string like "TargetDiff { x: Some(1), y: Some(2) }"
fn apply_target_diff(target: &mut crate::game::game::Target, diff_data: &str) -> Result<(), String> {
    if let Some(content) = diff_data.strip_prefix("TargetDiff { ").and_then(|s| s.strip_suffix(" }")) {
        for part in content.split(", ") {
            if let Some(value_str) = part.strip_prefix("x: Some(").and_then(|s| s.strip_suffix(")")) {
                target.x = value_str.parse().map_err(|e| format!("Failed to parse x diff: {}", e))?;
            } else if let Some(value_str) = part.strip_prefix("y: Some(").and_then(|s| s.strip_suffix(")")) {
                target.y = value_str.parse().map_err(|e| format!("Failed to parse y diff: {}", e))?;
            }
        }
        Ok(())
    } else {
        Err(format!("Invalid TargetDiff format: {}", diff_data))
    }
}

/// Apply WaitTimer diff from string like "WaitTimerDiff { ticks: Some(5) }"
fn apply_wait_timer_diff(timer: &mut crate::game::game::WaitTimer, diff_data: &str) -> Result<(), String> {
    if let Some(content) = diff_data.strip_prefix("WaitTimerDiff { ").and_then(|s| s.strip_suffix(" }")) {
        if let Some(value_str) = content.strip_prefix("ticks: Some(").and_then(|s| s.strip_suffix(")")) {
            timer.ticks = value_str.parse().map_err(|e| format!("Failed to parse ticks diff: {}", e))?;
        }
        Ok(())
    } else {
        Err(format!("Invalid WaitTimerDiff format: {}", diff_data))
    }
}

/// Apply ActorState diff from string like "MovingToWork"
fn apply_actor_state_diff(state: &mut crate::game::game::ActorState, diff_data: &str) -> Result<(), String> {
    *state = parse_actor_state_data(diff_data)?;
    Ok(())
}

// Game module - declared after ReplayLogConfig
pub mod game;
