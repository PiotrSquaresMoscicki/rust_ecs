use rand::Rng;
use rust_ecs::{Diff, DiffComponent, In, Out, System, World, WorldView};

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

// System skeletons
pub struct MovementSystem;
impl System for MovementSystem {
    type InComponents = (Actor, Position, Target);
    type OutComponents = (Position,);
    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // TODO: Move actors toward their target, avoiding obstacles
    }
    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}

pub struct WaitSystem;
impl System for WaitSystem {
    type InComponents = (Actor, WaitTimer, Target);
    type OutComponents = (WaitTimer, Target);
    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // TODO: Decrement wait timers, switch targets when done
    }
    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}

pub struct RenderSystem;
impl System for RenderSystem {
    type InComponents = (Position, Home, Work, Actor);
    type OutComponents = ();
    fn initialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
    fn update(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {
        // TODO: Print the 10x10 grid to the terminal
    }
    fn deinitialize(&mut self, _world: &mut WorldView<Self::InComponents, Self::OutComponents>) {}
}
