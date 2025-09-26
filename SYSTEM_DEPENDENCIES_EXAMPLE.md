# System Dependencies and Ordering - Usage Example

This example demonstrates both the Dependencies feature and the new Before/After system ordering feature implemented in the Rust ECS framework.

## Overview

There are now two approaches to system ordering:

1. **Dependencies Approach**: Systems declare what they depend on (existing feature)
2. **Before/After Approach**: Systems declare what they run before or after (new feature)

Both approaches can be used together and use the same topological sorting algorithm to determine execution order.

## Basic Usage - Dependencies Approach

```rust
use rust_ecs::{System, World, WorldView};

// Base system with no dependencies
struct MovementSystem;
impl System for MovementSystem {
    type InComponents = (Velocity,);
    type OutComponents = (Position,);
    type Dependencies = (); // No dependencies
    type Ordering = (); // No before/after constraints

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
    type Ordering = (); // No before/after constraints

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

## New Feature: Before/After System Ordering

The new Before/After approach provides an alternative way to declare system ordering constraints:

```rust
use rust_ecs::{System, World, WorldView, Before, After};

// Base system that runs first
struct InputSystem;
impl System for InputSystem {
    type InComponents = ();
    type OutComponents = ();
    type Dependencies = ();
    type Ordering = Before<(MovementSystem, PhysicsSystem)>; // Run before Movement and Physics
    
    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}

// System that runs after input but before rendering
struct MovementSystem;
impl System for MovementSystem {
    type InComponents = ();
    type OutComponents = ();
    type Dependencies = ();
    type Ordering = After<(InputSystem,)>; // Run after Input
    
    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}

// System that runs last
struct RenderSystem;
impl System for RenderSystem {
    type InComponents = ();
    type OutComponents = ();
    type Dependencies = ();
    type Ordering = After<(MovementSystem, PhysicsSystem)>; // Run after Movement and Physics
    
    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}
```

## Mixed Dependencies and Before/After

You can combine both approaches in the same system:

```rust
struct HybridSystem;
impl System for HybridSystem {
    type InComponents = ();
    type OutComponents = ();
    type Dependencies = (CoreSystem,); // Must run after CoreSystem (old approach)
    type Ordering = Before<(RenderSystem,)>; // Must run before RenderSystem (new approach)
    
    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}
```

## Main Function Example

```rust
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

- **Automatic Ordering**: Systems are automatically ordered by dependencies and/or before/after constraints during initialization, updates, and deinitialization
- **Two Approaches**: Use either Dependencies (old) or Before/After (new) or both together
- **Reverse Deinitialization**: Systems deinitialize in reverse dependency order (dependencies last)
- **Error Handling**: Circular dependencies and missing dependencies are handled gracefully with fallback to registration order
- **Backward Compatibility**: Existing systems work unchanged with `type Dependencies = ()` and `type Ordering = ()`
- **Multiple Constraints**: Supports up to 3 dependencies/constraints per tuple using tuple syntax

## Before/After Benefits

The new Before/After approach offers several advantages:

1. **Clearer Intent**: `Before<(RenderSystem,)>` is more explicit than `Dependencies = ()`
2. **Bi-directional**: One system can declare what it runs before AND after
3. **Less Coupling**: Systems don't need to know their dependencies, just their ordering constraints
4. **Composability**: Easy to add new systems without modifying existing system dependencies

## Output Example

```
InputSystem initialized
MovementSystem initialized (after InputSystem)
PhysicsSystem initialized (after InputSystem)
RenderSystem initialized (after MovementSystem and PhysicsSystem)

Frame 1
InputSystem processing input
MovementSystem updating entities
PhysicsSystem processing physics
RenderSystem rendering frame

Frame 2
InputSystem processing input
MovementSystem updating entities
PhysicsSystem processing physics
RenderSystem rendering frame

RenderSystem deinitialized (before MovementSystem and PhysicsSystem)
PhysicsSystem deinitialized (before InputSystem)
MovementSystem deinitialized (before InputSystem)
InputSystem deinitialized
```

The system dependencies and before/after constraints ensure that:
1. **InputSystem** always initializes and updates first
2. **MovementSystem** and **PhysicsSystem** run after InputSystem
3. **RenderSystem** runs after both MovementSystem and PhysicsSystem
4. Systems deinitialize in reverse order
5. This guarantees proper data flow: Input → Processing → Rendering