/// Determinism verification tests for the Tickle Fighting Engine.
///
/// These tests ensure that running the same input sequence twice produces
/// identical game state, which is the fundamental requirement for rollback
/// networking.
use tickle_core::{
    Direction, Facing, Health, HitboxManager, InputBuffer, InputState, LogicRect,
    LogicVec2, Position, PowerGauge, PreviousPosition, Pushbox, StateMachine, StateType, Velocity,
    BUTTON_A,
};
use tickle_core::systems::physics::{DEFAULT_FRICTION, DEFAULT_GRAVITY, GROUND_Y};
use tickle_network::snapshot::{FighterSnapshot, GameManagerSnapshot, GameSnapshot};
use tickle_network::DeterministicRng;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn default_fighter(x: i32, facing: i32) -> FighterSnapshot {
    FighterSnapshot {
        position: Position {
            pos: LogicVec2::new(x, 0),
        },
        previous_position: PreviousPosition {
            pos: LogicVec2::new(x, 0),
        },
        velocity: Velocity {
            vel: LogicVec2::ZERO,
        },
        facing: Facing { dir: facing },
        health: Health::new(10000),
        power_gauge: PowerGauge::new(),
        state_machine: StateMachine::new(),
        input_buffer: InputBuffer::new(),
        hitbox_manager: HitboxManager::new(Pushbox {
            rect: LogicRect::new(-1500, -8000, 3000, 8000),
        }),
        combo_count: 0,
    }
}

// DETERMINISM_TEST_PLACEHOLDER

fn default_game_state() -> GameSnapshot {
    GameSnapshot {
        fighters: [
            default_fighter(-20000, Facing::RIGHT),
            default_fighter(20000, Facing::LEFT),
        ],
        manager: GameManagerSnapshot {
            round_timer: 5400,
            round_number: 1,
            rng_state: 42,
            frame_number: 0,
        },
    }
}

/// Deterministic simulation step: applies physics to a single fighter snapshot.
/// This mirrors what the real game loop does, but operates on snapshot data
/// directly so we can verify determinism without the full ECS.
fn simulate_fighter_frame(
    fighter: &mut FighterSnapshot,
    input: InputState,
    rng: &mut DeterministicRng,
) {
    // Save previous position
    fighter.previous_position.pos = fighter.position.pos;

    // Push input to buffer
    fighter.input_buffer.push(input);

    // State machine transitions based on input
    let _ = fighter.state_machine.try_transition(&input);

    // Apply movement based on state
    match fighter.state_machine.current_state() {
        StateType::WalkForward => {
            fighter.velocity.vel.x = 400 * fighter.facing.dir;
        }
        StateType::WalkBackward => {
            fighter.velocity.vel.x = -300 * fighter.facing.dir;
        }
        StateType::Jump => {
            if fighter.state_machine.state_frame() == 0 {
                fighter.velocity.vel.y = 1800;
            }
        }
        _ => {}
    }

    // Use RNG to add slight variation (simulates game logic RNG usage)
    let _variation = rng.range(-1, 2);

    // Physics: gravity
    if fighter.position.pos.y > GROUND_Y || fighter.velocity.vel.y > 0 {
        fighter.velocity.vel.y += DEFAULT_GRAVITY;
    }

    // Physics: velocity integration
    fighter.position.pos.x += fighter.velocity.vel.x;
    fighter.position.pos.y += fighter.velocity.vel.y;

    // Ground detection
    if fighter.position.pos.y < GROUND_Y {
        fighter.position.pos.y = GROUND_Y;
        fighter.velocity.vel.y = 0;
        if fighter.state_machine.current_state() == StateType::Jump {
            fighter.state_machine.land();
        }
    }

    // Friction on ground
    if fighter.position.pos.y == GROUND_Y && fighter.velocity.vel.x != 0 {
        if fighter.velocity.vel.x > 0 {
            fighter.velocity.vel.x = (fighter.velocity.vel.x - DEFAULT_FRICTION).max(0);
        } else {
            fighter.velocity.vel.x = (fighter.velocity.vel.x + DEFAULT_FRICTION).min(0);
        }
    }

    // State machine update (auto-transitions)
    fighter.state_machine.update();
}

// DETERMINISM_TESTS_PLACEHOLDER

fn simulate_frame(state: &mut GameSnapshot, inputs: [InputState; 2]) {
    let mut rng = DeterministicRng::new(state.manager.rng_state);

    simulate_fighter_frame(&mut state.fighters[0], inputs[0], &mut rng);
    simulate_fighter_frame(&mut state.fighters[1], inputs[1], &mut rng);

    state.manager.rng_state = rng.state();
    state.manager.frame_number += 1;
    if state.manager.round_timer > 0 {
        state.manager.round_timer -= 1;
    }
}

/// Generate a deterministic input sequence using the RNG.
fn generate_input_sequence(seed: u64, length: usize) -> Vec<[InputState; 2]> {
    let mut rng = DeterministicRng::new(seed);
    let directions = [
        Direction::Neutral,
        Direction::Right,
        Direction::Left,
        Direction::Down,
        Direction::Up,
        Direction::DownRight,
        Direction::DownLeft,
    ];

    (0..length)
        .map(|_| {
            let d1 = directions[(rng.next() as usize) % directions.len()];
            let b1 = if rng.next() % 4 == 0 { BUTTON_A } else { 0 };
            let d2 = directions[(rng.next() as usize) % directions.len()];
            let b2 = if rng.next() % 4 == 0 { BUTTON_A } else { 0 };
            [InputState::new(b1, d1), InputState::new(b2, d2)]
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
fn determinism_1000_frames() {
    let inputs = generate_input_sequence(999, 1000);

    // Run 1
    let mut state1 = default_game_state();
    for frame_inputs in &inputs {
        simulate_frame(&mut state1, *frame_inputs);
    }

    // Run 2 (identical)
    let mut state2 = default_game_state();
    for frame_inputs in &inputs {
        simulate_frame(&mut state2, *frame_inputs);
    }

    assert_eq!(state1, state2, "1000-frame simulation diverged");
}

#[test]
fn determinism_snapshot_restore_continues_identically() {
    let inputs = generate_input_sequence(123, 500);

    // Run full 500 frames
    let mut full_run = default_game_state();
    for frame_inputs in &inputs {
        simulate_frame(&mut full_run, *frame_inputs);
    }

    // Run 250 frames, snapshot, then continue 250 more
    let mut partial_run = default_game_state();
    for frame_inputs in &inputs[..250] {
        simulate_frame(&mut partial_run, *frame_inputs);
    }
    let snapshot = partial_run.clone();

    // Restore from snapshot and continue
    let mut restored_run = snapshot;
    for frame_inputs in &inputs[250..] {
        simulate_frame(&mut restored_run, *frame_inputs);
    }

    assert_eq!(
        full_run, restored_run,
        "snapshot-restore produced different result"
    );
}

#[test]
fn determinism_different_inputs_diverge() {
    let inputs1 = generate_input_sequence(100, 100);
    let inputs2 = generate_input_sequence(200, 100);

    let mut state1 = default_game_state();
    for frame_inputs in &inputs1 {
        simulate_frame(&mut state1, *frame_inputs);
    }

    let mut state2 = default_game_state();
    for frame_inputs in &inputs2 {
        simulate_frame(&mut state2, *frame_inputs);
    }

    assert_ne!(
        state1, state2,
        "different inputs should produce different states"
    );
}

#[test]
fn determinism_rng_stays_in_sync() {
    let mut rng1 = DeterministicRng::new(42);
    let mut rng2 = DeterministicRng::new(42);

    // Simulate 1000 frames of RNG usage
    for _ in 0..1000 {
        let a = rng1.next();
        let b = rng2.next();
        assert_eq!(a, b);
        // Simulate occasional range calls
        if a % 3 == 0 {
            assert_eq!(rng1.range(0, 100), rng2.range(0, 100));
        }
    }
}

#[test]
fn determinism_idle_frames_stable() {
    let neutral = InputState::EMPTY;
    let mut state = default_game_state();

    // Run 1000 idle frames
    for _ in 0..1000 {
        simulate_frame(&mut state, [neutral, neutral]);
    }

    // Fighters should still be at starting positions (on ground, no movement)
    assert_eq!(state.fighters[0].position.pos.y, 0);
    assert_eq!(state.fighters[1].position.pos.y, 0);
    assert_eq!(state.fighters[0].velocity.vel, LogicVec2::ZERO);
    assert_eq!(state.fighters[1].velocity.vel, LogicVec2::ZERO);
}
