pub mod components;
pub mod utils;
pub mod movement_system;
pub mod wait_system;
pub mod render_system;
#[allow(clippy::module_inception)]
pub mod game;

// Re-export only what's needed
pub use components::*;
pub use movement_system::MovementSystem;
pub use wait_system::WaitSystem;
pub use render_system::RenderSystem;
pub use game::{initialize_game, run_game, run_game_replay};
