use serde::{Deserialize, Serialize};
use tickle_core::{
    Facing, Health, HitboxManager, InputBuffer, Position, PowerGauge, PreviousPosition,
    StateMachine, Velocity,
};

/// MUGEN-specific fighter state required for rollback determinism.
///
/// vars is stored as two [i32; 30] halves because the serde impl only supports
/// fixed-size arrays up to 32 elements. vars_lo covers var(0..29) and
/// vars_hi covers var(30..59).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MugenState {
    /// Integer variables var(0..29)
    pub vars_lo: [i32; 30],
    /// Integer variables var(30..59)
    pub vars_hi: [i32; 30],
    /// Control flag - whether player input affects state
    pub ctrl: bool,
    /// Current animation number
    pub anim_num: i32,
    /// Current animation element (1-based frame index)
    pub anim_elem: i32,
    /// Ticks into current animation element
    pub anim_time: i32,
    /// Whether current attack connected
    pub move_hit: bool,
    /// Whether current attack made contact (hit or guarded)
    pub move_contact: bool,
    /// Whether current attack was guarded
    pub move_guarded: bool,
    /// Previous state number
    pub prev_state_num: i32,
    /// Remaining hitstun frames
    pub hitstun_remaining: i32,
    /// Remaining blockstun frames
    pub blockstun_remaining: i32,
    /// Remaining ticks for active NotHitBy window (0 = not active)
    pub not_hit_by_ticks: i32,
    /// NotHitBy attribute filter string (empty = not active)
    pub not_hit_by_attr: String,
    /// Remaining ticks for active HitBy window (0 = not active)
    pub hit_by_ticks: i32,
    /// HitBy attribute filter string (empty = not active)
    pub hit_by_attr: String,
}

impl MugenState {
    /// Read a MUGEN integer variable by index (0..59).
    ///
    /// Returns 0 for out-of-range indices.
    pub fn get_var(&self, index: usize) -> i32 {
        if index < 30 {
            self.vars_lo[index]
        } else if index < 60 {
            self.vars_hi[index - 30]
        } else {
            0
        }
    }

    /// Write a MUGEN integer variable by index (0..59).
    ///
    /// Out-of-range indices are silently ignored.
    pub fn set_var(&mut self, index: usize, value: i32) {
        if index < 30 {
            self.vars_lo[index] = value;
        } else if index < 60 {
            self.vars_hi[index - 30] = value;
        }
    }
}

impl Default for MugenState {
    fn default() -> Self {
        Self {
            vars_lo: [0i32; 30],
            vars_hi: [0i32; 30],
            ctrl: true,
            anim_num: 0,
            anim_elem: 0,
            anim_time: 0,
            move_hit: false,
            move_contact: false,
            move_guarded: false,
            prev_state_num: 0,
            hitstun_remaining: 0,
            blockstun_remaining: 0,
            not_hit_by_ticks: 0,
            not_hit_by_attr: String::new(),
            hit_by_ticks: 0,
            hit_by_attr: String::new(),
        }
    }
}

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
    /// MUGEN-specific state for rollback compatibility
    pub mugen_state: MugenState,
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
#[allow(clippy::too_many_arguments)]
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
    mugen_state: MugenState,
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
        mugen_state,
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
    MugenState,
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
        snapshot.mugen_state.clone(),
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
            mugen_state: MugenState::default(),
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

        let snap = snapshot_fighter(
            &pos, &prev, &vel, &facing, &health, &gauge, &sm, &ib, &hm, 0,
            MugenState::default(),
        );
        let (rp, rprev, rv, rf, rh, rg, rsm, rib, rhm, rc, rms) = restore_fighter(&snap);

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
        assert_eq!(rms, MugenState::default());
    }

    #[test]
    fn snapshot_detects_difference() {
        let snap1 = default_game_snapshot();
        let mut snap2 = default_game_snapshot();
        assert_eq!(snap1, snap2);

        snap2.fighters[0].health.take_damage(100);
        assert_ne!(snap1, snap2);
    }

    #[test]
    fn test_mugen_state_default() {
        let ms = MugenState::default();
        assert_eq!(ms.vars_lo, [0i32; 30]);
        assert_eq!(ms.vars_hi, [0i32; 30]);
        assert!(ms.ctrl);
        assert!(!ms.move_hit);
    }

    #[test]
    fn test_mugen_state_in_snapshot() {
        let mut snap = default_fighter_snapshot();
        snap.mugen_state.set_var(0, 42);
        snap.mugen_state.ctrl = false;

        let cloned = snap.clone();
        assert_eq!(cloned.mugen_state.get_var(0), 42);
        assert!(!cloned.mugen_state.ctrl);
    }

    #[test]
    fn test_mugen_state_var_accessors() {
        let mut ms = MugenState::default();
        ms.set_var(0, 10);
        ms.set_var(29, 20);
        ms.set_var(30, 30);
        ms.set_var(59, 40);
        assert_eq!(ms.get_var(0), 10);
        assert_eq!(ms.get_var(29), 20);
        assert_eq!(ms.get_var(30), 30);
        assert_eq!(ms.get_var(59), 40);
        // Out-of-range returns 0
        assert_eq!(ms.get_var(60), 0);
    }
}
