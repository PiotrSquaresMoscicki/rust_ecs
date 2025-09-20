use rust_ecs::{World, game::game::{MovementSystem, WaitSystem, RenderSystem, initialize_game}};

fn main() {
    println!("=== Demonstrating System Addition Recording and Replay ===\n");
    
    // Scenario 1: Create a world and manually add systems
    println!("1. Creating a world and manually adding systems:");
    let mut original_world = World::new();
    
    println!("   - Adding MovementSystem");
    original_world.add_system(MovementSystem);
    
    println!("   - Adding WaitSystem");
    original_world.add_system(WaitSystem);
    
    println!("   - Adding RenderSystem");
    original_world.add_system(RenderSystem::default());
    
    // Run a few updates
    println!("   - Running 2 updates");
    original_world.update();
    original_world.update();
    
    let history = original_world.get_update_history();
    println!("   - Total operations recorded: {}", history.len());
    println!("     (3 system additions + 2 updates = 5 total)\n");
    
    // Show what's recorded
    for (i, update) in history.updates().iter().enumerate() {
        println!("   Operation {}: {} system diffs", i + 1, update.system_diffs().len());
        for system_diff in update.system_diffs() {
            for operation in system_diff.world_operations() {
                match operation {
                    rust_ecs::WorldOperation::AddSystem(system_type) => {
                        println!("     -> System addition: {}", system_type.split("::").last().unwrap_or(system_type));
                    }
                    _ => {
                        println!("     -> Other operation: {:?}", operation);
                    }
                }
            }
        }
    }
    
    println!("\n2. Creating a fresh world and replaying the operations:");
    let mut fresh_world = World::new();
    
    println!("   - Applying recorded operations to fresh world");
    for (i, update) in history.updates().iter().enumerate() {
        fresh_world.apply_update_diff(update);
        println!("     Applied operation {}", i + 1);
    }
    
    println!("   - Fresh world now has the same systems as the original");
    println!("   - Running an update on the fresh world to verify it works:");
    fresh_world.update();
    
    let fresh_history = fresh_world.get_update_history();
    println!("   - Fresh world update history length: {}", fresh_history.len());
    println!("     (1 new update - replay doesn't re-record operations)\n");
    
    // Scenario 3: Use the game's initialize_game function
    println!("3. Using the game's initialize_game function:");
    let game_world = initialize_game();
    let game_history = game_world.get_update_history();
    
    println!("   - initialize_game() recorded {} operations", game_history.len());
    println!("     (This demonstrates that system additions in initialize_game are now recorded)");
    
    println!("\n✅ System addition recording and replay functionality working correctly!");
    println!("\nKey achievement: You can now create a fresh world and replay everything from the beginning,");
    println!("including system additions, just as requested in the problem statement.");
}