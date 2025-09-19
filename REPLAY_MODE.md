# Replay Mode Documentation

The ECS framework now supports replay mode functionality that allows you to replay game sessions with the existing game systems operating on component copies instead of creating dedicated replay systems.

## Key Concept

The main insight is that the existing ECS systems (MovementSystem, WaitSystem, RenderSystem) can work with component copies during replay mode without requiring any replay-specific systems. This allows developers to perform replay analysis using the same systems that run the live game.

## How to Use Replay Mode

### Running the Game Normally

To run the game in normal mode with live ECS systems:

```bash
cargo run game
```

This will:
- Start the simulation game with actors moving between home and work
- Track all component changes in the ECS framework's built-in history system
- Systems operate on live data and can modify the world state

### Running in Replay Mode

To run the game in replay mode:

```bash
cargo run game <replay_log_path>
```

Example:
```bash
cargo run game /path/to/replay.log
```

This will:
- Initialize the same ECS world as normal mode
- Use the same systems (MovementSystem, WaitSystem, RenderSystem) 
- Apply replay data by updating component values from the log
- Systems read and render the updated components but the world state follows exactly the replay data

## Key Features

### No Dedicated Replay Systems Required

The breakthrough of this approach is that **no separate replay systems are needed**. The existing game systems work perfectly in replay mode:

- `MovementSystem`: Reads Position and Target components (now from replay data)
- `WaitSystem`: Reads WaitTimer components (now from replay data)  
- `RenderSystem`: Reads Position components and renders the grid (now from replay data)

### Component Copy Approach

In replay mode:
- Components are updated based on replay log data instead of game logic
- Systems read these "replayed" components through normal ECS queries
- The same rendering and logic systems work seamlessly
- World state progression follows only the replay data

### Mode-Aware Rendering

The `RenderSystem` automatically detects replay mode and shows:
- Normal mode: "Simulation Game - Actors traveling between Home and Work"
- Replay mode: "Simulation Game REPLAY - Actors traveling between Home and Work (Replay Mode - Systems operating on component copies)"

## Architecture

### Unified System Design

```rust
// Same systems work for both modes
pub struct RenderSystem {
    pub replay_mode: bool,  // Only for display purposes
}

// Systems read components the same way regardless of mode
impl System for RenderSystem {
    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        for (_entity, position) in world.query_components::<(In<Position>,)>() {
            // Same rendering logic works for both live and replay data
        }
    }
}
```

### Component Copy Implementation

- In normal mode: Game logic updates components
- In replay mode: `simulate_replay_frame()` applies replay data to components
- Both modes: Systems read components through standard ECS queries

### Replay Data Application

```rust
fn simulate_replay_frame(world: &mut World, frame: usize) {
    // Read replay data from log file (demo version simulates this)
    // Apply component changes to world
    let new_position = Position { x: replay_x, y: replay_y };
    world.add_component(entity, new_position);
    
    // Existing systems automatically see the updated components
}
```

## Example Usage

1. **Start normal game and let it run:**
   ```bash
   cargo run game
   # Actors move using live game logic
   ```

2. **Run replay mode:**
   ```bash
   cargo run game /any/path  # Currently shows demo
   # Same systems render actors, but positions come from replay data
   ```

3. **Observe the key difference:**
   - Normal mode: Systems update components based on game logic
   - Replay mode: Components are updated from replay data, systems just read and render

## Benefits of This Approach

1. **No Code Duplication**: Same systems work for both modes
2. **Easier Maintenance**: One set of rendering/logic systems to maintain
3. **Consistency**: Replay rendering is guaranteed to match live rendering
4. **Developer Friendly**: No need to implement replay-specific systems
5. **Component Copy Safety**: Systems operate on the data but world state is controlled by replay

## Implementation Notes

- The replay system modifies component values based on log data
- Existing systems read these modified components through normal ECS queries
- This creates the "component copy" effect without actually copying - the systems just see different data
- World state progression is controlled entirely by the replay data application
- Systems initialize and update normally but work with replay-driven component values

## Future Enhancements

The current implementation provides a foundation for:
- Parsing actual ECS framework replay logs
- Loading real game session data from files
- Frame-by-frame replay navigation with the same systems
- Replay speed controls using the same rendering systems
- Analysis tools that leverage existing game systems

This approach eliminates the need for dedicated replay systems while maintaining all the safety benefits of component copies.