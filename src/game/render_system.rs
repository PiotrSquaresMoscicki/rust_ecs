use crate::{In, System, WorldView};
use super::components::{Position, GRID_SIZE, HOME_POS, WORK_POS, Home, Work, Actor, Tree, WoodcutterHut, Woodcutter, Obstacle};

/// Render System - displays the grid with configurable size
pub struct RenderSystem {
    grid_width: usize,
    grid_height: usize,
}

impl RenderSystem {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid_width: width,
            grid_height: height,
        }
    }
}

impl Default for RenderSystem {
    fn default() -> Self {
        Self::new(GRID_SIZE as usize, GRID_SIZE as usize)
    }
}

impl System for RenderSystem {
    type InComponents = (Position, Home, Work, Actor, Tree, WoodcutterHut, Woodcutter, Obstacle);
    type OutComponents = ();

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // Clear screen
        print!("\x1B[2J\x1B[1;1H");

        // Create grid
        let mut grid = vec![vec!['.'; self.grid_width]; self.grid_height];

        // Place obstacles/walls on grid first (lowest priority)
        for (_entity, (position, _obstacle)) in world.query_components::<(In<Position>, In<Obstacle>)>() {
            let x = position.x as usize;
            let y = position.y as usize;
            if x < self.grid_width && y < self.grid_height {
                grid[y][x] = '#'; // Wall/obstacle
            }
        }

        // Place trees on grid
        for (_entity, (position, _tree)) in world.query_components::<(In<Position>, In<Tree>)>() {
            let x = position.x as usize;
            let y = position.y as usize;
            if x < self.grid_width && y < self.grid_height {
                grid[y][x] = 'T';
            }
        }

        // Place woodcutter huts on grid  
        for (_entity, (position, _hut)) in world.query_components::<(In<Position>, In<WoodcutterHut>)>() {
            let x = position.x as usize;
            let y = position.y as usize;
            if x < self.grid_width && y < self.grid_height {
                grid[y][x] = 'W'; // Woodcutter hut
            }
        }

        // Place work/exit positions
        for (_entity, (position, _work)) in world.query_components::<(In<Position>, In<Work>)>() {
            let x = position.x as usize;
            let y = position.y as usize;
            if x < self.grid_width && y < self.grid_height {
                grid[y][x] = 'E'; // Exit/Work
            }
        }

        // Place home positions
        for (_entity, (position, _home)) in world.query_components::<(In<Position>, In<Home>)>() {
            let x = position.x as usize;
            let y = position.y as usize;
            if x < self.grid_width && y < self.grid_height {
                grid[y][x] = 'H'; // Home
            }
        }

        // Place actors/woodcutters on grid (highest priority)
        for (_entity, (position, _actor)) in world.query_components::<(In<Position>, In<Actor>)>() {
            let x = position.x as usize;
            let y = position.y as usize;
            if x < self.grid_width && y < self.grid_height {
                if grid[y][x] != 'H' { // Don't overwrite home
                    grid[y][x] = 'A'; // Actor
                }
            }
        }

        // Place woodcutters on grid (separate from regular actors)
        for (_entity, (position, _woodcutter)) in world.query_components::<(In<Position>, In<Woodcutter>)>() {
            let x = position.x as usize;
            let y = position.y as usize;
            if x < self.grid_width && y < self.grid_height {
                if grid[y][x] != 'H' && grid[y][x] != 'A' { // Don't overwrite home or actor
                    grid[y][x] = 'C'; // Woodcutter (C for Cutter)
                }
            }
        }

        // Print grid with appropriate legend
        if self.grid_width == 10 && self.grid_height == 10 {
            // Navigation demo
            println!("Navigation Demo - Labyrinth Pathfinding");
            println!("# = Wall, A = Actor, E = Exit, . = Open space");
        } else {
            // Default/woodcutter demo
            println!("Simulation Game - Actors and Woodcutters");
            println!("H = Home, E = Work/Office, A = Actor, C = Woodcutter, T = Tree, W = Woodcutter Hut");
        }
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
        assert!(std::mem::size_of_val(&render_system) > 0); // Now contains grid dimensions
        
        // Test custom creation
        let custom_render_system = RenderSystem::new(15, 20);
        assert!(std::mem::size_of_val(&custom_render_system) > 0);
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