use super::components::{
    Actor, Carpenter, CarpenterHut, Navigation, Position, Target, WaitTimer, WoodcutterHut,
};
use super::utils::is_adjacent;
use crate::{In, Out, System, World, WorldView};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// Carpenter System - manages carpenter behavior for traveling between woodcutter huts and carpenter huts
pub struct CarpenterSystem;

impl System for CarpenterSystem {
    type InComponents = (
        Carpenter,
        Position,
        WaitTimer,
        Target,
        WoodcutterHut,
        CarpenterHut,
        Navigation,
    );
    type OutComponents = (Target, WaitTimer, Navigation);
    type InSystems = ();
    type OutSystems = ();

    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}

    fn update(&mut self, world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // Collect all woodcutter hut positions (where carpenters collect wood)
        let woodcutter_hut_positions: Vec<(i32, i32)> = world
            .query_components::<(In<Position>, In<WoodcutterHut>)>()
            .into_iter()
            .map(|(_, (pos, _))| (pos.x, pos.y))
            .collect();

        // Collect all carpenter hut positions (where carpenters work)
        let carpenter_hut_positions: Vec<(i32, i32)> = world
            .query_components::<(In<Position>, In<CarpenterHut>)>()
            .into_iter()
            .map(|(_, (pos, _))| (pos.x, pos.y))
            .collect();

        // Debug output
        if !woodcutter_hut_positions.is_empty() && !carpenter_hut_positions.is_empty() {
            println!(
                "  🔨 Carpenter: {} woodcutter huts, {} carpenter huts available",
                woodcutter_hut_positions.len(),
                carpenter_hut_positions.len()
            );
        }

        // Collect changes to apply after queries
        let mut target_changes = Vec::new();
        let mut timer_changes = Vec::new();
        let mut navigation_changes = Vec::new();

        // Query carpenters
        for (entity, (position, _carpenter, wait_timer, target, navigation)) in world
            .query_components::<(
                In<Position>,
                In<Carpenter>,
                Out<WaitTimer>,
                Out<Target>,
                Out<Navigation>,
            )>()
        {
            let current_pos = (position.x, position.y);
            let target_pos = (target.x, target.y);
            let is_near_target = is_adjacent(current_pos, target_pos) || current_pos == target_pos;

            // Determine current state based on target location
            let at_woodcutter_hut = woodcutter_hut_positions.contains(&target_pos);
            let at_carpenter_hut = carpenter_hut_positions.contains(&target_pos);

            if is_near_target {
                if at_woodcutter_hut {
                    // At woodcutter hut - collect wood for 5 ticks then go to carpenter hut
                    if wait_timer.ticks > 1 {
                        let old_timer = *wait_timer;
                        wait_timer.ticks -= 1;
                        timer_changes.push((entity, old_timer, *wait_timer));
                    } else {
                        // Wood collected - go to nearest carpenter hut
                        if let Some(&nearest_carpenter_hut) =
                            find_nearest_position(current_pos, &carpenter_hut_positions)
                        {
                            let old_target = *target;
                            target.x = nearest_carpenter_hut.0;
                            target.y = nearest_carpenter_hut.1;
                            target_changes.push((entity, old_target, *target));

                            // Set timer for working at carpenter hut
                            let old_timer = *wait_timer;
                            wait_timer.ticks = 8; // Work for 8 ticks
                            timer_changes.push((entity, old_timer, *wait_timer));

                            // Signal navigation recalculation for new target
                            let old_navigation = navigation.clone();
                            navigation.request_recalculation();
                            navigation_changes.push((entity, old_navigation, navigation.clone()));
                        }
                    }
                } else if at_carpenter_hut {
                    // At carpenter hut - work for 8 ticks then go to woodcutter hut
                    if wait_timer.ticks > 1 {
                        let old_timer = *wait_timer;
                        wait_timer.ticks -= 1;
                        timer_changes.push((entity, old_timer, *wait_timer));
                    } else {
                        // Work finished - go to nearest woodcutter hut for more wood
                        if let Some(&nearest_woodcutter_hut) =
                            find_nearest_position(current_pos, &woodcutter_hut_positions)
                        {
                            let old_target = *target;
                            target.x = nearest_woodcutter_hut.0;
                            target.y = nearest_woodcutter_hut.1;
                            target_changes.push((entity, old_target, *target));

                            // Set timer for collecting wood
                            let old_timer = *wait_timer;
                            wait_timer.ticks = 5; // Collect for 5 ticks
                            timer_changes.push((entity, old_timer, *wait_timer));

                            // Signal navigation recalculation for new target
                            let old_navigation = navigation.clone();
                            navigation.request_recalculation();
                            navigation_changes.push((entity, old_navigation, navigation.clone()));
                        }
                    }
                } else {
                    // At unknown location - go to nearest woodcutter hut
                    if let Some(&nearest_woodcutter_hut) =
                        find_nearest_position(current_pos, &woodcutter_hut_positions)
                    {
                        let old_target = *target;
                        target.x = nearest_woodcutter_hut.0;
                        target.y = nearest_woodcutter_hut.1;
                        target_changes.push((entity, old_target, *target));

                        // Set timer for collecting wood
                        let old_timer = *wait_timer;
                        wait_timer.ticks = 5;
                        timer_changes.push((entity, old_timer, *wait_timer));

                        // Signal navigation recalculation for new target
                        let old_navigation = navigation.clone();
                        navigation.request_recalculation();
                        navigation_changes.push((entity, old_navigation, navigation.clone()));
                    }
                }
            } else {
                // Not at target yet - ensure we're heading to the right place
                // This handles cases where the carpenter needs to retarget
                if !at_woodcutter_hut && !at_carpenter_hut {
                    // Target is neither a woodcutter hut nor carpenter hut - go to nearest woodcutter hut
                    if let Some(&nearest_woodcutter_hut) =
                        find_nearest_position(current_pos, &woodcutter_hut_positions)
                    {
                        if target_pos != nearest_woodcutter_hut {
                            let old_target = *target;
                            target.x = nearest_woodcutter_hut.0;
                            target.y = nearest_woodcutter_hut.1;
                            target_changes.push((entity, old_target, *target));

                            // Set timer for collecting wood
                            let old_timer = *wait_timer;
                            wait_timer.ticks = 5;
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
            println!(
                "  Carpenter {:?}: {} -> ({}, {})",
                entity,
                if woodcutter_hut_positions.contains(&(old_target.x, old_target.y)) {
                    "Woodcutter Hut"
                } else if carpenter_hut_positions.contains(&(old_target.x, old_target.y)) {
                    "Carpenter Hut"
                } else {
                    "Unknown"
                },
                new_target.x,
                new_target.y
            );
        }

        for (entity, old_timer, new_timer) in timer_changes {
            if old_timer.ticks != new_timer.ticks {
                println!(
                    "  Carpenter {:?}: Timer {} -> {}",
                    entity, old_timer.ticks, new_timer.ticks
                );
            }
        }

        // Navigation changes are handled through the mutable references
    }

    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}

/// Find the nearest position from a list of positions
fn find_nearest_position(from: (i32, i32), positions: &[(i32, i32)]) -> Option<&(i32, i32)> {
    positions.iter().min_by_key(|&&pos| {
        let dx = from.0 - pos.0;
        let dy = from.1 - pos.1;
        dx * dx + dy * dy
    })
}

/// Initialize a carpenter demo world with carpenters traveling between woodcutter and carpenter huts
pub fn initialize_carpenter_demo() -> World {
    println!("=== Carpenter System Demo ===");
    println!(
        "This demo showcases carpenters traveling between woodcutter huts and carpenter huts."
    );
    println!("Carpenters collect wood from woodcutter huts and work at carpenter huts.");

    let mut world = World::new();

    // Create two woodcutter huts (sources of wood for carpenters)
    println!("\nCreating 2 woodcutter huts...");
    let woodcutter_hut_positions = [
        (5, 3),   // Left side
        (25, 10), // Right side
    ];

    for (i, &pos) in woodcutter_hut_positions.iter().enumerate() {
        let hut_entity = world.create_entity();
        world.add_component(hut_entity, Position { x: pos.0, y: pos.1 });
        world.add_component(hut_entity, WoodcutterHut);
        println!("  Woodcutter Hut {} at ({}, {})", i + 1, pos.0, pos.1);
    }

    // Create one carpenter hut (workplace for carpenters)
    println!("\nCreating 1 carpenter hut...");
    let carpenter_hut_position = (15, 7); // Center
    let carpenter_hut_entity = world.create_entity();
    world.add_component(
        carpenter_hut_entity,
        Position {
            x: carpenter_hut_position.0,
            y: carpenter_hut_position.1,
        },
    );
    world.add_component(carpenter_hut_entity, CarpenterHut);
    println!(
        "  Carpenter Hut at ({}, {})",
        carpenter_hut_position.0, carpenter_hut_position.1
    );

    // Create 2 carpenters starting near the carpenter hut
    println!("\nCreating 2 carpenters...");
    let carpenter_positions = [
        (14, 7), // Near carpenter hut
        (16, 8), // Near carpenter hut
    ];

    for (i, &pos) in carpenter_positions.iter().enumerate() {
        let carpenter_entity = world.create_entity();
        world.add_component(carpenter_entity, Position { x: pos.0, y: pos.1 });
        world.add_component(carpenter_entity, Actor); // For rendering
        world.add_component(carpenter_entity, Carpenter);
        world.add_component(carpenter_entity, WaitTimer { ticks: 1 }); // Start immediately

        // Initially target the nearest woodcutter hut
        let nearest_woodcutter_hut = woodcutter_hut_positions
            .iter()
            .min_by_key(|&&hut_pos| {
                let dx = pos.0 - hut_pos.0;
                let dy = pos.1 - hut_pos.1;
                dx * dx + dy * dy
            })
            .unwrap();

        world.add_component(
            carpenter_entity,
            Target {
                x: nearest_woodcutter_hut.0,
                y: nearest_woodcutter_hut.1,
            },
        );
        world.add_component(carpenter_entity, Navigation::new()); // Add Navigation for pathfinding

        println!(
            "  Carpenter {} at ({}, {}) targeting woodcutter hut at ({}, {})",
            i + 1,
            pos.0,
            pos.1,
            nearest_woodcutter_hut.0,
            nearest_woodcutter_hut.1
        );
    }

    // Add systems
    world.add_system(super::navigation_system::NavigationSystem);
    world.add_system(CarpenterSystem);
    world.add_system(super::render_system::RenderSystem::default());

    // Initialize systems
    world.initialize_systems();

    println!("\nCarpenter demo world initialized!");
    println!("- 2 woodcutter huts (wood sources)");
    println!("- 1 carpenter hut (workplace)");
    println!("- 2 carpenters");

    world
}

/// Run the carpenter demo
pub fn run_carpenter_demo() {
    let mut world = initialize_carpenter_demo();

    println!("\nStarting simulation... (Press Ctrl+C to stop)");

    // Set up signal handler for graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    if let Err(e) = ctrlc::set_handler(move || {
        println!("\nShutdown signal received!");
        r.store(false, Ordering::SeqCst);
    }) {
        eprintln!("Error setting Ctrl+C handler: {}", e);
    }

    let mut frame = 0;
    while running.load(Ordering::SeqCst) {
        frame += 1;
        println!("\n--- Frame {} ---", frame);
        world.update();

        thread::sleep(Duration::from_millis(1000)); // 1 second between frames

        // Print frame diff after all system updates and after sleep
        world.print_last_frame_diff();
    }

    world.deinitialize_systems();
    println!("Carpenter demo simulation ended.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_carpenter_system_creation() {
        let system = CarpenterSystem;
        // Just verify it can be created - integration tests will test functionality
        drop(system);
    }

    #[test]
    fn test_find_nearest_position() {
        let positions = vec![(0, 0), (5, 5), (10, 10)];
        let from = (3, 3);
        let nearest = find_nearest_position(from, &positions);
        assert_eq!(nearest, Some(&(5, 5)));
    }

    #[test]
    fn test_carpenter_integration() {
        let mut world = World::new();

        // Create woodcutter hut
        let woodcutter_hut_entity = world.create_entity();
        world.add_component(woodcutter_hut_entity, Position { x: 5, y: 5 });
        world.add_component(woodcutter_hut_entity, WoodcutterHut);

        // Create carpenter hut
        let carpenter_hut_entity = world.create_entity();
        world.add_component(carpenter_hut_entity, Position { x: 10, y: 10 });
        world.add_component(carpenter_hut_entity, CarpenterHut);

        // Create carpenter
        let carpenter_entity = world.create_entity();
        world.add_component(carpenter_entity, Position { x: 5, y: 5 });
        world.add_component(carpenter_entity, Carpenter);
        world.add_component(carpenter_entity, WaitTimer { ticks: 1 });
        world.add_component(carpenter_entity, Target { x: 5, y: 5 }); // At woodcutter hut
        world.add_component(carpenter_entity, Navigation::new());

        // Add system
        world.add_system(CarpenterSystem);
        world.initialize_systems();

        // Run one update
        world.update();

        // Carpenter should have immediately switched to carpenter hut (timer was 1, not > 1)
        let target = world.get_component::<Target>(carpenter_entity).unwrap();
        assert_eq!(target.x, 10);
        assert_eq!(target.y, 10);

        // Timer should be set to 8 for working at carpenter hut
        let timer = world.get_component::<WaitTimer>(carpenter_entity).unwrap();
        assert_eq!(timer.ticks, 8);

        // Move carpenter to the carpenter hut position to simulate navigation
        world.remove_component::<Position>(carpenter_entity);
        world.add_component(carpenter_entity, Position { x: 10, y: 10 });

        // Run another update - now that carpenter is at carpenter hut, timer should decrement
        world.update();

        let timer = world.get_component::<WaitTimer>(carpenter_entity).unwrap();
        assert_eq!(timer.ticks, 7);
    }
}
