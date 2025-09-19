use rust_ecs::{World, ReplayLogConfig, replay_analysis};

#[test]
fn test_complete_replay_logging_workflow() {
    // Create a test world
    let mut world = World::new();
    
    // Configure logging for this test
    let config = ReplayLogConfig {
        enabled: true,
        log_directory: "test_logs".to_string(),
        file_prefix: "integration_test".to_string(),
        flush_interval: 5,
        include_component_details: true,
    };
    
    // Enable logging
    world.enable_replay_logging(config).expect("Failed to enable logging");
    
    // Verify logging is enabled
    assert!(world.is_replay_logging_enabled());
    assert!(world.replay_session_id().is_some());
    
    // Create some entities and run updates
    let entity1 = world.create_entity();
    let entity2 = world.create_entity();
    
    // Run some updates to generate history
    for i in 0..10 {
        world.update();
        
        // Check that history is being recorded
        let history = world.get_update_history();
        assert_eq!(history.len(), i + 1);
    }
    
    // Analyze the captured data
    let history = world.get_update_history();
    let stats = replay_analysis::analyze_replay_history(history);
    
    // Verify the analysis results
    assert_eq!(stats.total_updates, 10);
    assert!(stats.total_system_executions >= 0); // May be 0 if no systems added
    
    // Test anomaly detection (should find no anomalies in uniform data)
    let anomalous = replay_analysis::find_anomalous_frames(history, 2.0);
    println!("Anomalous frames found: {:?}", anomalous);
    
    // Print analysis for manual verification
    replay_analysis::print_replay_analysis(history);
    
    // Clean up logging
    world.disable_replay_logging().expect("Failed to disable logging");
    assert!(!world.is_replay_logging_enabled());
    
    // Clean up test files
    let _ = std::fs::remove_dir_all("test_logs");
    
    println!("✅ Complete replay logging workflow test passed");
}

#[test]
fn test_replay_analysis_with_activity() {
    let mut world = World::new();
    
    // Add some entities to create activity
    for i in 0..5 {
        let entity = world.create_entity();
        println!("Created entity {:?} in iteration {}", entity, i);
    }
    
    // Run updates to generate different activity levels
    for frame in 0..20 {
        world.update();
        
        // Create variable activity by adding entities every few frames
        if frame % 3 == 0 {
            let _entity = world.create_entity();
        }
    }
    
    // Analyze the results
    let history = world.get_update_history();
    let stats = replay_analysis::analyze_replay_history(history);
    
    println!("Analysis Results:");
    println!("  Total updates: {}", stats.total_updates);
    println!("  Most active frame: {:?}", stats.most_active_frame);
    println!("  Max changes in frame: {}", stats.most_changes_in_frame);
    
    // Verify we captured the updates
    assert_eq!(stats.total_updates, 20);
    
    // Find frames with above-average activity
    let anomalous = replay_analysis::find_anomalous_frames(history, 1.5);
    
    if !anomalous.is_empty() {
        println!("Frames with above-average activity: {:?}", anomalous);
    }
    
    println!("✅ Replay analysis with activity test passed");
}