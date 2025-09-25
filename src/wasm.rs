use wasm_bindgen::prelude::*;

// Import the `console.log` function from the `console` object.
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// Define a macro to provide `println!` style syntax
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// Entry point for the WebAssembly module
#[wasm_bindgen(start)]
pub fn main() {
    // Set up panic hook for better error messages
    console_error_panic_hook::set_once();
    
    console_log!("Hello World from Rust and WebAssembly!");
    console_log!("Rust ECS Framework is now running in the browser!");
}

// Export a function that can be called from JavaScript
#[wasm_bindgen]
pub fn greet(name: &str) {
    console_log!("Hello, {}! Welcome to Rust ECS in WebAssembly!", name);
}

// Export a simple function that demonstrates the ECS framework
#[wasm_bindgen]
pub fn run_simple_demo() {
    console_log!("Running simple ECS demo in WebAssembly...");
    
    // Create a basic world (simplified version without full ECS complexity for now)
    console_log!("Creating ECS World...");
    let mut world = crate::World::new();
    
    console_log!("Created world with {} entities", world.entity_count());
    
    // Create a simple entity
    let entity = world.create_entity();
    console_log!("Created entity: {:?}", entity);
    
    console_log!("ECS demo completed successfully!");
}