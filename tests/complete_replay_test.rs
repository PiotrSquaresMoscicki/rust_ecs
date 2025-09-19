use rust_ecs::*;
use std::fs;
use std::path::Path;

#[test]
fn test_complete_replay_workflow() {
    // Clean up any existing logs
    let _ = fs::remove_dir_all("test_replay_logs");
    
    // Step 1: Create a world and enable replay logging
    let mut world = World::new();
    
    // Add some entities
    let _entity1 = world.create_entity();
    let _entity2 = world.create_entity();
    
    // Enable logging
    world.enable_replay_logging_simple("test_replay_logs", "test_game", 5).unwrap();
    
    // Simulate some game updates
    for i in 0..5 {
        println!("Test update {}", i + 1);
        world.update();
    }
    
    // Finalize logging
    let session_id = world.replay_session_id().unwrap().to_string();
    world.disable_replay_logging().unwrap();
    
    // Step 2: Verify log file was created
    let log_file = format!("test_replay_logs/test_game_{}.log", session_id);
    assert!(Path::new(&log_file).exists(), "Replay log file should exist");
    
    // Step 3: Parse the replay log
    let replay_history = World::parse_replay_log_file(&log_file).unwrap();
    assert_eq!(replay_history.len(), 5, "Should have 5 recorded updates");
    
    println!("✅ Complete replay workflow test passed!");
    
    // Clean up
    let _ = fs::remove_dir_all("test_replay_logs");
}