//! Component Lifecycle Event Demo
//! 
//! This demo shows how ComponentAdded<T> and ComponentRemoved<T> events
//! are automatically created when components are added or removed from entities.

use rust_ecs::{World, WorldView, System, Entity, Event, ComponentAdded, ComponentRemoved, In};

/// A health component
#[derive(Debug, Clone)]
struct Health {
    value: i32,
    max_value: i32,
}

/// A player component
#[derive(Debug, Clone)]
struct Player {
    name: String,
    level: u32,
}

/// System that reacts to component lifecycle events
struct LifecycleObserverSystem {
    components_added: usize,
    components_removed: usize,
}

impl System for LifecycleObserverSystem {
    type InComponents = ();
    type OutComponents = ();
    type Dependencies = ();

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("LifecycleObserverSystem initialized");
    }

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // React to Health components being added
        let health_added_events = world.query_components::<(In<Event<ComponentAdded<Health>>>,)>();
        for (_event_entity, health_added) in &health_added_events {
            println!("🟢 Health component added to entity {:?}", health_added.entity);
            self.components_added += 1;
        }

        // React to Player components being added
        let player_added_events = world.query_components::<(In<Event<ComponentAdded<Player>>>,)>();
        for (_event_entity, player_added) in &player_added_events {
            println!("🟢 Player component added to entity {:?}", player_added.entity);
            self.components_added += 1;
        }

        // React to Health components being removed
        let health_removed_events = world.query_components::<(In<Event<ComponentRemoved<Health>>>,)>();
        for (_event_entity, health_removed) in &health_removed_events {
            println!("🔴 Health component removed from entity {:?}", health_removed.entity);
            self.components_removed += 1;
        }

        // React to Player components being removed
        let player_removed_events = world.query_components::<(In<Event<ComponentRemoved<Player>>>,)>();
        for (_event_entity, player_removed) in &player_removed_events {
            println!("🔴 Player component removed from entity {:?}", player_removed.entity);
            self.components_removed += 1;
        }
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("LifecycleObserverSystem processed {} additions and {} removals", 
            self.components_added, self.components_removed);
    }
}

fn main() {
    println!("=== Component Lifecycle Event Demo ===");
    println!("This demo shows automatic ComponentAdded<T> and ComponentRemoved<T> events.\n");

    let mut world = World::new();

    // Add the lifecycle observer system
    let observer_system = LifecycleObserverSystem {
        components_added: 0,
        components_removed: 0,
    };
    world.add_system(observer_system);
    world.initialize_systems();

    println!("Frame 1: Creating entities and adding components");
    
    // Create entities
    let player_entity = world.create_entity();
    let enemy_entity = world.create_entity();

    // Add components - this automatically creates ComponentAdded<T> events
    world.add_component(player_entity, Player { 
        name: "Hero".to_string(), 
        level: 5 
    });
    world.add_component(player_entity, Health { 
        value: 100, 
        max_value: 100 
    });
    
    world.add_component(enemy_entity, Health { 
        value: 80, 
        max_value: 80 
    });

    // Run frame - observer system will process ComponentAdded events
    world.update();
    println!();

    println!("Frame 2: Removing some components");
    
    // Remove components - this automatically creates ComponentRemoved<T> events
    world.remove_component::<Health>(enemy_entity);
    world.remove_component::<Player>(player_entity);

    // Run frame - observer system will process ComponentRemoved events
    world.update();
    println!();

    println!("Frame 3: Adding and removing in same frame");
    
    // Add a new component
    world.add_component(enemy_entity, Player { 
        name: "Goblin".to_string(), 
        level: 2 
    });
    
    // Remove another component
    world.remove_component::<Health>(player_entity);

    // Run frame - observer system will process both types of events
    world.update();
    println!();

    println!("Frame 4: No changes - observer system runs but finds no events");
    world.update();

    world.deinitialize_systems();

    println!("\n=== Demo Complete ===");
    println!("Key benefits of ComponentAdded<T> and ComponentRemoved<T> events:");
    println!("• Automatic creation when components are added/removed");
    println!("• Systems can react to component lifecycle changes");
    println!("• Events are automatically cleaned up each frame");
    println!("• No infinite recursion - events don't create more events");
    println!("• Type-safe and integrated with existing query system");
}