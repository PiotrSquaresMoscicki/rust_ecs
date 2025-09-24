// Test to verify the library can be used both as library and executable
use rust_ecs::*;

#[test]
fn test_library_imports() {
    // Test basic ECS functionality
    let mut world = World::new();
    let entity = world.create_entity();
    assert!(entity.entity_index == 0);
    
    // Test that basic ECS operations work
    let entity_count = world.entity_count();
    assert_eq!(entity_count, 1);
    
    // Test game module access
    let _pos = game::components::Position { x: 10, y: 5 };
    
    // Test that we can access ECS modules
    let _diff_trait = <i32 as Diff>::diff(&5, &10);
    
    // Test system creation
    struct TestSystem;
    impl System for TestSystem {
        type InComponents = ();
        type OutComponents = ();
        type Dependencies = ();
        
        fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    }
    
    world.add_system(TestSystem);
    world.initialize_systems();
    
    println!("✅ Library can be imported and used successfully!");
}

#[test]
fn test_game_module_access() {
    // Test that game module is accessible with all its components
    let mut world = World::new();
    let entity = world.create_entity();
    
    // Add game components
    world.add_component(entity, game::components::Position { x: 1, y: 2 });
    world.add_component(entity, game::components::Target { x: 3, y: 4 });
    
    // Query components
    let pos = world.get_component::<game::components::Position>(entity).unwrap();
    assert_eq!(pos.x, 1);
    assert_eq!(pos.y, 2);
    
    let target = world.get_component::<game::components::Target>(entity).unwrap();
    assert_eq!(target.x, 3);
    assert_eq!(target.y, 4);
    
    println!("✅ Game module components are accessible!");
}