use crate::Diff;

// Grid constants
pub const GRID_SIZE: i32 = 10;
pub const HOME_POS: (i32, i32) = (1, 1);
pub const WORK_POS: (i32, i32) = (6, 6); // Moved to center to avoid blocking woodcutter paths
pub const WAIT_TICKS: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct Home;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct Work;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct Actor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct Obstacle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct Target {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct WaitTimer {
    pub ticks: u32,
}

/// Navigation component for entities that need sophisticated pathfinding
#[derive(Debug, Clone, PartialEq, Eq, Diff)]
pub struct Navigation {
    pub path: Vec<(i32, i32)>,
    pub current_path_index: usize,
    pub needs_recalculation: bool,
}

impl Navigation {
    pub fn new() -> Self {
        Self {
            path: Vec::new(),
            current_path_index: 0,
            needs_recalculation: true,
        }
    }

    pub fn set_path(&mut self, path: Vec<(i32, i32)>) {
        self.path = path;
        self.current_path_index = 0;
        self.needs_recalculation = false;
    }

    pub fn get_next_position(&self) -> Option<(i32, i32)> {
        if self.current_path_index < self.path.len() {
            Some(self.path[self.current_path_index])
        } else {
            None
        }
    }

    pub fn advance_path(&mut self) {
        if self.current_path_index < self.path.len() {
            self.current_path_index += 1;
        }
    }

    pub fn request_recalculation(&mut self) {
        self.needs_recalculation = true;
    }

    pub fn is_path_complete(&self) -> bool {
        self.current_path_index >= self.path.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Diff)]
#[allow(dead_code)]
pub enum ActorState {
    #[default]
    MovingToWork,
    MovingToHome,
    WaitingAtWork,
    WaitingAtHome,
}

// Woodcutter system components
#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct Tree;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct WoodcutterHut;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct Woodcutter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Diff)]
pub struct CarryingTree;