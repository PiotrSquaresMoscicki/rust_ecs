# System Ordering - Usage Example

This example demonstrates the new InSystems/OutSystems system ordering feature implemented in the Rust ECS framework.

## Overview

System ordering is now handled through two simple type declarations:

- **InSystems**: Systems that are executed before this system (systems this system depends on)
- **OutSystems**: Systems that are executed after this system (systems that depend on this system)

## Basic Usage

```rust
use rust_ecs::{System, World, WorldView};

// Base system with no ordering constraints
struct MovementSystem;
impl System for MovementSystem {
    type InComponents = (Velocity,);
    type OutComponents = (Position,);
    type InSystems = (); // No systems need to run before this
    type OutSystems = (); // No systems declared to run after this

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

// System that runs after MovementSystem
struct PhysicsSystem;
impl System for PhysicsSystem {
    type InComponents = (Position, Velocity);
    type OutComponents = ();
    type InSystems = (MovementSystem,); // MovementSystem runs before this
    type OutSystems = (); // No systems declared to run after this

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
```

## Key Features

- **Automatic Ordering**: Systems are automatically ordered by InSystems/OutSystems constraints
- **Bi-directional**: Systems can declare both what runs before them (InSystems) and what runs after them (OutSystems)
- **Simple Syntax**: Use familiar tuple syntax for multiple constraints
- **Error Handling**: Circular dependencies are handled gracefully with fallback to registration order
- **Multiple Constraints**: Supports up to 5 systems per tuple using tuple syntax

## InSystems/OutSystems Benefits

1. **Clearer Intent**: `type InSystems = (InputSystem,);` clearly shows what needs to run first
2. **Bi-directional**: One system can declare both InSystems and OutSystems
3. **Less Verbose**: No wrapper types or complex syntax needed
4. **Intuitive**: InSystems = what runs before me, OutSystems = what runs after me
5. **Composability**: Easy to add new systems without modifying existing system constraints

The system constraints ensure proper data flow and execution order in your ECS world.
