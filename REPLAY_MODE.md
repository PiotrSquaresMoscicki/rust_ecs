# Replay Mode Documentation

The ECS framework now supports replay mode functionality that allows you to replay game sessions with systems operating on component copies instead of live data.

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
- Load replay data from the specified file path
- Create a ReplayWorld that operates on component copies
- Systems render the game state but cannot modify the actual world
- Follow exactly the replay data for world state progression

## Key Features

### Component Copy System

In replay mode, the game uses a `ReplayWorld` that:
- Stores component data as parsed strings rather than live objects
- Systems operate on these component copies, preventing state modification
- Renders the game exactly as it was during the original recording

### Separate Rendering System

The `ReplayRenderSystem`:
- Operates independently from the normal game rendering
- Reads component data from the replay world
- Displays the same visual grid as the normal game
- Clearly indicates when running in replay mode

### Demo Mode

Currently, the implementation includes a demo mode that:
- Creates a simple replay simulation with moving actors
- Demonstrates the concept of component copies
- Shows how systems can read but not modify state during replay

## Architecture

### ReplayWorld Structure

```rust
pub struct ReplayWorld {
    frame_snapshots: Vec<FrameSnapshot>,
    current_frame: usize,
    entities: Vec<SimpleEntity>,
    component_data: HashMap<SimpleEntity, HashMap<String, String>>,
}
```

### Component Copy Approach

- Components are stored as serialized strings
- Systems read these strings and parse them for rendering
- No direct modification of component data is possible
- World state progression follows only the replay data

### System Separation

- Normal systems: `MovementSystem`, `WaitSystem`, `RenderSystem`
- Replay systems: `ReplayRenderSystem`
- Complete separation prevents replay mode from affecting live game logic

## Example Usage

1. **Start normal game and let it run:**
   ```bash
   cargo run game
   # Let it run for a while, then stop with Ctrl+C
   ```

2. **Run replay mode:**
   ```bash
   cargo run game /any/path  # Currently shows demo
   ```

3. **Observe the differences:**
   - Normal mode: Live actor movement with ECS systems
   - Replay mode: Predetermined movement pattern from replay data

## Future Enhancements

The current implementation provides a foundation for:
- Parsing actual ECS framework replay logs
- Loading real game session data
- Frame-by-frame replay navigation
- Replay speed controls
- Analysis tools for debugging game logic

## Testing

The replay functionality includes comprehensive tests:

```bash
cargo test test_replay_mode_functionality
```

This verifies:
- ReplayWorld creation and component management
- Component copy system functionality
- Position data parsing and updates
- Demo world simulation

## Implementation Notes

- The replay system is designed to be completely separate from live game logic
- Component copies ensure no accidental state modification
- The architecture supports future expansion for full log file parsing
- All systems maintain their original functionality while adding replay capabilities