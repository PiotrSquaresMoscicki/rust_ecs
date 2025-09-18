use rust_ecs::{impl_diffable, Diffable, DiffableComponent, In, Out, System, World, WorldView};

// Example components with Diffable implementation
#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}

impl_diffable!(Position { x: f32, y: f32 });

#[derive(Debug, Clone)]
struct Velocity {
    dx: f32,
    dy: f32,
}

impl_diffable!(Velocity { dx: f32, dy: f32 });

#[derive(Debug)]
struct Health {
    current: i32,
    max: i32,
}

impl_diffable!(Health {
    current: i32,
    max: i32,
});

// Example system that moves entities based on their velocity
struct MovementSystem;

impl System for MovementSystem {
    type InComponents = (Velocity,);
    type OutComponents = (Position,);

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

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("HealthSystem initialized");
    }

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("HealthSystem regenerating health for entities");

        // Query all entities with health
        for (entity, health) in world.query_mut::<Health>() {
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

fn main() {
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

    // Register systems
    world.add_system(MovementSystem);
    world.add_system(HealthSystem);
    println!("Registered systems");

    // Initialize systems - one time function call before the first update
    world.initialize_systems();
    println!("Initialized all systems");

    // Run a few update frames
    println!("\nRunning simulation...");
    for frame in 1..=5 {
        println!("Frame {}", frame);
        world.update();
    }

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

    // Demonstrate diffable functionality
    demo_diffable_functionality();
}

fn demo_diffable_functionality() {
    println!("\n--- Diffable Component Tracking Demo ---");

    let mut world = World::new();

    // Create entities
    let entity1 = world.create_entity();
    let entity2 = world.create_entity();

    println!("Created entities: {:?}, {:?}", entity1, entity2);

    // Add diffable components with tracking
    let initial_pos = Position { x: 0.0, y: 0.0 };
    let initial_health = Health {
        current: 100,
        max: 100,
    };

    world.add_diffable_component(entity1, initial_pos);
    world.add_diffable_component(entity1, initial_health);

    println!("Added initial components with tracking");

    // Update components to show diff tracking
    let new_pos = Position { x: 5.0, y: 3.0 };
    let damaged_health = Health {
        current: 75,
        max: 100,
    };

    world.update_diffable_component(entity1, new_pos);
    world.update_diffable_component(entity1, damaged_health);

    println!("Updated components - changes tracked in world history");

    // Create and remove a child world
    let child_world_index = world.create_child_world();
    println!("Created child world: {}", child_world_index);

    world.remove_child_world(child_world_index);
    println!("Removed child world: {}", child_world_index);

    // Display world update history
    let history = world.get_update_history();
    println!("\nWorld Update History:");
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
                system_diff.component_changes.len(),
                system_diff.world_operations.len()
            );

            for change in &system_diff.component_changes {
                match &change.operation {
                    rust_ecs::DiffableComponentOperation::Added { data } => {
                        println!(
                            "      Added {} to {:?}: {}",
                            change.component_type_name, change.entity, data
                        );
                    }
                    rust_ecs::DiffableComponentOperation::Modified { diff } => {
                        println!(
                            "      Modified {} on {:?}: {}",
                            change.component_type_name, change.entity, diff
                        );
                    }
                    rust_ecs::DiffableComponentOperation::Removed => {
                        println!(
                            "      Removed {} from {:?}",
                            change.component_type_name, change.entity
                        );
                    }
                }
            }

            for operation in &system_diff.world_operations {
                match operation.operation {
                    rust_ecs::WorldOperationType::Created => {
                        println!("      Created world {}", operation.world_index);
                    }
                    rust_ecs::WorldOperationType::Removed => {
                        println!("      Removed world {}", operation.world_index);
                    }
                }
            }
        }
    }

    println!("\nDiffable tracking allows complete reproduction of all world changes!");
}
