use criterion::{black_box, criterion_group, Criterion};
use std::time::{Duration, Instant};
use std::collections::HashMap;

fn main() {
    println!("Running game performance benchmarks...");
    run_manual_benchmarks();
}

// Run manual benchmarks if criterion doesn't work
fn run_manual_benchmarks() {
    println!("=== Game Performance Benchmarks ===\n");

    // Benchmark configurations
    let test_iterations = [50, 100, 200];
    let num_runs = 10;

    for &iterations in &test_iterations {
        println!("Testing with {} iterations:", iterations);

        // Benchmark without replay
        let without_replay_times: Vec<Duration> = (0..num_runs)
            .map(|_| run_game_simulation_without_replay(iterations))
            .collect();

        // Benchmark with replay
        let with_replay_times: Vec<Duration> = (0..num_runs)
            .map(|_| run_game_simulation_with_replay(iterations))
            .collect();

        // Calculate statistics
        let avg_without = average_duration(&without_replay_times);
        let avg_with = average_duration(&with_replay_times);
        let overhead = calculate_overhead(avg_without, avg_with);

        println!("  Without replay: {:?} (avg)", avg_without);
        println!("  With replay:    {:?} (avg)", avg_with);
        println!("  Overhead:       {:.2}%", overhead);
        println!();
    }

    // Generate detailed report
    println!("=== Detailed Performance Analysis ===");
    generate_performance_report();
}

fn average_duration(durations: &[Duration]) -> Duration {
    let total_nanos: u128 = durations.iter().map(|d| d.as_nanos()).sum();
    Duration::from_nanos((total_nanos / durations.len() as u128) as u64)
}

fn calculate_overhead(baseline: Duration, with_overhead: Duration) -> f64 {
    let baseline_nanos = baseline.as_nanos() as f64;
    let overhead_nanos = with_overhead.as_nanos() as f64;
    ((overhead_nanos - baseline_nanos) / baseline_nanos) * 100.0
}

fn generate_performance_report() {
    println!("Game Performance Benchmark Results");
    println!("=====================================");
    println!("This benchmark compares the performance of the ECS game simulation");
    println!("with and without replay recording enabled.");
    println!();
    println!("Test Configuration:");
    println!("- Game simulates 3 actors moving between home and work positions");
    println!("- Each iteration represents one game frame/update cycle");
    println!("- Replay recording includes component change tracking and log writing");
    println!();
    
    // Run comprehensive benchmark
    let iterations = 500;
    let runs = 20;
    
    println!("Running comprehensive benchmark ({} iterations, {} runs)...", iterations, runs);
    
    let mut without_replay_times = Vec::new();
    let mut with_replay_times = Vec::new();
    
    for i in 0..runs {
        print!("Run {}/{}... ", i + 1, runs);
        
        let without_time = run_game_simulation_without_replay(iterations);
        let with_time = run_game_simulation_with_replay(iterations);
        
        without_replay_times.push(without_time);
        with_replay_times.push(with_time);
        
        println!("Done");
    }
    
    // Statistical analysis
    let avg_without = average_duration(&without_replay_times);
    let avg_with = average_duration(&with_replay_times);
    let min_without = without_replay_times.iter().min().unwrap();
    let max_without = without_replay_times.iter().max().unwrap();
    let min_with = with_replay_times.iter().min().unwrap();
    let max_with = with_replay_times.iter().max().unwrap();
    
    let overhead_percentage = calculate_overhead(avg_without, avg_with);
    
    println!();
    println!("Results:");
    println!("--------");
    println!("Without Replay Recording:");
    println!("  Average: {:?}", avg_without);
    println!("  Min:     {:?}", min_without);
    println!("  Max:     {:?}", max_without);
    println!();
    println!("With Replay Recording:");
    println!("  Average: {:?}", avg_with);
    println!("  Min:     {:?}", min_with);
    println!("  Max:     {:?}", max_with);
    println!();
    println!("Performance Impact:");
    println!("  Replay overhead: {:.2}%", overhead_percentage);
    println!("  Slowdown factor: {:.2}x", avg_with.as_nanos() as f64 / avg_without.as_nanos() as f64);
    
    // Calculate per-frame overhead
    let per_frame_overhead = Duration::from_nanos(
        ((avg_with.as_nanos() - avg_without.as_nanos()) / iterations as u128) as u64
    );
    println!("  Per-frame overhead: {:?}", per_frame_overhead);
    
    println!();
    println!("Analysis:");
    println!("---------");
    if overhead_percentage < 5.0 {
        println!("✓ Low overhead: Replay recording has minimal performance impact");
    } else if overhead_percentage < 15.0 {
        println!("⚠ Moderate overhead: Replay recording has noticeable but acceptable impact");
    } else {
        println!("⚠ High overhead: Replay recording significantly impacts performance");
    }
    
    println!();
    println!("Recommendations:");
    println!("---------------");
    if overhead_percentage < 10.0 {
        println!("- Replay recording can be enabled in production for debugging");
        println!("- Performance impact is within acceptable limits");
    } else {
        println!("- Consider enabling replay recording only during development/testing");
        println!("- Optimize replay logging for production use");
    }
}

// Simplified game state representation
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
        
        // Simulate movement system
        for actor in &mut self.actors {
            if actor.wait_timer > 0 {
                actor.wait_timer -= 1;
            } else {
                // Move towards target
                let target_x = actor.target.x;
                let target_y = actor.target.y;
                let home_x = self.home_pos.x;
                let home_y = self.home_pos.y;
                let work_x = self.work_pos.x;
                let work_y = self.work_pos.y;
                
                // Simple pathfinding - move one step towards target
                if actor.position.x < target_x {
                    actor.position.x += 1;
                } else if actor.position.x > target_x {
                    actor.position.x -= 1;
                } else if actor.position.y < target_y {
                    actor.position.y += 1;
                } else if actor.position.y > target_y {
                    actor.position.y -= 1;
                }
                
                // Switch target when reached
                if actor.position.x == actor.target.x && actor.position.y == actor.target.y {
                    if actor.target.x == home_x && actor.target.y == home_y {
                        actor.target = Position { x: work_x, y: work_y };
                    } else {
                        actor.target = Position { x: home_x, y: home_y };
                    }
                    actor.wait_timer = 10; // Wait at destination
                }
            }
        }
    }

    fn update_with_logging(&mut self, replay_log: &mut ReplayLog) {
        self.update_count += 1;
        
        // Record the frame start
        replay_log.record_frame_start(self.update_count);
        
        // Simulate movement system with logging
        for actor in &mut self.actors {
            let old_wait_timer = actor.wait_timer;
            let old_position = actor.position.clone();
            let old_target = actor.target.clone();
            
            if actor.wait_timer > 0 {
                actor.wait_timer -= 1;
            } else {
                // Move towards target
                let target_x = actor.target.x;
                let target_y = actor.target.y;
                let home_x = self.home_pos.x;
                let home_y = self.home_pos.y;
                let work_x = self.work_pos.x;
                let work_y = self.work_pos.y;
                
                // Simple pathfinding - move one step towards target
                if actor.position.x < target_x {
                    actor.position.x += 1;
                } else if actor.position.x > target_x {
                    actor.position.x -= 1;
                } else if actor.position.y < target_y {
                    actor.position.y += 1;
                } else if actor.position.y > target_y {
                    actor.position.y -= 1;
                }
                
                // Switch target when reached
                if actor.position.x == actor.target.x && actor.position.y == actor.target.y {
                    if actor.target.x == home_x && actor.target.y == home_y {
                        actor.target = Position { x: work_x, y: work_y };
                    } else {
                        actor.target = Position { x: home_x, y: home_y };
                    }
                    actor.wait_timer = 10; // Wait at destination
                }
            }
            
            // Log changes (this simulates the replay recording overhead)
            if old_wait_timer != actor.wait_timer {
                replay_log.record_component_change(actor.id, "WaitTimer", format!("{}", actor.wait_timer));
            }
            if old_position.x != actor.position.x || old_position.y != actor.position.y {
                replay_log.record_component_change(actor.id, "Position", format!("({}, {})", actor.position.x, actor.position.y));
            }
            if old_target.x != actor.target.x || old_target.y != actor.target.y {
                replay_log.record_component_change(actor.id, "Target", format!("({}, {})", actor.target.x, actor.target.y));
            }
        }
        
        // Record the frame end
        replay_log.record_frame_end(self.update_count);
    }

    fn move_actor_towards_target(&self, actor: &mut Actor) {
        let target_x = actor.target.x;
        let target_y = actor.target.y;
        
        // Simple pathfinding - move one step towards target
        if actor.position.x < target_x {
            actor.position.x += 1;
        } else if actor.position.x > target_x {
            actor.position.x -= 1;
        } else if actor.position.y < target_y {
            actor.position.y += 1;
        } else if actor.position.y > target_y {
            actor.position.y -= 1;
        }
    }
}

// Simple replay log to simulate the overhead of recording
#[derive(Clone, Debug)]
struct ReplayLog {
    entries: Vec<String>,
    component_changes: HashMap<String, String>,
    write_buffer: Vec<u8>,
}

impl ReplayLog {
    fn new() -> Self {
        ReplayLog {
            entries: Vec::new(),
            component_changes: HashMap::new(),
            write_buffer: Vec::with_capacity(1024),
        }
    }

    fn record_frame_start(&mut self, frame: usize) {
        let entry = format!("FRAME_START: {}", frame);
        self.entries.push(entry.clone());
        self.write_buffer.extend_from_slice(entry.as_bytes());
    }

    fn record_frame_end(&mut self, frame: usize) {
        let entry = format!("FRAME_END: {}", frame);
        self.entries.push(entry.clone());
        self.write_buffer.extend_from_slice(entry.as_bytes());
    }

    fn record_component_change(&mut self, entity_id: usize, component: &str, value: String) {
        let key = format!("{}:{}", entity_id, component);
        self.component_changes.insert(key.clone(), value.clone());
        let entry = format!("COMPONENT_CHANGE: {} = {}", key, value);
        self.entries.push(entry.clone());
        self.write_buffer.extend_from_slice(entry.as_bytes());
    }

    fn flush_to_disk(&mut self) {
        // Simulate file I/O overhead by processing the buffer
        let serialized = String::from_utf8_lossy(&self.write_buffer);
        let _line_count = serialized.lines().count();
        
        // Simulate compression/serialization overhead
        let _compressed_size = self.write_buffer.len() / 2; // Simulate compression
        
        // Clear buffers (simulate flushing)
        self.entries.clear();
        self.write_buffer.clear();
    }
}

// Simulate game execution without replay recording
fn run_game_simulation_without_replay(iterations: usize) -> Duration {
    let start = Instant::now();
    
    let mut world = GameWorld::new();
    
    for _i in 0..iterations {
        world.update();
    }
    
    start.elapsed()
}

// Simulate game execution with replay recording
fn run_game_simulation_with_replay(iterations: usize) -> Duration {
    let start = Instant::now();
    
    let mut world = GameWorld::new();
    let mut replay_log = ReplayLog::new();
    
    for i in 0..iterations {
        world.update_with_logging(&mut replay_log);
        
        // Simulate periodic flushing to disk (every 10 frames)
        if i % 10 == 0 {
            replay_log.flush_to_disk();
        }
    }
    
    // Final flush
    replay_log.flush_to_disk();
    
    start.elapsed()
}

// Criterion benchmarks (if needed)
fn _benchmark_game_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_performance");
    group.measurement_time(Duration::from_secs(30));
    group.sample_size(20);

    group.bench_function("game_without_replay", |b| {
        b.iter(|| {
            black_box(run_game_simulation_without_replay(black_box(100)));
        });
    });

    group.bench_function("game_with_replay", |b| {
        b.iter(|| {
            black_box(run_game_simulation_with_replay(black_box(100)));
        });
    });

    group.finish();
}

criterion_group!(benches, _benchmark_game_performance);