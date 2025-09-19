# Rust ECS Framework

A Rust Entity-Component-System (ECS) framework designed for high debuggability and developer experience. Unlike traditional ECS implementations focused on raw performance, this framework emphasizes **change tracking** and **replay functionality** to help developers debug complex system interactions.

## Key Features

- **Type-Safe System Definitions**: Systems declare their input and output components, enabling compile-time safety and runtime change tracking
- **Component Querying**: Flexible component queries with both immutable and mutable access
- **Change Tracking**: Comprehensive tracking of all component changes for debugging and replay
- **Replay Functionality**: Ability to replay entire game sessions to reproduce bugs
- **Entity Management**: Full lifecycle management of entities and their components
- **Developer-Friendly**: Clear APIs and extensive debugging capabilities

## Architecture

The framework follows the standard ECS pattern with some unique additions:

- **Entity**: A unique identifier (wrapper around `usize`)
- **Component**: Data assigned to entities (any Rust type that implements `'static`)
- **System**: Logic that operates on components, with declared input/output component types
- **World**: Container for entities, components, and systems with change tracking
- **WorldView**: Type-safe wrapper that provides controlled access to world data for systems

## Core Concepts

### Systems with Input/Output Declaration

```rust
use rust_ecs::{System, WorldView, Entity};

struct MovementSystem;

impl System for MovementSystem {
    type InputComponents = (Velocity,);      // Components we read from
    type OutputComponents = (Position,);     // Components we modify

    fn initialize(&mut self, world: &mut WorldView<Self::InputComponents, Self::OutputComponents>) {
        // One-time initialization
    }

    fn update(&mut self, world: &mut WorldView<Self::InputComponents, Self::OutputComponents>) {
        // Get entities with velocity and update their positions
        let entities_with_velocity: Vec<(Entity, Velocity)> = world.query::<Velocity>()
            .into_iter()
            .map(|(entity, velocity)| (entity, velocity.clone()))
            .collect();
        
        for (entity, velocity) in entities_with_velocity {
            if let Some(position) = world.get_component_mut::<Position>(entity) {
                position.x += velocity.dx;
                position.y += velocity.dy;
            }
        }
    }

    fn deinitialize(&mut self, world: &mut WorldView<Self::InputComponents, Self::OutputComponents>) {
        // Cleanup when system is removed
    }
}
```

### Basic Usage

```rust
use rust_ecs::{World, Entity};

#[derive(Debug, Clone)]
struct Position { x: f32, y: f32 }

#[derive(Debug, Clone)]
struct Velocity { dx: f32, dy: f32 }

fn main() {
    // Create world
    let mut world = World::new();

    // Create entities
    let player = world.create_entity();
    let enemy = world.create_entity();

    // Add components
    world.add_component(player, Position { x: 0.0, y: 0.0 });
    world.add_component(player, Velocity { dx: 1.0, dy: 0.0 });
    world.add_component(enemy, Position { x: 10.0, y: 5.0 });

    // Add systems
    world.add_system(MovementSystem);

    // Initialize systems
    world.initialize_systems();

    // Game loop
    loop {
        world.update();
        // ... render, handle input, etc.
    }
}
```

### Component Querying

```rust
// Query all entities with a specific component type
let positions: Vec<(Entity, &Position)> = world.query::<Position>();

// Query with mutable access
let mut positions: Vec<(Entity, &mut Position)> = world.query_mut::<Position>();

// Get a specific component for an entity
if let Some(position) = world.get_component::<Position>(entity) {
    println!("Entity is at ({}, {})", position.x, position.y);
}

// Get entities that have a specific component type
let entities_with_health = world.entities_with_component::<Health>();
```

### Replay and Debugging

```rust
// Get the world's update history
let history = world.get_update_history();

// Replay the entire history in a new world instance
let replay_world = World::replay_history(history);
```

## Running the Demo

```bash
cargo run
```

This will run a demonstration showing:
- Entity creation and component management
- System execution with movement and health regeneration
- Component querying and modification
- Update history tracking
- Replay functionality

## Running Tests

```bash
cargo test
```

The test suite covers:
- Core ECS functionality
- Component querying and iteration
- Entity lifecycle management
- System execution
- Integration tests

## Future Enhancements

This implementation provides the foundation for a debuggable ECS. Future improvements could include:

- More sophisticated change tracking with detailed component diffs
- Advanced querying with multiple component types and filters
- Nested world support for hierarchical game structures
- Performance optimizations while maintaining debuggability
- Editor integration for visual debugging
- Serialization support for save/load functionality

## License

This project is open source. See the repository for license details.