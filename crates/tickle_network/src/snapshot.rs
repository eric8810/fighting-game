use serde::{Deserialize, Serialize};
use tickle_core::{
    Facing, Health, HitboxManager, InputBuffer, Position, PowerGauge,
    PreviousPosition, StateMachine, Velocity,
};

/// Complete snapshot of a single fighter's state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FighterSnapshot {
    pub position: Position,
    pub previous_position: PreviousPosition,
    pub velocity: Velocity,
    pub facing: Facing,
    pub health: Health,
    pub power_gauge: PowerGauge,
    pub state_machine: StateMachine,
    pub input_buffer: InputBuffer,
    pub hitbox_manager: HitboxManager,
    pub combo_count: u32,
}

/// Game-level state that isn't per-fighter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameManagerSnapshot {
    pub round_timer: u32,
    pub round_number: u32,
    pub rng_state: u64,
    pub frame_number: u32,
}

/// Complete game state snapshot for rollback.
/// Contains everything needed to restore the game to an exact frame.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameSnapshot {
    pub fighters: [FighterSnapshot; 2],
    pub manager: GameManagerSnapshot,
}

/// Extracts a FighterSnapshot from individual components.
pub fn snapshot_fighter(
    position: &Position,
    previous_position: &PreviousPosition,
    velocity: &Velocity,
    facing: &Facing,
    health: &Health,
    power_gauge: &PowerGauge,
    state_machine: &StateMachine,
    input_buffer: &InputBuffer,
    hitbox_manager: &HitboxManager,
    combo_count: u32,
) -> FighterSnapshot {
    FighterSnapshot {
        position: *position,
        previous_position: *previous_position,
        velocity: *velocity,
        facing: *facing,
        health: *health,
        power_gauge: *power_gauge,
        state_machine: state_machine.clone(),
        input_buffer: input_buffer.clone(),
        hitbox_manager: hitbox_manager.clone(),
        combo_count,
    }
}

/// Restores fighter components from a snapshot.
/// Returns the individual components as a tuple.
pub fn restore_fighter(
    snapshot: &FighterSnapshot,
) -> (
    Position,
    PreviousPosition,
    Velocity,
    Facing,
    Health,
    PowerGauge,
    StateMachine,
    InputBuffer,
    HitboxManager,
    u32,
) {
    (
        snapshot.position,
        snapshot.previous_position,
        snapshot.velocity,
        snapshot.facing,
        snapshot.health,
        snapshot.power_gauge,
        snapshot.state_machine.clone(),
        snapshot.input_buffer.clone(),
        snapshot.hitbox_manager.clone(),
        snapshot.combo_count,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use tickle_core::{LogicRect, LogicVec2, Pushbox};

    fn default_fighter_snapshot() -> FighterSnapshot {
        FighterSnapshot {
            position: Position {
                pos: LogicVec2::new(0, 0),
            },
            previous_position: PreviousPosition {
                pos: LogicVec2::new(0, 0),
            },
            velocity: Velocity {
                vel: LogicVec2::new(0, 0),
            },
            facing: Facing { dir: Facing::RIGHT },
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

    fn default_game_snapshot() -> GameSnapshot {
        GameSnapshot {
            fighters: [default_fighter_snapshot(), default_fighter_snapshot()],
            manager: GameManagerSnapshot {
                round_timer: 5400,
                round_number: 1,
                rng_state: 12345,
                frame_number: 0,
            },
        }
    }

    #[test]
    fn snapshot_roundtrip() {
        let snap = default_game_snapshot();
        let cloned = snap.clone();
        assert_eq!(snap, cloned);
    }

    #[test]
    fn snapshot_fighter_roundtrip() {
        let pos = Position {
            pos: LogicVec2::new(5000, 0),
        };
        let prev = PreviousPosition {
            pos: LogicVec2::new(4600, 0),
        };
        let vel = Velocity {
            vel: LogicVec2::new(400, 0),
        };
        let facing = Facing { dir: Facing::RIGHT };
        let health = Health::new(10000);
        let gauge = PowerGauge::new();
        let sm = StateMachine::new();
        let ib = InputBuffer::new();
        let hm = HitboxManager::new(Pushbox {
            rect: LogicRect::new(-1500, -8000, 3000, 8000),
        });

        let snap = snapshot_fighter(&pos, &prev, &vel, &facing, &health, &gauge, &sm, &ib, &hm, 0);
        let (rp, rprev, rv, rf, rh, rg, rsm, rib, rhm, rc) = restore_fighter(&snap);

        assert_eq!(rp, pos);
        assert_eq!(rprev, prev);
        assert_eq!(rv, vel);
        assert_eq!(rf, facing);
        assert_eq!(rh, health);
        assert_eq!(rg, gauge);
        assert_eq!(rsm.current_state(), sm.current_state());
        assert_eq!(rsm.state_frame(), sm.state_frame());
        assert_eq!(rib, ib);
        assert_eq!(rhm, hm);
        assert_eq!(rc, 0);
    }

    #[test]
    fn snapshot_detects_difference() {
        let snap1 = default_game_snapshot();
        let mut snap2 = default_game_snapshot();
        assert_eq!(snap1, snap2);

        snap2.fighters[0].health.take_damage(100);
        assert_ne!(snap1, snap2);
    }
}
