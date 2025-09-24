//! Event System Demo
//! 
//! This demo shows how to use the Event<T> system for dispatching short-lived
//! events that are automatically cleaned up at the end of each frame.

use rust_ecs::{World, WorldView, System, Event, In};

/// Event data representing shots fired in combat
#[derive(Debug, Clone)]
struct ShotsFired {
    damage: i32,
    target_id: usize,
}

/// Event data representing an explosion
#[derive(Debug, Clone)]
struct ExplosionEvent {
    radius: f32,
    damage: i32,
    position: (f32, f32),
}

/// Component representing a soldier in combat
#[derive(Debug, Clone)]
struct Soldier {
    id: usize,
    health: i32,
    position: (f32, f32),
}

/// System that processes ShotsFired events
struct CombatSystem {
    shots_processed: usize,
    explosions_processed: usize,
}

impl System for CombatSystem {
    type InComponents = ();
    type OutComponents = ();
    type Dependencies = ();

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("CombatSystem initialized");
    }

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // Process ShotsFired events - query for entities with both Soldier and Event<ShotsFired>
        let shot_results = world.query_components::<(In<Soldier>, In<Event<ShotsFired>>)>();
        
        for (_entity, (soldier, shots_fired)) in &shot_results {
            println!("Soldier {} fired shots dealing {} damage to target {}",
                soldier.id, shots_fired.damage, shots_fired.target_id);
            self.shots_processed += 1;
        }

        // Process ExplosionEvent events
        let explosion_results = world.query_components::<(In<Event<ExplosionEvent>>,)>();
        
        for (_entity, explosion) in &explosion_results {
            println!("Explosion at ({}, {}) with radius {} dealing {} damage",
                explosion.position.0, explosion.position.1, 
                explosion.radius, explosion.damage);
            self.explosions_processed += 1;
        }
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("CombatSystem processed {} shots and {} explosions", 
            self.shots_processed, self.explosions_processed);
    }
}

fn main() {
    println!("=== Event System Demo ===");
    println!("This demo shows Event<T> dispatching and automatic cleanup.\n");

    let mut world = World::new();

    // Create soldiers
    let soldier1 = world.create_entity();
    let soldier2 = world.create_entity();
    let soldier3 = world.create_entity();

    world.add_component(soldier1, Soldier { id: 1, health: 100, position: (10.0, 10.0) });
    world.add_component(soldier2, Soldier { id: 2, health: 80, position: (20.0, 15.0) });
    world.add_component(soldier3, Soldier { id: 3, health: 90, position: (30.0, 20.0) });

    // Add combat system
    let combat_system = CombatSystem { shots_processed: 0, explosions_processed: 0 };
    world.add_system(combat_system);
    world.initialize_systems();

    println!("Frame 1: Soldier 1 fires at Soldier 2");
    // Dispatch ShotsFired event by adding Event<ShotsFired> component
    world.add_component(soldier1, Event::new(ShotsFired { damage: 25, target_id: 2 }));
    
    // Verify event exists
    assert!(world.get_component::<Event<ShotsFired>>(soldier1).is_some());
    println!("Event dispatched - Event<ShotsFired> component added to soldier 1");
    
    // Run frame - system will process events, then events will be auto-cleaned
    world.update();
    
    // Events should be gone after frame
    assert!(world.get_component::<Event<ShotsFired>>(soldier1).is_none());
    println!("Event automatically cleaned up after frame\n");

    println!("Frame 2: Multiple events - shots and explosion");
    // Dispatch multiple events
    world.add_component(soldier2, Event::new(ShotsFired { damage: 30, target_id: 3 }));
    world.add_component(soldier3, Event::new(ShotsFired { damage: 20, target_id: 1 }));
    world.add_component(soldier1, Event::new(ExplosionEvent { 
        radius: 5.0, 
        damage: 40, 
        position: (25.0, 17.0) 
    }));

    // Verify all events exist
    assert!(world.get_component::<Event<ShotsFired>>(soldier2).is_some());
    assert!(world.get_component::<Event<ShotsFired>>(soldier3).is_some());
    assert!(world.get_component::<Event<ExplosionEvent>>(soldier1).is_some());
    println!("Multiple events dispatched");

    // Run frame
    world.update();

    // All events should be cleaned up
    assert!(world.get_component::<Event<ShotsFired>>(soldier2).is_none());
    assert!(world.get_component::<Event<ShotsFired>>(soldier3).is_none());
    assert!(world.get_component::<Event<ExplosionEvent>>(soldier1).is_none());
    println!("All events automatically cleaned up after frame\n");

    println!("Frame 3: No events - system runs but finds nothing to process");
    world.update();

    world.deinitialize_systems();

    println!("\n=== Demo Complete ===");
    println!("Key benefits of Event<T> system:");
    println!("• Events are type-safe and can contain any data");
    println!("• Systems query events using normal component queries");
    println!("• Events are automatically cleaned up - no manual memory management");
    println!("• Events only exist for one frame - perfect for short-lived notifications");
}