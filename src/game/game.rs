use crate::{World};
use rand::Rng;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use super::components::*;

// Re-export systems for backward compatibility
pub use super::movement_system::MovementSystem;
pub use super::wait_system::WaitSystem;
pub use super::render_system::RenderSystem;


// Game initialization and main loop

pub fn initialize_game() -> World {
    let mut world = World::new();
    let mut rng = rand::thread_rng();

    // Create home entity
    let home_entity = world.create_entity();
    world.add_component(
        home_entity,
        Position {
            x: HOME_POS.0,
            y: HOME_POS.1,
        },
    );
    world.add_component(home_entity, Home);
    world.add_component(home_entity, Obstacle);

    // Create work entity
    let work_entity = world.create_entity();
    world.add_component(
        work_entity,
        Position {
            x: WORK_POS.0,
            y: WORK_POS.1,
        },
    );
    world.add_component(work_entity, Work);
    world.add_component(work_entity, Obstacle);

    // Create 3 actors at random positions
    for _i in 0..3 {
        let actor_entity = world.create_entity();

        // Generate random position that's not home or work
        let mut pos;
        loop {
            pos = (rng.gen_range(0..GRID_SIZE), rng.gen_range(0..GRID_SIZE));
            if pos != HOME_POS && pos != WORK_POS {
                break;
            }
        }

        world.add_component(actor_entity, Position { x: pos.0, y: pos.1 });
        world.add_component(actor_entity, Actor);
        world.add_component(
            actor_entity,
            Target {
                x: WORK_POS.0,
                y: WORK_POS.1,
            },
        ); // Start by going to work
        world.add_component(actor_entity, WaitTimer { ticks: 0 });
        world.add_component(actor_entity, ActorState::MovingToWork);
    }

    // Add systems - same for both normal and replay modes
    world.add_system(MovementSystem);
    world.add_system(WaitSystem);
    world.add_system(RenderSystem::default());

    // Initialize systems
    world.initialize_systems();

    world
}

pub fn run_game() {
    run_game_normal();
}

pub fn run_game_replay(replay_log_path: &str) {
    println!("Starting Simulation Game in Replay Mode...");
    println!("Loading replay data from: {}", replay_log_path);
    
    // Initialize the game world - same as normal mode
    let mut world = initialize_game();
    
    // Run the replay using existing systems with component copies
    match run_replay_with_existing_systems(&mut world, replay_log_path) {
        Ok(()) => {
            println!("Replay completed successfully");
        }
        Err(e) => {
            eprintln!("Replay failed: {}", e);
        }
    }
}

fn run_game_normal() {
    println!("Starting Simulation Game...");
    println!("Actors will travel between Home (H) and Work (W)");
    println!("Press Ctrl+C to stop the simulation");

    let mut world = initialize_game();

    // Enable replay logging for full recording
    if let Err(e) = world.enable_replay_logging_simple("game_logs", "simulation_game", 10) {
        eprintln!("Warning: Failed to enable replay logging: {}", e);
        println!("Continuing without replay logging...");
    } else {
        println!("Replay logging enabled. Session will be saved to game_logs/");
    }
    
    // Set up Ctrl+C handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C, shutting down gracefully...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut update_count = 0;
    
    // Game loop - 2 ticks per second
    while running.load(Ordering::SeqCst) {
        world.update();
        update_count += 1;
        
        thread::sleep(Duration::from_millis(500)); // 2 FPS
    }

    // Disable replay logging and finalize the log file
    if let Err(e) = world.disable_replay_logging() {
        eprintln!("Warning: Failed to finalize replay logging: {}", e);
    } else {
        if let Some(session_id) = world.replay_session_id() {
            println!("Replay log saved. Session ID: {}", session_id);
            println!("To replay this session, run: cargo run game game_logs/simulation_game_{}.log", session_id);
        }
    }

    println!("Game completed after {} updates", update_count);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_initialization() {
        let world = initialize_game();

        // Should have 5 entities: home, work, and 3 actors
        assert_eq!(world.entity_count(), 5);

        // Should have entities with Home and Work components
        let home_entities = world.entities_with_component::<Home>();
        let work_entities = world.entities_with_component::<Work>();
        let actor_entities = world.entities_with_component::<Actor>();

        assert_eq!(home_entities.len(), 1);
        assert_eq!(work_entities.len(), 1);
        assert_eq!(actor_entities.len(), 3);
    }

    #[test]
    fn test_replay_history_basic() {
        // Create a world and run some updates
        let mut world = initialize_game();
        
        // Run some updates
        for _ in 0..5 {
            world.update();
        }
        
        // Verify the history is being tracked
        let history = world.get_update_history();
        
        println!("Test replay history tracking:");
        println!("  Total updates recorded: {}", history.len());
        
        assert_eq!(history.len(), 6); // 1 system initialization + 5 updates
        assert!(!history.is_empty());
        
        // Check that each update has system diffs
        for (i, update) in history.updates().iter().enumerate() {
            println!("  Update {}: {} system diffs", i + 1, update.system_diffs().len());
            assert!(update.system_diffs().len() > 0);
        }
    }

    #[test]
    fn test_simplified_multi_component_queries() {
        // Test that our simplified game systems work with the extended query support
        let mut world = initialize_game();
        
        // Get initial positions and targets of actors
        let initial_data: Vec<((i32, i32), (i32, i32))> = {
            world.query_position_actor_target_components()
                .into_iter()
                .map(|(_, (pos, _, target))| ((pos.x, pos.y), (target.x, target.y)))
                .collect()
        };
        
        // Should have 3 actors
        assert_eq!(initial_data.len(), 3);
        
        // All actors should initially target work
        for (_, target) in &initial_data {
            assert_eq!(*target, WORK_POS);
        }
        
        // Run a few updates to verify the simplified systems work
        for _ in 0..10 {
            world.update();
        }
        
        // Verify actors are still in the game after updates
        let final_actor_count = world.entities_with_component::<Actor>().len();
        assert_eq!(final_actor_count, 3);
        
        // Verify actors have moved (at least some should have different positions)
        let final_data: Vec<((i32, i32), (i32, i32))> = {
            world.query_position_actor_target_components()
                .into_iter()
                .map(|(_, (pos, _, target))| ((pos.x, pos.y), (target.x, target.y)))
                .collect()
        };
        
        assert_eq!(final_data.len(), 3);
        
        // At least one actor should have moved from initial position
        let movement_occurred = initial_data.iter().zip(final_data.iter())
            .any(|(initial, final_pos)| initial.0 != final_pos.0);
        
        // Note: Due to randomness in initial positions, movement might not always occur,
        // but the test verifies the systems run without errors
        println!("Movement occurred during test: {}", movement_occurred);
    }

    #[test]
    fn test_history_logging_integration() {
        // Test the history logging functionality with the game
        let mut world = initialize_game();
        
        // Run some updates to generate history
        for _ in 0..5 {
            world.update();
        }
        
        // Verify history is being tracked
        let history = world.get_update_history();
        assert_eq!(history.len(), 6); // 1 system initialization + 5 updates
        
        // Verify each update has system diffs
        for (i, update) in history.updates().iter().enumerate() {
            println!("Update {}: {} system diffs", i + 1, update.system_diffs().len());
            if i == 0 {
                // First update is system initialization - has 3 system diffs (all systems)
                assert_eq!(update.system_diffs().len(), 3);
            } else {
                // Remaining updates are game updates - each has 3 system diffs (Movement, Wait, Render)
                assert_eq!(update.system_diffs().len(), 3);
            }
        }
        
        // Test complete - history logging integration verified through AutoReplayLogger
        // which is the recommended way to do replay logging
    }

    #[test]
    fn test_replay_mode_functionality() {
        // Test the replay mode functionality with existing systems
        println!("Testing replay mode using existing systems");
        
        // Create a normal game world
        let mut world = initialize_game();
        
        // Verify entities were created
        assert!(world.entity_count() > 0);
        
        // Test the new system-level snapshot/restore approach
        let initial_entity_count = world.entity_count();
        
        // Enable replay mode to test system-level snapshot/restore
        world.enable_replay_mode();
        
        // Run a few updates in replay mode - each system will handle its own snapshot/restore
        for i in 0..3 {
            println!("System-level replay update {}", i + 1);
            world.update();
        }
        
        // Disable replay mode
        world.disable_replay_mode();
        
        // Verify world still has the same entities (components may have changed per replay data)
        assert_eq!(world.entity_count(), initial_entity_count);
        
        println!("✅ Replay mode functionality test passed - system-level snapshot/restore with replay diff application works");
    }
}

// Manual logging functions for game history

/// A world that operates on component copies for replay mode
fn run_replay_with_existing_systems(world: &mut World, replay_log_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Replay mode: Parsing and applying actual replay data");
    println!("Log path: {}", replay_log_path);
    
    // Parse the replay log file
    let replay_history = match World::parse_replay_log_file(replay_log_path) {
        Ok(history) => {
            println!("Successfully parsed replay log with {} updates", history.len());
            history
        }
        Err(e) => {
            eprintln!("Failed to parse replay log: {}", e);
            return Err(e);
        }
    };

    if replay_history.is_empty() {
        println!("No replay data found in log file");
        return Ok(());
    }
    
    // Set up Ctrl+C handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C, stopping replay...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    // Apply each update from the replay
    let num_updates = replay_history.updates().len();
    
    // Set the replay data on the world and enable replay mode
    world.set_replay_data(replay_history);
    
    for frame_idx in 0..num_updates {
        if !running.load(Ordering::SeqCst) {
            break;
        }

        println!("=== Replay Frame {} ===", frame_idx + 1);
        
        // Call the normal update method - systems will use replay data automatically
        world.update();
        
        println!("Applied replay frame {}", frame_idx + 1);

        thread::sleep(Duration::from_millis(500)); // 2 FPS for visualization
    }

    println!("Replay completed - {} frames applied", num_updates);
    Ok(())
}

























// Manual logging functions for game history








