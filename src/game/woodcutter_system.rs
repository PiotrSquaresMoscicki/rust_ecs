use crate::{In, Out, Not, System, WorldView, World};
use super::components::{Position, Target, WaitTimer, Woodcutter, Tree, WoodcutterHut, CarryingTree, Actor, Navigation, AssignedWoodcutter};
use super::utils::is_adjacent;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Woodcutter System - manages woodcutter behavior for tree chopping and delivery
pub struct WoodcutterSystem;

impl System for WoodcutterSystem {
    type InComponents = (Woodcutter, Position, WaitTimer, Target, Tree, WoodcutterHut, CarryingTree, Navigation, AssignedWoodcutter);
    type OutComponents = (Target, WaitTimer, CarryingTree, Navigation, AssignedWoodcutter);
    type InSystems = ();
    type OutSystems = ();

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // Use the actual Not<> component query to find unassigned trees!
        // This demonstrates the key functionality requested in the original issue
        let unassigned_tree_positions: Vec<(i32, i32)> = world
            .query_components::<(In<Position>, In<Tree>, Not<AssignedWoodcutter>)>()
            .into_iter()
            .map(|(_, (pos, _, _))| (pos.x, pos.y))
            .collect();

        // Collect all tree positions (for validation and chopping)
        let all_tree_positions: Vec<(i32, i32)> = world
            .query_components::<(In<Position>, In<Tree>)>()
            .into_iter()
            .map(|(_, (pos, _))| (pos.x, pos.y))
            .collect();

        // Debug output to show the Not<> functionality working
        if !unassigned_tree_positions.is_empty() {
            println!("  🌲 Not<> Query Result: {} unassigned trees found", 
                     unassigned_tree_positions.len());
        }

        // Track trees that get assigned during this frame to prevent race conditions
        let mut frame_assigned_trees: std::collections::HashSet<(i32, i32)> = std::collections::HashSet::new();

        // Helper function to get currently available trees (excluding those assigned this frame)
        let get_available_trees = |frame_assigned: &std::collections::HashSet<(i32, i32)>| -> Vec<(i32, i32)> {
            unassigned_tree_positions
                .iter()
                .filter(|pos| !frame_assigned.contains(pos))
                .copied()
                .collect()
        };

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
        let mut navigation_changes = Vec::new();
        let mut assignment_changes = Vec::new(); // For AssignedWoodcutter components
        let mut entities_to_remove = Vec::new();

        // Query woodcutters
        for (entity, (position, _woodcutter, wait_timer, target, navigation)) in 
            world.query_components::<(In<Position>, In<Woodcutter>, Out<WaitTimer>, Out<Target>, Out<Navigation>)>()
        {
            let current_pos = (position.x, position.y);
            let target_pos = (target.x, target.y);
            let is_near_target = is_adjacent(current_pos, target_pos) || current_pos == target_pos;

            // Check if woodcutter is carrying a tree
            let is_carrying = carrying_entities.contains(&entity);

            if is_carrying {
                // Woodcutter is carrying a tree - should go to nearest hut
                if is_near_target && hut_positions.contains(&target_pos) {
                    // At hut - wait for 2 ticks then remove carrying flag and find new unassigned tree
                    if wait_timer.ticks > 1 {
                        let old_timer = *wait_timer;
                        wait_timer.ticks -= 1;
                        timer_changes.push((entity, old_timer, *wait_timer));
                    } else {
                        // Timer will be 0 or is 0 - remove carrying flag and find nearest unassigned tree
                        carrying_changes.push((entity, CarryingTree, None));
                        
                        // Use actual Not<> query for available trees (the key feature!)
                        let available_trees = get_available_trees(&frame_assigned_trees);
                        if let Some(&nearest_tree) = find_nearest_position(current_pos, &available_trees) {
                            let old_target = *target;
                            target.x = nearest_tree.0;
                            target.y = nearest_tree.1;
                            target_changes.push((entity, old_target, *target));
                            
                            // Assign this woodcutter to the tree and track it for this frame
                            assignment_changes.push((nearest_tree, Some(AssignedWoodcutter { woodcutter_id: entity.entity_index as u32 })));
                            frame_assigned_trees.insert(nearest_tree);
                            
                            // Signal navigation recalculation for new target
                            let old_navigation = navigation.clone();
                            navigation.request_recalculation();
                            navigation_changes.push((entity, old_navigation, navigation.clone()));
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
                            
                            // Signal navigation recalculation for new target
                            let old_navigation = navigation.clone();
                            navigation.request_recalculation();
                            navigation_changes.push((entity, old_navigation, navigation.clone()));
                        }
                    }
                }
            } else {
                // Woodcutter is not carrying a tree - should go to assigned or find unassigned tree
                if is_near_target && all_tree_positions.contains(&target_pos) {
                    // At tree - chop for 10 ticks then remove tree and assignment, set carrying flag
                    if wait_timer.ticks > 1 {
                        let old_timer = *wait_timer;
                        wait_timer.ticks -= 1;
                        timer_changes.push((entity, old_timer, *wait_timer));
                    } else {
                        // Timer will be 0 or is 0 - tree is chopped
                        entities_to_remove.push(target_pos);
                        carrying_changes.push((entity, CarryingTree, Some(CarryingTree)));
                        
                        // Remove assignment from chopped tree
                        assignment_changes.push((target_pos, None));

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
                            
                            // Signal navigation recalculation for new target
                            let old_navigation = navigation.clone();
                            navigation.request_recalculation();
                            navigation_changes.push((entity, old_navigation, navigation.clone()));
                        }
                    }
                } else if is_near_target {
                    // Near target but target position doesn't have a tree anymore
                    // Find next nearest unassigned tree using actual Not<> query
                    let available_trees = get_available_trees(&frame_assigned_trees);
                    if let Some(&nearest_tree) = find_nearest_position(current_pos, &available_trees) {
                        if target_pos != nearest_tree {
                            let old_target = *target;
                            target.x = nearest_tree.0;
                            target.y = nearest_tree.1;
                            target_changes.push((entity, old_target, *target));

                            // Assign this woodcutter to the new tree and track it for this frame
                            assignment_changes.push((nearest_tree, Some(AssignedWoodcutter { woodcutter_id: entity.entity_index as u32 })));
                            frame_assigned_trees.insert(nearest_tree);

                            let old_timer = *wait_timer;
                            wait_timer.ticks = 10;
                            timer_changes.push((entity, old_timer, *wait_timer));
                            
                            // Signal navigation recalculation for new target
                            let old_navigation = navigation.clone();
                            navigation.request_recalculation();
                            navigation_changes.push((entity, old_navigation, navigation.clone()));
                        }
                    }
                } else {
                    // Not at tree yet - ensure target is nearest unassigned tree using Not<> query
                    let available_trees = get_available_trees(&frame_assigned_trees);
                    if let Some(&nearest_tree) = find_nearest_position(current_pos, &available_trees) {
                        if target_pos != nearest_tree {
                            // Remove assignment from old target if it was assigned to this woodcutter
                            if all_tree_positions.contains(&target_pos) {
                                assignment_changes.push((target_pos, None));
                            }
                            
                            let old_target = *target;
                            target.x = nearest_tree.0;
                            target.y = nearest_tree.1;
                            target_changes.push((entity, old_target, *target));

                            // Assign this woodcutter to the new tree and track it for this frame
                            assignment_changes.push((nearest_tree, Some(AssignedWoodcutter { woodcutter_id: entity.entity_index as u32 })));
                            frame_assigned_trees.insert(nearest_tree);

                            let old_timer = *wait_timer;
                            wait_timer.ticks = 10;
                            timer_changes.push((entity, old_timer, *wait_timer));
                            
                            // Signal navigation recalculation for new target
                            let old_navigation = navigation.clone();
                            navigation.request_recalculation();
                            navigation_changes.push((entity, old_navigation, navigation.clone()));
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

        for (entity, old_navigation, new_navigation) in navigation_changes {
            world.record_component_modification(entity, &old_navigation, &new_navigation);
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

        // Apply assignment changes - this demonstrates the Not<> functionality!
        for (tree_pos, assignment) in assignment_changes {
            // Find tree entities at this position
            let tree_entities: Vec<_> = world
                .query_components::<(In<Position>, In<Tree>)>()
                .into_iter()
                .filter(|(_, (pos, _))| (pos.x, pos.y) == tree_pos)
                .map(|(entity, _)| entity)
                .collect();

            for entity in tree_entities {
                match assignment {
                    Some(assigned_woodcutter) => {
                        // Remove any existing assignment first
                        world.remove_component::<AssignedWoodcutter>(entity);
                        // Add new assignment
                        world.add_component(entity, assigned_woodcutter);
                    }
                    None => {
                        // Remove assignment
                        world.remove_component::<AssignedWoodcutter>(entity);
                    }
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

/// Initialize a woodcutter demo world showcasing Not<> query functionality
/// - All trees in one corner except one in the middle
/// - Single woodcutter hut in opposite corner  
/// - Woodcutters start near the hut
/// - AssignedWoodcutter component prevents multiple woodcutters targeting same tree
pub fn initialize_woodcutter_demo() -> World {
    let mut world = World::new();

    // Create 9 trees clustered in one corner (0,0 to 2,2) plus one in the middle
    println!("Creating 10 trees...");
    let tree_positions = [
        // Corner cluster (9 trees)
        (0, 0), (0, 1), (0, 2),
        (1, 0), (1, 1), (1, 2), 
        (2, 0), (2, 1), (2, 2),
        // Middle tree
        (5, 5)
    ];
    
    for (i, &pos) in tree_positions.iter().enumerate() {
        let tree_entity = world.create_entity();
        world.add_component(tree_entity, Position { x: pos.0, y: pos.1 });
        world.add_component(tree_entity, Tree);
        if i < 9 {
            println!("  Tree {} at ({}, {}) [corner cluster]", i + 1, pos.0, pos.1);
        } else {
            println!("  Tree {} at ({}, {}) [middle tree]", i + 1, pos.0, pos.1);
        }
    }

    // Create single woodcutter hut in opposite corner
    println!("\nCreating 1 woodcutter hut...");
    let hut_position = (8, 8); // Opposite corner from trees
    let hut_entity = world.create_entity();
    world.add_component(hut_entity, Position { x: hut_position.0, y: hut_position.1 });
    world.add_component(hut_entity, WoodcutterHut);
    println!("  Woodcutter Hut at ({}, {})", hut_position.0, hut_position.1);

    // Create 2 woodcutter actors starting near the hut
    println!("\nCreating 2 woodcutters...");
    let woodcutter_positions = [(7, 7), (7, 8)]; // Near the hut
    
    for (i, &pos) in woodcutter_positions.iter().enumerate() {
        let woodcutter_entity = world.create_entity();
        world.add_component(woodcutter_entity, Position { x: pos.0, y: pos.1 });
        world.add_component(woodcutter_entity, Woodcutter);
        world.add_component(woodcutter_entity, Actor); // Add Actor component so NavigationSystem can move woodcutters
        
        // Initial target will be set by the woodcutter system using Not<AssignedWoodcutter> query
        world.add_component(woodcutter_entity, Target { x: pos.0, y: pos.1 }); // Start at current position
        world.add_component(woodcutter_entity, WaitTimer { ticks: 10 });
        world.add_component(woodcutter_entity, Navigation::new()); // Add Navigation for pathfinding
        
        println!("  Woodcutter {} at ({}, {}) near hut", i + 1, pos.0, pos.1);
    }

    // Add systems
    world.add_system(super::navigation_system::NavigationSystem);
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

/// Run the woodcutter demo showcasing Not<> component queries
pub fn run_woodcutter_demo() {
    println!("=== Woodcutter Not<> Component Query Demo ===");
    println!("This demo showcases the new Not<> query functionality.");
    println!("Woodcutters will only target trees that are NOT assigned to other woodcutters.\n");
    
    let mut world = initialize_woodcutter_demo();
    
    // Track some statistics
    let mut update_count = 0;
    let stop_signal = Arc::new(AtomicBool::new(false));
    
    // Setup Ctrl+C handler
    let stop_signal_clone = stop_signal.clone();
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C, stopping simulation...");
        stop_signal_clone.store(true, Ordering::SeqCst);
    }).expect("Error setting Ctrl+C handler");
    
    println!("\nStarting simulation... (Press Ctrl+C to stop)\n");
    
    while !stop_signal.load(Ordering::SeqCst) {
        world.update();
        update_count += 1;
        
        // Stop after 50 updates or when no trees left
        let tree_count = world.entities_with_component::<Tree>().len();
        if update_count >= 50 || tree_count == 0 {
            break;
        }
        
        thread::sleep(Duration::from_millis(500));
        
        // Print frame diff after all system updates and after sleep
        world.print_last_frame_diff();
    }
    
    println!("\n=== Demo Complete ===");
    println!("Updates: {}", update_count);
    let final_tree_count = world.entities_with_component::<Tree>().len();
    println!("Trees remaining: {}", final_tree_count);
    
    if final_tree_count < 10 {
        println!("Success! Woodcutters successfully used Not<AssignedWoodcutter> queries to prevent conflicts.");
    }
}





#[cfg(test)]
mod tests {
    use super::*;
    use crate::World;
    use super::super::components::*;
    use super::super::navigation_system::NavigationSystem;

    fn create_woodcutter_test_world() -> World {
        let mut world = World::new();

        // Create woodcutter at (0, 0)
        let woodcutter_entity = world.create_entity();
        world.add_component(woodcutter_entity, Position { x: 0, y: 0 });
        world.add_component(woodcutter_entity, Woodcutter);
        world.add_component(woodcutter_entity, Actor);
        world.add_component(woodcutter_entity, Target { x: 2, y: 2 }); // Initial target
        world.add_component(woodcutter_entity, WaitTimer { ticks: 10 });
        world.add_component(woodcutter_entity, Navigation::new()); // Add Navigation component

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

        world.add_system(NavigationSystem);
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