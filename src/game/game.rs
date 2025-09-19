use rand::Rng;
use crate::{Diff, DiffComponent, In, Out, System, World, WorldView};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Home;

// Manually implement Diff for unit struct
impl Diff for Home {
    type Diff = ();
    fn diff(&self, _other: &Self) -> Option<Self::Diff> {
        None // Unit structs are always the same
    }
    fn apply_diff(&mut self, _diff: &Self::Diff) {
        // Nothing to apply for unit structs
    }
}

impl DiffComponent for Home {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Work;

impl Diff for Work {
    type Diff = ();
    fn diff(&self, _other: &Self) -> Option<Self::Diff> {
        None
    }
    fn apply_diff(&mut self, _diff: &Self::Diff) {}
}

impl DiffComponent for Work {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Actor;

impl Diff for Actor {
    type Diff = ();
    fn diff(&self, _other: &Self) -> Option<Self::Diff> {
        None
    }
    fn apply_diff(&mut self, _diff: &Self::Diff) {}
}

impl DiffComponent for Actor {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Obstacle;

impl Diff for Obstacle {
    type Diff = ();
    fn diff(&self, _other: &Self) -> Option<Self::Diff> {
        None
    }
    fn apply_diff(&mut self, _diff: &Self::Diff) {}
}

impl DiffComponent for Obstacle {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct Target {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaitTimer {
    pub ticks: u32,
}

// Manually implement Diff for WaitTimer since u32 doesn't implement Diff
impl Diff for WaitTimer {
    type Diff = u32;
    
    fn diff(&self, other: &Self) -> Option<Self::Diff> {
        if self.ticks != other.ticks {
            Some(other.ticks)
        } else {
            None
        }
    }
    
    fn apply_diff(&mut self, diff: &Self::Diff) {
        self.ticks = *diff;
    }
}

impl DiffComponent for WaitTimer {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorState {
    MovingToWork,
    MovingToHome,
    WaitingAtWork,
    WaitingAtHome,
}

impl Default for ActorState {
    fn default() -> Self {
        ActorState::MovingToWork
    }
}

impl Diff for ActorState {
    type Diff = ActorState;
    
    fn diff(&self, other: &Self) -> Option<Self::Diff> {
        if self != other {
            Some(*other)
        } else {
            None
        }
    }
    
    fn apply_diff(&mut self, diff: &Self::Diff) {
        *self = *diff;
    }
}

impl DiffComponent for ActorState {}

// Movement System - handles actor movement with obstacle avoidance
pub struct MovementSystem;
impl System for MovementSystem {
    type InComponents = (Actor, Position, Target);
    type OutComponents = (Position,);
    
    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    
    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // Collect all obstacle positions first
        let mut obstacles = HashSet::new();
        
        // Add home and work positions as obstacles
        obstacles.insert(HOME_POS);
        obstacles.insert(WORK_POS);
        
        // Collect all actor positions (they are obstacles to each other)
        let actor_positions: Vec<(i32, i32)> = world
            .query_components::<(In<Position>, In<Actor>)>()
            .into_iter()
            .map(|(_, (pos, _))| (pos.x, pos.y))
            .collect();
        
        for pos in &actor_positions {
            obstacles.insert(*pos);
        }
        
        // Move each actor
        for (entity, position) in world.query_components::<(Out<Position>,)>() {
            // We need to get target separately due to query limitations
            let current_pos = (position.x, position.y);
            
            // For now, simple movement - we'll improve this later
            // This is a simplified version that moves toward work first
            let target_pos = WORK_POS;
            
            if current_pos == target_pos {
                continue; // Already at target
            }
            
            // Calculate next move using simple pathfinding
            let next_pos = calculate_next_move(current_pos, target_pos, &obstacles);
            
            // Update position if we can move
            if next_pos != current_pos && is_valid_position(next_pos) && !obstacles.contains(&next_pos) {
                position.x = next_pos.0;
                position.y = next_pos.1;
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
        // Due to query limitations, we'll process wait timers and targets separately
        for (entity, wait_timer) in world.query_components::<(Out<WaitTimer>,)>() {
            if wait_timer.ticks > 0 {
                wait_timer.ticks -= 1;
            }
        }
        
        // Process target switching (simplified)
        for (entity, target) in world.query_components::<(Out<Target>,)>() {
            // Simple toggle between home and work
            if target.x == HOME_POS.0 && target.y == HOME_POS.1 {
                target.x = WORK_POS.0;
                target.y = WORK_POS.1;
            } else {
                target.x = HOME_POS.0;
                target.y = HOME_POS.1;
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
        
        // Place entities on grid - simplified approach
        // We'll query all positions and determine entity type by checking components individually
        for (entity, position) in world.query_components::<(In<Position>,)>() {
            let x = position.x as usize;
            let y = position.y as usize;
            
            if x < GRID_SIZE as usize && y < GRID_SIZE as usize {
                // Check what type of entity this is by position
                if (position.x, position.y) == HOME_POS {
                    grid[y][x] = 'H';
                } else if (position.x, position.y) == WORK_POS {
                    grid[y][x] = 'W';
                } else {
                    grid[y][x] = 'A'; // Assume it's an actor
                }
            }
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

fn calculate_next_move(current: (i32, i32), target: (i32, i32), obstacles: &HashSet<(i32, i32)>) -> (i32, i32) {
    let (cx, cy) = current;
    let (tx, ty) = target;
    
    // Calculate direction
    let dx = if tx > cx { 1 } else if tx < cx { -1 } else { 0 };
    let dy = if ty > cy { 1 } else if ty < cy { -1 } else { 0 };
    
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
    world.add_component(home_entity, Position { x: HOME_POS.0, y: HOME_POS.1 });
    world.add_component(home_entity, Home);
    world.add_component(home_entity, Obstacle);
    
    // Create work entity
    let work_entity = world.create_entity();
    world.add_component(work_entity, Position { x: WORK_POS.0, y: WORK_POS.1 });
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
        world.add_component(actor_entity, Target { x: WORK_POS.0, y: WORK_POS.1 }); // Start by going to work
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
}
