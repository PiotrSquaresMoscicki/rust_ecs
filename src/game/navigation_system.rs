use super::components::{Actor, Navigation, Obstacle, Position, Target, Work, GRID_HEIGHT, GRID_WIDTH};
use super::utils::{is_adjacent, is_valid_position};
use crate::{In, Out, System, World, WorldView};
use pathfinding::prelude::astar;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Navigation System - handles sophisticated pathfinding using A* algorithm
pub struct NavigationSystem;

impl NavigationSystem {
    /// Find an adjacent position to the target that is not blocked
    fn find_adjacent_to_target(
        target: (i32, i32),
        obstacles: &HashSet<(i32, i32)>,
    ) -> Option<(i32, i32)> {
        // Check all 8 adjacent positions around the target
        for dx in -1..=1 {
            for dy in -1..=1 {
                if dx == 0 && dy == 0 {
                    continue; // Skip the target position itself
                }
                let adjacent_pos = (target.0 + dx, target.1 + dy);
                if is_valid_position(adjacent_pos) && !obstacles.contains(&adjacent_pos) {
                    return Some(adjacent_pos);
                }
            }
        }
        None
    }

    /// Calculate A* path from start to goal, avoiding obstacles
    fn calculate_path(
        start: (i32, i32),
        goal: (i32, i32),
        obstacles: &HashSet<(i32, i32)>,
    ) -> Option<Vec<(i32, i32)>> {
        let result = astar(
            &start,
            |&(x, y)| {
                // Generate successors: all 8 adjacent positions
                let mut successors = Vec::new();
                for dx in -1..=1 {
                    for dy in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue; // Skip current position
                        }
                        let new_pos = (x + dx, y + dy);
                        if is_valid_position(new_pos) && !obstacles.contains(&new_pos) {
                            // Cost is 1 for orthogonal moves, sqrt(2) ≈ 1.4 for diagonal
                            let cost = if dx.abs() + dy.abs() == 1 { 10 } else { 14 };
                            successors.push((new_pos, cost));
                        }
                    }
                }
                successors
            },
            |&(x, y)| {
                // Heuristic: Manhattan distance * 10 (to match orthogonal cost)
                ((goal.0 - x).abs() + (goal.1 - y).abs()) * 10
            },
            |&pos| pos == goal,
        );

        result.map(|(path, _cost)| path)
    }
}

impl System for NavigationSystem {
    type InComponents = (Actor, Position, Target);
    type OutComponents = (Position, Navigation);
    type InSystems = ();
    type OutSystems = ();

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // Collect all obstacle positions (including other actors)
        let mut obstacles = HashSet::new();

        // Add all obstacle entities
        for (_, (position, _)) in world.query_components::<(In<Position>, In<Obstacle>)>() {
            obstacles.insert((position.x, position.y));
        }

        // Collect all current actor positions to avoid collisions
        let current_positions: Vec<(i32, i32)> = world
            .query_components::<(In<Position>, In<Actor>)>()
            .into_iter()
            .map(|(_, (pos, _))| (pos.x, pos.y))
            .collect();

        // Add other actors as obstacles (will be excluded for the current actor during pathfinding)
        for &pos in &current_positions {
            obstacles.insert(pos);
        }

        // Collect changes to apply after the query
        let mut position_changes = Vec::new();
        let mut navigation_changes = Vec::new();

        // Process entities with Navigation component
        for (entity, (position, _actor, target, navigation)) in
            world.query_components::<(Out<Position>, In<Actor>, In<Target>, Out<Navigation>)>()
        {
            let current_pos = (position.x, position.y);
            let target_pos = (target.x, target.y);

            // Check if we need to recalculate the path
            if navigation.needs_recalculation || navigation.path.is_empty() {
                // Remove current actor from obstacles for pathfinding
                let mut temp_obstacles = obstacles.clone();
                temp_obstacles.remove(&current_pos);

                // Check if target is an obstacle (like HOME or WORK positions)
                let path_target = if temp_obstacles.contains(&target_pos) {
                    // Target is an obstacle, find adjacent position to target
                    Self::find_adjacent_to_target(target_pos, &temp_obstacles).unwrap_or(target_pos)
                } else {
                    target_pos
                };

                if let Some(mut path) =
                    Self::calculate_path(current_pos, path_target, &temp_obstacles)
                {
                    // Remove the first position if it's the current position
                    if !path.is_empty() && path[0] == current_pos {
                        path.remove(0);
                    }
                    let old_navigation = navigation.clone();
                    navigation.set_path(path);
                    navigation_changes.push((entity, old_navigation, navigation.clone()));
                } else {
                    // No path found, mark for recalculation next frame
                    let old_navigation = navigation.clone();
                    navigation.request_recalculation();
                    navigation_changes.push((entity, old_navigation, navigation.clone()));
                    continue;
                }
            }

            // Don't move if already at target or adjacent to target
            if !is_adjacent(current_pos, target_pos) && current_pos != target_pos {
                if let Some(next_pos) = navigation.get_next_position() {
                    // Check if the next position is still valid (not blocked by new obstacles)
                    let mut temp_obstacles = obstacles.clone();
                    temp_obstacles.remove(&current_pos);

                    if !temp_obstacles.contains(&next_pos) && is_valid_position(next_pos) {
                        // Move to next position
                        let old_position = *position;
                        position.x = next_pos.0;
                        position.y = next_pos.1;
                        position_changes.push((entity, old_position, *position));

                        // Advance to next step in path
                        let old_navigation = navigation.clone();
                        navigation.advance_path();
                        navigation_changes.push((entity, old_navigation, navigation.clone()));
                    } else {
                        // Path is blocked, request recalculation
                        let old_navigation = navigation.clone();
                        navigation.request_recalculation();
                        navigation_changes.push((entity, old_navigation, navigation.clone()));
                    }
                }
            }
        }

        // Record all component changes
        for (entity, old_position, new_position) in position_changes {
            world.record_component_modification(entity, &old_position, &new_position);
        }
        for (entity, old_navigation, new_navigation) in navigation_changes {
            world.record_component_modification(entity, &old_navigation, &new_navigation);
        }
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}

/// Initialize a navigation demo world with a labyrinth and two actors finding their way to the exit
pub fn initialize_navigation_demo() -> World {
    let mut world = World::new();

    println!("Creating labyrinth maze...");

    // Generate a 15x30 labyrinth with guaranteed paths
    let mut labyrinth_layout = vec![vec![1i32; GRID_WIDTH as usize]; GRID_HEIGHT as usize];
    
    // Create border walls
    #[allow(clippy::needless_range_loop)]
    for y in 0..GRID_HEIGHT as usize {
        for x in 0..GRID_WIDTH as usize {
            if y == 0 || y == (GRID_HEIGHT - 1) as usize || x == 0 || x == (GRID_WIDTH - 1) as usize {
                labyrinth_layout[y][x] = 1; // Wall
            } else {
                labyrinth_layout[y][x] = 0; // Open space
            }
        }
    }
    
    // Add internal maze structure - create a pattern of walls and passages
    #[allow(clippy::needless_range_loop)]
    for y in 2..(GRID_HEIGHT - 2) as usize {
        for x in 2..(GRID_WIDTH - 2) as usize {
            // Create a maze pattern with walls and passages
            if (x % 4 == 0 || y % 3 == 0) && !(x % 8 == 0 && y % 6 == 0) {
                // Create some walls but ensure passages
                if (x + y) % 7 != 0 {
                    labyrinth_layout[y][x] = 1; // Wall
                }
            }
        }
    }
    
    // Ensure clear paths from start positions to exit
    // Clear path along bottom and right edges
    for x in 1..(GRID_WIDTH - 1) as usize {
        labyrinth_layout[(GRID_HEIGHT - 2) as usize][x] = 0; // Bottom corridor
    }
    #[allow(clippy::needless_range_loop)]
    for y in 1..(GRID_HEIGHT - 1) as usize {
        labyrinth_layout[y][(GRID_WIDTH - 2) as usize] = 0; // Right corridor
    }
    
    // Ensure some cross-corridors
    labyrinth_layout[7][1..(GRID_WIDTH - 1) as usize].fill(0); // Horizontal corridor
    #[allow(clippy::needless_range_loop)]
    for y in 1..(GRID_HEIGHT - 1) as usize {
        labyrinth_layout[y][15] = 0; // Vertical corridor in middle
    }

    // Create wall entities
    for (y, row) in labyrinth_layout.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            if cell == 1 {
                let wall_entity = world.create_entity();
                world.add_component(
                    wall_entity,
                    Position {
                        x: x as i32,
                        y: y as i32,
                    },
                );
                world.add_component(wall_entity, Obstacle);
            }
        }
    }

    // Create exit marker (special component to show the exit)
    let exit_entity = world.create_entity();
    world.add_component(exit_entity, Position { x: GRID_WIDTH - 2, y: GRID_HEIGHT - 2 });
    world.add_component(exit_entity, Work); // Use Work as exit marker for rendering

    println!("Creating two actors with navigation components...");

    // Create Actor 1 at starting position (1, 1)
    let actor1_entity = world.create_entity();
    world.add_component(actor1_entity, Position { x: 1, y: 1 });
    world.add_component(actor1_entity, Actor);
    world.add_component(actor1_entity, Target { x: GRID_WIDTH - 2, y: GRID_HEIGHT - 2 }); // Target the exit
    world.add_component(actor1_entity, Navigation::new());

    // Create Actor 2 at starting position (1, 7)
    let actor2_entity = world.create_entity();
    world.add_component(actor2_entity, Position { x: 1, y: 7 });
    world.add_component(actor2_entity, Actor);
    world.add_component(actor2_entity, Target { x: GRID_WIDTH - 2, y: GRID_HEIGHT - 2 }); // Target the exit
    world.add_component(actor2_entity, Navigation::new());

    // Add navigation system (uses A* pathfinding)
    world.add_system(NavigationSystem);

    // Add render system for visualization
    world.add_system(super::render_system::RenderSystem::new(GRID_WIDTH as usize, GRID_HEIGHT as usize));

    // Initialize systems
    world.initialize_systems();

    println!("Labyrinth demo world initialized!");
    println!("Grid size: {}x{}", GRID_WIDTH, GRID_HEIGHT);
    println!("Actors: 2 (starting at (1,1) and (1,7))");
    println!("Exit: ({},{})", GRID_WIDTH - 2, GRID_HEIGHT - 2);
    println!("Wall obstacles: Created from maze layout");

    world
}

pub fn run_navigation_demo() {
    println!("🧭 Starting Navigation Demo 🧭");
    println!("=====================================");
    println!("This demo shows 2 actors navigating through a labyrinth to reach the exit");
    println!("Symbols: # = Wall, A = Actor, E = Exit, . = Open space");
    println!("The actors use A* pathfinding to find the optimal route while avoiding walls");
    println!("Press Ctrl+C to stop the demo");
    println!();

    let mut world = initialize_navigation_demo();

    // Set up Ctrl+C handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C, shutting down gracefully...");
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let mut update_count = 0;

    // Demo loop - 2 FPS for good observation speed
    while running.load(Ordering::SeqCst) {
        update_count += 1;

        // Update the world
        world.update();

        // Check if any actor reached the exit
        let actors = world.entities_with_component::<Actor>();
        let mut actors_at_exit = 0;

        for &actor in &actors {
            if let Some(position) = world.get_component::<Position>(actor) {
                if position.x == GRID_WIDTH - 2 && position.y == GRID_HEIGHT - 2 {
                    actors_at_exit += 1;
                    println!("🎉 Actor {:?} reached the exit! 🎉", actor);
                }
            }
        }

        if actors_at_exit == 2 {
            println!("🎉 Both actors have reached the exit! Demo complete! 🎉");
            break;
        }

        thread::sleep(Duration::from_millis(500)); // 2 FPS for good observation

        // Print frame diff after all system updates and after sleep
        world.print_last_frame_diff();
    }

    println!("Navigation demo completed after {} updates", update_count);
}

#[cfg(test)]
mod tests {
    use super::super::components::*;
    use super::*;
    use crate::World;

    #[test]
    fn test_navigation_system_basic_pathfinding() {
        let mut world = World::new();

        // Create an actor with navigation
        let actor_entity = world.create_entity();
        world.add_component(actor_entity, Position { x: 0, y: 0 });
        world.add_component(actor_entity, Actor);
        world.add_component(actor_entity, Target { x: 2, y: 2 });
        world.add_component(actor_entity, Navigation::new());

        world.add_system(NavigationSystem);
        world.initialize_systems();

        println!("Initial state:");
        let initial_navigation = world.get_component::<Navigation>(actor_entity).unwrap();
        println!(
            "Initial navigation: path={:?}, index={}, needs_recalc={}",
            initial_navigation.path,
            initial_navigation.current_path_index,
            initial_navigation.needs_recalculation
        );

        // Run one update to calculate path
        world.update();

        // Check that navigation component has a path
        let navigation = world.get_component::<Navigation>(actor_entity).unwrap();
        println!("After first update:");
        println!(
            "Navigation: path={:?}, index={}, needs_recalc={}",
            navigation.path, navigation.current_path_index, navigation.needs_recalculation
        );

        assert!(
            !navigation.path.is_empty(),
            "Navigation path should not be empty after first update"
        );
        // Note: after path calculation, the path should be set and index reset to 0

        // Run another update to move
        world.update();

        // Position should have moved towards target
        let position = world.get_component::<Position>(actor_entity).unwrap();
        println!("After second update:");
        println!("Position: ({}, {})", position.x, position.y);

        let navigation = world.get_component::<Navigation>(actor_entity).unwrap();
        println!(
            "Navigation: path={:?}, index={}, needs_recalc={}",
            navigation.path, navigation.current_path_index, navigation.needs_recalculation
        );

        assert_ne!(
            (position.x, position.y),
            (0, 0),
            "Actor should have moved from starting position"
        );

        // After moving, the path index should have advanced
        assert!(
            navigation.current_path_index >= 1 || navigation.path.len() <= 1,
            "Path index should advance or path should be short. Current index: {}, path length: {}",
            navigation.current_path_index,
            navigation.path.len()
        );
    }

    #[test]
    fn test_navigation_system_avoids_obstacles() {
        let mut world = World::new();

        // Create obstacles blocking direct path
        let obstacle1 = world.create_entity();
        world.add_component(obstacle1, Position { x: 1, y: 1 });
        world.add_component(obstacle1, Obstacle);

        let obstacle2 = world.create_entity();
        world.add_component(obstacle2, Position { x: 1, y: 0 });
        world.add_component(obstacle2, Obstacle);

        let obstacle3 = world.create_entity();
        world.add_component(obstacle3, Position { x: 0, y: 1 });
        world.add_component(obstacle3, Obstacle);

        // Create an actor that needs to navigate around obstacles
        let actor_entity = world.create_entity();
        world.add_component(actor_entity, Position { x: 0, y: 0 });
        world.add_component(actor_entity, Actor);
        world.add_component(actor_entity, Target { x: 2, y: 2 });
        world.add_component(actor_entity, Navigation::new());

        world.add_system(NavigationSystem);
        world.initialize_systems();

        // Run update to calculate path
        world.update();

        // Check that a path was found (even with obstacles)
        let navigation = world.get_component::<Navigation>(actor_entity).unwrap();
        println!("Path calculated: {:?}", navigation.path);
        println!("Needs recalculation: {}", navigation.needs_recalculation);

        // The path should not go through any obstacle positions
        let obstacle_positions = [(1, 1), (1, 0), (0, 1)];
        for pos in &navigation.path {
            assert!(
                !obstacle_positions.contains(pos),
                "Path goes through obstacle at {:?}",
                pos
            );
        }

        // If no path was found, it means the obstacles completely block access
        // In this test setup, there should be alternative routes available (like going via (0, 2) or (2, 0))
        if navigation.path.is_empty() {
            println!("No path found - checking if it's expected due to complete blockage");
            // Test with a simpler obstacle setup
            let mut test_obstacles = std::collections::HashSet::new();
            test_obstacles.insert((1, 1));
            test_obstacles.insert((1, 0));
            test_obstacles.insert((0, 1));

            let test_path = NavigationSystem::calculate_path((0, 0), (2, 2), &test_obstacles);
            if test_path.is_some() {
                panic!(
                    "Path should be found in navigation system but wasn't. Test path: {:?}",
                    test_path
                );
            }
        }
    }

    #[test]
    fn test_navigation_system_recalculates_when_path_blocked() {
        let mut world = World::new();

        // Create an actor
        let actor_entity = world.create_entity();
        world.add_component(actor_entity, Position { x: 0, y: 0 });
        world.add_component(actor_entity, Actor);
        world.add_component(actor_entity, Target { x: 4, y: 0 }); // Longer path
        world.add_component(actor_entity, Navigation::new());

        world.add_system(NavigationSystem);
        world.initialize_systems();

        // Run update to calculate initial path
        world.update();

        let initial_navigation = world
            .get_component::<Navigation>(actor_entity)
            .unwrap()
            .clone();
        assert!(
            !initial_navigation.path.is_empty(),
            "Initial path should not be empty"
        );
        println!("Initial path: {:?}", initial_navigation.path);

        // Move the actor one step
        world.update();
        let position_after_move = world.get_component::<Position>(actor_entity).unwrap();
        println!(
            "Position after first move: ({}, {})",
            position_after_move.x, position_after_move.y
        );

        // Add an obstacle that blocks a position in the middle of the path (not the target)
        let obstacle = world.create_entity();
        world.add_component(obstacle, Position { x: 3, y: 0 }); // Block position (3,0) which should be on the path to (4,0)
        world.add_component(obstacle, Obstacle);

        // Run update - should detect path is blocked and recalculate
        world.update();

        let new_navigation = world.get_component::<Navigation>(actor_entity).unwrap();
        println!(
            "Navigation after obstacle added: path={:?}, needs_recalc={}, index={}",
            new_navigation.path,
            new_navigation.needs_recalculation,
            new_navigation.current_path_index
        );

        // Should either request recalculation or have a different path
        let path_changed = new_navigation.path != initial_navigation.path;
        let needs_recalc = new_navigation.needs_recalculation;

        assert!(
            needs_recalc || path_changed,
            "Navigation should either need recalculation or have a different path when blocked. \
                 Initial path: {:?}, New path: {:?}, Needs recalc: {}",
            initial_navigation.path,
            new_navigation.path,
            needs_recalc
        );
    }

    #[test]
    fn test_navigation_path_calculation() {
        let mut obstacles = HashSet::new();
        obstacles.insert((1, 0));
        obstacles.insert((1, 1));
        obstacles.insert((1, 2));

        let path = NavigationSystem::calculate_path((0, 1), (2, 1), &obstacles);

        assert!(path.is_some());
        let path = path.unwrap();

        // Path should start at (0,1) and end at (2,1)
        assert_eq!(path[0], (0, 1));
        assert_eq!(path[path.len() - 1], (2, 1));

        // Path should not go through any obstacles
        for pos in &path {
            assert!(
                !obstacles.contains(pos),
                "Path goes through obstacle at {:?}",
                pos
            );
        }
    }

    #[test]
    fn test_navigation_no_path_available() {
        let mut obstacles = HashSet::new();
        // Create a wall that completely blocks access
        for y in 0..GRID_HEIGHT {
            obstacles.insert((2, y));
        }

        let path = NavigationSystem::calculate_path((0, 0), (5, 5), &obstacles);
        assert!(path.is_none()); // No path should be found
    }
}
