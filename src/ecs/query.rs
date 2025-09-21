//! Component querying functionality
//!
//! This module provides the query system for accessing components in the ECS.

use std::marker::PhantomData;
use crate::ecs::core::{Entity, Out, In};

/// Trait for component types that can be queried immutably
pub trait QueryComponent<'a> {
    type Item;

    fn get_component(world: &'a crate::ecs::World, entity: Entity) -> Option<Self::Item>;
}

/// Trait for components that can appear in multi-component queries
pub trait MixedQueryComponent<'a> {
    type Item;

    fn get_mixed_component(world: &'a crate::ecs::World, entity: Entity) -> Option<Self::Item>;
}

/// Trait for multi-component queries
pub trait MixedMultiQuery<'a> {
    type Item;

    fn query_mixed(world: &'a crate::ecs::World) -> Vec<(Entity, Self::Item)>;
}

// Implement QueryComponent for direct component access
impl<'a, T: 'static> QueryComponent<'a> for T {
    type Item = &'a T;

    fn get_component(world: &'a crate::ecs::World, entity: Entity) -> Option<Self::Item> {
        world.get_component::<T>(entity)
    }
}

// Implement QueryComponent for mutable component access through Out<T>
impl<'a, T: 'static> QueryComponent<'a> for Out<T> {
    type Item = &'a mut T;

    fn get_component(world: &'a crate::ecs::World, entity: Entity) -> Option<Self::Item> {
        // This is handled specially in WorldView
        unreachable!("Out<T> queries should be handled by WorldView")
    }
}

// Implement MixedQueryComponent for direct component access
impl<'a, T: 'static> MixedQueryComponent<'a> for T {
    type Item = &'a T;

    fn get_mixed_component(world: &'a crate::ecs::World, entity: Entity) -> Option<Self::Item> {
        world.get_component::<T>(entity)
    }
}

// Implement MixedQueryComponent for mutable component access through Out<T>
impl<'a, T: 'static> MixedQueryComponent<'a> for Out<T> {
    type Item = &'a mut T;

    fn get_mixed_component(world: &'a crate::ecs::World, entity: Entity) -> Option<Self::Item> {
        // This is handled specially in WorldView
        unreachable!("Out<T> queries should be handled by WorldView")
    }
}

// Implement MixedMultiQuery for single component
impl<'a, A> MixedMultiQuery<'a> for A
where
    A: MixedQueryComponent<'a> + 'static,
{
    type Item = A::Item;

    fn query_mixed(world: &'a crate::ecs::World) -> Vec<(Entity, Self::Item)> {
        // Implementation would be in WorldView
        unreachable!("MixedMultiQuery should be implemented in WorldView")
    }
}

// Implement MixedMultiQuery for tuples of components (up to 15 components)
macro_rules! impl_mixed_multi_query {
    ($($component:ident),+) => {
        impl<'a, $($component),+> MixedMultiQuery<'a> for ($($component,)+)
        where
            $($component: MixedQueryComponent<'a> + 'static,)+
        {
            type Item = ($($component::Item,)+);

            fn query_mixed(world: &'a crate::ecs::World) -> Vec<(Entity, Self::Item)> {
                // Implementation would be in WorldView
                unreachable!("MixedMultiQuery should be implemented in WorldView")
            }
        }
    };
}

// Generate implementations for tuples of 2 to 15 components
impl_mixed_multi_query!(A, B);
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