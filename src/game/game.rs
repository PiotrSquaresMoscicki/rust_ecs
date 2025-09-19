use crate::{Diff, In, Out, System, World, WorldView, ReplayLogConfig};
use rand::Rng;
use std::collections::HashSet;
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

        // Move each actor individually
        let mut actor_data: Vec<((i32, i32), (i32, i32))> = Vec::new();

        // First, collect all actor positions and their targets
        for (_entity, (position, _actor, target)) in
            world.query_components::<(In<Position>, In<Actor>, In<Target>)>()
        {
            actor_data.push(((position.x, position.y), (target.x, target.y)));
        }

        // Now update positions
        let mut update_index = 0;
        for (_entity, position) in world.query_components::<(Out<Position>,)>() {
            // Only update actor positions (skip home and work)
            if update_index < actor_data.len() {
                let (current_pos, target_pos) = actor_data[update_index];

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
                update_index += 1;
            }
        }
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}

// Wait System - handles wait timers and target switching
pub struct WaitSystem;
impl System for WaitSystem {
    type InComponents = (Actor, WaitTimer, Target, Position);
    type OutComponents = (WaitTimer, Target);

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // Collect actor data first - use simpler queries due to framework limitations
        let mut actor_positions: Vec<(i32, i32)> = Vec::new();
        let mut actor_targets: Vec<(i32, i32)> = Vec::new();
        let mut wait_times: Vec<u32> = Vec::new();

        // Collect positions
        for (_entity, (position, _actor)) in world.query_components::<(In<Position>, In<Actor>)>() {
            actor_positions.push((position.x, position.y));
        }

        // Collect targets
        for (_entity, target) in world.query_components::<(In<Target>,)>() {
            actor_targets.push((target.x, target.y));
        }

        // Collect wait times
        for (_entity, wait_timer) in world.query_components::<(In<WaitTimer>,)>() {
            wait_times.push(wait_timer.ticks);
        }

        // Calculate updates needed
        let mut updates: Vec<(bool, bool, u32)> = Vec::new();
        for i in 0..actor_positions
            .len()
            .min(actor_targets.len())
            .min(wait_times.len())
        {
            let current_pos = actor_positions[i];
            let target_pos = actor_targets[i];
            let current_ticks = wait_times[i];

            let is_near_target = is_adjacent(current_pos, target_pos) || current_pos == target_pos;
            let should_switch = is_near_target && current_ticks == 0;
            let new_ticks = if is_near_target && current_ticks > 0 {
                current_ticks - 1
            } else if should_switch {
                WAIT_TICKS
            } else {
                current_ticks
            };

            updates.push((is_near_target, should_switch, new_ticks));
        }

        // Update wait timers
        let mut timer_index = 0;
        for (_entity, wait_timer) in world.query_components::<(Out<WaitTimer>,)>() {
            if timer_index < updates.len() {
                wait_timer.ticks = updates[timer_index].2;
                timer_index += 1;
            }
        }

        // Update targets
        let mut target_index = 0;
        for (_entity, target) in world.query_components::<(Out<Target>,)>() {
            if target_index < updates.len() && target_index < actor_targets.len() {
                let should_switch = updates[target_index].1;
                let current_target = actor_targets[target_index];

                if should_switch {
                    // Switch target between home and work
                    if current_target == HOME_POS {
                        target.x = WORK_POS.0;
                        target.y = WORK_POS.1;
                    } else {
                        target.x = HOME_POS.0;
                        target.y = HOME_POS.1;
                    }
                }
                target_index += 1;
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
    println!("Starting Simulation Game...");
    println!("Actors will travel between Home (H) and Work (W)");
    println!("Press Ctrl+C to stop the simulation");

    let mut world = initialize_game();
    
    // Enable replay logging with automatic file generation
    let replay_config = ReplayLogConfig {
        enabled: true,
        log_directory: "game_replay_logs".to_string(),
        file_prefix: "actor_simulation".to_string(),
        flush_interval: 20, // Flush every 20 updates for this fast game
        include_component_details: true,
    };
    
    match world.enable_replay_logging(replay_config) {
        Ok(()) => {
            if let Some(session_id) = world.replay_session_id() {
                println!("Replay logging enabled - Session ID: {}", session_id);
                println!("Logs will be saved to: game_replay_logs/actor_simulation_{}.log", session_id);
            }
        }
        Err(e) => {
            eprintln!("Failed to enable replay logging: {}", e);
            println!("Game will continue without logging");
        }
    }

    // Game loop - 2 ticks per second
    loop {
        world.update();
        thread::sleep(Duration::from_millis(500)); // 2 FPS
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
    fn test_replay_logging_integration() {
        use crate::{replay_analysis, ReplayLogConfig};
        
        // Create a world and enable logging
        let mut world = initialize_game();
        
        let replay_config = ReplayLogConfig {
            enabled: true,
            log_directory: "test_replay_logs".to_string(),
            file_prefix: "test_session".to_string(),
            flush_interval: 5,
            include_component_details: true,
        };
        
        world.enable_replay_logging(replay_config).expect("Failed to enable logging");
        
        // Run some updates
        for _ in 0..10 {
            world.update();
        }
        
        // Analyze the replay data
        let history = world.get_update_history();
        let stats = replay_analysis::analyze_replay_history(history);
        
        println!("Test replay analysis:");
        println!("  Total updates: {}", stats.total_updates);
        println!("  Total system executions: {}", stats.total_system_executions);
        println!("  Component types involved: {:?}", stats.component_types_involved);
        
        assert_eq!(stats.total_updates, 10);
        assert!(stats.total_system_executions > 0);
        
        // Clean up logging
        world.disable_replay_logging().expect("Failed to disable logging");
        
        // Clean up test directory
        let _ = std::fs::remove_dir_all("test_replay_logs");
    }
}
