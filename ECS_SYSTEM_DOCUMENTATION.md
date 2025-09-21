# ECS System Documentation

## Table of Contents
1. [Overview](#overview)
2. [Core Concepts](#core-concepts)
3. [Architecture](#architecture)
4. [Change Tracking and Replay System](#change-tracking-and-replay-system)
5. [Component System](#component-system)
6. [Query System](#query-system)
7. [World Management](#world-management)
8. [Usage Examples](#usage-examples)
9. [Best Practices](#best-practices)
10. [API Reference](#api-reference)

## Overview

This Rust ECS (Entity Component System) framework is designed with **high debuggability** and **developer experience** as primary goals, rather than raw performance. The framework's unique approach centers around **change tracking** and **replay functionality** to help developers debug complex system interactions.

### Key Features

- **Type-Safe System Definitions**: Systems explicitly declare input and output components
- **Comprehensive Change Tracking**: All component modifications are tracked automatically
- **Full Replay Capability**: Complete game sessions can be replayed frame-by-frame
- **Developer-Friendly APIs**: Clear, intuitive interfaces with extensive debugging support
- **Compile-Time Safety**: Type system prevents many common ECS mistakes

## Core Concepts

### Entity

An **Entity** is a unique identifier that represents a game object. It's a lightweight wrapper around a `usize`:

```rust
pub struct Entity {
    id: usize,
    generation: usize,
}
```

Entities serve as keys to associate components together. They have no behavior or data themselves.

### Component

A **Component** is pure data attached to an entity. Any Rust type that implements `'static` can be a component:

```rust
#[derive(Debug, Clone, Diff)]
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
```

Components should implement the `Diff` trait (often via the derive macro) to enable change tracking.

### System

A **System** implements game logic by operating on components. Systems must declare their input and output components:

```rust
pub trait System {
    /// Components the system reads from (immutable access)
    type InComponents;
    /// Components the system reads from and writes to (mutable access)
    type OutComponents;

    /// Called once before the first update
    fn initialize(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>);
    
    /// Called every frame to update the system
    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>);
    
    /// Called when the system is removed or world shuts down
    fn deinitialize(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>);
}
```

### World

A **World** is the container for all entities, components, and systems. It manages:

- Entity lifecycle (creation/deletion)
- Component storage and access
- System execution and management
- Change tracking and history
- Replay functionality

### WorldView

A **WorldView** provides type-safe, controlled access to world data for systems. It ensures systems can only access components they've declared as inputs or outputs.

## Architecture

### System Wrapper Pattern

The framework uses a `SystemWrapper<S>` to encapsulate systems and provide change tracking:

```rust
struct SystemWrapper<S: System> {
    system: S,
    // Internal change tracking state
}
```

When a system executes:
1. A snapshot of output components is created
2. The system's update method is called with a WorldView
3. Changes are detected by comparing with the snapshot
4. All changes are recorded in the world's update history

### Type Safety Through Generic Constraints

The `WorldView<InComponents, OutComponents>` uses Rust's type system to enforce access control:

- Systems can only query components declared in `InComponents` or `OutComponents`
- Mutable access is only allowed for `OutComponents`
- Compile-time errors prevent accessing undeclared components

## Change Tracking and Replay System

### Diff Trait

The `Diff` trait is central to change tracking:

```rust
pub trait Diff {
    type Diff: Clone + std::fmt::Debug;
    
    fn diff(&self, other: &Self) -> Option<Self::Diff>;
    fn apply_diff(&mut self, diff: &Self::Diff);
    fn diff_to_string(diff: &Self::Diff) -> String;
}
```

### Automatic Change Detection

Every system execution creates diffs for all modified components:

1. **Before System Execution**: Snapshot all output components
2. **After System Execution**: Compare current state with snapshot
3. **Record Changes**: Store diffs in update history

### Replay Functionality

The framework can replay entire game sessions:

```rust
// Enable replay logging
let config = ReplayLogConfig {
    enabled: true,
    log_directory: "game_logs".to_string(),
    file_prefix: "my_game".to_string(),
    flush_interval: 50,
    include_component_details: true,
};
world.enable_replay_logging(config)?;

// Run game normally...

// Later, replay from log file
let replay_world = World::parse_and_replay_log("game_logs/my_game_123456.log")?;
```

### Replay Log Format

Replay logs use a structured format:

```
UPDATE 1
SYSTEMS: 1
  SYSTEM 0
    COMPONENT_CHANGES: 3
      MOD Entity(0, 0) Position PositionDiff { x: Some(1), y: Some(1) }
      MOD Entity(0, 1) Position PositionDiff { x: Some(2), y: Some(2) }
      MOD Entity(0, 2) Position PositionDiff { x: Some(3), y: Some(3) }
    WORLD_OPERATIONS: 0
```

## Component System

### Component Storage

Components are stored in a type-erased HashMap:

```rust
components: HashMap<TypeId, HashMap<Entity, Box<dyn Any>>>
```

This allows different component types while maintaining type safety through the `TypeId` key.

### Component Lifecycle

1. **Addition**: `world.add_component(entity, component)`
2. **Access**: Through queries or direct entity lookup
3. **Modification**: Via mutable references from queries
4. **Removal**: `world.remove_component::<T>(entity)`

### Diff Derive Macro

The `#[derive(Diff)]` macro automatically implements change tracking:

```rust
#[derive(Debug, Diff)]
struct Player {
    name: String,
    score: i32,
    position: Position,
}

// Generates PlayerDiff struct and Diff implementation
```

## Query System

### Basic Queries

Query for entities with specific components:

```rust
// Immutable access
let positions: Vec<(Entity, &Position)> = world.query::<Position>();

// Mutable access  
let mut positions: Vec<(Entity, &mut Position)> = world.query_mut::<Position>();
```

### Multi-Component Queries

Query entities with multiple components:

```rust
// Query entities that have both Position and Velocity
for (entity, (position, velocity)) in world.multi_query::<(Position, Velocity)>() {
    // position is &Position, velocity is &Velocity
}

// Mixed mutable/immutable access
for (entity, (velocity, mut position)) in world.multi_query::<(Velocity, Out<Position>)>() {
    position.x += velocity.dx;
    position.y += velocity.dy;
}
```

### Query Components

The framework supports up to 15 components in a single query using tuple implementations.

## World Management

### Entity Management

```rust
// Create entity
let entity = world.create_entity();

// Add components
world.add_component(entity, Position { x: 0.0, y: 0.0 });
world.add_component(entity, Velocity { dx: 1.0, dy: 0.0 });

// Check if entity has component
if world.has_component::<Position>(entity) {
    // ...
}

// Remove component
world.remove_component::<Position>(entity);

// Remove entity (and all its components)
world.remove_entity(entity);
```

### System Management

```rust
// Add system
world.add_system(MovementSystem);

// Initialize all systems (call once before game loop)
world.initialize_systems();

// Update all systems (call every frame)
world.update();

// Systems are automatically deinitialized when world is dropped
```

### Update History

```rust
// Get update history for debugging
let history = world.get_update_history();

// Replay history in a new world
let replay_world = World::replay_history(history);
```

## Usage Examples

### Basic ECS Setup

```rust
use rust_ecs::{World, System, WorldView, Diff};

#[derive(Debug, Clone, Diff)]
struct Position { x: f32, y: f32 }

#[derive(Debug, Clone, Diff)]
struct Velocity { dx: f32, dy: f32 }

struct MovementSystem;

impl System for MovementSystem {
    type InComponents = (Velocity,);
    type OutComponents = (Position,);

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("MovementSystem initialized");
    }

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        for (entity, (velocity, mut position)) in world.multi_query::<(Velocity, Out<Position>)>() {
            position.x += velocity.dx;
            position.y += velocity.dy;
        }
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("MovementSystem deinitialized");
    }
}

fn main() {
    let mut world = World::new();
    
    // Create entities
    let player = world.create_entity();
    world.add_component(player, Position { x: 0.0, y: 0.0 });
    world.add_component(player, Velocity { dx: 1.0, dy: 0.5 });
    
    // Add systems
    world.add_system(MovementSystem);
    world.initialize_systems();
    
    // Game loop
    for _ in 0..10 {
        world.update();
    }
}
```

### Replay and Debugging

```rust
// Enable detailed replay logging
let config = ReplayLogConfig {
    enabled: true,
    log_directory: "debug_logs".to_string(),
    file_prefix: "debug_session".to_string(),
    flush_interval: 1, // Flush every update for detailed debugging
    include_component_details: true,
};

world.enable_replay_logging(config)?;

// Run simulation
for i in 0..100 {
    world.update();
    
    // Check for problematic state
    if some_error_condition(&world) {
        println!("Error detected at frame {}", i);
        break;
    }
}

// Disable logging to finalize file
world.disable_replay_logging()?;

// Later, debug by replaying
let replay_world = World::parse_and_replay_log("debug_logs/debug_session_*.log")?;
```

## Best Practices

### Component Design

1. **Keep Components Small**: Focus on single responsibilities
2. **Use Derive Macro**: `#[derive(Diff)]` for automatic change tracking
3. **Prefer Composition**: Multiple small components over large ones
4. **Implement Debug**: Always derive or implement `Debug` for components

### System Design

1. **Declare Dependencies Clearly**: Be explicit about input/output components
2. **Minimize Component Access**: Only declare components you actually use
3. **Avoid Side Effects**: Keep systems pure and predictable
4. **Use Initialization**: Set up system state in `initialize()`, not `update()`

### Performance Considerations

1. **Batch Operations**: Process multiple entities together when possible
2. **Minimize Allocations**: Reuse collections where appropriate
3. **Query Efficiently**: Use specific queries rather than broad component access
4. **Profile Replay Overhead**: Monitor change tracking impact in performance-critical code

### Debugging Workflow

1. **Enable Replay Logging**: Use detailed logging during development
2. **Reproduce Issues**: Use replay logs to recreate problematic scenarios
3. **Isolate Systems**: Test systems individually when debugging
4. **Monitor Change History**: Review update history for unexpected modifications

## API Reference

### Core Types

- `Entity`: Unique entity identifier
- `World`: Main ECS container
- `WorldView<I, O>`: Type-safe system interface
- `System`: Trait for implementing game logic

### Component Operations

- `world.add_component<T>(entity, component)`
- `world.get_component<T>(entity) -> Option<&T>`
- `world.get_component_mut<T>(entity) -> Option<&mut T>`
- `world.remove_component<T>(entity)`
- `world.has_component<T>(entity) -> bool`

### Query Operations

- `world.query<T>() -> Vec<(Entity, &T)>`
- `world.query_mut<T>() -> Vec<(Entity, &mut T)>`
- `world.multi_query<(T1, T2, ...)>() -> Vec<(Entity, (T1, T2, ...))>`
- `world.entities_with_component<T>() -> Vec<Entity>`

### System Operations

- `world.add_system<S: System>(system)`
- `world.initialize_systems()`
- `world.update()`

### Replay Operations

- `world.enable_replay_logging(config)`
- `world.disable_replay_logging()`
- `World::parse_and_replay_log(path)`
- `World::replay_history(history)`

For complete API documentation, run `cargo doc --open`.