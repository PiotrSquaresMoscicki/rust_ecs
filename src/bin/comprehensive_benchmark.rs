use std::collections::HashMap;
use std::time::Instant;

/// Comprehensive benchmark comparing game performance across three scenarios:
/// 1. No recording
/// 2. Text-based replay recording  
/// 3. Binary-based replay recording
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Comprehensive Game Performance Benchmark ===\n");
    println!("Testing three scenarios:");
    println!("1. Game without any replay recording");
    println!("2. Game with text-based replay recording");
    println!("3. Game with binary-based replay recording (NEW)\n");

    // Test configuration
    let iterations = 500;
    let num_runs = 20;

    println!("Configuration:");
    println!("- {} game iterations per run", iterations);
    println!("- {} runs for statistical accuracy", num_runs);
    println!("- 3 actors moving in simulation");
    println!("- Movement, wait, and render systems\n");

    // Run benchmarks
    let no_recording_times = run_benchmark_no_recording(iterations, num_runs);
    let text_recording_times = run_benchmark_text_recording(iterations, num_runs);
    let binary_recording_times = run_benchmark_binary_recording(iterations, num_runs);

    // Calculate statistics
    let stats_no_recording = calculate_stats(&no_recording_times);
    let stats_text_recording = calculate_stats(&text_recording_times);
    let stats_binary_recording = calculate_stats(&binary_recording_times);

    // Print results
    print_benchmark_results(
        &stats_no_recording,
        &stats_text_recording,
        &stats_binary_recording,
        iterations,
    );

    Ok(())
}

#[derive(Debug)]
struct BenchmarkStats {
    avg_duration: std::time::Duration,
    min_duration: std::time::Duration,
    max_duration: std::time::Duration,
    median_duration: std::time::Duration,
}

fn run_benchmark_no_recording(iterations: usize, num_runs: usize) -> Vec<std::time::Duration> {
    println!("Running benchmark 1/3: No replay recording...");
    (0..num_runs)
        .map(|i| {
            print!("  Run {}/{}\r", i + 1, num_runs);
            run_game_simulation_no_recording(iterations)
        })
        .collect()
}

fn run_benchmark_text_recording(iterations: usize, num_runs: usize) -> Vec<std::time::Duration> {
    println!("\nRunning benchmark 2/3: Text-based replay recording...");
    (0..num_runs)
        .map(|i| {
            print!("  Run {}/{}\r", i + 1, num_runs);
            run_game_simulation_text_recording(iterations)
        })
        .collect()
}

fn run_benchmark_binary_recording(iterations: usize, num_runs: usize) -> Vec<std::time::Duration> {
    println!("\nRunning benchmark 3/3: Binary-based replay recording...");
    (0..num_runs)
        .map(|i| {
            print!("  Run {}/{}\r", i + 1, num_runs);
            run_game_simulation_binary_recording(iterations)
        })
        .collect()
}

fn calculate_stats(durations: &[std::time::Duration]) -> BenchmarkStats {
    let total_nanos: u128 = durations.iter().map(|d| d.as_nanos()).sum();
    let avg_duration =
        std::time::Duration::from_nanos((total_nanos / durations.len() as u128) as u64);

    let min_duration = *durations.iter().min().unwrap();
    let max_duration = *durations.iter().max().unwrap();

    let mut sorted = durations.to_vec();
    sorted.sort();
    let median_duration = sorted[sorted.len() / 2];

    BenchmarkStats {
        avg_duration,
        min_duration,
        max_duration,
        median_duration,
    }
}

fn print_benchmark_results(
    no_recording: &BenchmarkStats,
    text_recording: &BenchmarkStats,
    binary_recording: &BenchmarkStats,
    iterations: usize,
) {
    println!("\n=== BENCHMARK RESULTS ===\n");

    // Performance Summary Table
    println!("Performance Summary ({} iterations):", iterations);
    println!("┌─────────────────────────┬─────────────┬─────────────┬─────────────┬─────────────┐");
    println!("│ Scenario                │ Average     │ Minimum     │ Maximum     │ Median      │");
    println!("├─────────────────────────┼─────────────┼─────────────┼─────────────┼─────────────┤");
    println!(
        "│ No Recording            │ {:>10.3}µs │ {:>10.3}µs │ {:>10.3}µs │ {:>10.3}µs │",
        no_recording.avg_duration.as_nanos() as f64 / 1000.0,
        no_recording.min_duration.as_nanos() as f64 / 1000.0,
        no_recording.max_duration.as_nanos() as f64 / 1000.0,
        no_recording.median_duration.as_nanos() as f64 / 1000.0
    );
    println!(
        "│ Text Recording          │ {:>10.3}µs │ {:>10.3}µs │ {:>10.3}µs │ {:>10.3}µs │",
        text_recording.avg_duration.as_nanos() as f64 / 1000.0,
        text_recording.min_duration.as_nanos() as f64 / 1000.0,
        text_recording.max_duration.as_nanos() as f64 / 1000.0,
        text_recording.median_duration.as_nanos() as f64 / 1000.0
    );
    println!(
        "│ Binary Recording        │ {:>10.3}µs │ {:>10.3}µs │ {:>10.3}µs │ {:>10.3}µs │",
        binary_recording.avg_duration.as_nanos() as f64 / 1000.0,
        binary_recording.min_duration.as_nanos() as f64 / 1000.0,
        binary_recording.max_duration.as_nanos() as f64 / 1000.0,
        binary_recording.median_duration.as_nanos() as f64 / 1000.0
    );
    println!(
        "└─────────────────────────┴─────────────┴─────────────┴─────────────┴─────────────┘\n"
    );

    // Overhead Analysis
    let text_overhead_percent =
        calculate_overhead_percent(no_recording.avg_duration, text_recording.avg_duration);
    let binary_overhead_percent =
        calculate_overhead_percent(no_recording.avg_duration, binary_recording.avg_duration);
    let binary_vs_text_improvement =
        calculate_improvement_percent(text_recording.avg_duration, binary_recording.avg_duration);

    println!("Overhead Analysis:");
    println!("┌─────────────────────────┬─────────────┬─────────────┬─────────────┐");
    println!("│ Comparison              │ Overhead %  │ Slowdown    │ Per Frame   │");
    println!("├─────────────────────────┼─────────────┼─────────────┼─────────────┤");
    println!(
        "│ Text vs No Recording    │ {:>10.1}% │ {:>10.1}x │ {:>10.1}µs │",
        text_overhead_percent,
        text_recording.avg_duration.as_nanos() as f64 / no_recording.avg_duration.as_nanos() as f64,
        (text_recording.avg_duration.as_nanos() - no_recording.avg_duration.as_nanos()) as f64
            / (iterations as f64 * 1000.0)
    );
    println!(
        "│ Binary vs No Recording  │ {:>10.1}% │ {:>10.1}x │ {:>10.1}µs │",
        binary_overhead_percent,
        binary_recording.avg_duration.as_nanos() as f64
            / no_recording.avg_duration.as_nanos() as f64,
        (binary_recording.avg_duration.as_nanos() - no_recording.avg_duration.as_nanos()) as f64
            / (iterations as f64 * 1000.0)
    );
    println!(
        "│ Binary vs Text (impr.)  │ {:>10.1}% │ {:>10.1}x │ {:>10.1}µs │",
        -binary_vs_text_improvement,
        text_recording.avg_duration.as_nanos() as f64
            / binary_recording.avg_duration.as_nanos() as f64,
        (text_recording.avg_duration.as_nanos() - binary_recording.avg_duration.as_nanos()) as f64
            / (iterations as f64 * 1000.0)
    );
    println!("└─────────────────────────┴─────────────┴─────────────┴─────────────┘\n");

    // Performance Recommendations
    println!("Performance Analysis:");
    println!("━━━━━━━━━━━━━━━━━━━━━");

    if text_overhead_percent < 10.0 {
        println!("✓ Text Recording: Low overhead, suitable for production debugging");
    } else if text_overhead_percent < 50.0 {
        println!("⚠ Text Recording: Moderate overhead, consider for development only");
    } else {
        println!("⚠ Text Recording: High overhead, development/testing only");
    }

    if binary_overhead_percent < 5.0 {
        println!("✓ Binary Recording: Minimal overhead, excellent for production");
    } else if binary_overhead_percent < 25.0 {
        println!("✓ Binary Recording: Low overhead, good for production with monitoring");
    } else {
        println!("⚠ Binary Recording: Moderate overhead, selective use recommended");
    }

    println!("\nKey Findings:");
    println!(
        "• Binary recording is {:.1}x faster than text recording",
        text_recording.avg_duration.as_nanos() as f64
            / binary_recording.avg_duration.as_nanos() as f64
    );
    println!(
        "• Binary format reduces replay overhead by {:.1}% vs text format",
        binary_vs_text_improvement
    );

    if binary_overhead_percent < 10.0 {
        println!("• Binary recording achieves production-ready performance");
    }

    println!("\nRecommendations:");
    if binary_overhead_percent < text_overhead_percent / 2.0 {
        println!("🚀 Use binary recording for production - significant performance advantage");
    }
    println!("📊 Use text recording for development debugging - human readable logs");
    println!("⚡ Disable recording for performance-critical code paths");
}

fn calculate_overhead_percent(
    baseline: std::time::Duration,
    with_overhead: std::time::Duration,
) -> f64 {
    let baseline_nanos = baseline.as_nanos() as f64;
    let overhead_nanos = with_overhead.as_nanos() as f64;
    ((overhead_nanos - baseline_nanos) / baseline_nanos) * 100.0
}

fn calculate_improvement_percent(before: std::time::Duration, after: std::time::Duration) -> f64 {
    let before_nanos = before.as_nanos() as f64;
    let after_nanos = after.as_nanos() as f64;
    ((before_nanos - after_nanos) / before_nanos) * 100.0
}

// Game simulation implementations
fn run_game_simulation_no_recording(iterations: usize) -> std::time::Duration {
    let start = Instant::now();
    let mut world = GameWorld::new();

    for _i in 0..iterations {
        world.update();
    }

    start.elapsed()
}

fn run_game_simulation_text_recording(iterations: usize) -> std::time::Duration {
    let start = Instant::now();
    let mut world = GameWorld::new();
    let mut text_log = TextReplayLog::new();

    for i in 0..iterations {
        world.update_with_text_logging(&mut text_log);

        // Simulate periodic flushing (every 10 frames)
        if i % 10 == 0 {
            text_log.flush_to_disk();
        }
    }

    text_log.flush_to_disk();
    start.elapsed()
}

fn run_game_simulation_binary_recording(iterations: usize) -> std::time::Duration {
    let start = Instant::now();
    let mut world = GameWorld::new();
    let mut binary_log = BinaryReplayLog::new();

    for i in 0..iterations {
        world.update_with_binary_logging(&mut binary_log);

        // Simulate periodic flushing (every 10 frames)
        if i % 10 == 0 {
            binary_log.flush_to_disk();
        }
    }

    binary_log.flush_to_disk();
    start.elapsed()
}

// Game World Implementation
#[derive(Clone, Debug)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Clone, Debug)]
struct Actor {
    id: usize,
    wait_timer: u32,
    target: Position,
    position: Position,
}

#[derive(Clone, Debug)]
struct GameWorld {
    actors: Vec<Actor>,
    home_pos: Position,
    work_pos: Position,
    update_count: usize,
}

impl GameWorld {
    fn new() -> Self {
        GameWorld {
            actors: vec![
                Actor {
                    id: 1,
                    wait_timer: 0,
                    target: Position { x: 6, y: 8 },
                    position: Position { x: 2, y: 2 },
                },
                Actor {
                    id: 2,
                    wait_timer: 5,
                    target: Position { x: 1, y: 1 },
                    position: Position { x: 4, y: 6 },
                },
                Actor {
                    id: 3,
                    wait_timer: 3,
                    target: Position { x: 6, y: 8 },
                    position: Position { x: 3, y: 3 },
                },
            ],
            home_pos: Position { x: 1, y: 1 },
            work_pos: Position { x: 6, y: 8 },
            update_count: 0,
        }
    }

    fn update(&mut self) {
        self.update_count += 1;

        for actor in &mut self.actors {
            GameWorld::update_actor(&self.home_pos, &self.work_pos, actor);
        }
    }

    fn update_with_text_logging(&mut self, log: &mut TextReplayLog) {
        self.update_count += 1;
        log.record_frame_start(self.update_count);

        for actor in &mut self.actors {
            let old_wait_timer = actor.wait_timer;
            let old_position = actor.position.clone();
            let old_target = actor.target.clone();

            GameWorld::update_actor(&self.home_pos, &self.work_pos, actor);

            // Log changes
            if old_wait_timer != actor.wait_timer {
                log.record_component_change(actor.id, "WaitTimer", format!("{}", actor.wait_timer));
            }
            if old_position.x != actor.position.x || old_position.y != actor.position.y {
                log.record_component_change(
                    actor.id,
                    "Position",
                    format!("({}, {})", actor.position.x, actor.position.y),
                );
            }
            if old_target.x != actor.target.x || old_target.y != actor.target.y {
                log.record_component_change(
                    actor.id,
                    "Target",
                    format!("({}, {})", actor.target.x, actor.target.y),
                );
            }
        }

        log.record_frame_end(self.update_count);
    }

    fn update_with_binary_logging(&mut self, log: &mut BinaryReplayLog) {
        self.update_count += 1;
        log.record_frame_start(self.update_count);

        for actor in &mut self.actors {
            let old_wait_timer = actor.wait_timer;
            let old_position = actor.position.clone();
            let old_target = actor.target.clone();

            GameWorld::update_actor(&self.home_pos, &self.work_pos, actor);

            // Log changes in binary format
            if old_wait_timer != actor.wait_timer {
                log.record_component_change_binary(actor.id, "WaitTimer", &actor.wait_timer);
            }
            if old_position.x != actor.position.x || old_position.y != actor.position.y {
                log.record_component_change_binary(
                    actor.id,
                    "Position",
                    &(actor.position.x, actor.position.y),
                );
            }
            if old_target.x != actor.target.x || old_target.y != actor.target.y {
                log.record_component_change_binary(
                    actor.id,
                    "Target",
                    &(actor.target.x, actor.target.y),
                );
            }
        }

        log.record_frame_end(self.update_count);
    }

    fn update_actor(home_pos: &Position, work_pos: &Position, actor: &mut Actor) {
        if actor.wait_timer > 0 {
            actor.wait_timer -= 1;
        } else {
            // Move towards target
            if actor.position.x < actor.target.x {
                actor.position.x += 1;
            } else if actor.position.x > actor.target.x {
                actor.position.x -= 1;
            } else if actor.position.y < actor.target.y {
                actor.position.y += 1;
            } else if actor.position.y > actor.target.y {
                actor.position.y -= 1;
            }

            // Switch target when reached
            if actor.position.x == actor.target.x && actor.position.y == actor.target.y {
                if actor.target.x == home_pos.x && target.y == home_pos.y {
                    actor.target = work_pos.clone();
                } else {
                    actor.target = home_pos.clone();
                }
                actor.wait_timer = 10;
            }
        }
    }
}

// Text Replay Log Implementation
#[derive(Clone, Debug)]
struct TextReplayLog {
    entries: Vec<String>,
    component_changes: HashMap<String, String>,
}

impl TextReplayLog {
    fn new() -> Self {
        TextReplayLog {
            entries: Vec::new(),
            component_changes: HashMap::new(),
        }
    }

    fn record_frame_start(&mut self, frame: usize) {
        self.entries.push(format!("FRAME_START: {}", frame));
    }

    fn record_frame_end(&mut self, frame: usize) {
        self.entries.push(format!("FRAME_END: {}", frame));
    }

    fn record_component_change(&mut self, entity_id: usize, component: &str, value: String) {
        let key = format!("{}:{}", entity_id, component);
        self.component_changes.insert(key.clone(), value.clone());
        self.entries
            .push(format!("COMPONENT_CHANGE: {} = {}", key, value));
    }

    fn flush_to_disk(&mut self) {
        // Simulate text serialization overhead
        let serialized = self.entries.join("\n");
        let _bytes = serialized.as_bytes();
        self.entries.clear();
    }
}

// Binary Replay Log Implementation
#[derive(Clone, Debug)]
struct BinaryReplayLog {
    binary_entries: Vec<Vec<u8>>,
    component_data: HashMap<String, Vec<u8>>,
}

impl BinaryReplayLog {
    fn new() -> Self {
        BinaryReplayLog {
            binary_entries: Vec::new(),
            component_data: HashMap::new(),
        }
    }

    fn record_frame_start(&mut self, frame: usize) {
        // Simulate binary encoding of frame start
        let data = frame.to_le_bytes().to_vec();
        self.binary_entries.push(data);
    }

    fn record_frame_end(&mut self, frame: usize) {
        // Simulate binary encoding of frame end
        let data = frame.to_le_bytes().to_vec();
        self.binary_entries.push(data);
    }

    fn record_component_change_binary<T>(&mut self, entity_id: usize, component: &str, value: &T)
    where
        T: serde::Serialize,
    {
        let key = format!("{}:{}", entity_id, component);

        // Simulate binary serialization (more efficient than text)
        if let Ok(binary_data) = bincode::serialize(value) {
            self.component_data.insert(key, binary_data.clone());
            self.binary_entries.push(binary_data);
        }
    }

    fn flush_to_disk(&mut self) {
        // Simulate binary data writing (more efficient than text)
        let mut total_bytes = 0;
        for entry in &self.binary_entries {
            total_bytes += entry.len();
        }
        // Simulate writing total_bytes to disk
        self.binary_entries.clear();
    }
}
