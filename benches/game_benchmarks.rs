use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::{Duration, Instant};
use std::collections::HashMap;

// Simple benchmark structure for game performance
fn benchmark_game_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_performance");
    
    // Set reasonable timeout for game benchmarks
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(20);

    // Benchmark: Game without replay recording
    group.bench_function("game_without_replay", |b| {
        b.iter(|| {
            black_box(run_game_simulation_without_replay(black_box(100)));
        });
    });

    // Benchmark: Game with replay recording
    group.bench_function("game_with_replay", |b| {
        b.iter(|| {
            black_box(run_game_simulation_with_replay(black_box(100)));
        });
    });

    group.finish();
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
                Actor { id: 1, wait_timer: 0, target: Position { x: 6, y: 8 } },
                Actor { id: 2, wait_timer: 0, target: Position { x: 1, y: 1 } },
                Actor { id: 3, wait_timer: 0, target: Position { x: 6, y: 8 } },
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
                // Simple movement simulation
                if actor.target.x == self.home_pos.x && actor.target.y == self.home_pos.y {
                    actor.target = self.work_pos.clone();
                } else {
                    actor.target = self.home_pos.clone();
                }
                actor.wait_timer = 10; // Reset wait timer
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
            let old_target = actor.target.clone();
            
            if actor.wait_timer > 0 {
                actor.wait_timer -= 1;
            } else {
                // Simple movement simulation
                if actor.target.x == self.home_pos.x && actor.target.y == self.home_pos.y {
                    actor.target = self.work_pos.clone();
                } else {
                    actor.target = self.home_pos.clone();
                }
                actor.wait_timer = 10; // Reset wait timer
            }
            
            // Log changes (this simulates the replay recording overhead)
            if old_wait_timer != actor.wait_timer {
                replay_log.record_component_change(actor.id, "WaitTimer", format!("{}", actor.wait_timer));
            }
            if old_target.x != actor.target.x || old_target.y != actor.target.y {
                replay_log.record_component_change(actor.id, "Target", format!("({}, {})", actor.target.x, actor.target.y));
            }
        }
        
        // Record the frame end
        replay_log.record_frame_end(self.update_count);
    }
}

// Simple replay log to simulate the overhead of recording
#[derive(Clone, Debug)]
struct ReplayLog {
    entries: Vec<String>,
    component_changes: HashMap<String, String>,
}

impl ReplayLog {
    fn new() -> Self {
        ReplayLog {
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
        self.entries.push(format!("COMPONENT_CHANGE: {} = {}", key, value));
    }

    fn flush_to_disk(&mut self) {
        // Simulate file I/O overhead
        let serialized = self.entries.join("\n");
        let _bytes = serialized.as_bytes();
        self.entries.clear(); // Simulate flushing
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

criterion_group!(benches, benchmark_game_performance);
criterion_main!(benches);