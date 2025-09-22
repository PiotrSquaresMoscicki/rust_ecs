use crate::ecs::diff::{Diff, DiffComponent};
use crate::ecs::impl_diff;

// Grid constants
pub const GRID_SIZE: i32 = 10;
pub const HOME_POS: (i32, i32) = (1, 1);
pub const WORK_POS: (i32, i32) = (6, 8);
pub const WAIT_TICKS: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

// Use manual implementation for Position
impl_diff!(Position { x: i32, y: i32 });

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Home;

// Manual implementation for unit structs
impl Diff for Home {
    type Diff = ();

    fn diff(&self, _other: &Self) -> Option<Self::Diff> {
        None // Unit structs are always the same
    }

    fn apply_diff(&mut self, _diff: &Self::Diff) {
        // Nothing to apply for unit structs
    }
}

impl DiffComponent for Home {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Work;

impl Diff for Work {
    type Diff = ();

    fn diff(&self, _other: &Self) -> Option<Self::Diff> {
        None // Unit structs are always the same
    }

    fn apply_diff(&mut self, _diff: &Self::Diff) {
        // Nothing to apply for unit structs
    }
}

impl DiffComponent for Work {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Actor;

impl Diff for Actor {
    type Diff = ();

    fn diff(&self, _other: &Self) -> Option<Self::Diff> {
        None // Unit structs are always the same
    }

    fn apply_diff(&mut self, _diff: &Self::Diff) {
        // Nothing to apply for unit structs
    }
}

impl DiffComponent for Actor {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Obstacle;

impl Diff for Obstacle {
    type Diff = ();

    fn diff(&self, _other: &Self) -> Option<Self::Diff> {
        None // Unit structs are always the same
    }

    fn apply_diff(&mut self, _diff: &Self::Diff) {
        // Nothing to apply for unit structs
    }
}

impl DiffComponent for Obstacle {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Target {
    pub x: i32,
    pub y: i32,
}

impl_diff!(Target { x: i32, y: i32 });

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaitTimer {
    pub ticks: u32,
}

impl_diff!(WaitTimer { ticks: u32 });

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum ActorState {
    #[default]
    MovingToWork,
    MovingToHome,
    WaitingAtWork,
    WaitingAtHome,
}

impl Diff for ActorState {
    type Diff = ActorState;

    fn diff(&self, other: &Self) -> Option<Self::Diff> {
        if self != other {
            Some(*other)
        } else {
            None
        }
    }

    fn apply_diff(&mut self, diff: &Self::Diff) {
        *self = *diff;
    }
}

impl DiffComponent for ActorState {}
