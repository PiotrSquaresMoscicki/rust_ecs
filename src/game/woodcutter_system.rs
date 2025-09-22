use crate::{In, Out, System, WorldView, World};
use super::components::{Position, Target, WaitTimer, Woodcutter, Tree, WoodcutterHut, CarryingTree, Actor};
use super::utils::is_adjacent;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Woodcutter System - manages woodcutter behavior for tree chopping and delivery
pub struct WoodcutterSystem;

impl System for WoodcutterSystem {
    type InComponents = (Woodcutter, Position, WaitTimer, Target, Tree, WoodcutterHut, CarryingTree);
    type OutComponents = (Target, WaitTimer, CarryingTree);

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // First, collect all tree and woodcutter hut positions
        let tree_positions: Vec<(i32, i32)> = world
            .query_components::<(In<Position>, In<Tree>)>()
            .into_iter()
            .map(|(_, (pos, _))| (pos.x, pos.y))
            .collect();

        let hut_positions: Vec<(i32, i32)> = world
            .query_components::<(In<Position>, In<WoodcutterHut>)>()
            .into_iter()
            .map(|(_, (pos, _))| (pos.x, pos.y))
            .collect();

        // Collect all carrying states
        let carrying_entities: std::collections::HashSet<_> = world
            .query_components::<(In<CarryingTree>,)>()
            .into_iter()
            .map(|(entity, _)| entity)
            .collect();

        // Collect changes to apply after queries
        let mut target_changes = Vec::new();
        let mut timer_changes = Vec::new();
        let mut carrying_changes = Vec::new();
        let mut entities_to_remove = Vec::new();

        // Query woodcutters
        for (entity, (position, _woodcutter, wait_timer, target)) in 
            world.query_components::<(In<Position>, In<Woodcutter>, Out<WaitTimer>, Out<Target>)>()
        {
            let current_pos = (position.x, position.y);
            let target_pos = (target.x, target.y);
            let is_near_target = is_adjacent(current_pos, target_pos) || current_pos == target_pos;

            // Check if woodcutter is carrying a tree
            let is_carrying = carrying_entities.contains(&entity);

            if is_carrying {
                // Woodcutter is carrying a tree - should go to nearest hut
                if is_near_target && hut_positions.contains(&target_pos) {
                    // At hut - wait for 2 ticks then remove carrying flag and find new tree
                    if wait_timer.ticks > 1 {
                        let old_timer = *wait_timer;
                        wait_timer.ticks -= 1;
                        timer_changes.push((entity, old_timer, *wait_timer));
                    } else {
                        // Timer will be 0 or is 0 - remove carrying flag and find nearest tree
                        carrying_changes.push((entity, CarryingTree, None));
                        
                        if let Some(&nearest_tree) = find_nearest_position(current_pos, &tree_positions) {
                            let old_target = *target;
                            target.x = nearest_tree.0;
                            target.y = nearest_tree.1;
                            target_changes.push((entity, old_target, *target));
                        }
                    }
                } else {
                    // Not at hut yet - ensure target is nearest hut
                    if let Some(&nearest_hut) = find_nearest_position(current_pos, &hut_positions) {
                        if target_pos != nearest_hut {
                            let old_target = *target;
                            target.x = nearest_hut.0;
                            target.y = nearest_hut.1;
                            target_changes.push((entity, old_target, *target));
                        }
                    }
                }
            } else {
                // Woodcutter is not carrying a tree - should go to nearest tree
                if is_near_target && tree_positions.contains(&target_pos) {
                    // At tree - chop for 10 ticks then remove tree and set carrying flag
                    if wait_timer.ticks > 1 {
                        let old_timer = *wait_timer;
                        wait_timer.ticks -= 1;
                        timer_changes.push((entity, old_timer, *wait_timer));
                    } else {
                        // Timer will be 0 or is 0 - tree is chopped, remove tree and set carrying flag
                        entities_to_remove.push(target_pos);
                        carrying_changes.push((entity, CarryingTree, Some(CarryingTree)));

                        // Set timer to 2 for hut delivery
                        let old_timer = *wait_timer;
                        wait_timer.ticks = 2;
                        timer_changes.push((entity, old_timer, *wait_timer));

                        // Find nearest hut
                        if let Some(&nearest_hut) = find_nearest_position(current_pos, &hut_positions) {
                            let old_target = *target;
                            target.x = nearest_hut.0;
                            target.y = nearest_hut.1;
                            target_changes.push((entity, old_target, *target));
                        }
                    }
                } else if is_near_target {
                    // Near target but target position doesn't have a tree anymore
                    // Find next nearest tree
                    if let Some(&nearest_tree) = find_nearest_position(current_pos, &tree_positions) {
                        if target_pos != nearest_tree {
                            let old_target = *target;
                            target.x = nearest_tree.0;
                            target.y = nearest_tree.1;
                            target_changes.push((entity, old_target, *target));

                            let old_timer = *wait_timer;
                            wait_timer.ticks = 10;
                            timer_changes.push((entity, old_timer, *wait_timer));
                        }
                    }
                } else {
                    // Not at tree yet - ensure target is nearest tree and reset timer to 10
                    if let Some(&nearest_tree) = find_nearest_position(current_pos, &tree_positions) {
                        if target_pos != nearest_tree {
                            let old_target = *target;
                            target.x = nearest_tree.0;
                            target.y = nearest_tree.1;
                            target_changes.push((entity, old_target, *target));

                            let old_timer = *wait_timer;
                            wait_timer.ticks = 10;
                            timer_changes.push((entity, old_timer, *wait_timer));
                        }
                    }
                }
            }
        }

        // Apply all changes
        for (entity, old_target, new_target) in target_changes {
            world.record_component_modification(entity, &old_target, &new_target);
        }

        for (entity, old_timer, new_timer) in timer_changes {
            world.record_component_modification(entity, &old_timer, &new_timer);
        }

        for (entity, _component, add_or_remove) in carrying_changes {
            match add_or_remove {
                Some(carrying) => {
                    world.add_component(entity, carrying);
                }
                None => {
                    world.remove_component::<CarryingTree>(entity);
                }
            }
        }

        // Remove chopped trees
        for tree_pos in entities_to_remove {
            // Find and remove tree entities at this position
            let tree_entities: Vec<_> = world
                .query_components::<(In<Position>, In<Tree>)>()
                .into_iter()
                .filter(|(_, (pos, _))| (pos.x, pos.y) == tree_pos)
                .map(|(entity, _)| entity)
                .collect();

            for entity in tree_entities {
                world.remove_entity(entity);
            }
        }
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}

/// Find the nearest position from a list of positions
fn find_nearest_position(from: (i32, i32), positions: &[(i32, i32)]) -> Option<&(i32, i32)> {
    positions
        .iter()
        .min_by_key(|&&(x, y)| {
            let dx = (from.0 - x).abs();
            let dy = (from.1 - y).abs();
            dx + dy // Manhattan distance
        })
}

/// Initialize a woodcutter demo world with 10 trees, 2 woodcutter huts, and 2 woodcutters
pub fn initialize_woodcutter_demo() -> World {
    let mut world = World::new();

    // Create 10 trees at fixed positions for reproducibility
    println!("Creating 10 trees...");
    let tree_positions = [
        (0, 0), (9, 0), (0, 9), (9, 9), // corners
        (4, 4), (5, 5), (3, 6), (6, 3), // middle area
        (1, 4), (7, 1)  // scattered
    ];
    
    for (i, &pos) in tree_positions.iter().enumerate() {
        let tree_entity = world.create_entity();
        world.add_component(tree_entity, Position { x: pos.0, y: pos.1 });
        world.add_component(tree_entity, Tree);
        println!("  Tree {} at ({}, {})", i + 1, pos.0, pos.1);
    }

    // Create 2 woodcutter huts at fixed positions
    println!("\nCreating 2 woodcutter huts...");
    let hut_positions = [(2, 8), (8, 2)]; // Corner positions
    for (i, &pos) in hut_positions.iter().enumerate() {
        let hut_entity = world.create_entity();
        world.add_component(hut_entity, Position { x: pos.0, y: pos.1 });
        world.add_component(hut_entity, WoodcutterHut);
        println!("  Woodcutter Hut {} at ({}, {})", i + 1, pos.0, pos.1);
    }

    // Create 2 woodcutter actors at fixed positions
    println!("\nCreating 2 woodcutters...");
    let woodcutter_positions = [(1, 1), (8, 8)]; // Fixed positions for reproducibility
    
    for (i, &pos) in woodcutter_positions.iter().enumerate() {
        let woodcutter_entity = world.create_entity();
        world.add_component(woodcutter_entity, Position { x: pos.0, y: pos.1 });
        world.add_component(woodcutter_entity, Woodcutter);
        world.add_component(woodcutter_entity, Actor); // Add Actor component so MovementSystem can move woodcutters
        
        // Find nearest tree as initial target
        let nearest_tree = tree_positions.iter()
            .min_by_key(|&&(tx, ty)| {
                let dx = (pos.0 - tx).abs();
                let dy = (pos.1 - ty).abs();
                dx + dy
            })
            .unwrap_or(&tree_positions[0]);
        
        world.add_component(woodcutter_entity, Target { x: nearest_tree.0, y: nearest_tree.1 });
        world.add_component(woodcutter_entity, WaitTimer { ticks: 10 });
        
        println!("  Woodcutter {} at ({}, {}) targeting tree at ({}, {})", 
                 i + 1, pos.0, pos.1, nearest_tree.0, nearest_tree.1);
    }

    // Add systems
    world.add_system(super::movement_system::MovementSystem);
    world.add_system(WoodcutterSystem);
    world.add_system(super::render_system::RenderSystem::default());

    // Initialize systems
    world.initialize_systems();

    println!("\nWoodcutter demo world initialized!");
    println!("- 10 trees");
    println!("- 2 woodcutter huts");
    println!("- 2 woodcutters");
    
    world
}

/// Log the state of each woodcutter
pub fn log_woodcutter_states(world: &World, update_count: u32) {
    println!("=== Update {} - Woodcutter States ===", update_count);
    
    let woodcutter_entities = world.entities_with_component::<Woodcutter>();
    
    for (i, &entity) in woodcutter_entities.iter().enumerate() {
        let pos = world.get_component::<Position>(entity).unwrap();
        let target = world.get_component::<Target>(entity).unwrap();
        let timer = world.get_component::<WaitTimer>(entity).unwrap();
        let carrying = world.get_component::<CarryingTree>(entity);
        
        println!("Woodcutter {} (Entity {:?}):", i + 1, entity);
        println!("  Position: ({}, {})", pos.x, pos.y);
        println!("  Target: ({}, {})", target.x, target.y);
        println!("  Timer: {} ticks", timer.ticks);
        println!("  Carrying tree: {}", carrying.is_some());
        
        // Determine what the woodcutter is doing
        let action = if carrying.is_some() {
            "Carrying tree to hut"
        } else {
            "Going to chop tree"
        };
        println!("  Action: {}", action);
        println!();
    }
    
    // Log total trees remaining
    let tree_count = world.entities_with_component::<Tree>().len();
    println!("Trees remaining: {}", tree_count);
    println!();
}

/// Run the woodcutter demo
pub fn run_woodcutter_demo() {
    println!("🌲 Starting Woodcutter Demo 🌲");
    println!("=====================================");
    println!("This demo shows 2 woodcutters chopping 10 trees and delivering them to 2 huts");
    println!("Symbols: T=Tree, W=Woodcutter Hut, C=Woodcutter, H=Home, O=Work/Office");
    println!("Press Ctrl+C to stop the demo");
    println!();

    let mut world = initialize_woodcutter_demo();

    // Set up Ctrl+C handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C, shutting down gracefully...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");

    let mut update_count = 0;
    
    // Demo loop - 1 tick per second for easier observation
    while running.load(Ordering::SeqCst) {
        update_count += 1;
        
        // Log woodcutter states before update
        log_woodcutter_states(&world, update_count);
        
        // Update the world
        world.update();
        
        // Check if all trees are chopped
        let tree_count = world.entities_with_component::<Tree>().len();
        if tree_count == 0 {
            println!("🎉 All trees have been chopped! Demo complete! 🎉");
            break;
        }
        
        thread::sleep(Duration::from_millis(1000)); // 1 FPS for better observation
    }

    println!("Woodcutter demo completed after {} updates", update_count);
    let final_tree_count = world.entities_with_component::<Tree>().len();
    println!("Final tree count: {}", final_tree_count);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;
    use super::super::components::*;

    fn create_woodcutter_test_world() -> World {
        let mut world = World::new();

        // Create woodcutter at (0, 0)
        let woodcutter_entity = world.create_entity();
        world.add_component(woodcutter_entity, Position { x: 0, y: 0 });
        world.add_component(woodcutter_entity, Woodcutter);
        world.add_component(woodcutter_entity, Target { x: 2, y: 2 }); // Initial target
        world.add_component(woodcutter_entity, WaitTimer { ticks: 10 });

        // Create three trees
        let tree1 = world.create_entity();
        world.add_component(tree1, Position { x: 2, y: 2 });
        world.add_component(tree1, Tree);

        let tree2 = world.create_entity();
        world.add_component(tree2, Position { x: 4, y: 4 });
        world.add_component(tree2, Tree);

        let tree3 = world.create_entity();
        world.add_component(tree3, Position { x: 6, y: 6 });
        world.add_component(tree3, Tree);

        // Create woodcutter hut
        let hut = world.create_entity();
        world.add_component(hut, Position { x: 8, y: 8 });
        world.add_component(hut, WoodcutterHut);

        world.add_system(WoodcutterSystem);
        world.initialize_systems();

        world
    }

    #[test]
    fn test_woodcutter_system_creation() {
        let system = WoodcutterSystem;
        // Test that the system can be created
        assert_eq!(std::mem::size_of_val(&system), 0); // Zero-sized struct
    }

    #[test]
    fn test_woodcutter_targets_nearest_tree() {
        let world = create_woodcutter_test_world();

        // Get woodcutter entity
        let woodcutter_entities = world.entities_with_component::<Woodcutter>();
        assert_eq!(woodcutter_entities.len(), 1);
        let woodcutter_entity = woodcutter_entities[0];

        // Initial target should be set to nearest tree (2, 2)
        let target = world.get_component::<Target>(woodcutter_entity).unwrap();
        assert_eq!((target.x, target.y), (2, 2));
    }

    #[test]
    fn test_woodcutter_chops_tree_over_time() {
        let mut world = create_woodcutter_test_world();

        // Move woodcutter to tree position
        let woodcutter_entities = world.entities_with_component::<Woodcutter>();
        let woodcutter_entity = woodcutter_entities[0];
        
        // Update position to be at tree
        world.remove_component::<Position>(woodcutter_entity);
        world.add_component(woodcutter_entity, Position { x: 2, y: 2 });

        // Set timer to 1 (will become 0 after the update)
        world.remove_component::<WaitTimer>(woodcutter_entity);
        world.add_component(woodcutter_entity, WaitTimer { ticks: 1 });

        // Run one update to trigger tree chopping
        world.update();

        // After the update, woodcutter should be carrying tree
        let carrying = world.get_component::<CarryingTree>(woodcutter_entity);
        assert!(carrying.is_some());

        // Tree should be removed
        let tree_count_after = world.entities_with_component::<Tree>().len();
        assert_eq!(tree_count_after, 2); // Started with 3, one should be removed
    }

    #[test]
    fn test_find_nearest_position() {
        let positions = vec![(5, 5), (10, 10), (2, 3)];
        let nearest = find_nearest_position((0, 0), &positions);
        assert_eq!(nearest, Some(&(2, 3))); // Closest to (0,0)

        let nearest = find_nearest_position((6, 6), &positions);
        assert_eq!(nearest, Some(&(5, 5))); // Closest to (6,6)
    }

    #[test]
    fn test_woodcutter_integration() {
        let mut world = create_woodcutter_test_world();

        // Get initial tree count
        let initial_tree_count = world.entities_with_component::<Tree>().len();
        assert_eq!(initial_tree_count, 3);

        // Simulate one complete cycle - manually move woodcutter to first tree and set timer to 1
        let woodcutter_entities = world.entities_with_component::<Woodcutter>();
        let woodcutter_entity = woodcutter_entities[0];
        
        world.remove_component::<Position>(woodcutter_entity);
        world.add_component(woodcutter_entity, Position { x: 2, y: 2 });
        world.remove_component::<WaitTimer>(woodcutter_entity);
        world.add_component(woodcutter_entity, WaitTimer { ticks: 1 });

        // Run update to chop first tree
        world.update();

        // Should have one less tree and woodcutter should be carrying
        let tree_count_after_chop = world.entities_with_component::<Tree>().len();
        assert_eq!(tree_count_after_chop, 2);

        let carrying = world.get_component::<CarryingTree>(woodcutter_entity);
        assert!(carrying.is_some());

        // Move to hut and set timer to 1
        world.remove_component::<Position>(woodcutter_entity);
        world.add_component(woodcutter_entity, Position { x: 8, y: 8 });
        world.remove_component::<WaitTimer>(woodcutter_entity);
        world.add_component(woodcutter_entity, WaitTimer { ticks: 1 });

        // Run update to deliver at hut
        world.update();

        // Should no longer be carrying
        let carrying = world.get_component::<CarryingTree>(woodcutter_entity);
        assert!(carrying.is_none());

        // Should target next nearest tree
        let target = world.get_component::<Target>(woodcutter_entity).unwrap();
        // Should target (6,6) which is closer to (8,8) than (4,4)
        assert_eq!((target.x, target.y), (6, 6));
    }

    #[test]
    fn test_woodcutter_complete_cycle_demonstration() {
        let mut world = create_woodcutter_test_world();

        // Test the complete cycle: woodcutter starts at (0,0), goes to tree at (2,2), chops it, 
        // then goes to hut at (8,8), delivers it, then targets next nearest tree

        let woodcutter_entities = world.entities_with_component::<Woodcutter>();
        let woodcutter_entity = woodcutter_entities[0];

        // Initial state: woodcutter should target nearest tree
        let initial_target = world.get_component::<Target>(woodcutter_entity).unwrap();
        assert_eq!((initial_target.x, initial_target.y), (2, 2));

        // Step 1: Move woodcutter to tree and chop it
        world.remove_component::<Position>(woodcutter_entity);
        world.add_component(woodcutter_entity, Position { x: 2, y: 2 });
        world.remove_component::<WaitTimer>(woodcutter_entity);
        world.add_component(woodcutter_entity, WaitTimer { ticks: 1 });

        let trees_before = world.entities_with_component::<Tree>().len();
        world.update();

        // After chopping: should be carrying tree, one less tree exists, timer set to 2, targeting hut
        let carrying = world.get_component::<CarryingTree>(woodcutter_entity);
        assert!(carrying.is_some());
        let trees_after = world.entities_with_component::<Tree>().len();
        assert_eq!(trees_after, trees_before - 1);
        let timer = world.get_component::<WaitTimer>(woodcutter_entity).unwrap();
        assert_eq!(timer.ticks, 2);
        let target = world.get_component::<Target>(woodcutter_entity).unwrap();
        assert_eq!((target.x, target.y), (8, 8)); // Should target hut

        // Step 2: Move woodcutter to hut and deliver
        world.remove_component::<Position>(woodcutter_entity);
        world.add_component(woodcutter_entity, Position { x: 8, y: 8 });
        world.remove_component::<WaitTimer>(woodcutter_entity);
        world.add_component(woodcutter_entity, WaitTimer { ticks: 1 });

        world.update();

        // After delivery: should not be carrying, target next nearest tree
        let carrying_after = world.get_component::<CarryingTree>(woodcutter_entity);
        assert!(carrying_after.is_none());
        let final_target = world.get_component::<Target>(woodcutter_entity).unwrap();
        // Should target (6,6) which is the nearest remaining tree from (8,8)
        assert_eq!((final_target.x, final_target.y), (6, 6));

        println!("✅ Woodcutter system demonstration complete!");
        println!("- Started with 3 trees");
        println!("- Chopped down 1 tree");
        println!("- Carried tree to hut");
        println!("- Now targeting next tree at (6,6)");
        println!("- {} trees remaining", world.entities_with_component::<Tree>().len());
    }
}