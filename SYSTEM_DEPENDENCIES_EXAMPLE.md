# System Dependencies - Usage Example

This example demonstrates the new system dependencies feature implemented in the Rust ECS framework.

## Basic Usage

```rust
use rust_ecs::{System, World, WorldView};

// Base system with no dependencies
struct MovementSystem;
impl System for MovementSystem {
    type InComponents = (Velocity,);
    type OutComponents = (Position,);
    type Dependencies = (); // No dependencies

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("MovementSystem initialized");
    }

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // Update entity positions based on velocity
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("MovementSystem deinitialized");
    }
}

// System that depends on MovementSystem
struct PhysicsSystem;
impl System for PhysicsSystem {
    type InComponents = (Position, Velocity);
    type OutComponents = ();
    type Dependencies = (MovementSystem,); // Depends on MovementSystem

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("PhysicsSystem initialized (after MovementSystem)");
    }

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // Process physics after movement has been updated
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        println!("PhysicsSystem deinitialized (before MovementSystem)");
    }
}

fn main() {
    let mut world = World::new();
    
    // Add systems in any order - dependencies will be resolved automatically
    world.add_system(PhysicsSystem);    // Added first, but will initialize second
    world.add_system(MovementSystem);   // Added second, but will initialize first
    
    // Systems will initialize in dependency order: MovementSystem -> PhysicsSystem
    world.initialize_systems();
    
    // Systems will update in dependency order: MovementSystem -> PhysicsSystem
    world.update();
    
    // Systems will deinitialize in reverse order: PhysicsSystem -> MovementSystem
    world.deinitialize_systems();
}
```

## Multiple Dependencies

```rust
struct RenderSystem;
impl System for RenderSystem {
    type InComponents = (Position, Sprite);
    type OutComponents = ();
    // Render system depends on both movement and physics
    type Dependencies = (MovementSystem, PhysicsSystem);
    
    // This system will initialize/update after both dependencies
}
```

## Dependency Chain

```rust
struct SystemA;
impl System for SystemA {
    type Dependencies = (); // No dependencies
}

struct SystemB;
impl System for SystemB {
    type Dependencies = (SystemA,); // Depends on A
}

struct SystemC;
impl System for SystemC {
    type Dependencies = (SystemB,); // Depends on B (which depends on A)
}

// Execution order will be: A -> B -> C
```

## Key Features

- **Automatic Ordering**: Systems are automatically ordered by dependencies during initialization, updates, and deinitialization
- **Reverse Deinitialization**: Systems deinitialize in reverse dependency order (dependencies last)
- **Error Handling**: Circular dependencies and missing dependencies are handled gracefully with fallback to registration order
- **Backward Compatibility**: Existing systems work unchanged with `type Dependencies = ()`
- **Multiple Dependencies**: Supports up to 5 dependencies per system using tuple syntax

## Output Example

```
MovementSystem initialized
PhysicsSystem initialized (after MovementSystem)

Frame 1
MovementSystem updating entities
PhysicsSystem processing physics (after MovementSystem)

Frame 2
MovementSystem updating entities
PhysicsSystem processing physics (after MovementSystem)

PhysicsSystem deinitialized (before MovementSystem)
MovementSystem deinitialized
```

The system dependencies ensure that:
1. **MovementSystem** always initializes and updates before **PhysicsSystem**
2. **PhysicsSystem** deinitializes before **MovementSystem**
3. This guarantees that physics calculations always work with the most up-to-date positions