//! Test suite for the event system and component lifecycle tracking
//!
//! This module tests:
//! - Event<T> components are automatically cleaned up at end of frame
//! - ComponentAdded<T> components are automatically generated and cleaned up
//! - ComponentRemoved<T> components are automatically generated and cleaned up

use rust_ecs::*;
use rust_ecs::ecs::{Event, ComponentAdded, ComponentRemoved};

#[derive(Debug, Clone, PartialEq)]
struct ShotsFired {
    count: u32,
    damage: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct Soldier {
    name: String,
    health: f32,
}

#[derive(Debug, Clone, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

#[test]
fn test_event_component_cleanup() {
    let mut world = World::new();
    
    // Create an entity
    let entity = world.create_entity();
    
    // Add an Event<ShotsFired> component
    let shots_fired = ShotsFired { count: 5, damage: 10.0 };
    world.add_component(entity, Event::new(shots_fired));
    
    // Verify the event component exists
    assert!(world.get_component::<Event<ShotsFired>>(entity).is_some());
    
    // Update the world (this should clean up event components)
    world.update();
    
    // Verify the event component is cleaned up
    assert!(world.get_component::<Event<ShotsFired>>(entity).is_none());
}

#[test]
fn test_component_added_lifecycle() {
    let mut world = World::new();
    
    // Create an entity
    let entity = world.create_entity();
    
    // Use WorldView to add a component (this should generate ComponentAdded<T>)
    {
        let mut world_view = WorldView::<(), ()>::new(&mut world);
        let soldier = Soldier { name: "John".to_string(), health: 100.0 };
        world_view.add_component(entity, soldier);
    }
    
    // Verify both the original component and ComponentAdded<T> exist
    assert!(world.get_component::<Soldier>(entity).is_some());
    assert!(world.get_component::<ComponentAdded<Soldier>>(entity).is_some());
    
    // Update the world (this should clean up ComponentAdded<T>)
    world.update();
    
    // Verify the original component still exists but ComponentAdded<T> is cleaned up
    assert!(world.get_component::<Soldier>(entity).is_some());
    assert!(world.get_component::<ComponentAdded<Soldier>>(entity).is_none());
}

#[test]
fn test_component_removed_lifecycle() {
    let mut world = World::new();
    
    // Create an entity and add a component
    let entity = world.create_entity();
    let position = Position { x: 10.0, y: 20.0 };
    world.add_component(entity, position.clone());
    
    // Verify the component exists
    assert!(world.get_component::<Position>(entity).is_some());
    
    // Use WorldView to remove the component (this should generate ComponentRemoved<T>)
    let removed_position = {
        let mut world_view = WorldView::<(), ()>::new(&mut world);
        world_view.remove_component::<Position>(entity)
    };
    
    // Verify the component was removed and ComponentRemoved<T> was created
    assert!(world.get_component::<Position>(entity).is_none());
    assert!(world.get_component::<ComponentRemoved<Position>>(entity).is_some());
    assert_eq!(removed_position, Some(position.clone()));
    
    // Verify the data in ComponentRemoved<T> matches the original component
    let component_removed = world.get_component::<ComponentRemoved<Position>>(entity).unwrap();
    assert_eq!(component_removed.data, position);
    
    // Update the world (this should clean up ComponentRemoved<T>)
    world.update();
    
    // Verify ComponentRemoved<T> is cleaned up
    assert!(world.get_component::<ComponentRemoved<Position>>(entity).is_none());
}

#[test]
fn test_event_system_with_systems() {
    // Test that systems can query for events and they work correctly
    
    struct ShotEventSystem;
    
    impl System for ShotEventSystem {
        type InComponents = (In<Soldier>, In<Event<ShotsFired>>);
        type OutComponents = ();
        type Dependencies = ();
        
        fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        
        fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
            let entities_with_shots: Vec<(Entity, (Soldier, Event<ShotsFired>))> = 
                world.query_components::<(In<Soldier>, In<Event<ShotsFired>>)>()
                    .into_iter()
                    .map(|(entity, (soldier, shots))| (entity, ((*soldier).clone(), (*shots).clone())))
                    .collect();
            
            // Verify we can access the event data
            for (_entity, (_soldier, shots_fired)) in entities_with_shots {
                assert_eq!(shots_fired.count, 3);
                assert_eq!(shots_fired.damage, 15.0);
            }
        }
        
        fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    }
    
    let mut world = World::new();
    
    // Create an entity with soldier and shot event
    let entity = world.create_entity();
    world.add_component(entity, Soldier { name: "Alice".to_string(), health: 80.0 });
    world.add_component(entity, Event::new(ShotsFired { count: 3, damage: 15.0 }));
    
    // Add the system
    world.add_system(ShotEventSystem);
    
    // Initialize and update (system should process the event)
    world.initialize_systems();
    world.update();
    
    // Verify the event is cleaned up after the frame
    assert!(world.get_component::<Event<ShotsFired>>(entity).is_none());
}

#[test]
fn test_component_lifecycle_with_systems() {
    // Test that systems can query for ComponentAdded and ComponentRemoved events
    
    struct LifecycleSystem;
    
    impl System for LifecycleSystem {
        type InComponents = (In<ComponentAdded<Position>>, In<ComponentRemoved<Position>>);
        type OutComponents = ();
        type Dependencies = ();
        
        fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        
        fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
            // Query for ComponentAdded<Position> events
            let added_events: Vec<(Entity, ComponentAdded<Position>)> = 
                world.query_components::<(In<ComponentAdded<Position>>,)>()
                    .into_iter()
                    .map(|(entity, added)| (entity, added.clone()))
                    .collect();
            
            // Query for ComponentRemoved<Position> events
            let removed_events: Vec<(Entity, ComponentRemoved<Position>)> = 
                world.query_components::<(In<ComponentRemoved<Position>>,)>()
                    .into_iter()
                    .map(|(entity, removed)| (entity, removed.clone()))
                    .collect();
            
            // For this test, we expect exactly one of each type
            assert_eq!(added_events.len(), 1);
            assert_eq!(removed_events.len(), 1);
            
            // Verify the removed component data
            let removed_position = &removed_events[0].1;
            assert_eq!(removed_position.x, 5.0);
            assert_eq!(removed_position.y, 10.0);
        }
        
        fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    }
    
    let mut world = World::new();
    
    // Create entities and manipulate components to generate lifecycle events
    let entity1 = world.create_entity();
    let entity2 = world.create_entity();
    
    // Add a position component to entity2 to be removed later
    world.add_component(entity2, Position { x: 5.0, y: 10.0 });
    
    // Use WorldView to add/remove components (generates lifecycle events)
    {
        let mut world_view = WorldView::<(), ()>::new(&mut world);
        // Add component to entity1 (generates ComponentAdded)
        world_view.add_component(entity1, Position { x: 1.0, y: 2.0 });
        // Remove component from entity2 (generates ComponentRemoved)
        world_view.remove_component::<Position>(entity2);
    }
    
    // Add the system and run
    world.add_system(LifecycleSystem);
    world.initialize_systems();
    world.update();
    
    // Verify lifecycle events are cleaned up after the frame
    assert!(world.get_component::<ComponentAdded<Position>>(entity1).is_none());
    assert!(world.get_component::<ComponentRemoved<Position>>(entity2).is_none());
}

#[test]
fn test_multiple_events_same_frame() {
    let mut world = World::new();
    
    // Create multiple entities with events
    let entity1 = world.create_entity();
    let entity2 = world.create_entity();
    let entity3 = world.create_entity();
    
    world.add_component(entity1, Event::new(ShotsFired { count: 1, damage: 5.0 }));
    world.add_component(entity2, Event::new(ShotsFired { count: 2, damage: 10.0 }));
    world.add_component(entity3, Event::new(ShotsFired { count: 3, damage: 15.0 }));
    
    // Verify all events exist
    assert!(world.get_component::<Event<ShotsFired>>(entity1).is_some());
    assert!(world.get_component::<Event<ShotsFired>>(entity2).is_some());
    assert!(world.get_component::<Event<ShotsFired>>(entity3).is_some());
    
    // Update the world
    world.update();
    
    // Verify all events are cleaned up
    assert!(world.get_component::<Event<ShotsFired>>(entity1).is_none());
    assert!(world.get_component::<Event<ShotsFired>>(entity2).is_none());
    assert!(world.get_component::<Event<ShotsFired>>(entity3).is_none());
}

#[test]
fn test_problem_statement_scenario() {
    // This test demonstrates the exact scenario described in the problem statement
    
    struct ShotEventSystem {
        processed_events: Vec<(Entity, ShotsFired)>,
    }
    
    impl System for ShotEventSystem {
        type InComponents = (In<Soldier>, In<Event<ShotsFired>>);
        type OutComponents = ();
        type Dependencies = ();
        
        fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
        
        fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
            // Query as described in the problem statement:
            // for (entity, (soldier, shots_fired)) in query_components::<Soldier, Event<ShotsFired>>()
            let entities_with_shots: Vec<(Entity, (Soldier, Event<ShotsFired>))> = 
                world.query_components::<(In<Soldier>, In<Event<ShotsFired>>)>()
                    .into_iter()
                    .map(|(entity, (soldier, shots))| (entity, (soldier.clone(), shots.clone())))
                    .collect();
            
            // Process the events
            for (entity, (soldier, shots_fired)) in entities_with_shots {
                println!("Soldier {} fired {} shots for {} damage each", 
                         soldier.name, shots_fired.count, shots_fired.damage);
                self.processed_events.push((entity, shots_fired.data));
            }
        }
        
        fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    }
    
    let mut world = World::new();
    
    // Create entities with soldiers
    let soldier1 = world.create_entity();
    let soldier2 = world.create_entity();
    
    world.add_component(soldier1, Soldier { name: "Alice".to_string(), health: 100.0 });
    world.add_component(soldier2, Soldier { name: "Bob".to_string(), health: 80.0 });
    
    // Dispatch events as described in the problem statement:
    // "I add component of type Event<ShotsFired> to an entity I want to dispatch the event for"
    world.add_component(soldier1, Event::new(ShotsFired { count: 3, damage: 15.0 }));
    world.add_component(soldier2, Event::new(ShotsFired { count: 2, damage: 20.0 }));
    
    // Verify events exist before processing
    assert!(world.get_component::<Event<ShotsFired>>(soldier1).is_some());
    assert!(world.get_component::<Event<ShotsFired>>(soldier2).is_some());
    
    // Add the system that reacts to ShotsFired events
    let mut shot_system = ShotEventSystem { processed_events: Vec::new() };
    world.add_system(shot_system);
    
    // Initialize and update the world (systems should process the events)
    world.initialize_systems();
    world.update();
    
    // Verify that the events are automatically cleaned up at the end of the frame
    // as stated in the problem: "the world automatically removes all components of type Event<...> at the end of the frame"
    assert!(world.get_component::<Event<ShotsFired>>(soldier1).is_none());
    assert!(world.get_component::<Event<ShotsFired>>(soldier2).is_none());
    
    // The soldiers should still exist (they're not events)
    assert!(world.get_component::<Soldier>(soldier1).is_some());
    assert!(world.get_component::<Soldier>(soldier2).is_some());
}