//! Performance test: simulates 1000+ frames of gameplay and measures timing.
//!
//! Run with: cargo test -p game --release --test perf_test -- --nocapture

use std::time::{Duration, Instant};

use hecs::World;
use tickle_core::{
    Direction, Facing, Health, InputState, LogicVec2, Position, PreviousPosition, StateMachine,
    StateType, Velocity, BUTTON_A,
};

// Constants matching game/src/main.rs
const STAGE_WIDTH: i32 = 80_000;
const GROUND_Y: i32 = 0;
const GRAVITY: i32 = -80;
const MOVE_SPEED: i32 = 400;
const JUMP_VEL: i32 = 1800;
const FRICTION: i32 = 50;

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
        Facing { dir: Facing::RIGHT },
        StateMachine::new(),
        Health::new(10_000),
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
        Facing { dir: Facing::RIGHT },
        StateMachine::new(),
        Health::new(10_000),
    ));
}

fn logic_update(world: &mut World, p1_input: &InputState, p2_input: &InputState) {
    for (_, (pos, prev)) in world.query_mut::<(&Position, &mut PreviousPosition)>() {
        prev.pos = pos.pos;
    }
    for (_, (_, vel, sm)) in world.query_mut::<(&Player1, &mut Velocity, &mut StateMachine)>() {
        apply_input(vel, sm, p1_input);
    }
    for (_, (_, vel, sm)) in world.query_mut::<(&Player2, &mut Velocity, &mut StateMachine)>() {
        apply_input(vel, sm, p2_input);
    }

    for (_, (pos, vel, sm)) in
        world.query_mut::<(&mut Position, &mut Velocity, &mut StateMachine)>()
    {
        if pos.pos.y > GROUND_Y || vel.vel.y > 0 {
            vel.vel.y += GRAVITY;
        }
        if pos.pos.y <= GROUND_Y && sm.current_state() != StateType::Jump {
            if vel.vel.x > 0 {
                vel.vel.x = (vel.vel.x - FRICTION).max(0);
            } else if vel.vel.x < 0 {
                vel.vel.x = (vel.vel.x + FRICTION).min(0);
            }
        }
        pos.pos.x += vel.vel.x;
        pos.pos.y += vel.vel.y;
        if pos.pos.y < GROUND_Y {
            pos.pos.y = GROUND_Y;
            vel.vel.y = 0;
            sm.land();
        }
        pos.pos.x = pos.pos.x.clamp(0, STAGE_WIDTH);
        sm.update();
    }
}

fn apply_input(vel: &mut Velocity, sm: &mut StateMachine, input: &InputState) {
    let was_jumping = sm.current_state() == StateType::Jump;
    sm.try_transition(input);
    match sm.current_state() {
        StateType::WalkForward => vel.vel.x = MOVE_SPEED,
        StateType::WalkBackward => vel.vel.x = -MOVE_SPEED,
        StateType::Jump if !was_jumping => {
            vel.vel.y = JUMP_VEL;
        }
        _ => {}
    }
}

/// Generate a varied input sequence that exercises different states.
fn generate_input_sequence(frame: usize) -> (InputState, InputState) {
    let cycle = frame % 240; // 4-second cycle at 60fps
    let p1 = match cycle {
        0..=29 => InputState::new(0, Direction::Right), // walk right
        30..=59 => InputState::new(0, Direction::Left), // walk left
        60..=64 => InputState::new(0, Direction::Up),   // jump
        65..=89 => InputState::EMPTY,                   // idle (airborne)
        90..=94 => InputState::new(BUTTON_A, Direction::Neutral), // attack
        95..=119 => InputState::EMPTY,                  // idle
        120..=149 => InputState::new(0, Direction::Right), // walk right
        150..=154 => InputState::new(0, Direction::Up), // jump
        155..=179 => InputState::EMPTY,                 // idle
        180..=209 => InputState::new(0, Direction::Down), // crouch
        _ => InputState::EMPTY,                         // idle
    };
    // P2 does the mirror
    let p2 = match cycle {
        0..=29 => InputState::new(0, Direction::Left),
        30..=59 => InputState::new(0, Direction::Right),
        60..=64 => InputState::new(0, Direction::Up),
        65..=89 => InputState::EMPTY,
        90..=94 => InputState::new(BUTTON_A, Direction::Neutral),
        95..=119 => InputState::EMPTY,
        120..=149 => InputState::new(0, Direction::Left),
        150..=154 => InputState::new(0, Direction::Up),
        155..=179 => InputState::EMPTY,
        180..=209 => InputState::new(0, Direction::Down),
        _ => InputState::EMPTY,
    };
    (p1, p2)
}

/// Runs N frames of logic updates and returns per-frame timing stats.
struct PerfStats {
    total: Duration,
    min: Duration,
    max: Duration,
    mean: Duration,
    p99: Duration,
    frame_count: usize,
}

impl std::fmt::Display for PerfStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "frames={}, total={:.2}ms, mean={:.3}us, min={:.3}us, max={:.3}us, p99={:.3}us",
            self.frame_count,
            self.total.as_secs_f64() * 1000.0,
            self.mean.as_secs_f64() * 1_000_000.0,
            self.min.as_secs_f64() * 1_000_000.0,
            self.max.as_secs_f64() * 1_000_000.0,
            self.p99.as_secs_f64() * 1_000_000.0,
        )
    }
}

fn run_perf_test(frame_count: usize) -> PerfStats {
    let mut world = World::new();
    spawn_fighters(&mut world);

    let mut timings = Vec::with_capacity(frame_count);

    for frame in 0..frame_count {
        let (p1_input, p2_input) = generate_input_sequence(frame);
        let start = Instant::now();
        logic_update(&mut world, &p1_input, &p2_input);
        timings.push(start.elapsed());
    }

    timings.sort();
    let total: Duration = timings.iter().sum();
    let min = timings[0];
    let max = *timings.last().unwrap();
    let mean = total / frame_count as u32;
    let p99_idx = (frame_count as f64 * 0.99) as usize;
    let p99 = timings[p99_idx.min(frame_count - 1)];

    PerfStats {
        total,
        min,
        max,
        mean,
        p99,
        frame_count,
    }
}

#[test]
fn test_1000_frames_performance() {
    // Warmup
    let _ = run_perf_test(100);

    let stats = run_perf_test(1000);
    println!("\n=== Performance Test: 1000 frames ===");
    println!("{}", stats);

    // At 60 FPS, each logic frame budget is ~16.67ms.
    // The spec says logic frame time should be < 8ms.
    // With just 2 entities and no rendering, we expect sub-millisecond.
    let budget = Duration::from_millis(8);
    assert!(
        stats.p99 < budget,
        "p99 frame time ({:.3}us) exceeds 8ms budget",
        stats.p99.as_secs_f64() * 1_000_000.0
    );
    assert!(
        stats.max < Duration::from_millis(16),
        "max frame time ({:.3}us) exceeds 16ms hard limit",
        stats.max.as_secs_f64() * 1_000_000.0
    );
}

#[test]
fn test_10000_frames_stability() {
    let stats = run_perf_test(10_000);
    println!("\n=== Stability Test: 10000 frames ===");
    println!("{}", stats);

    // Check that performance doesn't degrade over time (no memory leaks
    // causing slowdown). Compare first 1000 vs last 1000 frames.
    let mut world = World::new();
    spawn_fighters(&mut world);

    let mut early_times = Vec::new();
    let mut late_times = Vec::new();

    for frame in 0..10_000 {
        let (p1, p2) = generate_input_sequence(frame);
        let start = Instant::now();
        logic_update(&mut world, &p1, &p2);
        let elapsed = start.elapsed();

        if frame < 1000 {
            early_times.push(elapsed);
        } else if frame >= 9000 {
            late_times.push(elapsed);
        }
    }

    let early_mean: Duration = early_times.iter().sum::<Duration>() / early_times.len() as u32;
    let late_mean: Duration = late_times.iter().sum::<Duration>() / late_times.len() as u32;

    println!(
        "Early 1000 mean: {:.3}us, Late 1000 mean: {:.3}us",
        early_mean.as_secs_f64() * 1_000_000.0,
        late_mean.as_secs_f64() * 1_000_000.0,
    );

    // Late frames should not be more than 5x slower than early frames.
    // (generous margin for OS scheduling jitter)
    let ratio = late_mean.as_nanos() as f64 / early_mean.as_nanos().max(1) as f64;
    println!("Late/Early ratio: {:.2}x", ratio);
    assert!(
        ratio < 5.0,
        "Performance degradation detected: late frames are {:.2}x slower",
        ratio
    );
}

#[test]
fn test_fixed_timestep_determinism() {
    // Verify that the same input sequence produces identical world state.
    let mut world1 = World::new();
    let mut world2 = World::new();
    spawn_fighters(&mut world1);
    spawn_fighters(&mut world2);

    for frame in 0..600 {
        let (p1, p2) = generate_input_sequence(frame);
        logic_update(&mut world1, &p1, &p2);
        logic_update(&mut world2, &p1, &p2);
    }

    // Compare all positions
    let positions1: Vec<_> = world1
        .query_mut::<&Position>()
        .into_iter()
        .map(|(_, p)| p.pos)
        .collect();
    let positions2: Vec<_> = world2
        .query_mut::<&Position>()
        .into_iter()
        .map(|(_, p)| p.pos)
        .collect();

    assert_eq!(
        positions1, positions2,
        "Determinism violation: positions diverged after 600 frames"
    );
    println!("\n=== Determinism Test ===");
    println!("600 frames with identical inputs -> identical positions (deterministic)");
}

#[test]
fn test_fixed_timestep_accumulator() {
    // Simulate the accumulator pattern manually to verify logic update counts.
    let logic_dt: f64 = 1.0 / 60.0;
    let mut accumulator: f64 = 0.0;
    let mut logic_count = 0u32;

    // 960 render frames at 240Hz = 4 seconds
    for _ in 0..960 {
        accumulator += 1.0 / 240.0;
        while accumulator >= logic_dt {
            logic_count += 1;
            accumulator -= logic_dt;
        }
    }

    println!("\n=== 240Hz Render Accumulator Test ===");
    println!(
        "960 render frames at 240Hz -> {} logic updates",
        logic_count
    );
    assert_eq!(
        logic_count, 240,
        "Expected 240 logic updates for 4s at 60Hz"
    );
}

#[test]
fn test_ecs_query_overhead() {
    // Measure overhead of hecs queries with varying entity counts.
    let entity_counts = [2, 10, 50, 100];

    println!("\n=== ECS Query Overhead ===");
    for &count in &entity_counts {
        let mut world = World::new();
        for i in 0..count {
            let x = (i as i32) * 10000;
            world.spawn((
                Position {
                    pos: LogicVec2::new(x, 0),
                },
                PreviousPosition {
                    pos: LogicVec2::new(x, 0),
                },
                Velocity {
                    vel: LogicVec2::ZERO,
                },
                Facing { dir: Facing::RIGHT },
                StateMachine::new(),
                Health::new(10_000),
            ));
        }

        let _input = InputState::EMPTY;
        // Warmup
        for _ in 0..100 {
            for (_, (pos, prev)) in world.query_mut::<(&Position, &mut PreviousPosition)>() {
                prev.pos = pos.pos;
            }
        }

        let start = Instant::now();
        let iterations = 10_000;
        for _ in 0..iterations {
            for (_, (pos, prev)) in world.query_mut::<(&Position, &mut PreviousPosition)>() {
                prev.pos = pos.pos;
            }
            for (_, (pos, vel, sm)) in
                world.query_mut::<(&mut Position, &mut Velocity, &mut StateMachine)>()
            {
                if pos.pos.y > GROUND_Y || vel.vel.y > 0 {
                    vel.vel.y += GRAVITY;
                }
                pos.pos.x += vel.vel.x;
                pos.pos.y += vel.vel.y;
                if pos.pos.y < GROUND_Y {
                    pos.pos.y = GROUND_Y;
                    vel.vel.y = 0;
                    sm.land();
                }
                sm.update();
            }
        }
        let elapsed = start.elapsed();
        let per_iter = elapsed / iterations;

        println!(
            "  {} entities: {:.3}us/frame ({:.1}us total for {} iters)",
            count,
            per_iter.as_secs_f64() * 1_000_000.0,
            elapsed.as_secs_f64() * 1_000_000.0,
            iterations,
        );
    }
}
