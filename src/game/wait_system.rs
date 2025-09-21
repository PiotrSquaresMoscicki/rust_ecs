use crate::{In, Out, System, WorldView};
use super::components::{Actor, Position, Target, WaitTimer, HOME_POS, WORK_POS, WAIT_TICKS};
use super::utils::is_adjacent;

/// Wait System - handles wait timers and target switching
/// Simplified thanks to extended query support for up to 16 components!
pub struct WaitSystem;

impl System for WaitSystem {
    type InComponents = (Actor, WaitTimer, Target, Position);
    type OutComponents = (WaitTimer, Target);

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // Collect changes to apply after the query
        let mut wait_timer_changes = Vec::new();
        let mut target_changes = Vec::new();

        // Now we can query all actor components together thanks to extended query support!
        for (entity, (position, _actor, wait_timer, target)) in 
            world.query_components::<(In<Position>, In<Actor>, Out<WaitTimer>, Out<Target>)>()
        {
            let current_pos = (position.x, position.y);
            let target_pos = (target.x, target.y);
            let current_ticks = wait_timer.ticks;

            let is_near_target = is_adjacent(current_pos, target_pos) || current_pos == target_pos;
            let should_switch = is_near_target && current_ticks == 0;

            // Update wait timer
            let old_wait_timer = *wait_timer;
            if is_near_target && current_ticks > 0 {
                wait_timer.ticks = current_ticks - 1;
            } else if should_switch {
                wait_timer.ticks = WAIT_TICKS;
            }
            
            // Store wait timer change if it was modified
            if old_wait_timer.ticks != wait_timer.ticks {
                wait_timer_changes.push((entity, old_wait_timer, *wait_timer));
            }

            // Update target if needed
            if should_switch {
                let old_target = *target;
                // Switch target between home and work
                if target_pos == HOME_POS {
                    target.x = WORK_POS.0;
                    target.y = WORK_POS.1;
                } else {
                    target.x = HOME_POS.0;
                    target.y = HOME_POS.1;
                }
                
                // Store target change
                target_changes.push((entity, old_target, *target));
            }
        }
        
        // Record all component changes
        for (entity, old_wait_timer, new_wait_timer) in wait_timer_changes {
            world.record_component_modification(entity, &old_wait_timer, &new_wait_timer);
        }
        
        for (entity, old_target, new_target) in target_changes {
            world.record_component_modification(entity, &old_target, &new_target);
        }
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;
    use super::super::components::*;

    fn create_test_world_with_waiting_actor() -> World {
        let mut world = World::new();
        
        // Create an actor near HOME position
        let actor_entity = world.create_entity();
        world.add_component(actor_entity, Position { x: HOME_POS.0, y: HOME_POS.1 + 1 }); // Adjacent to home
        world.add_component(actor_entity, Actor);
        world.add_component(actor_entity, Target { x: HOME_POS.0, y: HOME_POS.1 });
        world.add_component(actor_entity, WaitTimer { ticks: 3 });
        
        // Add the wait system
        world.add_system(WaitSystem);
        world.initialize_systems();
        
        world
    }

    #[test]
    fn test_wait_system_decreases_timer() {
        let mut world = create_test_world_with_waiting_actor();
        
        let actor_entities = world.entities_with_component::<Actor>();
        let actor_entity = actor_entities[0];
        
        // Initial timer should be 3
        let initial_timer = world.get_component::<WaitTimer>(actor_entity).unwrap();
        assert_eq!(initial_timer.ticks, 3);
        
        // Run one update
        world.update();
        
        // Timer should decrease by 1
        let new_timer = world.get_component::<WaitTimer>(actor_entity).unwrap();
        assert_eq!(new_timer.ticks, 2);
    }

    #[test]
    fn test_wait_system_switches_target_when_timer_reaches_zero() {
        let mut world = World::new();
        
        // Create an actor at HOME position with 0 timer
        let actor_entity = world.create_entity();
        world.add_component(actor_entity, Position { x: HOME_POS.0, y: HOME_POS.1 });
        world.add_component(actor_entity, Actor);
        world.add_component(actor_entity, Target { x: HOME_POS.0, y: HOME_POS.1 });
        world.add_component(actor_entity, WaitTimer { ticks: 0 });
        
        world.add_system(WaitSystem);
        world.initialize_systems();
        
        // Run update
        world.update();
        
        // Target should switch to WORK and timer should reset
        let new_target = world.get_component::<Target>(actor_entity).unwrap();
        let new_timer = world.get_component::<WaitTimer>(actor_entity).unwrap();
        
        assert_eq!(new_target.x, WORK_POS.0);
        assert_eq!(new_target.y, WORK_POS.1);
        assert_eq!(new_timer.ticks, WAIT_TICKS);
    }

    #[test]
    fn test_wait_system_switches_from_work_to_home() {
        let mut world = World::new();
        
        // Create an actor at WORK position with 0 timer
        let actor_entity = world.create_entity();
        world.add_component(actor_entity, Position { x: WORK_POS.0, y: WORK_POS.1 });
        world.add_component(actor_entity, Actor);
        world.add_component(actor_entity, Target { x: WORK_POS.0, y: WORK_POS.1 });
        world.add_component(actor_entity, WaitTimer { ticks: 0 });
        
        world.add_system(WaitSystem);
        world.initialize_systems();
        
        // Run update
        world.update();
        
        // Target should switch to HOME and timer should reset
        let new_target = world.get_component::<Target>(actor_entity).unwrap();
        let new_timer = world.get_component::<WaitTimer>(actor_entity).unwrap();
        
        assert_eq!(new_target.x, HOME_POS.0);
        assert_eq!(new_target.y, HOME_POS.1);
        assert_eq!(new_timer.ticks, WAIT_TICKS);
    }

    #[test]
    fn test_wait_system_no_change_when_not_near_target() {
        let mut world = World::new();
        
        // Create an actor far from target
        let actor_entity = world.create_entity();
        world.add_component(actor_entity, Position { x: 5, y: 5 });
        world.add_component(actor_entity, Actor);
        world.add_component(actor_entity, Target { x: HOME_POS.0, y: HOME_POS.1 });
        world.add_component(actor_entity, WaitTimer { ticks: 3 });
        
        world.add_system(WaitSystem);
        world.initialize_systems();
        
        // Run update
        world.update();
        
        // Nothing should change since actor is not near target
        let timer = world.get_component::<WaitTimer>(actor_entity).unwrap();
        let target = world.get_component::<Target>(actor_entity).unwrap();
        
        assert_eq!(timer.ticks, 3);
        assert_eq!(target.x, HOME_POS.0);
        assert_eq!(target.y, HOME_POS.1);
    }
}