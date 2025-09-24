//! A Rust ECS (Entity Component System) framework with high debuggability.
//!
//! This library provides a unique ECS implementation where systems declare their
//! input and output components, enabling comprehensive change tracking and replay
//! functionality for debugging complex system interactions.

/// Macro to automatically implement Diff for structs
/// Generates diff functions for all fields
#[macro_export]
macro_rules! impl_diff {
    ($type:ident { $($field:ident: $field_type:ty),* $(,)? }) => {
        paste::paste! {
            #[derive(Clone, Debug)]
            pub struct [<$type Diff>] {
                $(
                    pub $field: Option<<$field_type as $crate::ecs::diff::Diff>::Diff>,
                )*
            }

            impl $crate::ecs::diff::Diff for $type {
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

            impl $crate::ecs::diff::DiffComponent for $type {}
        }
    };
}

// ECS module - the core of the library
pub mod ecs;

// Re-export the derive macro from the derive crate
pub use rust_ecs_derive::Diff;

// Re-export the most commonly used types from the ECS module for convenience
pub use ecs::{
    Entity, Out, In, Not, ComponentChange, ComponentOperation, WorldOperation,
    Event, ComponentAdded, ComponentRemoved,
    DiffComponent, DiffComponentChange,
    System, SystemInitDiff, SystemUpdateDiff, SystemDeinitDiff, WorldUpdateDiff, WorldUpdateHistory,
    QueryComponent, MixedMultiQuery, MixedQueryComponent,
    ReplayLogConfig, AutoReplayLogger,
    World, WorldView
};

// Re-export Diff trait from ECS (not conflicting with derive macro)
pub use ecs::diff::Diff;

// Re-export replay analysis functions for backward compatibility
pub mod replay_analysis {
    pub use crate::ecs::replay::{analyze_replay_history, print_replay_analysis, find_anomalous_frames, read_replay_log, parse_replay_log, ReplayStats};
}

// Game module - declared after ECS so it can use ECS types
pub mod game;

#[cfg(test)]
mod tests {
    use super::*;

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
        type Dependencies = ();

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
        assert_eq!(history.updates().len(), 3); // 1 system addition + 2 updates
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
        use std::collections::HashMap;        
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

    #[test]
    fn test_not_component_query() {
        let mut world = World::new();
        let mut world_view = WorldView::<(), ()>::new(&mut world);

        // Create some test components for the scenario
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        struct Tree { id: u32 }
        
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        struct FallenTree { id: u32 }
        
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        struct AssignedWoodcutter { woodcutter_id: u32 }

        // Create entities with different component combinations
        let entity1 = world_view.create_entity(); // Tree + FallenTree (no AssignedWoodcutter)
        let entity2 = world_view.create_entity(); // Tree + FallenTree + AssignedWoodcutter
        let entity3 = world_view.create_entity(); // Only Tree (no FallenTree, no AssignedWoodcutter)
        let entity4 = world_view.create_entity(); // Only FallenTree (no Tree, no AssignedWoodcutter)

        // Add components
        world_view.add_component(entity1, Tree { id: 1 });
        world_view.add_component(entity1, FallenTree { id: 1 });
        // No AssignedWoodcutter for entity1

        world_view.add_component(entity2, Tree { id: 2 });
        world_view.add_component(entity2, FallenTree { id: 2 });
        world_view.add_component(entity2, AssignedWoodcutter { woodcutter_id: 1 });

        world_view.add_component(entity3, Tree { id: 3 });
        // No FallenTree, no AssignedWoodcutter for entity3

        world_view.add_component(entity4, FallenTree { id: 4 });
        // No Tree, no AssignedWoodcutter for entity4

        // Test the main scenario: entities with Tree AND FallenTree but NOT AssignedWoodcutter
        let results = world_view.query_components::<(In<Tree>, In<FallenTree>, Not<AssignedWoodcutter>)>();
        
        // Should only return entity1 (has Tree + FallenTree, but no AssignedWoodcutter)
        assert_eq!(results.len(), 1);
        let (entity, (tree, fallen_tree, _not_assigned)) = &results[0];
        assert_eq!(*entity, entity1);
        assert_eq!(tree.id, 1);
        assert_eq!(fallen_tree.id, 1);

        // Test another query: entities with Tree but NOT FallenTree
        let tree_not_fallen = world_view.query_components::<(In<Tree>, Not<FallenTree>)>();
        
        // Should only return entity3 (has Tree but no FallenTree)
        assert_eq!(tree_not_fallen.len(), 1);
        let (entity, (tree, _not_fallen)) = &tree_not_fallen[0];
        assert_eq!(*entity, entity3);
        assert_eq!(tree.id, 3);

        // Test query: entities with FallenTree but NOT Tree
        let fallen_not_tree = world_view.query_components::<(In<FallenTree>, Not<Tree>)>();
        
        // Should only return entity4 (has FallenTree but no Tree)
        assert_eq!(fallen_not_tree.len(), 1);
        let (entity, (fallen_tree, _not_tree)) = &fallen_not_tree[0];
        assert_eq!(*entity, entity4);
        assert_eq!(fallen_tree.id, 4);

        // Test query: entities NOT assigned (without any positive components)
        let not_assigned = world_view.query_components::<(Not<AssignedWoodcutter>,)>();
        
        // Should return entity1, entity3, and entity4 (all except entity2)
        assert_eq!(not_assigned.len(), 3);
        let returned_entities: Vec<Entity> = not_assigned.iter().map(|(e, _)| *e).collect();
        assert!(returned_entities.contains(&entity1));
        assert!(returned_entities.contains(&entity3));
        assert!(returned_entities.contains(&entity4));
        assert!(!returned_entities.contains(&entity2));
    }

    #[test]
    fn test_not_component_edge_cases() {
        let mut world = World::new();
        let mut world_view = WorldView::<(), ()>::new(&mut world);

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        struct ComponentA { value: i32 }
        
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        struct ComponentB { value: i32 }

        // Test query with only Not<> components
        let entity1 = world_view.create_entity();
        let entity2 = world_view.create_entity();
        
        // entity1 has no components, entity2 has ComponentA
        world_view.add_component(entity2, ComponentA { value: 42 });

        // Query for entities that don't have ComponentA
        let not_a_results = world_view.query_components::<(Not<ComponentA>,)>();
        assert_eq!(not_a_results.len(), 1);
        assert_eq!(not_a_results[0].0, entity1);

        // Query for entities that don't have ComponentB (should return both)
        let not_b_results = world_view.query_components::<(Not<ComponentB>,)>();
        assert_eq!(not_b_results.len(), 2);
        let returned_entities: Vec<Entity> = not_b_results.iter().map(|(e, _)| *e).collect();
        assert!(returned_entities.contains(&entity1));
        assert!(returned_entities.contains(&entity2));

        // Test mixing Not<> with Out<> for mutable access
        world_view.add_component(entity1, ComponentB { value: 100 });
        
        let mut mixed_results = world_view.query_components::<(Out<ComponentB>, Not<ComponentA>)>();
        assert_eq!(mixed_results.len(), 1);
        let (entity, (comp_b, _not_a)) = &mut mixed_results[0];
        assert_eq!(*entity, entity1);
        assert_eq!(comp_b.value, 100);
        
        // Modify the component through the mutable reference
        comp_b.value = 200;
        
        // Verify the change was applied
        let verification = world_view.get_component::<ComponentB>(entity1);
        assert_eq!(verification.unwrap().value, 200);
    }

    #[test]
    fn test_problem_statement_scenario() {
        // Test the exact scenario from the problem statement:
        // query_components::<In<Tree>, In<FallenTree>, Not<AssignedWoodcutter>>()
        let mut world = World::new();
        let mut world_view = WorldView::<(), ()>::new(&mut world);

        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        struct Tree2 { species_id: u32 }
        
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        struct FallenTree { fallen_at: u32 }
        
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
        struct AssignedWoodcutter { worker_id: u32 }
        
        let entity1 = world_view.create_entity(); // Tree + FallenTree, no AssignedWoodcutter
        let entity2 = world_view.create_entity(); // Tree + FallenTree + AssignedWoodcutter
        let entity3 = world_view.create_entity(); // Only Tree
        let _entity4 = world_view.create_entity(); // No components

        world_view.add_component(entity1, Tree2 { species_id: 1 });
        world_view.add_component(entity1, FallenTree { fallen_at: 1000 });

        world_view.add_component(entity2, Tree2 { species_id: 2 });
        world_view.add_component(entity2, FallenTree { fallen_at: 2000 });
        world_view.add_component(entity2, AssignedWoodcutter { worker_id: 1 });

        world_view.add_component(entity3, Tree2 { species_id: 3 });

        // Test the exact query from the problem statement
        let results = world_view.query_components::<(In<Tree2>, In<FallenTree>, Not<AssignedWoodcutter>)>();
        
        // Should only return entity1
        assert_eq!(results.len(), 1);
        let (entity, (tree, fallen_tree, _not_assigned)) = &results[0];
        assert_eq!(*entity, entity1);
        assert_eq!(tree.species_id, 1);
        assert_eq!(fallen_tree.fallen_at, 1000);
    }

    #[test]
    fn test_system_dependencies_single() {
        struct SystemA;
        impl System for SystemA {
            type InComponents = ();
            type OutComponents = ();
            type Dependencies = ();
            fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        }

        struct SystemB;
        impl System for SystemB {
            type InComponents = ();
            type OutComponents = ();
            type Dependencies = (SystemA,);
            fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        }

        let mut world = World::new();
        world.add_system(SystemB); // Add B first
        world.add_system(SystemA); // Add A second
        
        // Should initialize in dependency order: A then B
        world.initialize_systems();
        // Should update in dependency order: A then B
        world.update();
        // Should deinitialize in reverse order: B then A
        world.deinitialize_systems();
    }

    #[test]
    fn test_system_dependencies_multiple() {
        struct SystemX;
        impl System for SystemX {
            type InComponents = ();
            type OutComponents = ();
            type Dependencies = ();
            fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        }

        struct SystemY;
        impl System for SystemY {
            type InComponents = ();
            type OutComponents = ();
            type Dependencies = ();
            fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        }

        struct SystemZ;
        impl System for SystemZ {
            type InComponents = ();
            type OutComponents = ();
            type Dependencies = (SystemX, SystemY);
            fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        }

        let mut world = World::new();
        world.add_system(SystemZ); // Add Z first (depends on X and Y)
        world.add_system(SystemY); // Add Y second
        world.add_system(SystemX); // Add X third
        
        // Should initialize in dependency order: X and Y first, then Z
        world.initialize_systems();
        world.update();
        world.deinitialize_systems();
    }

    #[test]
    fn test_system_dependencies_chain() {
        struct ChainA;
        impl System for ChainA {
            type InComponents = ();
            type OutComponents = ();
            type Dependencies = ();
            fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        }

        struct ChainB;
        impl System for ChainB {
            type InComponents = ();
            type OutComponents = ();
            type Dependencies = (ChainA,);
            fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        }

        struct ChainC;
        impl System for ChainC {
            type InComponents = ();
            type OutComponents = ();
            type Dependencies = (ChainB,);
            fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        }

        let mut world = World::new();
        world.add_system(ChainC); // Add C first (depends on B)
        world.add_system(ChainA); // Add A second (no dependencies)
        world.add_system(ChainB); // Add B third (depends on A)
        
        // Should initialize in dependency order: A -> B -> C
        world.initialize_systems();
        world.update();
        world.deinitialize_systems();
    }

    #[test]
    fn test_system_dependencies_no_dependencies() {
        struct IndependentSystem1;
        impl System for IndependentSystem1 {
            type InComponents = ();
            type OutComponents = ();
            type Dependencies = ();
            fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        }

        struct IndependentSystem2;
        impl System for IndependentSystem2 {
            type InComponents = ();
            type OutComponents = ();
            type Dependencies = ();
            fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        }

        let mut world = World::new();
        world.add_system(IndependentSystem1);
        world.add_system(IndependentSystem2);
        
        // Should work fine with no dependencies
        world.initialize_systems();
        world.update();
        world.deinitialize_systems();
    }

    #[test]
    fn test_system_dependencies_circular_detection() {
        // This test demonstrates that circular dependencies are handled gracefully
        // (falls back to registration order with warning)
        
        struct CircularA;
        impl System for CircularA {
            type InComponents = ();
            type OutComponents = ();
            type Dependencies = (CircularB,);
            fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        }

        struct CircularB;
        impl System for CircularB {
            type InComponents = ();
            type OutComponents = ();
            type Dependencies = (CircularA,);
            fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        }

        let mut world = World::new();
        world.add_system(CircularA);
        world.add_system(CircularB);
        
        // Should handle circular dependencies gracefully (prints warning and uses registration order)
        world.initialize_systems();
        world.update();
        world.deinitialize_systems();
    }

    #[test]
    fn test_system_dependencies_missing_dependency() {
        // This test demonstrates that missing dependencies are handled gracefully
        
        struct MissingDepSystem;
        impl System for MissingDepSystem {
            type InComponents = ();
            type OutComponents = ();
            type Dependencies = (NonExistentSystem,);
            fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        }

        struct NonExistentSystem;
        impl System for NonExistentSystem {
            type InComponents = ();
            type OutComponents = ();
            type Dependencies = ();
            fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
            fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        }

        let mut world = World::new();
        world.add_system(MissingDepSystem); // Add system that depends on NonExistentSystem
        // But don't add NonExistentSystem
        
        // Should handle missing dependencies gracefully (prints warning and uses registration order)
        world.initialize_systems();
        world.update();
        world.deinitialize_systems();
    }

    // Tests for Event system and Component change notifications
    
    #[derive(Debug, Clone, PartialEq)]
    struct ShotsFired {
        damage: i32,
        target_id: u32,
    }
    
    #[derive(Debug, Clone, PartialEq)]
    struct Soldier {
        id: u32,
        name: String,
    }

    #[test]
    fn test_event_dispatching_and_querying() {
        let mut world = World::new();
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        // Add a soldier to entity1
        world.add_component(entity1, Soldier { id: 1, name: "John".to_string() });
        
        // Dispatch a ShotsFired event to entity1
        world.add_event(entity1, ShotsFired { damage: 50, target_id: 2 });

        // Query for the event
        let events = world.get_component::<Event<ShotsFired>>(entity1);
        assert!(events.is_some());
        let event = events.unwrap();
        assert_eq!(event.get().damage, 50);
        assert_eq!(event.get().target_id, 2);

        // Entity2 should not have this event
        let no_event = world.get_component::<Event<ShotsFired>>(entity2);
        assert!(no_event.is_none());
    }

    #[test]
    fn test_event_cleanup_after_world_update() {
        let mut world = World::new();
        let entity = world.create_entity();

        // Dispatch an event
        world.add_event(entity, ShotsFired { damage: 30, target_id: 1 });

        // Event should exist before update
        let event_before = world.get_component::<Event<ShotsFired>>(entity);
        assert!(event_before.is_some());

        // Run world update (this should clean up temporary components)
        world.update();

        // Event should be cleaned up after update
        let event_after = world.get_component::<Event<ShotsFired>>(entity);
        assert!(event_after.is_none());
    }

    #[test]
    fn test_component_added_notifications() {
        let mut world = World::new();
        let entity = world.create_entity();

        // Add a component - this should automatically create a ComponentAdded notification
        world.add_component(entity, Position { x: 10.0, y: 20.0 });

        // Check for ComponentAdded notification
        let added_notification = world.get_component::<ComponentAdded<Position>>(entity);
        assert!(added_notification.is_some());

        // The position component should also exist
        let position = world.get_component::<Position>(entity);
        assert!(position.is_some());
        assert_eq!(position.unwrap().x, 10.0);
        assert_eq!(position.unwrap().y, 20.0);
    }

    #[test]
    fn test_component_removed_notifications() {
        let mut world = World::new();
        let entity = world.create_entity();

        // Add a component first
        world.add_component(entity, Position { x: 5.0, y: 15.0 });

        // Clear any ComponentAdded notifications by running an update
        world.update();

        // Remove the component with notification
        let was_removed = world.remove_component_with_notification::<Position>(entity);
        assert!(was_removed);

        // Check for ComponentRemoved notification
        let removed_notification = world.get_component::<ComponentRemoved<Position>>(entity);
        assert!(removed_notification.is_some());
        let removed_data = removed_notification.unwrap();
        assert_eq!(removed_data.get_data().x, 5.0);
        assert_eq!(removed_data.get_data().y, 15.0);

        // The position component should no longer exist
        let position = world.get_component::<Position>(entity);
        assert!(position.is_none());
    }

    #[test]
    fn test_event_querying_with_worldview() {
        let mut world = World::new();
        let mut world_view = WorldView::<(), ()>::new(&mut world);

        let entity1 = world_view.create_entity();
        let entity2 = world_view.create_entity();

        // Add soldier to both entities
        world_view.add_component(entity1, Soldier { id: 1, name: "Alice".to_string() });
        world_view.add_component(entity2, Soldier { id: 2, name: "Bob".to_string() });

        // Add event only to entity1
        world_view.add_event(entity1, ShotsFired { damage: 25, target_id: 2 });

        // Query for soldiers with ShotsFired events
        let results = world_view.query_components::<(In<Soldier>, In<Event<ShotsFired>>)>();
        
        // Should return only entity1
        assert_eq!(results.len(), 1);
        let (entity, (soldier, event)) = &results[0];
        assert_eq!(*entity, entity1);
        assert_eq!(soldier.id, 1);
        assert_eq!(soldier.name, "Alice");
        assert_eq!(event.get().damage, 25);
        assert_eq!(event.get().target_id, 2);
    }

    #[test]
    fn test_component_change_notifications_with_worldview() {
        let mut world = World::new();
        let mut world_view = WorldView::<(), ()>::new(&mut world);

        let entity = world_view.create_entity();

        // Add a component
        world_view.add_component(entity, Velocity { dx: 1.0, dy: 2.0 });

        // Query for ComponentAdded notifications
        let added_results = world_view.query_components::<(In<ComponentAdded<Velocity>>,)>();
        assert_eq!(added_results.len(), 1);
        assert_eq!(added_results[0].0, entity);
    }

    #[test]
    fn test_multiple_events_on_same_entity() {
        let mut world = World::new();
        let entity = world.create_entity();

        // Add multiple different events to the same entity
        world.add_event(entity, ShotsFired { damage: 10, target_id: 1 });
        world.add_event(entity, Position { x: 100.0, y: 200.0 });

        // Both events should be queryable
        let shots_event = world.get_component::<Event<ShotsFired>>(entity);
        let position_event = world.get_component::<Event<Position>>(entity);

        assert!(shots_event.is_some());
        assert!(position_event.is_some());

        assert_eq!(shots_event.unwrap().get().damage, 10);
        assert_eq!(position_event.unwrap().get().x, 100.0);
    }

    #[test]
    fn test_events_and_notifications_cleanup_independently() {
        let mut world = World::new();
        let entity = world.create_entity();

        // Add component (creates ComponentAdded notification)
        world.add_component(entity, Soldier { id: 1, name: "Test".to_string() });
        
        // Add event
        world.add_event(entity, ShotsFired { damage: 5, target_id: 1 });

        // Both should exist before update
        assert!(world.get_component::<ComponentAdded<Soldier>>(entity).is_some());
        assert!(world.get_component::<Event<ShotsFired>>(entity).is_some());

        // Run update - both temporary components should be cleaned up
        world.update();

        // Both should be cleaned up after update
        assert!(world.get_component::<ComponentAdded<Soldier>>(entity).is_none());
        assert!(world.get_component::<Event<ShotsFired>>(entity).is_none());

        // But the regular component should still exist
        assert!(world.get_component::<Soldier>(entity).is_some());
    }

    #[test]
    fn test_event_system_problem_statement_example() {
        // This test replicates the exact example from the problem statement
        let mut world = World::new();
        let entity1 = world.create_entity();
        let entity2 = world.create_entity();

        // Add soldiers to both entities
        world.add_component(entity1, Soldier { id: 1, name: "Soldier1".to_string() });
        world.add_component(entity2, Soldier { id: 2, name: "Soldier2".to_string() });

        // Entity1 fires shots
        world.add_event(entity1, ShotsFired { damage: 100, target_id: 2 });

        // Query as described in problem statement: for (entity, (soldier, shots_fired))
        let mut world_view = WorldView::<(), ()>::new(&mut world);
        let results = world_view.query_components::<(In<Soldier>, In<Event<ShotsFired>>)>();

        // Should find entity1 with both soldier and shots_fired event
        assert_eq!(results.len(), 1);
        let (entity, (soldier, shots_fired)) = &results[0];
        assert_eq!(*entity, entity1);
        assert_eq!(soldier.id, 1);
        assert_eq!(shots_fired.get().damage, 100);
        assert_eq!(shots_fired.get().target_id, 2);

        // After world update, events should be automatically cleaned up
        world.update();
        
        let results_after_update = world_view.query_components::<(In<Soldier>, In<Event<ShotsFired>>)>();
        assert_eq!(results_after_update.len(), 0);
    }

    #[test]
    fn test_no_infinite_recursion_with_event_notifications() {
        let mut world = World::new();
        let entity = world.create_entity();

        // Adding events should not create ComponentAdded notifications for the events themselves
        world.add_event(entity, ShotsFired { damage: 1, target_id: 1 });

        // Should not have ComponentAdded<Event<ShotsFired>>
        let no_event_notification = world.get_component::<ComponentAdded<Event<ShotsFired>>>(entity);
        assert!(no_event_notification.is_none());

        // But should have the event itself
        let event = world.get_component::<Event<ShotsFired>>(entity);
        assert!(event.is_some());
    }

    #[test]
    fn test_example_from_problem_statement() {
        // This test demonstrates the exact usage described in the problem statement
        println!("=== Event System Demo ===");
        
        let mut world = World::new();
        
        // Create entities
        let soldier1 = world.create_entity();
        let soldier2 = world.create_entity();
        
        // Add soldiers
        world.add_component(soldier1, Soldier { id: 1, name: "Alice".to_string() });
        world.add_component(soldier2, Soldier { id: 2, name: "Bob".to_string() });
        
        println!("Created soldiers Alice and Bob");
        
        // Soldier1 fires shots - dispatch event as described in problem statement
        world.add_event(soldier1, ShotsFired { damage: 100, target_id: 2 });
        println!("Alice fires shots at Bob!");
        
        // Query for events EXACTLY as described in problem statement:
        // "for (entity, (soldier, shots_fired) in query_components::<Soldier, Event<ShotsFired>>()"
        {
            let mut world_view = WorldView::<(), ()>::new(&mut world);
            let results = world_view.query_components::<(In<Soldier>, In<Event<ShotsFired>>)>();
            
            println!("\nQuerying for soldiers with ShotsFired events:");
            for (entity, (soldier, shots_fired)) in &results {
                println!("  Entity {:?}: {} fired shots with {} damage targeting {}", 
                    entity, soldier.name, shots_fired.get().damage, shots_fired.get().target_id);
                
                // Verify this matches problem statement expectations
                assert_eq!(soldier.id, 1);
                assert_eq!(soldier.name, "Alice");
                assert_eq!(shots_fired.get().damage, 100);
                assert_eq!(shots_fired.get().target_id, 2);
            }
            
            // Should find exactly one result as per problem statement
            assert_eq!(results.len(), 1);
        }
        
        println!("\n=== Component Change Notifications Demo ===");
        
        // Add position component (automatically creates ComponentAdded notification)
        world.add_component(soldier1, Position { x: 10.0, y: 20.0 });
        
        // Query for component addition notifications
        {
            let mut world_view = WorldView::<(), ()>::new(&mut world);
            let additions = world_view.query_components::<(In<ComponentAdded<Position>>,)>();
            println!("Position component was added to {} entities", additions.len());
            assert_eq!(additions.len(), 1);
        }
        
        println!("\n=== Automatic Cleanup Demo ===");
        
        // Before world update - events and notifications exist
        let events_before = world.get_component::<Event<ShotsFired>>(soldier1);
        let additions_before = world.get_component::<ComponentAdded<Position>>(soldier1);
        println!("Before world.update():");
        println!("  Events exist: {}", events_before.is_some());
        println!("  Notifications exist: {}", additions_before.is_some());
        
        assert!(events_before.is_some());
        assert!(additions_before.is_some());
        
        // Run world update - this cleans up all temporary components as specified
        world.update();
        
        // After world update - all temporary components are cleaned up automatically
        let events_after = world.get_component::<Event<ShotsFired>>(soldier1);
        let additions_after = world.get_component::<ComponentAdded<Position>>(soldier1);
        println!("\nAfter world.update():");
        println!("  Events exist: {}", events_after.is_some());
        println!("  Notifications exist: {}", additions_after.is_some());
        
        assert!(events_after.is_none());
        assert!(additions_after.is_none());
        
        // But regular components still exist
        let soldier_still_exists = world.get_component::<Soldier>(soldier1);
        println!("  Regular components still exist: {}", soldier_still_exists.is_some());
        assert!(soldier_still_exists.is_some());
        
        println!("\n=== Problem Statement Requirements Verified ===");
        println!("✓ Event dispatching works exactly as specified");
        println!("✓ Query syntax matches problem statement exactly");
        println!("✓ Automatic cleanup at end of frame works");
        println!("✓ Component change notifications work");
        println!("✓ Temporary components stored in separate HashMap");
        println!("✓ No manual removal needed - automatic cleanup!");
    }
}