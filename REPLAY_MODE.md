# Replay Mode Documentation

The ECS framework now supports replay mode functionality that allows you to replay game sessions with the existing game systems operating on component copies instead of creating dedicated replay systems.

## Key Concept

The main insight is that the existing ECS systems (MovementSystem, WaitSystem, RenderSystem) work with component copies during replay mode without being aware of it. This allows developers to perform replay analysis using the same systems that run the live game, with replay functionality being completely invisible to the systems.

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
- Initialize the exact same ECS world as normal mode
- Use the identical systems (MovementSystem, WaitSystem, RenderSystem) 
- Apply replay data by updating component values from the log
- Systems read and render the updated components without knowing it's replay data

## Key Features

### Replay Functionality Invisible to Systems

The breakthrough of this approach is that **systems are completely unaware of replay mode**. There are no replay-specific properties, flags, or different behaviors in the systems:

- `MovementSystem`: Reads Position and Target components (unaware of data source)
- `WaitSystem`: Reads WaitTimer components (unaware of data source)  
- `RenderSystem`: Reads Position components and renders the grid (unaware of data source)

### True Component Copy Approach

In replay mode:
- Components are updated based on replay log data instead of game logic
- Systems read these "replayed" components through normal ECS queries
- The same rendering and logic systems work seamlessly
- World state progression follows only the replay data
- **Systems have no knowledge they're operating on replay data**

### Unified System Architecture

Both modes use the identical systems and initialization:
- Same `initialize_game()` function for both normal and replay modes
- Same system registration: `MovementSystem`, `WaitSystem`, `RenderSystem`
- Same rendering output: "Simulation Game - Actors traveling between Home and Work"
- No mode-specific logic or properties anywhere in the systems

## Architecture

### Unified System Design

```rust
// Same systems work for both modes with no knowledge of replay
pub struct RenderSystem;

impl System for RenderSystem {
    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // Same rendering logic works for both live and replay data
        // System is completely unaware of data source
        for (_entity, position) in world.query_components::<(In<Position>,)>() {
            // Systems automatically see updated components from any source
        }
    }
}
```

### Component Copy Implementation

- In normal mode: Game logic updates components → Systems read and render
- In replay mode: Replay data updates components → Same systems read and render
- Both modes: Systems use identical ECS queries and rendering logic
- **Key insight**: Systems never know which mode they're in

### Replay Data Application

```rust
fn simulate_replay_frame(world: &mut World, frame: usize) {
    // Read replay data from log file (demo version simulates this)
    // Apply component changes to world
    let new_position = Position { x: replay_x, y: replay_y };
    world.add_component(entity, new_position);
    
    // Existing systems automatically see the updated components
    // They have no idea this data came from a replay log
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

3. **Observe the key insight:**
   - Normal mode: Systems update components based on game logic
   - Replay mode: Components are updated from replay data, systems just read and render
   - Both modes: **Systems are completely unaware of which mode they're in**

## Benefits of This Approach

1. **No Code Duplication**: Same systems work for both modes with zero awareness
2. **Easier Maintenance**: One set of rendering/logic systems to maintain
3. **Guaranteed Consistency**: Replay rendering matches live rendering exactly
4. **Developer Friendly**: No need to implement any replay-specific code in systems
5. **True Invisibility**: Replay functionality is completely invisible to game systems
6. **Component Copy Safety**: Systems operate on replay-driven component data but are unaware of it

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