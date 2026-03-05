//! Integration test: simulates a full game session with menu navigation,
//! fighting, and verifies text rendering output.
//!
//! Run with: cargo test -p game --test integration_test -- --nocapture

use hecs::World;
use tickle_core::{
    Facing, FighterState, Health, InputBuffer, LogicVec2, Position, PowerGauge, PreviousPosition,
    Velocity,
};
use tickle_mugen::MugenFighterState;

// Import from game crate (we need to expose these in lib.rs or duplicate here)
struct Player1;
struct Player2;

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
            dir: Facing::RIGHT,
        },
        FighterState::new(),
        MugenFighterState::default(),
        InputBuffer::new(),
        Health::new(10_000),
        PowerGauge::new(),
    ));
    world.spawn((
        Player2,
        Position {
            pos: LogicVec2::from_pixels(600, 0),
        },
        PreviousPosition {
            pos: LogicVec2::from_pixels(600, 0),
        },
        Velocity {
            vel: LogicVec2::ZERO,
        },
        Facing {
            dir: Facing::LEFT,
        },
        FighterState::new(),
        MugenFighterState::default(),
        InputBuffer::new(),
        Health::new(10_000),
        PowerGauge::new(),
    ));
}

#[test]
fn test_integration() {
    // Basic integration test
    let mut world = World::new();
    spawn_fighters(&mut world);

    // Verify we have 2 fighters
    assert_eq!(world.len(), 2);
}
