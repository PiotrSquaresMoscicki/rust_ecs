//! Component querying functionality
//!
//! This module provides the query system for accessing components in the ECS.

use crate::ecs::core::{Entity, In, Not, Out};
use std::collections::HashSet;

/// Trait for component types that can be queried immutably
pub trait QueryComponent<'a> {
    type Item;

    fn get_component(world: &'a crate::ecs::world::World, entity: Entity) -> Option<Self::Item>;
}

/// Trait for components that can appear in multi-component queries
pub trait MixedQueryComponent<'a> {
    type Item;

    fn get_mixed_component(
        world: &'a mut crate::ecs::world::World,
        entity: Entity,
    ) -> Option<Self::Item>;
}

/// Trait for multi-component queries
pub trait MixedMultiQuery<'a> {
    type Item;

    fn query_mixed(world: &'a mut crate::ecs::world::World) -> Vec<(Entity, Self::Item)>;
}

// Implement QueryComponent for In<T> wrapper specifically
impl<'a, T: 'static> QueryComponent<'a> for In<T> {
    type Item = &'a T;

    fn get_component(_world: &'a crate::ecs::world::World, _entity: Entity) -> Option<Self::Item> {
        // This is handled specially in WorldView
        unreachable!("In<T> queries should be handled by WorldView")
    }
}

// Implement QueryComponent for mutable component access through Out<T>
impl<'a, T: 'static> QueryComponent<'a> for Out<T> {
    type Item = &'a mut T;

    fn get_component(_world: &'a crate::ecs::world::World, _entity: Entity) -> Option<Self::Item> {
        // This is handled specially in WorldView
        unreachable!("Out<T> queries should be handled by WorldView")
    }
}

// Implement MixedQueryComponent for In<T> wrapper specifically
impl<'a, T: 'static> MixedQueryComponent<'a> for In<T> {
    type Item = &'a T;

    fn get_mixed_component(
        world: &'a mut crate::ecs::world::World,
        entity: Entity,
    ) -> Option<Self::Item> {
        world.get_component::<T>(entity)
    }
}

// Implement MixedQueryComponent for mutable component access through Out<T>
impl<'a, T: 'static> MixedQueryComponent<'a> for Out<T> {
    type Item = &'a mut T;

    fn get_mixed_component(
        world: &'a mut crate::ecs::world::World,
        entity: Entity,
    ) -> Option<Self::Item> {
        use std::any::TypeId;

        // First check regular components
        if let Some(component) = world
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
        {
            return Some(component);
        }

        // Then check temporary components (Events, ComponentAdded, ComponentRemoved)
        world
            .temporary_components
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

// Implement MixedQueryComponent for Not<T> - returns () when component is NOT present
impl<'a, T: 'static> MixedQueryComponent<'a> for Not<T> {
    type Item = ();

    fn get_mixed_component(
        world: &'a mut crate::ecs::world::World,
        entity: Entity,
    ) -> Option<Self::Item> {
        use std::any::TypeId;

        // Check if the component exists in regular components
        let has_regular_component = world
            .components
            .get(&TypeId::of::<T>())
            .map(|components| components.iter().any(|(e, _)| *e == entity))
            .unwrap_or(false);

        // Check if the component exists in temporary components
        let has_temporary_component = world
            .temporary_components
            .get(&TypeId::of::<T>())
            .map(|components| components.iter().any(|(e, _)| *e == entity))
            .unwrap_or(false);

        // Return Some(()) if the component does NOT exist in either storage
        if !has_regular_component && !has_temporary_component {
            Some(())
        } else {
            None
        }
    }
}

// Concrete implementations for 1 component
impl<'a, A> MixedMultiQuery<'a> for (A,)
where
    A: MixedQueryComponent<'a> + 'static,
{
    type Item = A::Item;

    fn query_mixed(world: &'a mut crate::ecs::world::World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();

        // Get all entities from both regular and temporary component storage
        let mut all_entities = std::collections::HashSet::new();

        // Add entities from regular entity list
        for entity in &world.entities {
            all_entities.insert(*entity);
        }

        // Add entities that have temporary components
        for components in world.temporary_components.values() {
            for (entity, _) in components {
                all_entities.insert(*entity);
            }
        }

        for entity in all_entities {
            unsafe {
                let world_ptr = world as *mut crate::ecs::world::World;
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

    fn query_mixed(world: &'a mut crate::ecs::world::World) -> Vec<(Entity, Self::Item)> {
        let mut results = Vec::new();

        // Get all entities from both regular and temporary component storage
        let mut all_entities = HashSet::new();

        // Add entities from regular entity list
        for entity in &world.entities {
            all_entities.insert(*entity);
        }

        // Add entities that have temporary components
        for components in world.temporary_components.values() {
            for (entity, _) in components {
                all_entities.insert(*entity);
            }
        }

        for entity in all_entities {
            unsafe {
                let world_ptr = world as *mut crate::ecs::world::World;
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

// Implementation macro for generating query implementations for multiple component counts
// Note: The generic parameter names (A, B, C, etc.) are intentionally uppercase as they represent
// type parameters in generic implementations. The snake_case warning is suppressed for this macro.
#[allow(non_snake_case)]
macro_rules! impl_mixed_multi_query {
    ($($generic:ident),+) => {
        #[allow(non_snake_case)]
        impl<'a, $($generic),+> MixedMultiQuery<'a> for ($($generic,)+)
        where
            $(
                $generic: MixedQueryComponent<'a> + 'static,
            )+
        {
            type Item = ($($generic::Item,)+);

            fn query_mixed(world: &'a mut crate::ecs::world::World) -> Vec<(Entity, Self::Item)> {
                let mut results = Vec::new();

                // Get all entities from both regular and temporary component storage
                let mut all_entities = HashSet::new();

                // Add entities from regular entity list
                for entity in &world.entities {
                    all_entities.insert(*entity);
                }

                // Add entities that have temporary components
                for components in world.temporary_components.values() {
                    for (entity, _) in components {
                        all_entities.insert(*entity);
                    }
                }

                for entity in all_entities {
                    unsafe {
                        let world_ptr = world as *mut crate::ecs::world::World;
                        $(
                            let $generic = $generic::get_mixed_component(&mut *world_ptr, entity);
                        )+

                        if let ($(Some($generic),)+) = ($($generic,)+) {
                            results.push((entity, ($($generic,)+)));
                        }
                    }
                }

                results
            }
        }
    };
}

// Generate implementations for up to 16 components
// These macro invocations use uppercase type parameters as part of the macro design pattern.
// The non_snake_case warning is suppressed within the macro definition.
impl_mixed_multi_query!(A, B, C);
impl_mixed_multi_query!(A, B, C, D);
impl_mixed_multi_query!(A, B, C, D, E);
impl_mixed_multi_query!(A, B, C, D, E, F);
impl_mixed_multi_query!(A, B, C, D, E, F, G);
impl_mixed_multi_query!(A, B, C, D, E, F, G, H);
impl_mixed_multi_query!(A, B, C, D, E, F, G, H, I);
impl_mixed_multi_query!(A, B, C, D, E, F, G, H, I, J);
impl_mixed_multi_query!(A, B, C, D, E, F, G, H, I, J, K);
impl_mixed_multi_query!(A, B, C, D, E, F, G, H, I, J, K, L);
impl_mixed_multi_query!(A, B, C, D, E, F, G, H, I, J, K, L, M);
impl_mixed_multi_query!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
impl_mixed_multi_query!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
impl_mixed_multi_query!(A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
