use rust_ecs_derive::Diff;

// Grid constants
pub const GRID_SIZE: i32 = 10;
pub const HOME_POS: (i32, i32) = (1, 1);
pub const WORK_POS: (i32, i32) = (6, 8);
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Diff)]
#[allow(dead_code)]
pub enum ActorState {
    #[default]
    MovingToWork,
    MovingToHome,
    WaitingAtWork,
    WaitingAtHome,
}