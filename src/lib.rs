//! A simple ECS (Entity Component System) library for Rust.
//!
//! This library provides basic functionality for managing entities,
//! components, and systems in a game or simulation.

/// A dummy function to demonstrate the library.
/// Returns the sum of two numbers.
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// A placeholder ECS World structure.
#[derive(Debug, Default)]
pub struct World {
    entity_count: u32,
}

impl World {
    /// Creates a new empty world.
    pub fn new() -> Self {
        Self { entity_count: 0 }
    }

    /// Creates a new entity and returns its ID.
    pub fn create_entity(&mut self) -> u32 {
        let id = self.entity_count;
        self.entity_count += 1;
        id
    }

    /// Returns the current number of entities.
    pub fn entity_count(&self) -> u32 {
        self.entity_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_function() {
        assert_eq!(add(2, 3), 5);
        assert_eq!(add(-1, 1), 0);
        assert_eq!(add(0, 0), 0);
    }

    #[test]
    fn test_world_creation() {
        let world = World::new();
        assert_eq!(world.entity_count(), 0);
    }

    #[test]
    fn test_entity_creation() {
        let mut world = World::new();

        let entity1 = world.create_entity();
        assert_eq!(entity1, 0);
        assert_eq!(world.entity_count(), 1);

        let entity2 = world.create_entity();
        assert_eq!(entity2, 1);
        assert_eq!(world.entity_count(), 2);
    }

    #[test]
    fn test_world_default() {
        let world = World::default();
        assert_eq!(world.entity_count(), 0);
    }
}
