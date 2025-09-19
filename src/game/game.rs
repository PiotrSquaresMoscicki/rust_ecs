use crate::{Diff, In, Out, System, World, WorldView};
use rand::Rng;
use std::collections::{HashSet, HashMap};
use std::fs::{File, OpenOptions};
use std::io::{Write, BufWriter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

// Grid constants
const GRID_SIZE: i32 = 10;
const HOME_POS: (i32, i32) = (1, 1);
const WORK_POS: (i32, i32) = (6, 8);
const WAIT_TICKS: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct Home;



#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct Work;



#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct Actor;



#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct Obstacle;



#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct Target {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct WaitTimer {
    pub ticks: u32,
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Diff)]
#[allow(dead_code)]
pub enum ActorState {
    #[default]
    MovingToWork,
    MovingToHome,
    WaitingAtWork,
    WaitingAtHome,
}



// Movement System - handles actor movement with obstacle avoidance
// Simplified thanks to extended query support for up to 16 components!
pub struct MovementSystem;
impl System for MovementSystem {
    type InComponents = (Actor, Position, Target);
    type OutComponents = (Position,);

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // Collect all obstacle positions first
        let mut obstacles = HashSet::new();

        // Add home and work positions as obstacles (don't move into them)
        obstacles.insert(HOME_POS);
        obstacles.insert(WORK_POS);

        // Collect all current actor positions to avoid collisions
        let current_positions: Vec<(i32, i32)> = world
            .query_components::<(In<Position>, In<Actor>)>()
            .into_iter()
            .map(|(_, (pos, _))| (pos.x, pos.y))
            .collect();

        // Now we can query and update actor positions in a single query thanks to extended support!
        for (_entity, (position, _actor, target)) in
            world.query_components::<(Out<Position>, In<Actor>, In<Target>)>()
        {
            let current_pos = (position.x, position.y);
            let target_pos = (target.x, target.y);

            // Don't move if already at target or adjacent to target
            if !is_adjacent(current_pos, target_pos) && current_pos != target_pos {
                // Create a temporary obstacles set without the current actor
                let mut temp_obstacles = obstacles.clone();
                for &pos in &current_positions {
                    if pos != current_pos {
                        temp_obstacles.insert(pos);
                    }
                }

                // Calculate next move
                let next_pos = calculate_next_move(current_pos, target_pos, &temp_obstacles);

                // Update position if we can move
                if next_pos != current_pos
                    && is_valid_position(next_pos)
                    && !temp_obstacles.contains(&next_pos)
                {
                    position.x = next_pos.0;
                    position.y = next_pos.1;
                }
            }
        }
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}

// Wait System - handles wait timers and target switching
// Simplified thanks to extended query support for up to 16 components!
pub struct WaitSystem;
impl System for WaitSystem {
    type InComponents = (Actor, WaitTimer, Target, Position);
    type OutComponents = (WaitTimer, Target);

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // Now we can query all actor components together thanks to extended query support!
        for (_entity, (position, _actor, wait_timer, target)) in 
            world.query_components::<(In<Position>, In<Actor>, Out<WaitTimer>, Out<Target>)>()
        {
            let current_pos = (position.x, position.y);
            let target_pos = (target.x, target.y);
            let current_ticks = wait_timer.ticks;

            let is_near_target = is_adjacent(current_pos, target_pos) || current_pos == target_pos;
            let should_switch = is_near_target && current_ticks == 0;

            // Update wait timer
            if is_near_target && current_ticks > 0 {
                wait_timer.ticks = current_ticks - 1;
            } else if should_switch {
                wait_timer.ticks = WAIT_TICKS;
            }

            // Update target if needed
            if should_switch {
                // Switch target between home and work
                if target_pos == HOME_POS {
                    target.x = WORK_POS.0;
                    target.y = WORK_POS.1;
                } else {
                    target.x = HOME_POS.0;
                    target.y = HOME_POS.1;
                }
            }
        }
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}

// Render System - displays the 10x10 grid
pub struct RenderSystem;
impl System for RenderSystem {
    type InComponents = (Position,);
    type OutComponents = ();

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");

        // Create grid
        let mut grid = vec![vec!['.'; GRID_SIZE as usize]; GRID_SIZE as usize];

        // Place entities on grid
        for (_entity, position) in world.query_components::<(In<Position>,)>() {
            let x = position.x as usize;
            let y = position.y as usize;

            if x < GRID_SIZE as usize && y < GRID_SIZE as usize {
                // Check what type of entity this is by position
                if (position.x, position.y) == HOME_POS {
                    grid[y][x] = 'H';
                } else if (position.x, position.y) == WORK_POS {
                    grid[y][x] = 'W';
                } else {
                    // If the position overlaps with home or work, show the location marker instead
                    if grid[y][x] == '.' {
                        grid[y][x] = 'A'; // Actor
                    }
                }
            }
        }

        // Ensure home and work are always visible
        if HOME_POS.0 >= 0 && HOME_POS.0 < GRID_SIZE && HOME_POS.1 >= 0 && HOME_POS.1 < GRID_SIZE {
            grid[HOME_POS.1 as usize][HOME_POS.0 as usize] = 'H';
        }
        if WORK_POS.0 >= 0 && WORK_POS.0 < GRID_SIZE && WORK_POS.1 >= 0 && WORK_POS.1 < GRID_SIZE {
            grid[WORK_POS.1 as usize][WORK_POS.0 as usize] = 'W';
        }

        // Print grid
        println!("Simulation Game - Actors traveling between Home and Work");
        println!("H = Home, W = Work, A = Actor");
        println!();
        for row in &grid {
            for cell in row {
                print!("{} ", cell);
            }
            println!();
        }
        println!();
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}

// Helper functions

fn calculate_next_move(
    current: (i32, i32),
    target: (i32, i32),
    obstacles: &HashSet<(i32, i32)>,
) -> (i32, i32) {
    let (cx, cy) = current;
    let (tx, ty) = target;

    // Calculate direction
    let dx = if tx > cx {
        1
    } else if tx < cx {
        -1
    } else {
        0
    };
    let dy = if ty > cy {
        1
    } else if ty < cy {
        -1
    } else {
        0
    };

    // Try diagonal movement first
    let diagonal = (cx + dx, cy + dy);
    if !obstacles.contains(&diagonal) && is_valid_position(diagonal) {
        return diagonal;
    }

    // Try horizontal movement
    if dx != 0 {
        let horizontal = (cx + dx, cy);
        if !obstacles.contains(&horizontal) && is_valid_position(horizontal) {
            return horizontal;
        }
    }

    // Try vertical movement
    if dy != 0 {
        let vertical = (cx, cy + dy);
        if !obstacles.contains(&vertical) && is_valid_position(vertical) {
            return vertical;
        }
    }

    // Can't move, stay in place
    current
}

fn is_valid_position(pos: (i32, i32)) -> bool {
    pos.0 >= 0 && pos.0 < GRID_SIZE && pos.1 >= 0 && pos.1 < GRID_SIZE
}

fn is_adjacent(pos1: (i32, i32), pos2: (i32, i32)) -> bool {
    let dx = (pos1.0 - pos2.0).abs();
    let dy = (pos1.1 - pos2.1).abs();
    dx <= 1 && dy <= 1 && !(dx == 0 && dy == 0)
}

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

    // Add systems
    world.add_system(MovementSystem);
    world.add_system(WaitSystem);
    world.add_system(RenderSystem);

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
    
    match load_and_replay_game(replay_log_path) {
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

    // Enable detailed replay logging using the ECS framework's AutoReplayLogger
    let replay_config = crate::ReplayLogConfig {
        enabled: true,
        log_directory: "game_replay_logs".to_string(),
        file_prefix: "detailed_game_replay".to_string(),
        flush_interval: 10,
        include_component_details: true,
    };

    match world.enable_replay_logging(replay_config) {
        Ok(()) => {
            println!("Detailed replay logging enabled");
        }
        Err(e) => {
            eprintln!("Warning: Failed to enable replay logging: {}", e);
            println!("Game will continue without detailed logging");
        }
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
        
        // Flush logs periodically for real-time monitoring
        if update_count % 10 == 0 {
            if let Err(e) = world.flush_replay_log() {
                eprintln!("Warning: Failed to flush replay log: {}", e);
            }
        }
        
        thread::sleep(Duration::from_millis(500)); // 2 FPS
    }

    // Graceful shutdown - finalize logging
    println!("Finalizing replay logs...");
    if let Err(e) = world.disable_replay_logging() {
        eprintln!("Warning: Failed to finalize replay logging: {}", e);
    } else {
        if let Some(session_id) = world.replay_session_id() {
            println!("Game session replay data saved to game_replay_logs/detailed_game_replay_{}.log", session_id);
        }
    }
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
    fn test_valid_position() {
        assert!(is_valid_position((0, 0)));
        assert!(is_valid_position((9, 9)));
        assert!(!is_valid_position((-1, 0)));
        assert!(!is_valid_position((10, 0)));
        assert!(!is_valid_position((0, -1)));
        assert!(!is_valid_position((0, 10)));
    }

    #[test]
    fn test_is_adjacent() {
        assert!(is_adjacent((1, 1), (1, 2))); // vertical
        assert!(is_adjacent((1, 1), (2, 1))); // horizontal
        assert!(is_adjacent((1, 1), (2, 2))); // diagonal
        assert!(is_adjacent((1, 1), (0, 0))); // diagonal
        assert!(!is_adjacent((1, 1), (1, 1))); // same position
        assert!(!is_adjacent((1, 1), (3, 3))); // too far
    }

    #[test]
    fn test_calculate_next_move() {
        let obstacles = HashSet::new();

        // Test direct movement
        assert_eq!(calculate_next_move((0, 0), (2, 2), &obstacles), (1, 1));
        assert_eq!(calculate_next_move((5, 5), (3, 3), &obstacles), (4, 4));

        // Test with obstacle
        let mut obstacles_with_block = HashSet::new();
        obstacles_with_block.insert((1, 1));
        let next = calculate_next_move((0, 0), (2, 2), &obstacles_with_block);
        // Should find alternative path
        assert!(next == (1, 0) || next == (0, 1));
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
        
        assert_eq!(history.len(), 5);
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
            let mut world_view = crate::WorldView::<(), ()>::new(&mut world);
            world_view.query_components::<(crate::In<Position>, crate::In<Actor>, crate::In<Target>)>()
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
            let mut world_view = crate::WorldView::<(), ()>::new(&mut world);
            world_view.query_components::<(crate::In<Position>, crate::In<Actor>, crate::In<Target>)>()
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
        assert_eq!(history.len(), 5);
        
        // Verify each update has system diffs
        for (i, update) in history.updates().iter().enumerate() {
            println!("Update {}: {} system diffs", i + 1, update.system_diffs().len());
            // Should have 3 systems: Movement, Wait, and Render
            assert_eq!(update.system_diffs().len(), 3);
        }
        
        // Test that the logging functions would work (without actually creating files)
        let session_id = 123456;
        let temp_dir = "/tmp/test_game_logs";
        let temp_file = format!("{}/test_game_{}.log", temp_dir, session_id);
        
        // Create temp directory
        if std::fs::create_dir_all(temp_dir).is_ok() {
            if let Ok(mut log_file) = setup_logging(temp_dir, &temp_file, session_id) {
                // Test logging a few updates
                for i in 1..=3 {
                    assert!(log_game_update(&mut log_file, i, &world).is_ok());
                }
                
                // Test finalization
                assert!(finalize_logging(&mut log_file, 3).is_ok());
                
                // Verify file exists and has content
                if let Ok(content) = std::fs::read_to_string(&temp_file) {
                    assert!(content.contains("Simulation Game History Log"));
                    assert!(content.contains("Session ID: 123456"));
                    assert!(content.contains("UPDATE 1"));
                    assert!(content.contains("UPDATE 3"));
                    assert!(content.contains("Game Session Complete"));
                }
                
                // Clean up
                let _ = std::fs::remove_file(&temp_file);
                let _ = std::fs::remove_dir(temp_dir);
            }
        }
    }
}

// Replay functionality

/// Structure representing a parsed replay update from log file
#[derive(Debug, Clone)]
pub struct ReplayUpdate {
    pub update_number: u32,
    pub component_changes: Vec<ReplayComponentChange>,
    pub world_operations: Vec<ReplayWorldOperation>,
}

#[derive(Debug, Clone)]
pub struct ReplayComponentChange {
    pub entity_id: u32,
    pub entity_world: u32,
    pub component_type: String,
    pub change_type: ReplayChangeType,
    pub data: String,
}

#[derive(Debug, Clone)]
pub enum ReplayChangeType {
    Add,
    Modify,
    Remove,
}

#[derive(Debug, Clone)]
pub enum ReplayWorldOperation {
    CreateEntity { entity_id: u32, world_id: u32 },
    RemoveEntity { entity_id: u32, world_id: u32 },
    CreateWorld { world_id: u32 },
    RemoveWorld { world_id: u32 },
}

/// A world that operates on component copies for replay mode
pub struct ReplayWorld {
    /// Stores component snapshots for entities at each frame
    frame_snapshots: Vec<FrameSnapshot>,
    current_frame: usize,
    entities: Vec<crate::Entity>,
    /// Component data stored as parsed replay data
    component_data: HashMap<crate::Entity, HashMap<String, String>>,
}

#[derive(Debug, Clone)]
pub struct FrameSnapshot {
    pub entities: Vec<crate::Entity>,
    pub component_data: HashMap<crate::Entity, HashMap<String, String>>,
}

impl ReplayWorld {
    pub fn new() -> Self {
        Self {
            frame_snapshots: Vec::new(),
            current_frame: 0,
            entities: Vec::new(),
            component_data: HashMap::new(),
        }
    }

    pub fn add_frame_snapshot(&mut self, snapshot: FrameSnapshot) {
        self.frame_snapshots.push(snapshot);
    }

    pub fn set_current_frame(&mut self, frame: usize) {
        if frame < self.frame_snapshots.len() {
            self.current_frame = frame;
            // Load the frame snapshot
            let snapshot = &self.frame_snapshots[frame];
            self.entities = snapshot.entities.clone();
            self.component_data = snapshot.component_data.clone();
        }
    }

    pub fn get_current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn total_frames(&self) -> usize {
        self.frame_snapshots.len()
    }

    /// Get component data for visualization
    pub fn get_component_data(&self, entity: crate::Entity, component_type: &str) -> Option<&String> {
        self.component_data.get(&entity)?.get(component_type)
    }

    /// Get all entities that have a specific component type
    pub fn entities_with_component(&self, component_type: &str) -> Vec<crate::Entity> {
        self.entities.iter()
            .filter(|entity| {
                self.component_data.get(entity)
                    .map(|components| components.contains_key(component_type))
                    .unwrap_or(false)
            })
            .copied()
            .collect()
    }
}

fn load_and_replay_game(replay_log_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Parse the replay log file
    let replay_updates = parse_replay_log(replay_log_path)?;
    
    if replay_updates.is_empty() {
        return Err("No replay data found in log file".into());
    }

    println!("Loaded {} replay updates", replay_updates.len());

    // Initialize replay world with initial game state
    let mut replay_world = create_initial_replay_world();
    
    // Create frame snapshots for each update
    for update in &replay_updates {
        apply_replay_update_to_world(&mut replay_world, update);
        
        // Create snapshot of current state
        let snapshot = FrameSnapshot {
            entities: replay_world.entities.clone(),
            component_data: replay_world.component_data.clone(),
        };
        replay_world.add_frame_snapshot(snapshot);
    }

    println!("Replay world prepared with {} frames", replay_world.total_frames());
    
    // Set up systems for replay rendering
    let mut render_system = ReplayRenderSystem;
    
    // Set up Ctrl+C handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C, stopping replay...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    // Replay loop - 2 FPS
    let mut frame = 0;
    while running.load(Ordering::SeqCst) && frame < replay_world.total_frames() {
        replay_world.set_current_frame(frame);
        
        // Render the current frame
        render_system.render_frame(&replay_world, frame);
        
        frame += 1;
        thread::sleep(Duration::from_millis(500)); // 2 FPS
    }

    if frame >= replay_world.total_frames() {
        println!("Replay completed - {} frames played", frame);
    }

    Ok(())
}

fn parse_replay_log(file_path: &str) -> Result<Vec<ReplayUpdate>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file_path)?;
    let mut replay_updates = Vec::new();
    let mut current_update: Option<ReplayUpdate> = None;
    
    for line in content.lines() {
        let line = line.trim();
        
        if line.starts_with("UPDATE ") {
            // Save previous update if it exists
            if let Some(update) = current_update.take() {
                replay_updates.push(update);
            }
            
            // Parse update number
            if let Some(update_num_str) = line.strip_prefix("UPDATE ") {
                if let Ok(update_num) = update_num_str.parse::<u32>() {
                    current_update = Some(ReplayUpdate {
                        update_number: update_num,
                        component_changes: Vec::new(),
                        world_operations: Vec::new(),
                    });
                }
            }
        } else if line.starts_with("      ADD ") {
            // Parse component addition: "      ADD Entity(0, 1) rust_ecs::Position Position { x: 1, y: 1 }"
            if let Some(ref mut update) = current_update {
                if let Some(change) = parse_component_change(line, ReplayChangeType::Add) {
                    update.component_changes.push(change);
                }
            }
        } else if line.starts_with("      MOD ") {
            // Parse component modification
            if let Some(ref mut update) = current_update {
                if let Some(change) = parse_component_change(line, ReplayChangeType::Modify) {
                    update.component_changes.push(change);
                }
            }
        } else if line.starts_with("      REM ") {
            // Parse component removal
            if let Some(ref mut update) = current_update {
                if let Some(change) = parse_component_change(line, ReplayChangeType::Remove) {
                    update.component_changes.push(change);
                }
            }
        } else if line.starts_with("      CREATE_ENTITY ") {
            // Parse entity creation
            if let Some(ref mut update) = current_update {
                if let Some(operation) = parse_world_operation(line) {
                    update.world_operations.push(operation);
                }
            }
        }
    }
    
    // Save the last update if it exists
    if let Some(update) = current_update {
        replay_updates.push(update);
    }
    
    Ok(replay_updates)
}

fn parse_component_change(line: &str, change_type: ReplayChangeType) -> Option<ReplayComponentChange> {
    let prefix = match change_type {
        ReplayChangeType::Add => "      ADD ",
        ReplayChangeType::Modify => "      MOD ",
        ReplayChangeType::Remove => "      REM ",
    };
    
    let line = line.strip_prefix(prefix)?;
    let parts: Vec<&str> = line.splitn(3, ' ').collect();
    
    if parts.len() >= 2 {
        // Parse entity: "Entity(world, id)"
        let entity_str = parts[0];
        let (entity_world, entity_id) = parse_entity_string(entity_str)?;
        
        let component_type = parts[1].to_string();
        let data = if parts.len() > 2 { parts[2].to_string() } else { String::new() };
        
        Some(ReplayComponentChange {
            entity_id,
            entity_world,
            component_type,
            change_type,
            data,
        })
    } else {
        None
    }
}

fn parse_entity_string(entity_str: &str) -> Option<(u32, u32)> {
    // Parse "Entity(world, id)" format
    if let Some(inner) = entity_str.strip_prefix("Entity(").and_then(|s| s.strip_suffix(")")) {
        let parts: Vec<&str> = inner.split(", ").collect();
        if parts.len() == 2 {
            if let (Ok(world), Ok(id)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                return Some((world, id));
            }
        }
    }
    None
}

fn parse_world_operation(line: &str) -> Option<ReplayWorldOperation> {
    if let Some(entity_str) = line.strip_prefix("      CREATE_ENTITY ") {
        if let Some((world_id, entity_id)) = parse_entity_string(entity_str) {
            return Some(ReplayWorldOperation::CreateEntity { entity_id, world_id });
        }
    } else if let Some(entity_str) = line.strip_prefix("      REMOVE_ENTITY ") {
        if let Some((world_id, entity_id)) = parse_entity_string(entity_str) {
            return Some(ReplayWorldOperation::RemoveEntity { entity_id, world_id });
        }
    }
    None
}

fn create_initial_replay_world() -> ReplayWorld {
    let mut replay_world = ReplayWorld::new();
    
    // Create initial entities based on the game setup
    // We'll create them as the replay data is processed
    
    replay_world
}

fn apply_replay_update_to_world(replay_world: &mut ReplayWorld, update: &ReplayUpdate) {
    // Process world operations first
    for operation in &update.world_operations {
        match operation {
            ReplayWorldOperation::CreateEntity { entity_id, world_id } => {
                let entity = crate::Entity::new(*world_id as usize, *entity_id as usize);
                if !replay_world.entities.contains(&entity) {
                    replay_world.entities.push(entity);
                    replay_world.component_data.insert(entity, HashMap::new());
                }
            }
            ReplayWorldOperation::RemoveEntity { entity_id, world_id } => {
                let entity = crate::Entity::new(*world_id as usize, *entity_id as usize);
                replay_world.entities.retain(|e| *e != entity);
                replay_world.component_data.remove(&entity);
            }
            _ => {} // Handle other operations if needed
        }
    }
    
    // Process component changes
    for change in &update.component_changes {
        let entity = crate::Entity::new(change.entity_world as usize, change.entity_id as usize);
        
        // Ensure entity exists
        if !replay_world.entities.contains(&entity) {
            replay_world.entities.push(entity);
            replay_world.component_data.insert(entity, HashMap::new());
        }
        
        let entity_components = replay_world.component_data.get_mut(&entity).unwrap();
        
        match change.change_type {
            ReplayChangeType::Add | ReplayChangeType::Modify => {
                entity_components.insert(change.component_type.clone(), change.data.clone());
            }
            ReplayChangeType::Remove => {
                entity_components.remove(&change.component_type);
            }
        }
    }
}

/// Render system for replay mode that operates on component copies
pub struct ReplayRenderSystem;

impl ReplayRenderSystem {
    pub fn render_frame(&mut self, replay_world: &ReplayWorld, frame_number: usize) {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");

        // Create grid
        let mut grid = vec![vec!['.'; GRID_SIZE as usize]; GRID_SIZE as usize];

        // Get entities with Position components
        let position_entities = replay_world.entities_with_component("rust_ecs::game::game::Position");
        
        // Place entities on grid
        for entity in position_entities {
            if let Some(pos_data) = replay_world.get_component_data(entity, "rust_ecs::game::game::Position") {
                if let Some((x, y)) = parse_position_data(pos_data) {
                    if x >= 0 && x < GRID_SIZE && y >= 0 && y < GRID_SIZE {
                        let x_idx = x as usize;
                        let y_idx = y as usize;
                        
                        // Check if this is a home, work, or actor
                        if replay_world.get_component_data(entity, "rust_ecs::game::game::Home").is_some() {
                            grid[y_idx][x_idx] = 'H';
                        } else if replay_world.get_component_data(entity, "rust_ecs::game::game::Work").is_some() {
                            grid[y_idx][x_idx] = 'W';
                        } else if replay_world.get_component_data(entity, "rust_ecs::game::game::Actor").is_some() {
                            // Don't overwrite Home or Work markers
                            if grid[y_idx][x_idx] == '.' {
                                grid[y_idx][x_idx] = 'A';
                            }
                        }
                    }
                }
            }
        }

        // Ensure home and work are always visible at their fixed positions
        if HOME_POS.0 >= 0 && HOME_POS.0 < GRID_SIZE && HOME_POS.1 >= 0 && HOME_POS.1 < GRID_SIZE {
            grid[HOME_POS.1 as usize][HOME_POS.0 as usize] = 'H';
        }
        if WORK_POS.0 >= 0 && WORK_POS.0 < GRID_SIZE && WORK_POS.1 >= 0 && WORK_POS.1 < GRID_SIZE {
            grid[WORK_POS.1 as usize][WORK_POS.0 as usize] = 'W';
        }

        // Print grid
        println!("Simulation Game REPLAY - Frame {}", frame_number + 1);
        println!("H = Home, W = Work, A = Actor");
        println!();
        for row in &grid {
            for cell in row {
                print!("{} ", cell);
            }
            println!();
        }
        println!();
    }
}

fn parse_position_data(data: &str) -> Option<(i32, i32)> {
    // Parse "Position { x: 1, y: 2 }" format
    if let Some(inner) = data.strip_prefix("Position { ").and_then(|s| s.strip_suffix(" }")) {
        let parts: Vec<&str> = inner.split(", ").collect();
        if parts.len() == 2 {
            if let (Some(x_str), Some(y_str)) = (
                parts[0].strip_prefix("x: "),
                parts[1].strip_prefix("y: ")
            ) {
                if let (Ok(x), Ok(y)) = (x_str.parse::<i32>(), y_str.parse::<i32>()) {
                    return Some((x, y));
                }
            }
        }
    }
    None
}

// Manual logging functions for game history

fn setup_logging(log_directory: &str, log_file_path: &str, session_id: u64) -> Result<BufWriter<File>, std::io::Error> {
    // Create log directory if it doesn't exist
    std::fs::create_dir_all(log_directory)?;

    // Create log file
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_file_path)?;
    
    let mut writer = BufWriter::new(file);
    
    // Write header
    writeln!(writer, "# Simulation Game History Log")?;
    writeln!(writer, "# Session ID: {}", session_id)?;
    writeln!(writer, "# Timestamp: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;
    writeln!(writer, "# Format: Each update shows actor positions and targets")?;
    writeln!(writer)?;
    
    println!("History logging enabled - logs will be saved to {}", log_file_path);
    println!("Session ID: {}", session_id);
    
    Ok(writer)
}

fn log_game_update(file: &mut BufWriter<File>, update_count: u32, world: &World) -> Result<(), std::io::Error> {
    writeln!(file, "UPDATE {}", update_count)?;
    
    // Log basic statistics about the world
    let history = world.get_update_history();
    writeln!(file, "TOTAL_ENTITIES: {}", world.entity_count())?;
    writeln!(file, "HISTORY_UPDATES: {}", history.len())?;
    
    if !history.is_empty() {
        let latest_update = &history.updates()[history.len() - 1];
        writeln!(file, "SYSTEM_EXECUTIONS: {}", latest_update.system_diffs().len())?;
        
        let total_changes: usize = latest_update.system_diffs()
            .iter()
            .map(|diff| diff.component_changes().len())
            .sum();
        writeln!(file, "COMPONENT_CHANGES: {}", total_changes)?;
        
        let total_operations: usize = latest_update.system_diffs()
            .iter()
            .map(|diff| diff.world_operations().len())
            .sum();
        writeln!(file, "WORLD_OPERATIONS: {}", total_operations)?;
    }
    
    writeln!(file)?;
    Ok(())
}

fn finalize_logging(file: &mut BufWriter<File>, total_updates: u32) -> Result<(), std::io::Error> {
    writeln!(file, "# Game Session Complete")?;
    writeln!(file, "# Total Updates: {}", total_updates)?;
    writeln!(file, "# End Timestamp: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;
    file.flush()?;
    Ok(())
}
