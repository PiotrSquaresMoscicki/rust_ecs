use rust_ecs::{World, System, WorldView, Entity};

// Example components
#[derive(Debug)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug)]
struct Velocity {
    dx: f32,
    dy: f32,
}

#[derive(Debug)]
struct Health {
    current: i32,
    max: i32,
}

// Example system that moves entities based on their velocity
struct MovementSystem;

impl System for MovementSystem {
    type InputComponents = (Velocity,);
    type OutputComponents = (Position,);

    fn initialize(&mut self, _world: &mut WorldView<Self::InputComponents, Self::OutputComponents>) {
        println!("MovementSystem initialized");
    }

    fn update(&mut self, _world: &mut WorldView<Self::InputComponents, Self::OutputComponents>) {
        println!("MovementSystem updating entities with position and velocity");
        // In a complete implementation, this would iterate over entities with Position and Velocity
        // and update positions based on velocity
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InputComponents, Self::OutputComponents>) {
        println!("MovementSystem deinitialized");
    }
}

// Example system that handles health regeneration
struct HealthSystem;

impl System for HealthSystem {
    type InputComponents = ();
    type OutputComponents = (Health,);

    fn initialize(&mut self, _world: &mut WorldView<Self::InputComponents, Self::OutputComponents>) {
        println!("HealthSystem initialized");
    }

    fn update(&mut self, _world: &mut WorldView<Self::InputComponents, Self::OutputComponents>) {
        println!("HealthSystem regenerating health for entities");
        // In a complete implementation, this would iterate over entities with Health
        // and regenerate health over time
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InputComponents, Self::OutputComponents>) {
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
    world.add_component(player, Health { current: 100, max: 100 });

    world.add_component(enemy1, Position { x: 10.0, y: 5.0 });
    world.add_component(enemy1, Velocity { dx: -0.5, dy: 0.0 });
    world.add_component(enemy1, Health { current: 50, max: 50 });

    world.add_component(enemy2, Position { x: -5.0, y: 10.0 });
    world.add_component(enemy2, Health { current: 30, max: 30 });

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
    println!("This demonstrates the ECS framework with change tracking capabilities.");
    println!("In a full implementation, the world would track all component changes");
    println!("for replay functionality and debugging.");
}
