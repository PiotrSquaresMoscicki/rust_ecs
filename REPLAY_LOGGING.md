# ECS Replay Logging and Analysis

This document describes the automatic replay logging system implemented in the rust_ecs framework.

## Overview

The rust_ecs framework now includes comprehensive replay logging capabilities that automatically capture all game state changes for later analysis. This is particularly useful for:

- Debugging complex game behaviors
- Performance analysis
- Understanding system interactions
- Automated testing and verification
- Game analytics

## Features

### Automatic Logging
- **Zero-overhead when disabled**: No performance impact when logging is turned off
- **Comprehensive change tracking**: Captures all component modifications, entity operations, and world changes
- **File-based persistence**: Automatically saves logs to disk for later analysis
- **Configurable detail levels**: Choose what information to include in logs

### Analysis Tools
- **Statistical analysis**: Get insights into update frequency, system activity, and component usage
- **Anomaly detection**: Identify frames with unusual activity levels
- **Replay visualization**: Human-readable output of game state changes
- **Performance metrics**: Track changes per frame and system execution patterns

## Usage

### Basic Setup

```rust
use rust_ecs::{World, ReplayLogConfig};

let mut world = World::new();

// Configure replay logging
let config = ReplayLogConfig {
    enabled: true,
    log_directory: "game_logs".to_string(),
    file_prefix: "my_game".to_string(),
    flush_interval: 50,
    include_component_details: true,
};

// Enable logging
world.enable_replay_logging(config)?;

// Your game logic here...
for _ in 0..100 {
    world.update();
}

// Disable logging when done
world.disable_replay_logging()?;
```

### Configuration Options

```rust
pub struct ReplayLogConfig {
    /// Whether logging is enabled
    pub enabled: bool,
    
    /// Directory to save replay files
    pub log_directory: String,
    
    /// Base name for log files (timestamp will be appended)
    pub file_prefix: String,
    
    /// Maximum number of updates to keep in memory before flushing to disk
    pub flush_interval: usize,
    
    /// Whether to include detailed component changes in logs
    pub include_component_details: bool,
}
```

### Analysis Examples

```rust
use rust_ecs::replay_analysis;

// Get the world's update history
let history = world.get_update_history();

// Generate comprehensive statistics
let stats = replay_analysis::analyze_replay_history(history);
println!("Total updates: {}", stats.total_updates);
println!("Component types involved: {:?}", stats.component_types_involved);

// Print detailed analysis report
replay_analysis::print_replay_analysis(history);

// Find frames with unusual activity (2x average)
let anomalous = replay_analysis::find_anomalous_frames(history, 2.0);
println!("Anomalous frames: {:?}", anomalous);
```

## Log File Format

Log files use a structured text format for easy parsing and analysis:

```
# ECS Replay Log
# Session ID: 1234567890
# Timestamp: 2024-01-01 12:00:00 UTC
# Configuration: ReplayLogConfig { ... }

UPDATE 1
SYSTEMS: 3
  SYSTEM 0
    COMPONENT_CHANGES: 2
      MOD Entity(0, 5) Position Position { x: 1.0, y: 2.0 }
      ADD Entity(0, 6) Velocity Velocity { dx: 0.5, dy: 0.0 }
    WORLD_OPERATIONS: 1
      CREATE_ENTITY Entity(0, 7)

UPDATE 2
...
```

## Analysis Report Example

```
=== ECS Replay Analysis Report ===
Total Updates: 150
Total System Executions: 450
Total Component Changes: 1205
Total World Operations: 25
Entities Created: 12
Entities Removed: 3
Most Active Frame: 87 (with 45 changes)
Component Types Involved:
  - Position
  - Velocity
  - Health
  - Actor
  - Target
Average Changes per Frame: 8.03
=== End Report ===
```

## Demo Commands

The framework includes several demo commands:

```bash
# Run the main ECS demo
cargo run

# Run the actor simulation game
cargo run game

# Run the replay analysis demo
cargo run replay-demo
```

## Performance Considerations

- **Memory usage**: The logging system accumulates data in memory before flushing to disk. Adjust `flush_interval` based on your memory constraints.
- **Disk space**: Log files can grow large with detailed component tracking. Monitor disk usage in production.
- **I/O performance**: File writes are buffered, but frequent flushing may impact performance in write-heavy scenarios.

## Best Practices

1. **Enable logging during development and testing**, disable in production unless needed for analytics
2. **Use appropriate flush intervals** - smaller values use more I/O but reduce memory usage
3. **Monitor log file sizes** and implement log rotation for long-running applications
4. **Use the analysis tools** to understand your game's behavior patterns
5. **Test with logging enabled** to ensure your game logic is deterministic

## Integration with Tests

The logging system integrates seamlessly with your test suite:

```rust
#[test]
fn test_game_behavior() {
    let mut world = setup_test_world();
    
    // Enable logging for this test
    let config = ReplayLogConfig::default();
    world.enable_replay_logging(config)?;
    
    // Run your test scenario
    simulate_game_scenario(&mut world);
    
    // Analyze the results
    let history = world.get_update_history();
    let stats = replay_analysis::analyze_replay_history(history);
    
    // Verify expected behavior
    assert_eq!(stats.entities_created, 5);
    assert!(stats.total_component_changes > 0);
}
```

This replay logging system provides developers with powerful tools for understanding and debugging their ECS-based games and simulations.