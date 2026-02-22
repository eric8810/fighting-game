//! Integration test: simulates a full game session with menu navigation,
//! fighting, and verifies text rendering output.
//!
//! Run with: cargo test -p game --test integration_test -- --nocapture

use hecs::World;
use tickle_core::{
    Direction, Facing, Health, InputState, LogicVec2, Position, PowerGauge, PreviousPosition,
    StateMachine, StateType, Velocity, BUTTON_A,
};

// Import from game crate (we need to expose these in lib.rs or duplicate here)
struct Player1;
struct Player2;

const GROUND_Y: i32 = 0;
const GRAVITY: i32 = -80;
const MOVE_SPEED: i32 = 400;
const JUMP_VEL: i32 = 1800;
const FRICTION: i32 = 50;

fn spawn_fighters(world: &mut World) {
    world.spawn((
        Player1,
        Position {
            pos: LogicVec2::from_pixels(200, 0),
        },
        PreviousPosition {
            pos: LogicVec2::from_pixels(200, 0),
        },
        Velocity {
            vel: LogicVec2::ZERO,
        },
        Facing {
            dir: Direction::Right,
        },
        StateMachine::new(),
        Health::new(10_000),
        PowerGauge::new(3),
    ));
    world.spawn((
        Player2,
        Position {
            pos: LogicVec2::from_pixels(600, 0),
        },
        PreviousPosition {
            pos: LogicVec2::from_pixels(600, 0),
        },
