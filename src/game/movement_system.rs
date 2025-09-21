use crate::{In, Out, System, WorldView};
use std::collections::HashSet;
use super::components::{Actor, Position, Target, HOME_POS, WORK_POS};
use super::utils::{calculate_next_move, is_valid_position, is_adjacent};

/// Movement System - handles actor movement with obstacle avoidance
/// Simplified thanks to extended query support for up to 16 components!
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

        // Collect changes to apply after the query
        let mut changes = Vec::new();

        // Now we can query and update actor positions in a single query thanks to extended support!
        for (entity, (position, _actor, target)) in
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
                    let old_position = *position;
                    position.x = next_pos.0;
                    position.y = next_pos.1;
                    
                    // Store the change to record later
                    changes.push((entity, old_position, *position));
                }
            }
        }
        
        // Record all component changes
        for (entity, old_position, new_position) in changes {
            world.record_component_modification(entity, &old_position, &new_position);
        }
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;
    use super::super::components::*;

    fn create_test_world_with_actor() -> World {
        let mut world = World::new();
        
        // Create an actor
        let actor_entity = world.create_entity();
        world.add_component(actor_entity, Position { x: 2, y: 2 });
        world.add_component(actor_entity, Actor);
        world.add_component(actor_entity, Target { x: 5, y: 5 });
        
        // Add the movement system
        world.add_system(MovementSystem);
        world.initialize_systems();
        
        world
    }

    #[test]
    fn test_movement_system_basic() {
        let mut world = create_test_world_with_actor();
        
        // Get initial position
        let actor_entities = world.entities_with_component::<Actor>();
        assert_eq!(actor_entities.len(), 1);
        let actor_entity = actor_entities[0];
        
        let initial_pos = world.get_component::<Position>(actor_entity).unwrap();
        assert_eq!(initial_pos.x, 2);
        assert_eq!(initial_pos.y, 2);
        
        // Run one update
        world.update();
        
        // Position should have moved towards target
        let new_pos = world.get_component::<Position>(actor_entity).unwrap();
        // Should move diagonally towards (5,5)
        assert_eq!(new_pos.x, 3);
        assert_eq!(new_pos.y, 3);
    }

    #[test]
    fn test_movement_system_stops_when_adjacent() {
        let mut world = World::new();
        
        // Create an actor adjacent to target
        let actor_entity = world.create_entity();
        world.add_component(actor_entity, Position { x: 4, y: 5 }); // Adjacent to (5,5)
        world.add_component(actor_entity, Actor);
        world.add_component(actor_entity, Target { x: 5, y: 5 });
        
        world.add_system(MovementSystem);
        world.initialize_systems();
        
        // Run update
        world.update();
        
        // Position should not change (already adjacent)
        let pos = world.get_component::<Position>(actor_entity).unwrap();
        assert_eq!(pos.x, 4);
        assert_eq!(pos.y, 5);
    }

    #[test]
    fn test_movement_system_avoids_home_work_positions() {
        let mut world = World::new();
        
        // Create an actor that would move through HOME_POS (1,1)
        // Start at (0,0) and target (2,2) - direct path would go through (1,1)
        let actor_entity = world.create_entity();
        world.add_component(actor_entity, Position { x: 0, y: 0 });
        world.add_component(actor_entity, Actor);
        world.add_component(actor_entity, Target { x: 2, y: 2 });
        
        world.add_system(MovementSystem);
        world.initialize_systems();
        
        // Run update
        world.update();
        
        // Should move around HOME_POS, not through it
        let pos = world.get_component::<Position>(actor_entity).unwrap();
        assert_ne!((pos.x, pos.y), HOME_POS);
        // Should have moved from the starting position (the movement system should find an alternative path)
        // It should move to either (1,0) or (0,1) to avoid going through HOME_POS
        assert!((pos.x, pos.y) == (1, 0) || (pos.x, pos.y) == (0, 1));
    }
}