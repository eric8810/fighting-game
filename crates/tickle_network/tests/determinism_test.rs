use tickle_core::systems::physics::{DEFAULT_FRICTION, DEFAULT_GRAVITY, GROUND_Y};
/// Determinism verification tests for the Tickle Fighting Engine.
///
/// These tests ensure that running the same input sequence twice produces
/// identical game state, which is the fundamental requirement for rollback
/// networking.
use tickle_core::{
    Direction, Facing, Health, HitboxManager, InputBuffer, InputState, LogicRect, LogicVec2,
    Position, PowerGauge, PreviousPosition, Pushbox, StateMachine, Velocity, BUTTON_A,
    STATE_STAND, STATE_WALK_FORWARD, STATE_WALK_BACKWARD, STATE_JUMP_UP,
};
use tickle_network::snapshot::{FighterSnapshot, GameManagerSnapshot, GameSnapshot, MugenState};
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
        mugen_state: MugenState::default(),
    }
}

/// Simulate a single fighter frame with MUGEN state mutations.
/// This extends the regular physics simulation with deterministic changes
/// to MugenState fields (vars, ctrl, anim, move flags) to verify that
/// snapshot save/restore covers all MUGEN-specific state.
fn simulate_mugen_fighter_frame(
    fighter: &mut FighterSnapshot,
    input: InputState,
    rng: &mut DeterministicRng,
    frame: u32,
) {
    // Regular physics simulation
    simulate_fighter_frame(fighter, input, rng);

    let ms = &mut fighter.mugen_state;

    // Deterministic MUGEN state mutations based on frame number and RNG:

    // Cycle state_num through common states every 10 frames
    let state_cycle = [STATE_STAND, STATE_WALK_FORWARD, STATE_WALK_BACKWARD, STATE_JUMP_UP];
    let desired_state = state_cycle[(frame as usize / 10) % state_cycle.len()];
    if fighter.state_machine.current_state() != desired_state {
        // Store prev_state_num like the real controller does
        ms.prev_state_num = fighter.state_machine.current_state();
    }

    // Update vars deterministically: var(0) counts frames, var(1) accumulates RNG
    ms.set_var(0, frame as i32);
    let rng_val = rng.range(0, 1000);
    let prev_var1 = ms.get_var(1);
    ms.set_var(1, prev_var1.wrapping_add(rng_val));

    // var(10) and var(40) test both halves of the split storage (lo/hi)
    ms.set_var(10, (frame as i32) * 3);
    ms.set_var(40, (frame as i32) * 7);

    // Toggle ctrl every 15 frames
    ms.ctrl = (frame / 15).is_multiple_of(2);

    // Advance animation state deterministically
    ms.anim_time += 1;
    if ms.anim_time >= 5 {
        ms.anim_time = 0;
        ms.anim_elem += 1;
        if ms.anim_elem > 8 {
            ms.anim_elem = 1;
            ms.anim_num += 1;
        }
    }

    // Toggle move flags at specific frames
    ms.move_hit = frame % 20 < 5;
    ms.move_contact = frame % 20 < 8;
    ms.move_guarded = frame % 30 < 3;
}

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
        STATE_WALK_FORWARD => {
            fighter.velocity.vel.x = 400 * fighter.facing.dir;
        }
        STATE_WALK_BACKWARD => {
            fighter.velocity.vel.x = -300 * fighter.facing.dir;
        }
        STATE_JUMP_UP => {
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
        if fighter.state_machine.current_state() == STATE_JUMP_UP {
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

/// Simulate a full game frame with MUGEN state mutations for both fighters.
fn simulate_mugen_frame(state: &mut GameSnapshot, inputs: [InputState; 2]) {
    let mut rng = DeterministicRng::new(state.manager.rng_state);
    let frame = state.manager.frame_number;

    simulate_mugen_fighter_frame(&mut state.fighters[0], inputs[0], &mut rng, frame);
    simulate_mugen_fighter_frame(&mut state.fighters[1], inputs[1], &mut rng, frame);

    state.manager.rng_state = rng.state();
    state.manager.frame_number += 1;
    if state.manager.round_timer > 0 {
        state.manager.round_timer -= 1;
    }
}

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

#[test]
fn test_mugen_rollback_determinism() {
    let inputs = generate_input_sequence(777, 60);

    // --- Full run: simulate 60 frames with MUGEN state mutations ---
    let mut full_run = default_game_state();
    // Seed MugenState with non-default values so we verify they survive rollback
    full_run.fighters[0].mugen_state.set_var(5, 999);
    full_run.fighters[0].mugen_state.ctrl = false;
    full_run.fighters[0].mugen_state.anim_num = 200;
    full_run.fighters[1].mugen_state.set_var(30, -42);
    full_run.fighters[1].mugen_state.move_hit = true;

    for frame_inputs in &inputs {
        simulate_mugen_frame(&mut full_run, *frame_inputs);
    }

    // --- Snapshot run: simulate 30 frames, save, continue 30 more ---
    let mut snapshot_run = default_game_state();
    snapshot_run.fighters[0].mugen_state.set_var(5, 999);
    snapshot_run.fighters[0].mugen_state.ctrl = false;
    snapshot_run.fighters[0].mugen_state.anim_num = 200;
    snapshot_run.fighters[1].mugen_state.set_var(30, -42);
    snapshot_run.fighters[1].mugen_state.move_hit = true;

    for frame_inputs in &inputs[..30] {
        simulate_mugen_frame(&mut snapshot_run, *frame_inputs);
    }

    // Save snapshot at frame 30
    let snapshot_at_30 = snapshot_run.clone();

    // Verify snapshot captured non-trivial MUGEN state
    assert_eq!(snapshot_at_30.fighters[0].mugen_state.get_var(0), 29); // frame counter (0-indexed, last frame was 29)
    assert_ne!(snapshot_at_30.fighters[0].mugen_state.get_var(1), 0); // RNG accumulator should be non-zero
    assert_ne!(snapshot_at_30.fighters[0].mugen_state.get_var(10), 0);
    assert_ne!(snapshot_at_30.fighters[1].mugen_state.get_var(40), 0);

    // Continue to frame 60
    for frame_inputs in &inputs[30..] {
        simulate_mugen_frame(&mut snapshot_run, *frame_inputs);
    }

    // --- Rollback run: restore snapshot at 30, re-simulate to 60 ---
    let mut rollback_run = snapshot_at_30;
    for frame_inputs in &inputs[30..] {
        simulate_mugen_frame(&mut rollback_run, *frame_inputs);
    }

    // Core assertion: bit-identical state at frame 60
    assert_eq!(
        full_run, snapshot_run,
        "full run and snapshot run diverged at frame 60"
    );
    assert_eq!(
        full_run, rollback_run,
        "full run and rollback run diverged at frame 60 (rollback failed)"
    );

    // Verify specific MUGEN fields are correct and non-trivial
    for i in 0..2 {
        let ms = &full_run.fighters[i].mugen_state;
        // var(0) should be last frame number (59)
        assert_eq!(ms.get_var(0), 59, "fighter {i} var(0) should track frame");
        // var(1) should be non-zero (accumulated RNG values)
        assert_ne!(ms.get_var(1), 0, "fighter {i} var(1) should have RNG accumulation");
        // var(10) and var(40) verify both lo/hi halves of storage
        assert_eq!(ms.get_var(10), 59 * 3, "fighter {i} var(10) should be frame*3");
        assert_eq!(ms.get_var(40), 59 * 7, "fighter {i} var(40) should be frame*7");
        // ctrl toggles based on frame (59/15 = 3, 3%2 = 1, so ctrl=false)
        assert!(!ms.ctrl, "fighter {i} ctrl should be false at frame 59");
        // move_hit: 59%20 = 19, 19 < 5 is false
        assert!(!ms.move_hit, "fighter {i} move_hit should be false at frame 59");
        // move_contact: 59%20 = 19, 19 < 8 is false
        assert!(!ms.move_contact, "fighter {i} move_contact should be false at frame 59");
        // move_guarded: 59%30 = 29, 29 < 3 is false
        assert!(!ms.move_guarded, "fighter {i} move_guarded should be false at frame 59");
        // anim state should have advanced
        assert!(
            ms.anim_elem > 0 || ms.anim_num > 0,
            "fighter {i} animation should have advanced"
        );
    }
}
