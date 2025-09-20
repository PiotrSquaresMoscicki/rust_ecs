use rust_ecs::{World, game::game::{MovementSystem, WaitSystem, RenderSystem}};

#[test]
fn test_system_addition_recording_and_replay() {
    // Test that system additions are recorded and can be replayed
    
    // Create a world and add systems
    let mut original_world = World::new();
    original_world.add_system(MovementSystem);
    
    // Get the recorded history
    let history = original_world.get_update_history();
    
    // Verify that system addition was recorded
    assert_eq!(history.len(), 1, "Should have 1 recorded operation (system addition)");
    
    let first_update = &history.updates()[0];
    assert_eq!(first_update.system_diffs().len(), 1, "Should have 1 system diff");
    
    let system_diff = &first_update.system_diffs()[0];
    assert_eq!(system_diff.world_operations().len(), 1, "Should have 1 world operation");
    
    // Check that the operation is an AddSystem operation
    let operation = &system_diff.world_operations()[0];
    match operation {
        rust_ecs::WorldOperation::AddSystem(system_type) => {
            assert!(system_type.contains("MovementSystem"), "Should record MovementSystem addition");
        }
        _ => panic!("Expected AddSystem operation, got {:?}", operation),
    }
    
    // Now test replay: create a fresh world and apply the history
    let mut replay_world = World::new();
    
    // Apply the recorded operations
    for update in history.updates() {
        replay_world.apply_update_diff(update);
    }
    
    // Check how many updates we have so far (should be 0 - replaying doesn't record history)
    let replay_history_before_new_update = replay_world.get_update_history();
    println!("Replay world has {} updates after applying recorded operations", replay_history_before_new_update.len());
    assert_eq!(replay_history_before_new_update.len(), 0, "Replay world should have 0 updates after applying recorded operations (replay doesn't re-record)");
    
    // The replay world should now have the same systems as the original
    // We can verify by calling update() and checking that it records a new update with system diffs
    replay_world.update();
    
    // The replay world should now have 1 update: the new update after replay
    let replay_history = replay_world.get_update_history();
    println!("Replay world has {} updates after calling update()", replay_history.len());
    assert_eq!(replay_history.len(), 1, "Replay world should have 1 update after replay + 1 update call");
    
    // The update should have system diffs from the replayed system
    let update = &replay_history.updates()[0];
    assert!(update.system_diffs().len() >= 1, "Update should have at least 1 system diff from the replayed MovementSystem");
    
    println!("✅ System addition recording and replay test passed");
}

#[test]
fn test_multiple_system_additions_replay() {
    // Test multiple system additions
    let mut original_world = World::new();
    
    // Add multiple systems
    original_world.add_system(MovementSystem);
    original_world.add_system(WaitSystem);
    original_world.add_system(RenderSystem::default());
    
    // Run some updates
    original_world.update();
    original_world.update();
    
    let history = original_world.get_update_history();
    // Should have: 3 system additions + 2 updates = 5 total
    assert_eq!(history.len(), 5, "Should have 5 recorded operations");
    
    // Create a fresh world for replay
    let mut replay_world = World::new();
    
    // Apply all the recorded operations
    for update in history.updates() {
        replay_world.apply_update_diff(update);
    }
    
    // Verify the replay worked by running another update
    replay_world.update();
    
    let replay_history = replay_world.get_update_history();
    // Should have: 1 new update (replay doesn't re-record the original operations)
    assert_eq!(replay_history.len(), 1, "Replay world should have 1 update after replay + 1 update call");
    
    println!("✅ Multiple system additions replay test passed");
}

#[test]
fn test_empty_world_replay() {
    // Test that we can create a fresh world and replay everything from the beginning
    let mut original_world = World::new();
    
    // Add a system
    original_world.add_system(MovementSystem);
    
    // Run some updates
    original_world.update();
    original_world.update();
    
    // Get the complete history
    let complete_history = original_world.get_update_history();
    
    // Create a completely fresh world (simulating the problem statement requirement)
    let mut fresh_world = World::new();
    
    // Replay everything from the beginning
    for update in complete_history.updates() {
        fresh_world.apply_update_diff(update);
    }
    
    // The fresh world should now behave like the original
    fresh_world.update(); // This should work because the system was replayed
    
    let fresh_history = fresh_world.get_update_history();
    // Should have: 1 new update (replay doesn't re-record the original operations)
    assert_eq!(fresh_history.len(), 1, "Fresh world should have 1 update after full replay + 1 update call");
    
    println!("✅ Empty world complete replay test passed");
}