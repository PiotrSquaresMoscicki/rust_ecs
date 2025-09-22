use rust_ecs::ecs::core::{ComponentChange, ComponentOperation, Entity, WorldOperation};
use rust_ecs::ecs::diff::DiffComponentChange;
use rust_ecs::ecs::replay::{AutoReplayLogger, ReplayLogConfig};
use rust_ecs::ecs::system::{SystemUpdateDiff, WorldUpdateDiff};
use std::any::TypeId;
use std::time::Instant;

/// Benchmark the optimized replay system vs original
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Replay System Performance Benchmark ===\n");

    // Create test data
    let test_updates = create_test_data(500); // 500 updates like the original benchmark

    // Test 1: Original configuration
    println!("Testing original configuration...");
    let original_config = ReplayLogConfig::debug_full();
    let original_time = benchmark_replay_logging(&test_updates, original_config)?;

    // Test 2: Optimized performance configuration
    println!("Testing optimized performance configuration...");
    let optimized_config = ReplayLogConfig::optimized_performance();
    let optimized_time = benchmark_replay_logging(&test_updates, optimized_config)?;

    // Print results
    println!("\n=== Results ===");
    println!(
        "Original (full detail): {:.3}ms",
        original_time.as_secs_f64() * 1000.0
    );
    println!(
        "Optimized (minimal):    {:.3}ms",
        optimized_time.as_secs_f64() * 1000.0
    );

    let speedup = original_time.as_secs_f64() / optimized_time.as_secs_f64();
    println!("Speedup: {:.2}x faster", speedup);

    let reduction = (1.0 - (optimized_time.as_secs_f64() / original_time.as_secs_f64())) * 100.0;
    println!(
        "Performance improvement: {:.1}% reduction in overhead",
        reduction
    );

    Ok(())
}

fn create_test_data(num_updates: usize) -> Vec<WorldUpdateDiff> {
    let mut updates = Vec::new();

    for i in 0..num_updates {
        // Create mock system diffs similar to the game benchmark
        let mut system_diffs = Vec::new();

        // Movement system diff (3 entities moving)
        let mut movement_diff = SystemUpdateDiff::new();
        for entity_id in 0..3 {
            let entity = Entity::new(entity_id, 0);

            // Add mock component changes
            movement_diff.add_component_change(ComponentChange {
                entity,
                type_id: TypeId::of::<i32>(), // Mock position type
                operation: ComponentOperation::Modify,
            });

            // Add mock diff changes
            movement_diff.add_diff_change(DiffComponentChange::Modified {
                entity,
                type_name: "Position".to_string(),
                diff_string: format!(
                    "PositionDiff {{ x: Some({}), y: Some({}) }}",
                    (i + entity_id) % 10,
                    (i + entity_id + 1) % 10
                ),
            });
        }
        system_diffs.push(movement_diff);

        // Wait system diff
        let mut wait_diff = SystemUpdateDiff::new();
        // Occasionally add wait timer changes
        if i % 10 == 0 {
            let entity = Entity::new(0, 0);
            wait_diff.add_diff_change(DiffComponentChange::Modified {
                entity,
                type_name: "WaitTimer".to_string(),
                diff_string: "WaitTimerDiff { ticks: Some(5) }".to_string(),
            });
        }
        system_diffs.push(wait_diff);

        // Render system diff (usually no changes)
        let render_diff = SystemUpdateDiff::new();
        system_diffs.push(render_diff);

        updates.push(WorldUpdateDiff::new().with_system_diffs(system_diffs));
    }

    updates
}

fn benchmark_replay_logging(
    updates: &[WorldUpdateDiff],
    config: ReplayLogConfig,
) -> Result<std::time::Duration, Box<dyn std::error::Error>> {
    // Create temporary directory for test logs
    std::fs::create_dir_all(&config.log_directory)?;

    let mut logger = AutoReplayLogger::new(config);
    logger.initialize()?;

    let start = Instant::now();

    for update in updates {
        logger.log_update(update)?;
    }

    logger.finalize()?;

    let duration = start.elapsed();

    // Clean up test files
    let _ = std::fs::remove_dir_all(
        &logger
            .session_id()
            .split('_')
            .next()
            .unwrap_or("replay_logs"),
    );

    Ok(duration)
}
