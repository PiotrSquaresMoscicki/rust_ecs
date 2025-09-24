use rust_ecs::*;

// Define a trait that some components will implement
trait StateMachine {
    fn get_state(&self) -> &str;
    fn update(&mut self);
}

// AI component implementing StateMachine
#[derive(Debug, Clone, Diff)]
struct AIController {
    state: String,
    health: i32,
}

impl StateMachine for AIController {
    fn get_state(&self) -> &str {
        &self.state
    }
    
    fn update(&mut self) {
        if self.health < 50 {
            self.state = "retreating".to_string();
        } else {
            self.state = "attacking".to_string();
        }
    }
}

// Animation component implementing StateMachine
#[derive(Debug, Clone, Diff)]
struct AnimationController {
    current_animation: String,
    frame: i32,
}

impl StateMachine for AnimationController {
    fn get_state(&self) -> &str {
        &self.current_animation
    }
    
    fn update(&mut self) {
        self.frame += 1;
        if self.frame > 10 {
            self.current_animation = "idle".to_string();
            self.frame = 0;
        }
    }
}

// Regular component that does NOT implement StateMachine
#[derive(Debug, Clone, Diff)]
struct Transform {
    x: f32,
    y: f32,
}

fn main() {
    let mut world = World::new();
    let mut world_view = WorldView::<(), ()>::new(&mut world);

    // Register trait implementations
    world_view.register_trait_impl::<AIController, dyn StateMachine>();
    world_view.register_trait_impl::<AnimationController, dyn StateMachine>();

    // Create entities
    let enemy = world_view.create_entity();
    let player = world_view.create_entity();
    let static_object = world_view.create_entity();

    // Add components
    world_view.add_component(enemy, AIController { 
        state: "patrolling".to_string(), 
        health: 100 
    });
    world_view.add_component(player, AnimationController { 
        current_animation: "running".to_string(), 
        frame: 5 
    });
    world_view.add_component(static_object, Transform { x: 10.0, y: 20.0 });

    // Query for all entities with components implementing StateMachine
    let state_machines = world_view.query_components::<(InTrait<dyn StateMachine>,)>();
    println!("Found {} entities with StateMachine components:", state_machines.len());
    
    for (entity, _) in &state_machines {
        println!("  Entity {:?} has a component implementing StateMachine", entity);
    }

    // Query for specific combinations
    let ai_controllers = world_view.query_components::<(In<AIController>, InTrait<dyn StateMachine>)>();
    println!("\nFound {} AI controllers with StateMachine trait:", ai_controllers.len());
    
    for (entity, (ai, _)) in &ai_controllers {
        println!("  Entity {:?}: AI state = '{}', health = {}", entity, ai.state, ai.health);
    }

    // Query for entities with Transform (should not include StateMachine entities)
    let transforms = world_view.query_components::<(In<Transform>,)>();
    println!("\nFound {} entities with Transform:", transforms.len());
    
    for (entity, transform) in &transforms {
        println!("  Entity {:?}: position = ({}, {})", entity, transform.x, transform.y);
    }

    println!("\n✓ Trait-based component querying works as expected!");
    println!("✓ Same API as regular component queries");
    println!("✓ Can be combined with other query types");
}