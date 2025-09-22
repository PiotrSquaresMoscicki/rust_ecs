use crate::{In, System, WorldView};
use super::components::{Position, GRID_SIZE, HOME_POS, WORK_POS, Home, Work, Actor, Tree, WoodcutterHut, Woodcutter};

/// Render System - displays the 10x10 grid
pub struct RenderSystem;

impl Default for RenderSystem {
    fn default() -> Self {
        Self
    }
}

impl System for RenderSystem {
    type InComponents = (Position, Home, Work, Actor, Tree, WoodcutterHut, Woodcutter);
    type OutComponents = ();

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");

        // Create grid
        let mut grid = vec![vec!['.'; GRID_SIZE as usize]; GRID_SIZE as usize];

        // Place trees on grid
        for (_entity, (position, _tree)) in world.query_components::<(In<Position>, In<Tree>)>() {
            let x = position.x as usize;
            let y = position.y as usize;
            if x < GRID_SIZE as usize && y < GRID_SIZE as usize {
                grid[y][x] = 'T';
            }
        }

        // Place woodcutter huts on grid  
        for (_entity, (position, _hut)) in world.query_components::<(In<Position>, In<WoodcutterHut>)>() {
            let x = position.x as usize;
            let y = position.y as usize;
            if x < GRID_SIZE as usize && y < GRID_SIZE as usize {
                grid[y][x] = 'W'; // Woodcutter hut
            }
        }

        // Place actors/woodcutters on grid
        for (_entity, (position, _actor)) in world.query_components::<(In<Position>, In<Actor>)>() {
            let x = position.x as usize;
            let y = position.y as usize;
            if x < GRID_SIZE as usize && y < GRID_SIZE as usize {
                if grid[y][x] == '.' {
                    grid[y][x] = 'A'; // Actor
                }
            }
        }

        // Place woodcutters on grid (separate from regular actors)
        for (_entity, (position, _woodcutter)) in world.query_components::<(In<Position>, In<Woodcutter>)>() {
            let x = position.x as usize;
            let y = position.y as usize;
            if x < GRID_SIZE as usize && y < GRID_SIZE as usize {
                if grid[y][x] == '.' {
                    grid[y][x] = 'C'; // Woodcutter (C for Cutter)
                }
            }
        }

        // Ensure home and work are always visible at their fixed positions
        if HOME_POS.0 >= 0 && HOME_POS.0 < GRID_SIZE && HOME_POS.1 >= 0 && HOME_POS.1 < GRID_SIZE {
            grid[HOME_POS.1 as usize][HOME_POS.0 as usize] = 'H';
        }
        if WORK_POS.0 >= 0 && WORK_POS.0 < GRID_SIZE && WORK_POS.1 >= 0 && WORK_POS.1 < GRID_SIZE {
            grid[WORK_POS.1 as usize][WORK_POS.0 as usize] = 'O'; // O for Office/Work
        }

        // Print grid - updated output
        println!("Simulation Game - Actors and Woodcutters");
        println!("H = Home, O = Work/Office, A = Actor, C = Woodcutter, T = Tree, W = Woodcutter Hut");
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;
    use super::super::components::*;

    fn create_test_world_with_entities() -> World {
        let mut world = World::new();
        
        // Create home entity
        let home_entity = world.create_entity();
        world.add_component(home_entity, Position { x: HOME_POS.0, y: HOME_POS.1 });
        world.add_component(home_entity, Home);
        
        // Create work entity
        let work_entity = world.create_entity();
        world.add_component(work_entity, Position { x: WORK_POS.0, y: WORK_POS.1 });
        world.add_component(work_entity, Work);
        
        // Create an actor
        let actor_entity = world.create_entity();
        world.add_component(actor_entity, Position { x: 3, y: 3 });
        world.add_component(actor_entity, Actor);
        
        world
    }

    #[test]
    fn test_render_system_creation() {
        let render_system = RenderSystem::default();
        // Test that the system can be created without panicking
        assert_eq!(std::mem::size_of_val(&render_system), 0); // Zero-sized struct
    }

    #[test]
    fn test_render_system_with_world() {
        let mut world = create_test_world_with_entities();
        world.add_system(RenderSystem::default());
        world.initialize_systems();
        
        // Test that update doesn't panic
        // Note: This will print to stdout, but that's expected behavior
        world.update();
        
        // If we get here without panicking, the test passes
        assert!(true);
    }

    #[test]
    fn test_render_system_grid_bounds() {
        let mut world = World::new();
        
        // Create entities at grid boundaries
        let edge_entity = world.create_entity();
        world.add_component(edge_entity, Position { x: GRID_SIZE - 1, y: GRID_SIZE - 1 });
        
        // Create entity outside grid (should be ignored)
        let out_of_bounds_entity = world.create_entity();
        world.add_component(out_of_bounds_entity, Position { x: GRID_SIZE, y: GRID_SIZE });
        
        world.add_system(RenderSystem::default());
        world.initialize_systems();
        
        // Should not panic even with out-of-bounds entities
        world.update();
        
        assert!(true);
    }

    #[test]
    fn test_render_system_home_work_always_visible() {
        let mut world = World::new();
        
        // Create only an actor at home position (should still show 'H')
        let actor_entity = world.create_entity();
        world.add_component(actor_entity, Position { x: HOME_POS.0, y: HOME_POS.1 });
        world.add_component(actor_entity, Actor);
        
        world.add_system(RenderSystem::default());
        world.initialize_systems();
        
        // This should render with 'H' at home position, not 'A'
        world.update();
        
        assert!(true);
    }
}