//! Integration tests for the rust_ecs library.

use rust_ecs::{add, World};

#[test]
fn integration_test_add_function() {
    // Test the public add function
    assert_eq!(add(5, 7), 12);
    assert_eq!(add(-3, 3), 0);
}

#[test]
fn integration_test_world_functionality() {
    // Test the World struct and its methods
    let mut world = World::new();

    // Test initial state
    assert_eq!(world.entity_count(), 0);

    // Test entity creation
    let entity1 = world.create_entity();
    let entity2 = world.create_entity();
    let entity3 = world.create_entity();

    assert_eq!(entity1, 0);
    assert_eq!(entity2, 1);
    assert_eq!(entity3, 2);
    assert_eq!(world.entity_count(), 3);
}

#[test]
fn integration_test_multiple_worlds() {
    // Test that multiple worlds are independent
    let mut world1 = World::new();
    let mut world2 = World::new();

    world1.create_entity();
    world1.create_entity();

    world2.create_entity();

    assert_eq!(world1.entity_count(), 2);
    assert_eq!(world2.entity_count(), 1);
}
