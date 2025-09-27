use rust_ecs::{game, Diff, DiffComponent, In, Out, System, World, WorldView};
use std::env;

// Example components with Diff implementation using derive macro
#[derive(Debug, Diff)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Diff)]
struct Velocity {
    dx: f32,
    dy: f32,
}

#[derive(Debug, Diff)]
struct Health {
    current: i32,
    max: i32,
}

// Demo component to show derive macro in action
#[derive(Debug, Diff)]
struct Temperature {
    celsius: f32,
    pressure: f32,
}

// Example system that moves entities based on their velocity
struct MovementSystem;

impl System for MovementSystem {
    type InComponents = (Velocity,);
    type OutComponents = (Position,);
    type InSystems = ();
    type OutSystems = ();

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("MovementSystem initialized");
    }

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("MovementSystem updating entities with position and velocity");

        // Use the new multi-component query to get entities with both Position and Velocity
        // Position is immutable (In), Velocity is mutable (Out)
        let mut results = world.query_components::<(In<Position>, Out<Velocity>)>();

        for (entity, (position, velocity)) in &mut results {
            // Calculate new position based on velocity (but we can't modify position here)
            let new_x = position.x + velocity.dx;
            let new_y = position.y + velocity.dy;
            println!(
                "  Entity {:?} would move from ({:.1}, {:.1}) to ({:.1}, {:.1})",
                entity, position.x, position.y, new_x, new_y
            );

            // For demonstration, let's dampen the velocity over time
            velocity.dx *= 0.95;
            velocity.dy *= 0.95;
        }
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("MovementSystem deinitialized");
    }
}

// Example system that handles health regeneration
struct HealthSystem;

impl System for HealthSystem {
    type InComponents = ();
    type OutComponents = (Health,);
    type InSystems = ();
    type OutSystems = ();

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("HealthSystem initialized");
    }

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("HealthSystem regenerating health for entities");

        // Query all entities with health
        for (entity, health) in world.query_components::<(Out<Health>,)>() {
            if health.current < health.max {
                health.current = (health.current + 1).min(health.max);
                println!(
                    "  Regenerated health for entity {:?}: {}/{}",
                    entity, health.current, health.max
                );
            }
        }
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("HealthSystem deinitialized");
    }
}

// Example system that depends on MovementSystem - it should initialize/update after MovementSystem
struct PhysicsSystem;

impl System for PhysicsSystem {
    type InComponents = (Position, Velocity);
    type OutComponents = ();
    type InSystems = (MovementSystem,);
    type OutSystems = ();

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("PhysicsSystem initialized (depends on MovementSystem)");
    }

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("PhysicsSystem processing physics (after MovementSystem)");

        let results = world.query_components::<(In<Position>, In<Velocity>)>();
        for (entity, (position, velocity)) in &results {
            // Do some physics calculations that depend on updated positions/velocities
            let speed = (velocity.dx * velocity.dx + velocity.dy * velocity.dy).sqrt();
            if speed > 0.1 {
                println!(
                    "  Entity {:?} at ({:.1}, {:.1}) moving at speed {:.2}",
                    entity, position.x, position.y, speed
                );
            }
        }
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("PhysicsSystem deinitialized (depends on MovementSystem)");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Check if "game" argument is provided with optional replay file
    if args.len() > 1 && args[1] == "game" {
        if args.len() > 2 {
            // Replay mode: cargo run game <replay_log_path>
            let replay_path = &args[2];
            game::run_game_replay(replay_path);
        } else {
            // Normal game mode: cargo run game
            game::run_game();
        }
        return;
    }

    // Check if "demo" argument is provided
    if args.len() > 2 && args[1] == "demo" {
        match args[2].as_str() {
            "woodcutter" => {
                game::woodcutter_system::run_woodcutter_demo();
                return;
            }
            "navigation" => {
                game::navigation_system::run_navigation_demo();
                return;
            }
            "carpenter" => {
                game::carpenter_system::run_carpenter_demo();
                return;
            }
            _ => {
                eprintln!("Unknown demo type: {}", args[2]);
                eprintln!("Available demos: woodcutter, navigation, carpenter");
                return;
            }
        }
    }

    // Check if "replay-demo" argument is provided
    if args.len() > 1 && args[1] == "replay-demo" {
        demo_replay_analysis();
        return;
    }

    // Default behavior - run the ECS framework demo
    run_ecs_demo();
}

fn run_ecs_demo() {
    println!("Rust ECS Framework Demo");
    println!("=======================");

    // Create world
    let mut world = World::new();
    println!("Created new world");

    // Create some entities
    let player = world.create_entity();
    let enemy1 = world.create_entity();
    let enemy2 = world.create_entity();

    println!("Created {} entities", world.entity_count());

    // Add components to entities
    world.add_component(player, Position { x: 0.0, y: 0.0 });
    world.add_component(player, Velocity { dx: 1.0, dy: 0.0 });
    world.add_component(
        player,
        Health {
            current: 90,
            max: 100,
        },
    ); // Slightly damaged

    world.add_component(enemy1, Position { x: 10.0, y: 5.0 });
    world.add_component(enemy1, Velocity { dx: -0.5, dy: 0.0 });
    world.add_component(
        enemy1,
        Health {
            current: 25,
            max: 50,
        },
    ); // Heavily damaged

    world.add_component(enemy2, Position { x: -5.0, y: 10.0 });
    world.add_component(
        enemy2,
        Health {
            current: 1,
            max: 30,
        },
    ); // Almost dead

    println!("Added components to entities");

    // Register systems - note the order: PhysicsSystem depends on MovementSystem
    println!("\n--- System Dependency Demo ---");
    println!(
        "Adding systems: HealthSystem, PhysicsSystem (depends on MovementSystem), MovementSystem"
    );
    println!("Expected order: MovementSystem -> PhysicsSystem, HealthSystem (no dependencies)");

    world.add_system(HealthSystem); // No dependencies
    world.add_system(PhysicsSystem); // Depends on MovementSystem
    world.add_system(MovementSystem); // No dependencies
    println!("Registered systems");

    // Initialize systems - should be ordered by dependencies
    println!("\nInitializing systems (should respect dependencies):");
    world.initialize_systems();
    println!("Initialized all systems");

    // Run a few update frames
    println!("\nRunning simulation...");
    for frame in 1..=3 {
        println!("\nFrame {}", frame);
        world.update();
    }

    println!("\nDeinitializing systems (should be in reverse dependency order):");
    world.deinitialize_systems();

    println!("\nSimulation complete!");

    // Demonstrate replay functionality
    println!("\n--- Replay Functionality Demo ---");
    let history = world.get_update_history();
    let _replay_world = World::replay_history(history);

    println!("\nThis demonstrates the ECS framework with change tracking capabilities.");
    println!("The framework includes:");
    println!("- Type-safe system definitions with input/output component declarations");
    println!("- Component querying and iteration");
    println!("- World update history tracking for debugging");
    println!("- Replay functionality for reproducing game states");
    println!("- Entity and component management");

    // Demonstrate additional world functionality
    println!("\n--- Additional World Features ---");
    println!(
        "Entities with Position: {:?}",
        world.entities_with_component::<Position>()
    );
    println!(
        "Entities with Velocity: {:?}",
        world.entities_with_component::<Velocity>()
    );
    println!(
        "Entities with Health: {:?}",
        world.entities_with_component::<Health>()
    );

    // Demonstrate diff functionality and derive macro
    demo_diff_functionality();

    // Demonstrate the derive macro specifically
    demo_derive_macro();
}

fn demo_derive_macro() {
    println!("\n--- Derive Macro Demo ---");

    // Create instances of the Temperature component that uses #[derive(Diff)]
    let temp1 = Temperature {
        celsius: 20.0,
        pressure: 1013.25,
    };

    let temp2 = Temperature {
        celsius: 25.0, // Changed temperature
        pressure: 1013.25,
    };

    let temp3 = Temperature {
        celsius: 25.0,
        pressure: 1015.0, // Changed pressure
    };

    println!("Original temperature: {:?}", temp1);
    println!("Temperature with changed celsius: {:?}", temp2);
    println!("Temperature with changed pressure: {:?}", temp3);

    // Test diffing - this uses the automatically generated diff implementation
    if let Some(diff) = temp1.diff(&temp2) {
        println!("Diff (temp1 -> temp2): {:?}", diff);
    }

    if let Some(diff) = temp2.diff(&temp3) {
        println!("Diff (temp2 -> temp3): {:?}", diff);
    }

    // No diff when comparing identical values
    if temp1.diff(&temp1).is_none() {
        println!("No diff when comparing identical temperatures (as expected)");
    }

    println!("✅ Derive macro working perfectly! Components automatically implement Diff with #[derive(Diff)]");
}

fn demo_diff_functionality() {
    println!("\n--- Transparent Diff Tracking Demo ---");

    let mut world = World::new();

    // Create entities
    let entity1 = world.create_entity();
    let entity2 = world.create_entity();

    println!("Created entities: {:?}, {:?}", entity1, entity2);

    // Add components using standard methods - tracking happens automatically
    world.add_component(entity1, Position { x: 0.0, y: 0.0 });
    world.add_component(entity1, Velocity { dx: 1.0, dy: 0.5 });
    world.add_component(
        entity1,
        Health {
            current: 100,
            max: 100,
        },
    );

    world.add_component(entity2, Position { x: 10.0, y: 5.0 });
    world.add_component(entity2, Velocity { dx: -0.5, dy: 0.0 });

    println!("Added components using standard ECS methods");

    // Add a movement system that will automatically track changes
    world.add_system(MovementSystem);
    world.initialize_systems();

    println!("Added system - changes will be tracked automatically during updates");

    // Run updates - all component changes will be tracked transparently
    for frame in 1..=3 {
        println!("Frame {}", frame);
        world.update();
    }

    // Create and remove a child world to show world operation tracking
    let child_world_index = world.create_child_world();
    println!("Created child world: {}", child_world_index);

    world.remove_child_world(child_world_index);
    println!("Removed child world: {}", child_world_index);

    // Display world update history - shows all automatically tracked changes
    let history = world.get_update_history();
    println!("\nWorld Update History (Automatically Tracked):");
    for (i, update) in history.updates().iter().enumerate() {
        println!(
            "  Update {}: {} system diffs",
            i + 1,
            update.system_diffs().len()
        );
        for (j, system_diff) in update.system_diffs().iter().enumerate() {
            println!(
                "    System {}: {} component changes, {} world operations",
                j,
                system_diff.component_changes().len(),
                system_diff.world_operations().len()
            );

            for change in system_diff.component_changes() {
                println!("      {:?}", change);
            }

            for operation in system_diff.world_operations() {
                println!("      {:?}", operation);
            }
        }
    }

    println!("\nDemonstrated: Transparent change tracking without manual diff methods!");
    println!("Developers just use standard ECS methods - all tracking happens automatically.");
}

/// Demonstrate replay analysis functionality
fn demo_replay_analysis() {
    println!("\n=== Replay Analysis Demo ===");

    // Create a simple world for demonstration
    let mut world = World::new();

    // Enable replay logging
    let replay_config = rust_ecs::ReplayLogConfig {
        enabled: true,
        log_directory: "demo_replay_logs".to_string(),
        file_prefix: "demo_session".to_string(),
        flush_interval: 5,
        include_component_details: true,
    };

    match world.enable_replay_logging(replay_config) {
        Ok(()) => {
            println!("Replay logging enabled for demo");
        }
        Err(e) => {
            eprintln!("Failed to enable logging: {}", e);
            return;
        }
    }

    // Add some entities and components
    let entity1 = world.create_entity();
    let entity2 = world.create_entity();
    world.add_component(entity1, Position { x: 0.0, y: 0.0 });
    world.add_component(entity1, Velocity { dx: 1.0, dy: 0.5 });
    world.add_component(entity2, Position { x: 10.0, y: 10.0 });

    // Add a movement system
    world.add_system(MovementSystem);
    world.initialize_systems();

    // Run several updates
    for i in 0..15 {
        println!("Update {}", i + 1);
        world.update();
    }

    // Analyze the replay data
    let history = world.get_update_history();
    rust_ecs::replay_analysis::print_replay_analysis(history);

    // Find anomalous frames (frames with significantly more activity)
    let anomalous = rust_ecs::replay_analysis::find_anomalous_frames(history, 2.0);
    if !anomalous.is_empty() {
        println!("Anomalous frames (2x average activity): {:?}", anomalous);
    }

    // Clean up
    if let Err(e) = world.disable_replay_logging() {
        eprintln!("Failed to finalize logging: {}", e);
    }

    // Clean up demo directory
    let _ = std::fs::remove_dir_all("demo_replay_logs");

    println!("Replay analysis demo completed");
}
