use crate::air::Air;
use crate::cns::{Cns, Controller, Physics, StateController, StateTypeValue};
use crate::trigger::TriggerContext;
use tickle_core::components::{
    Facing, FighterState, Health, HitType, Hitbox, Position, PowerGauge, Velocity,
};
use tickle_core::math::{LogicRect, LogicVec2};
use tickle_core::state_constants::*;
use tickle_core::systems::physics::GROUND_Y;

/// A sound event emitted by the PlaySnd controller.
/// The game loop reads these from MugenFighterState and dispatches them
/// to the audio system (tickle_audio) for actual playback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MugenSoundEvent {
    /// Sound group number (matches SND file group).
    pub group: i32,
    /// Sound index within the group (matches SND file sound number).
    pub sound: i32,
}

/// Active HitDef data set by a HitDef controller during an attack state.
/// Stores the attack properties that will be applied when the attack hitbox
/// connects with the opponent's hurtbox.
#[derive(Clone, Debug)]
pub struct ActiveHitDef {
    /// Damage dealt on hit
    pub damage: i32,
    /// Damage dealt on guard
    pub guard_damage: i32,
    /// Horizontal velocity applied to grounded opponent on hit
    pub ground_velocity_x: i32,
    /// Vertical velocity applied to grounded opponent on hit
    pub ground_velocity_y: i32,
    /// Horizontal velocity applied to aerial opponent on hit
    pub air_velocity_x: i32,
    /// Vertical velocity applied to aerial opponent on hit
    pub air_velocity_y: i32,
    /// Horizontal velocity applied to opponent on guard
    pub guard_velocity_x: i32,
    /// Frames of hitstun on grounded hit
    pub ground_hittime: i32,
    /// Frames of blockstun on guard
    pub guard_hittime: i32,
    /// Frames of hitstun on aerial hit
    pub air_hittime: i32,
    /// Whether hit causes opponent to fall/launch
    pub fall: bool,
    /// Power gained by attacker on hit
    pub getpower_hit: i32,
    /// Power gained by attacker on guard
    pub getpower_guard: i32,
    /// Attack attribute string (e.g. "S.NA" = standing normal attack)
    pub attr: String,
    /// Pause frames for attacker (hitpause)
    pub pausetime_p1: i32,
    /// Pause frames for defender (hitstop)
    pub pausetime_p2: i32,
    /// Whether this HitDef has already connected (prevents multi-hit)
    pub has_hit: bool,
    /// State to transition attacker into on hit (p1stateno)
    pub p1stateno: Option<i32>,
    /// State to transition defender into on hit (p2stateno)
    pub p2stateno: Option<i32>,
}

impl Default for ActiveHitDef {
    fn default() -> Self {
        Self {
            damage: 0,
            guard_damage: 0,
            ground_velocity_x: 0,
            ground_velocity_y: 0,
            air_velocity_x: 0,
            air_velocity_y: 0,
            guard_velocity_x: 0,
            ground_hittime: 15,
            guard_hittime: 15,
            air_hittime: 20,
            fall: false,
            getpower_hit: 20,
            getpower_guard: 5,
            attr: String::new(),
            pausetime_p1: 0,
            pausetime_p2: 0,
            has_hit: false,
            p1stateno: None,
            p2stateno: None,
        }
    }
}

/// Per-fighter MUGEN runtime state that persists across frames.
/// This holds data needed by the controller executor that isn't part of
/// tickle_core's standard components.
#[derive(Clone, Debug)]
pub struct MugenFighterState {
    /// Integer variables var(0)..var(59)
    pub vars: [i32; 60],
    /// Control flag - whether player can act
    pub ctrl: bool,
    /// Current animation number
    pub anim_num: i32,
    /// Current animation element (1-based)
    pub anim_elem: i32,
    /// Ticks into current animation element
    pub anim_time: i32,
    /// Whether current attack connected
    pub move_hit: bool,
    /// Whether current attack made contact (hit or guarded)
    pub move_contact: bool,
    /// Whether current attack was guarded
    pub move_guarded: bool,
    /// Previous state number (before last ChangeState)
    pub prev_state_num: i32,
    /// Active command names this frame
    pub active_commands: Vec<String>,
    /// Assert special flags active this frame
    pub assert_flags: Vec<String>,
    /// Currently active HitDef (set by HitDef controller, consumed by collision)
    pub active_hitdef: Option<ActiveHitDef>,
    /// Sound events queued this frame by PlaySnd controllers.
    /// The game loop should drain these and dispatch to the audio system.
    pub pending_sounds: Vec<MugenSoundEvent>,
    /// Deterministic RNG state for VarRandom controller.
    /// Uses the same LCG as tickle_network::DeterministicRng to stay
    /// rollback-compatible without a cross-crate dependency.
    pub rng_state: u64,
    /// Combo hit counter for damage scaling. Resets when opponent exits hitstun.
    pub combo_count: u32,
    /// Remaining hitstun frames for this fighter (when being hit).
    pub hitstun_remaining: i32,
    /// Remaining blockstun frames for this fighter (when blocking).
    pub blockstun_remaining: i32,
    /// Active NotHitBy window: attacks matching this attr filter pass through.
    /// None when not active.
    pub not_hit_by: Option<(String, i32)>,
    /// Active HitBy window: only attacks matching this attr filter can connect.
    /// None when not active.
    pub hit_by: Option<(String, i32)>,
}

impl Default for MugenFighterState {
    fn default() -> Self {
        Self {
            vars: [0; 60],
            ctrl: true,
            anim_num: 0,
            anim_elem: 1,
            anim_time: 0,
            move_hit: false,
            move_contact: false,
            move_guarded: false,
            prev_state_num: 0,
            active_commands: Vec::new(),
            assert_flags: Vec::new(),
            active_hitdef: None,
            pending_sounds: Vec::new(),
            rng_state: 42,
            combo_count: 0,
            hitstun_remaining: 0,
            blockstun_remaining: 0,
            not_hit_by: None,
            hit_by: None,
        }
    }
}

/// Advance the deterministic LCG and return the next u32.
/// Uses the same constants as tickle_network::DeterministicRng.
fn rng_next(state: &mut u64) -> u32 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    (*state >> 32) as u32
}

/// Generate a random i32 in [min, max] (inclusive on both ends).
fn rng_range(state: &mut u64, min: i32, max: i32) -> i32 {
    if min >= max {
        return min;
    }
    let range = (max - min + 1) as u32; // +1 for inclusive max
    min + (rng_next(state) % range) as i32
}

/// Collects side effects from controller execution to be applied after iteration.
#[derive(Debug, Default)]
struct ControllerEffects {
    new_state: Option<i32>,
    new_ctrl: Option<bool>,
    vel_set_x: Option<i32>,
    vel_set_y: Option<i32>,
    vel_add_x: Option<i32>,
    vel_add_y: Option<i32>,
    pos_set_x: Option<i32>,
    pos_set_y: Option<i32>,
    pos_add_x: Option<i32>,
    pos_add_y: Option<i32>,
    var_ops: Vec<VarOp>,
    new_anim: Option<i32>,
    apply_gravity: bool,
    assert_flags: Vec<String>,
    hitdef: Option<ActiveHitDef>,
    sound_events: Vec<MugenSoundEvent>,
    not_hit_by: Option<(String, i32)>,
    hit_by: Option<(String, i32)>,
}

#[derive(Debug)]
enum VarOp {
    Set { index: usize, value: i32 },
    Add { index: usize, value: i32 },
}

/// Build a TriggerContext from the current fighter state.
fn build_trigger_context(
    cns: &Cns,
    fighter_state: &FighterState,
    mugen: &MugenFighterState,
    position: &Position,
    velocity: &Velocity,
    health: &Health,
    power: &PowerGauge,
) -> TriggerContext {
    let statetype = cns.get_state(fighter_state.state_num)
        .map(|sd| match sd.state_type {
            StateTypeValue::Standing  => 0,
            StateTypeValue::Crouching => 1,
            StateTypeValue::Aerial    => 2,
            StateTypeValue::Lying     => 3,
        })
        .unwrap_or(0);
    TriggerContext {
        time: fighter_state.state_frame,
        stateno: fighter_state.state_num,
        prev_stateno: mugen.prev_state_num,
        statetype,
        vel_x: velocity.vel.x,
        vel_y: velocity.vel.y,
        pos_y: position.pos.y,
        ctrl: mugen.ctrl,
        vars: mugen.vars,
        anim_num: mugen.anim_num,
        anim_elem: mugen.anim_elem,
        anim_time: mugen.anim_time,
        move_hit: mugen.move_hit,
        move_contact: mugen.move_contact,
        move_guarded: mugen.move_guarded,
        active_commands: mugen.active_commands.clone(),
        life: health.current,
        power: power.current,
        // P2 fields default to 0; populated by the caller when opponent data is available
        p2_stateno: 0,
        p2_life: 0,
        p2_bodydist_x: 0,
        p2_bodydist_y: 0,
        p2_statetype: 0,
        p2_movetype: 0,
        p2_ctrl: false,
        p2_vel_x: 0,
        p2_vel_y: 0,
    }
}

/// Execute a single controller and collect its effects.
fn execute_controller(
    controller: &Controller,
    ctx: &TriggerContext,
    effects: &mut ControllerEffects,
    _yaccel: f32,
    rng_state: &mut u64,
) {
    match controller {
        Controller::ChangeState { value, ctrl } => {
            let target = value.evaluate_int(ctx);
            effects.new_state = Some(target);
            if let Some(ctrl_expr) = ctrl {
                effects.new_ctrl = Some(ctrl_expr.evaluate_int(ctx) != 0);
            }
        }
        Controller::VelSet { x, y } => {
            if let Some(expr) = x {
                let val = (expr.evaluate_int(ctx) as f32 * 100.0) as i32;
                effects.vel_set_x = Some(val);
            }
            if let Some(expr) = y {
                let val = (expr.evaluate_int(ctx) as f32 * 100.0) as i32;
                effects.vel_set_y = Some(val);
            }
        }
        Controller::VelAdd { x, y } => {
            if let Some(expr) = x {
                let val = (expr.evaluate_int(ctx) as f32 * 100.0) as i32;
                effects.vel_add_x = Some(effects.vel_add_x.unwrap_or(0) + val);
            }
            if let Some(expr) = y {
                let val = (expr.evaluate_int(ctx) as f32 * 100.0) as i32;
                effects.vel_add_y = Some(effects.vel_add_y.unwrap_or(0) + val);
            }
        }
        Controller::PosAdd { x, y } => {
            if let Some(expr) = x {
                let val = (expr.evaluate_int(ctx) as f32 * 100.0) as i32;
                effects.pos_add_x = Some(effects.pos_add_x.unwrap_or(0) + val);
            }
            if let Some(expr) = y {
                let val = (expr.evaluate_int(ctx) as f32 * 100.0) as i32;
                effects.pos_add_y = Some(effects.pos_add_y.unwrap_or(0) + val);
            }
        }
        Controller::PosSet { x, y } => {
            if let Some(expr) = x {
                let val = (expr.evaluate_int(ctx) as f32 * 100.0) as i32;
                effects.pos_set_x = Some(val);
            }
            if let Some(expr) = y {
                let val = (expr.evaluate_int(ctx) as f32 * 100.0) as i32;
                effects.pos_set_y = Some(val);
            }
        }
        Controller::VarSet { var_num, value } => {
            let val = value.evaluate_int(ctx);
            effects.var_ops.push(VarOp::Set {
                index: *var_num as usize,
                value: val,
            });
        }
        Controller::VarAdd { var_num, value } => {
            let val = value.evaluate_int(ctx);
            effects.var_ops.push(VarOp::Add {
                index: *var_num as usize,
                value: val,
            });
        }
        Controller::CtrlSet { value } => {
            effects.new_ctrl = Some(value.evaluate_int(ctx) != 0);
        }
        Controller::ChangeAnim { value } => {
            effects.new_anim = Some(value.evaluate_int(ctx));
        }
        Controller::PlaySnd { group, sound } => {
            effects.sound_events.push(MugenSoundEvent {
                group: *group,
                sound: *sound,
            });
        }
        Controller::AssertSpecial { flags } => {
            effects.assert_flags.extend(flags.iter().cloned());
        }
        Controller::HitDef {
            attr,
            damage,
            guard_damage,
            ground_velocity_x,
            ground_velocity_y,
            air_velocity_x,
            air_velocity_y,
            guard_velocity_x,
            ground_hittime,
            guard_hittime,
            air_hittime,
            fall,
            getpower_hit,
            getpower_guard,
            pausetime_p1,
            pausetime_p2,
            p1stateno,
            p2stateno,
            ..
        } => {
            effects.hitdef = Some(ActiveHitDef {
                damage: *damage,
                guard_damage: *guard_damage,
                ground_velocity_x: (*ground_velocity_x * 100.0) as i32,
                ground_velocity_y: (*ground_velocity_y * 100.0) as i32,
                air_velocity_x: (*air_velocity_x * 100.0) as i32,
                air_velocity_y: (*air_velocity_y * 100.0) as i32,
                guard_velocity_x: (*guard_velocity_x * 100.0) as i32,
                ground_hittime: *ground_hittime,
                guard_hittime: *guard_hittime,
                air_hittime: *air_hittime,
                fall: *fall,
                getpower_hit: *getpower_hit,
                getpower_guard: *getpower_guard,
                attr: attr.clone(),
                pausetime_p1: *pausetime_p1,
                pausetime_p2: *pausetime_p2,
                has_hit: false,
                p1stateno: *p1stateno,
                p2stateno: *p2stateno,
            });
        }
        Controller::NotHitBy { attr, time } => {
            effects.not_hit_by = Some((attr.clone(), *time));
        }
        Controller::HitBy { attr, time } => {
            effects.hit_by = Some((attr.clone(), *time));
        }
        Controller::VarRandom {
            var_num,
            range_min,
            range_max,
        } => {
            let val = rng_range(rng_state, *range_min, *range_max);
            effects.var_ops.push(VarOp::Set {
                index: *var_num as usize,
                value: val,
            });
        }
        Controller::Gravity => {
            effects.apply_gravity = true;
        }
        Controller::Unknown(_) => {
            // Silently ignore unknown controllers
        }
    }
}

/// Apply collected effects to the fighter's state.
fn apply_effects(
    effects: &ControllerEffects,
    fighter_state: &mut FighterState,
    mugen: &mut MugenFighterState,
    position: &mut Position,
    velocity: &mut Velocity,
    cns: &Cns,
) {
    // Apply velocity set (absolute)
    if let Some(vx) = effects.vel_set_x {
        velocity.vel.x = vx;
    }
    if let Some(vy) = effects.vel_set_y {
        velocity.vel.y = vy;
    }

    // Apply velocity add (relative)
    if let Some(vx) = effects.vel_add_x {
        velocity.vel.x += vx;
    }
    if let Some(vy) = effects.vel_add_y {
        velocity.vel.y += vy;
    }

    // Apply position set (absolute)
    if let Some(px) = effects.pos_set_x {
        position.pos.x = px;
    }
    if let Some(py) = effects.pos_set_y {
        position.pos.y = py;
    }

    // Apply position add (relative)
    if let Some(px) = effects.pos_add_x {
        position.pos.x += px;
    }
    if let Some(py) = effects.pos_add_y {
        position.pos.y += py;
    }

    // Apply variable operations
    for op in &effects.var_ops {
        match op {
            VarOp::Set { index, value } => {
                if *index < 60 {
                    mugen.vars[*index] = *value;
                }
            }
            VarOp::Add { index, value } => {
                if *index < 60 {
                    mugen.vars[*index] += *value;
                }
            }
        }
    }

    // Apply ctrl
    if let Some(ctrl) = effects.new_ctrl {
        mugen.ctrl = ctrl;
    }

    // Apply animation change
    if let Some(anim) = effects.new_anim {
        mugen.anim_num = anim;
        mugen.anim_elem = 1;
        mugen.anim_time = 0;
    }

    // Apply gravity
    if effects.apply_gravity {
        let yaccel = cns.movement.yaccel;
        velocity.vel.y += (yaccel * 100.0) as i32;
    }

    // Collect assert flags
    if !effects.assert_flags.is_empty() {
        mugen
            .assert_flags
            .extend(effects.assert_flags.iter().cloned());
    }

    // Apply HitDef activation
    if let Some(ref hitdef) = effects.hitdef {
        mugen.active_hitdef = Some(hitdef.clone());
        // Clear move_hit/move_contact/move_guarded when a new HitDef is set
        mugen.move_hit = false;
        mugen.move_contact = false;
        mugen.move_guarded = false;
    }

    // Queue sound events for the game loop to dispatch to the audio system
    if !effects.sound_events.is_empty() {
        mugen.pending_sounds.extend(effects.sound_events.iter().cloned());
    }

    // Apply NotHitBy: attacks matching the filter pass through the fighter.
    // NotHitBy and HitBy are mutually exclusive — setting one clears the other.
    if let Some(ref nhb) = effects.not_hit_by {
        mugen.not_hit_by = Some(nhb.clone());
        mugen.hit_by = None;
    }

    // Apply HitBy: only attacks matching the filter can connect.
    if let Some(ref hb) = effects.hit_by {
        mugen.hit_by = Some(hb.clone());
        mugen.not_hit_by = None;
    }

    // Apply state change LAST (so velocity/position from this tick apply first)
    if let Some(new_state) = effects.new_state {
        enter_state(fighter_state, mugen, velocity, new_state, cns);
    }
}

/// Transition into a new state: reset state_frame, apply statedef entry properties.
fn enter_state(
    fighter_state: &mut FighterState,
    mugen: &mut MugenFighterState,
    velocity: &mut Velocity,
    new_state: i32,
    cns: &Cns,
) {
    mugen.prev_state_num = fighter_state.state_num;
    fighter_state.change_state(new_state);

    // Apply statedef entry properties
    if let Some(statedef) = cns.get_state(new_state) {
        if let Some(ref velset) = statedef.velset {
            velocity.vel.x = (velset.x * 100.0) as i32;
            velocity.vel.y = (velset.y * 100.0) as i32;
        }
        if let Some(ctrl) = statedef.ctrl {
            mugen.ctrl = ctrl != 0;
        }
        if let Some(anim) = statedef.anim {
            mugen.anim_num = anim;
            mugen.anim_elem = 1;
            mugen.anim_time = 0;
        }
    }
}

/// Evaluate and execute controllers from a list, accumulating effects.
/// Stops processing on the first ChangeState controller that fires
/// (MUGEN semantics: only one state change per tick per controller list).
fn process_controllers(
    controllers: &[StateController],
    ctx: &TriggerContext,
    effects: &mut ControllerEffects,
    yaccel: f32,
    rng_state: &mut u64,
) {
    for sc in controllers {
        if sc.should_fire(ctx) {
            execute_controller(&sc.controller, ctx, effects, yaccel, rng_state);
            // If a ChangeState fired, stop processing this controller list
            if matches!(sc.controller, Controller::ChangeState { .. }) && effects.new_state.is_some()
            {
                break;
            }
        }
    }
}

/// Main system entry point: execute MUGEN controllers for one fighter for one frame.
///
/// This processes the current state's controllers, applies statedef physics,
/// and advances the state frame counter.
///
/// Call this once per fighter per game tick.
pub fn mugen_controller_system(
    cns: &Cns,
    fighter_state: &mut FighterState,
    mugen: &mut MugenFighterState,
    position: &mut Position,
    velocity: &mut Velocity,
    health: &mut Health,
    power: &mut PowerGauge,
) {
    // Clear per-frame assert flags
    mugen.assert_flags.clear();

    let yaccel = cns.movement.yaccel;

    // 1. Process current state's controllers
    let mut effects = ControllerEffects::default();
    let ctx = build_trigger_context(cns, fighter_state, mugen, position, velocity, health, power);

    if let Some(statedef) = cns.get_state(fighter_state.state_num) {
        process_controllers(&statedef.controllers, &ctx, &mut effects, yaccel, &mut mugen.rng_state);
    }

    apply_effects(&effects, fighter_state, mugen, position, velocity, cns);

    // 2. Apply statedef physics for the current state (after controller effects)
    apply_statedef_physics(cns, fighter_state, position, velocity);

    // 3. Advance state frame
    fighter_state.advance_frame();
    mugen.anim_time += 1;
}

/// Apply physics behavior defined by the current statedef.
fn apply_statedef_physics(
    cns: &Cns,
    fighter_state: &FighterState,
    position: &mut Position,
    velocity: &mut Velocity,
) {
    let physics = cns
        .get_state(fighter_state.state_num)
        .and_then(|sd| sd.physics.as_ref());

    match physics {
        Some(Physics::Standing) => {
            // Standing physics: apply ground friction (MUGEN -y=up: y>=0 means at/on ground)
            if position.pos.y >= GROUND_Y {
                let friction = (cns.movement.stand_friction * 100.0) as i32;
                if velocity.vel.x > 0 {
                    velocity.vel.x = (velocity.vel.x - friction).max(0);
                } else if velocity.vel.x < 0 {
                    velocity.vel.x = (velocity.vel.x + friction).min(0);
                }
            }
        }
        Some(Physics::Crouching) => {
            // Crouching physics: apply crouch friction (MUGEN -y=up: y>=0 means at/on ground)
            if position.pos.y >= GROUND_Y {
                let friction = (cns.movement.crouch_friction * 100.0) as i32;
                if velocity.vel.x > 0 {
                    velocity.vel.x = (velocity.vel.x - friction).max(0);
                } else if velocity.vel.x < 0 {
                    velocity.vel.x = (velocity.vel.x + friction).min(0);
                }
            }
        }
        Some(Physics::Aerial) => {
            // Aerial physics: apply gravity
            velocity.vel.y += (cns.movement.yaccel * 100.0) as i32;
        }
        Some(Physics::None) | None => {
            // No automatic physics
        }
    }
}

/// Execute Statedef -1 global controllers.
/// These run every tick regardless of current state, and are used for
/// command-based state transitions (e.g., special move inputs).
///
/// Call this before `mugen_controller_system` so that global transitions
/// take priority.
pub fn mugen_global_controller_system(
    cns: &Cns,
    fighter_state: &mut FighterState,
    mugen: &mut MugenFighterState,
    position: &mut Position,
    velocity: &mut Velocity,
    health: &mut Health,
    power: &mut PowerGauge,
) {
    let yaccel = cns.movement.yaccel;
    let ctx = build_trigger_context(cns, fighter_state, mugen, position, velocity, health, power);

    let mut effects = ControllerEffects::default();
    process_controllers(&cns.global_state_controllers, &ctx, &mut effects, yaccel, &mut mugen.rng_state);

    apply_effects(&effects, fighter_state, mugen, position, velocity, cns);
}

/// Complete MUGEN tick for one fighter: Statedef -1 first, then current state.
///
/// This is the recommended entry point for game loops. It ensures the correct
/// MUGEN execution order: global controllers (Statedef -1) run first so that
/// command-based transitions take priority over in-state controllers.
pub fn mugen_tick(
    cns: &Cns,
    fighter_state: &mut FighterState,
    mugen: &mut MugenFighterState,
    position: &mut Position,
    velocity: &mut Velocity,
    health: &mut Health,
    power: &mut PowerGauge,
) {
    // Clear per-tick sound events so both phases accumulate fresh events
    mugen.pending_sounds.clear();

    // Phase 1: Statedef -1 (global controllers, e.g., command-based transitions)
    mugen_global_controller_system(cns, fighter_state, mugen, position, velocity, health, power);

    // Phase 2: Current state controllers + physics + frame advance
    mugen_controller_system(cns, fighter_state, mugen, position, velocity, health, power);
}

/// Result of a HitDef collision check between two fighters.
#[derive(Clone, Debug)]
pub struct MugenHitResult {
    /// Damage dealt to defender
    pub damage: i32,
    /// Hitstun frames applied to defender
    pub hitstun: i32,
    /// Knockback velocity applied to defender
    pub knockback: LogicVec2,
    /// Power gained by attacker
    pub attacker_power: i32,
    /// State number defender was put into
    pub defender_state: i32,
}

/// Get the attack hitboxes (Clsn1) for a fighter's current animation frame.
/// Returns hitbox rectangles in local space (relative to fighter position).
fn get_attack_clsn1(air: &Air, anim_num: i32, anim_elem: i32) -> Vec<LogicRect> {
    let action = match air.get_action(anim_num as u32) {
        Some(a) => a,
        None => return Vec::new(),
    };
    let frame_idx = (anim_elem - 1).max(0) as usize;
    if frame_idx >= action.frames.len() {
        return Vec::new();
    }
    let frame = &action.frames[frame_idx];

    // Frame-specific Clsn1 overrides action default
    let clsn1_boxes = frame.clsn1.as_ref().unwrap_or(&action.clsn1_default);

    clsn1_boxes
        .iter()
        .map(|c| {
            let left = c.left as i32 * 100;
            let top = c.top as i32 * 100;
            let right = c.right as i32 * 100;
            let bottom = c.bottom as i32 * 100;
            LogicRect::new(left, top, right - left, bottom - top)
        })
        .collect()
}

/// Get the hurtboxes (Clsn2) for a fighter's current animation frame.
/// Returns hurtbox rectangles in local space.
fn get_hurtbox_clsn2(air: &Air, anim_num: i32, anim_elem: i32) -> Vec<LogicRect> {
    let action = match air.get_action(anim_num as u32) {
        Some(a) => a,
        None => return Vec::new(),
    };
    let frame_idx = (anim_elem - 1).max(0) as usize;
    if frame_idx >= action.frames.len() {
        return Vec::new();
    }
    let frame = &action.frames[frame_idx];

    // Frame-specific Clsn2 overrides action default
    let clsn2_boxes = frame.clsn2.as_ref().unwrap_or(&action.clsn2_default);

    clsn2_boxes
        .iter()
        .map(|c| {
            let left = c.left as i32 * 100;
            let top = c.top as i32 * 100;
            let right = c.right as i32 * 100;
            let bottom = c.bottom as i32 * 100;
            LogicRect::new(left, top, right - left, bottom - top)
        })
        .collect()
}

/// Transform a local-space rect to world space by adding position offset
/// and optionally flipping horizontally based on facing direction.
fn to_world_rect(rect: LogicRect, pos: &Position, facing: &Facing) -> LogicRect {
    let mut r = LogicRect::new(
        rect.x + pos.pos.x,
        rect.y + pos.pos.y,
        rect.w,
        rect.h,
    );
    if facing.dir == Facing::LEFT {
        // Flip around fighter position: mirror the X coordinate
        r = r.flip_x(pos.pos.x);
    }
    r
}

/// Check if any attack hitbox intersects any defender hurtbox.
fn check_hitbox_overlap(
    attack_rects: &[LogicRect],
    attacker_pos: &Position,
    attacker_facing: &Facing,
    hurt_rects: &[LogicRect],
    defender_pos: &Position,
    defender_facing: &Facing,
) -> bool {
    for atk_rect in attack_rects {
        let world_atk = to_world_rect(*atk_rect, attacker_pos, attacker_facing);
        for def_rect in hurt_rects {
            let world_def = to_world_rect(*def_rect, defender_pos, defender_facing);
            if world_atk.intersects(world_def) {
                return true;
            }
        }
    }
    false
}

/// Data bundle for one fighter in the HitDef collision system.
pub struct MugenCollisionFighter<'a> {
    pub position: &'a Position,
    pub velocity: &'a mut Velocity,
    pub facing: &'a Facing,
    pub fighter_state: &'a mut FighterState,
    pub mugen: &'a mut MugenFighterState,
    pub health: &'a mut Health,
    pub power: &'a mut PowerGauge,
}

/// Check whether an attack attribute matches a NotHitBy/HitBy filter.
///
/// MUGEN attr format: "StateType,AttackType" — e.g. "S,NA" = standing normal attack.
/// The filter's state part can combine chars: "SCA" matches any stance.
/// The filter's attack part can list multiple codes separated by spaces: "NA SA".
///
/// Returns `true` if the attack is covered by the filter.
fn attack_attr_matches(filter: &str, attr: &str) -> bool {
    let (filter_state, filter_attack) = match filter.trim().find(',') {
        Some(i) => (&filter.trim()[..i], filter.trim()[i + 1..].trim()),
        None => return false,
    };
    let (attr_state, attr_attack) = match attr.trim().find(',') {
        Some(i) => (&attr.trim()[..i], attr.trim()[i + 1..].trim()),
        None => return false,
    };

    // State type match: each char in the attack's state string must appear in the filter's state string.
    // Empty filter_state is treated as a wildcard matching any state.
    let filter_state_upper = filter_state.trim().to_ascii_uppercase();
    let state_match = filter_state_upper.is_empty()
        || attr_state
            .trim()
            .chars()
            .any(|c| filter_state_upper.contains(c.to_ascii_uppercase()));

    if !state_match {
        return false;
    }

    // Attack type match: attr_attack must equal one of the whitespace-separated codes in filter_attack.
    let attr_attack_upper = attr_attack.to_ascii_uppercase();
    filter_attack
        .split_ascii_whitespace()
        .any(|code| code.to_ascii_uppercase() == attr_attack_upper)
}

/// HitDef collision system: checks if the attacker's active HitDef connects
/// with the defender, and applies damage/hitstun/knockback.
///
/// This uses AIR Clsn1 (attack) and Clsn2 (hurtbox) data to determine
/// collision. If no AIR data is available, falls back to a distance-based
/// check using the CnsSize.attack_dist parameter.
///
/// Returns Some(MugenHitResult) if a hit connected, None otherwise.
pub fn mugen_hitdef_collision(
    attacker: &mut MugenCollisionFighter<'_>,
    defender: &mut MugenCollisionFighter<'_>,
    air: Option<&Air>,
    attack_dist: i32,
) -> Option<MugenHitResult> {
    let hitdef = match attacker.mugen.active_hitdef.as_ref() {
        Some(hd) if !hd.has_hit => hd,
        _ => return None,
    };

    // NotHitBy: if the defender's filter covers this attack attribute, the hit is blocked.
    if let Some((ref filter, _)) = defender.mugen.not_hit_by {
        if attack_attr_matches(filter, &hitdef.attr) {
            return None;
        }
    }

    // HitBy: if the defender has a HitBy restriction and the attack does NOT match, block it.
    if let Some((ref filter, _)) = defender.mugen.hit_by {
        if !attack_attr_matches(filter, &hitdef.attr) {
            return None;
        }
    }

    let hit_connected = if let Some(air_data) = air {
        // Use AIR Clsn1/Clsn2 data for precise collision
        let atk_rects = get_attack_clsn1(air_data, attacker.mugen.anim_num, attacker.mugen.anim_elem);
        let def_rects = get_hurtbox_clsn2(air_data, defender.mugen.anim_num, defender.mugen.anim_elem);

        if atk_rects.is_empty() || def_rects.is_empty() {
            false
        } else {
            check_hitbox_overlap(
                &atk_rects,
                attacker.position,
                attacker.facing,
                &def_rects,
                defender.position,
                defender.facing,
            )
        }
    } else {
        // Fallback: distance-based check using attack_dist
        let dx = (attacker.position.pos.x - defender.position.pos.x).abs();
        let dy = (attacker.position.pos.y - defender.position.pos.y).abs();
        dx <= attack_dist * 100 && dy <= 8000
    };

    if !hit_connected {
        return None;
    }

    // Clone HitDef data before mutating
    let damage = hitdef.damage;
    let ground_hittime = hitdef.ground_hittime;
    let air_hittime = hitdef.air_hittime;
    let ground_velocity_x = hitdef.ground_velocity_x;
    let ground_velocity_y = hitdef.ground_velocity_y;
    let air_velocity_x = hitdef.air_velocity_x;
    let air_velocity_y = hitdef.air_velocity_y;
    let fall = hitdef.fall;
    let getpower_hit = hitdef.getpower_hit;
    let p2stateno = hitdef.p2stateno;

    // Mark HitDef as consumed
    if let Some(ref mut hd) = attacker.mugen.active_hitdef {
        hd.has_hit = true;
    }

    // Set attacker flags
    attacker.mugen.move_hit = true;
    attacker.mugen.move_contact = true;

    // Apply damage to defender
    defender.health.take_damage(damage);

    // Determine defender's hit state and knockback based on their position
    let is_defender_airborne = defender.position.pos.y > GROUND_Y;
    let (hitstun, knockback_x, knockback_y, defender_state) = if is_defender_airborne {
        // Air hit
        (
            air_hittime,
            air_velocity_x,
            air_velocity_y,
            p2stateno.unwrap_or(STATE_HIT_AIR_LIGHT),
        )
    } else if fall {
        // Ground hit with fall (launcher)
        (
            air_hittime,
            ground_velocity_x,
            ground_velocity_y,
            p2stateno.unwrap_or(STATE_HIT_AIR_LIGHT),
        )
    } else {
        // Regular ground hit
        (
            ground_hittime,
            ground_velocity_x,
            ground_velocity_y,
            p2stateno.unwrap_or(STATE_HIT_STAND_LIGHT),
        )
    };

    // Apply knockback (flip direction based on attacker facing)
    let dir = attacker.facing.dir;
    defender.velocity.vel.x = knockback_x * dir;
    defender.velocity.vel.y = knockback_y;

    // Put defender in hitstun state
    defender.fighter_state.change_state(defender_state);

    // Set hitstun timer
    defender.mugen.hitstun_remaining = hitstun;

    // Disable defender control during hitstun
    defender.mugen.ctrl = false;

    // Grant power to attacker
    attacker.power.add(getpower_hit);

    Some(MugenHitResult {
        damage,
        hitstun,
        knockback: LogicVec2::new(knockback_x * dir, knockback_y),
        attacker_power: getpower_hit,
        defender_state,
    })
}

/// Convenience function to build a tickle_core Hitbox from an ActiveHitDef.
/// Used when the game loop needs to create standard Hitbox components from
/// MUGEN HitDef data.
pub fn hitdef_to_hitbox(hitdef: &ActiveHitDef, rect: LogicRect) -> Hitbox {
    let hit_type = if hitdef.attr.contains('A') {
        HitType::Mid // Aerial attack
    } else if hitdef.attr.contains("HA") || hitdef.attr.to_uppercase().contains("HIGH") {
        HitType::High
    } else if hitdef.attr.to_lowercase().contains("low") {
        HitType::Low
    } else {
        HitType::Mid
    };

    Hitbox {
        rect,
        damage: hitdef.damage,
        hitstun: hitdef.ground_hittime,
        blockstun: hitdef.guard_hittime,
        knockback: LogicVec2::new(hitdef.ground_velocity_x, hitdef.ground_velocity_y),
        hit_type,
    }
}

/// Combo scaling: damage multiplier decreases with each successive hit.
/// Returns scaled damage. combo_count starts at 0 for the first hit.
fn combo_scaled_damage(base_damage: i32, combo_count: u32) -> i32 {
    let scale = match combo_count {
        0 | 1 => 100,
        2 => 100,
        3 => 80,
        4 => 70,
        5 => 60,
        _ => 50,
    };
    base_damage * scale / 100
}

/// Result of the full combat integration for one frame.
#[derive(Clone, Debug, Default)]
pub struct CombatFrameResult {
    /// Hit result for P1 attacking P2 (if a hit connected)
    pub p1_hit: Option<MugenHitResult>,
    /// Hit result for P2 attacking P1 (if a hit connected)
    pub p2_hit: Option<MugenHitResult>,
}

/// Full MUGEN combat system for two fighters in a single frame.
///
/// This integrates HitDef collision, guard handling, combo scaling,
/// hitstun/blockstun countdown, and knockdown. Call once per game frame
/// after both fighters' controller systems have run.
///
/// When `air` is provided, uses Clsn1/Clsn2 for precise collision.
/// Otherwise falls back to distance-based detection using `attack_dist`.
pub fn mugen_combat_frame(
    p1: &mut MugenCollisionFighter<'_>,
    p2: &mut MugenCollisionFighter<'_>,
    air: Option<&Air>,
    attack_dist: i32,
) -> CombatFrameResult {
    let mut result = CombatFrameResult::default();

    // Tick down hitstun/blockstun for both fighters
    tick_stun_timers(p1);
    tick_stun_timers(p2);

    // Check P1 attacking P2
    let p1_hit = resolve_attack(p1, p2, air, attack_dist);
    if let Some(ref hit) = p1_hit {
        // Apply combo scaling on the damage
        let scaled_damage = combo_scaled_damage(hit.damage, p1.mugen.combo_count);
        let extra_damage = hit.damage - scaled_damage;
        if extra_damage > 0 {
            // Heal back the excess damage that was applied at full in mugen_hitdef_collision
            p2.health.heal(extra_damage);
        }
        p1.mugen.combo_count += 1;
    }
    result.p1_hit = p1_hit;

    // Check P2 attacking P1
    let p2_hit = resolve_attack(p2, p1, air, attack_dist);
    if let Some(ref hit) = p2_hit {
        let scaled_damage = combo_scaled_damage(hit.damage, p2.mugen.combo_count);
        let extra_damage = hit.damage - scaled_damage;
        if extra_damage > 0 {
            p1.health.heal(extra_damage);
        }
        p2.mugen.combo_count += 1;
    }
    result.p2_hit = p2_hit;

    result
}

/// Check if the defender is currently blocking (in a guard state).
fn is_guarding(fighter_state: &FighterState) -> bool {
    is_guard_state(fighter_state.state_num)
}

/// Resolve one attacker's HitDef against a defender.
/// Handles guard detection: if the defender is blocking, applies guard damage
/// and blockstun instead of full hit damage and hitstun.
fn resolve_attack(
    attacker: &mut MugenCollisionFighter<'_>,
    defender: &mut MugenCollisionFighter<'_>,
    air: Option<&Air>,
    attack_dist: i32,
) -> Option<MugenHitResult> {
    // Check if defender is in a guard state
    if is_guarding(defender.fighter_state) {
        return resolve_guarded_attack(attacker, defender, air, attack_dist);
    }

    // Normal hit resolution using existing mugen_hitdef_collision
    mugen_hitdef_collision(attacker, defender, air, attack_dist)
}

/// Handle a guarded (blocked) attack: apply guard damage and blockstun.
fn resolve_guarded_attack(
    attacker: &mut MugenCollisionFighter<'_>,
    defender: &mut MugenCollisionFighter<'_>,
    air: Option<&Air>,
    attack_dist: i32,
) -> Option<MugenHitResult> {
    let hitdef = match attacker.mugen.active_hitdef.as_ref() {
        Some(hd) if !hd.has_hit => hd,
        _ => return None,
    };

    // Check collision (same logic as mugen_hitdef_collision)
    let hit_connected = if let Some(air_data) = air {
        let atk_rects = get_attack_clsn1(air_data, attacker.mugen.anim_num, attacker.mugen.anim_elem);
        let def_rects = get_hurtbox_clsn2(air_data, defender.mugen.anim_num, defender.mugen.anim_elem);
        if atk_rects.is_empty() || def_rects.is_empty() {
            false
        } else {
            check_hitbox_overlap(
                &atk_rects, attacker.position, attacker.facing,
                &def_rects, defender.position, defender.facing,
            )
        }
    } else {
        let dx = (attacker.position.pos.x - defender.position.pos.x).abs();
        let dy = (attacker.position.pos.y - defender.position.pos.y).abs();
        dx <= attack_dist * 100 && dy <= 8000
    };

    if !hit_connected {
        return None;
    }

    // Extract guard data before mutating
    let guard_damage = hitdef.guard_damage;
    let guard_hittime = hitdef.guard_hittime;
    let guard_velocity_x = hitdef.guard_velocity_x;
    let getpower_guard = hitdef.getpower_guard;

    // Mark HitDef as consumed
    if let Some(ref mut hd) = attacker.mugen.active_hitdef {
        hd.has_hit = true;
    }

    // Set attacker flags (contact but not hit)
    attacker.mugen.move_contact = true;
    attacker.mugen.move_guarded = true;

    // Apply guard damage
    defender.health.take_damage(guard_damage);

    // Apply guard pushback
    let dir = attacker.facing.dir;
    defender.velocity.vel.x = guard_velocity_x * dir;

    // Set blockstun
    defender.mugen.blockstun_remaining = guard_hittime;

    // Grant guard power to attacker
    attacker.power.add(getpower_guard);

    Some(MugenHitResult {
        damage: guard_damage,
        hitstun: guard_hittime,
        knockback: LogicVec2::new(guard_velocity_x * dir, 0),
        attacker_power: getpower_guard,
        defender_state: defender.fighter_state.state_num,
    })
}

/// Count down hitstun and blockstun timers. When they expire, transition
/// the fighter back to an appropriate recovery state.
fn tick_stun_timers(fighter: &mut MugenCollisionFighter<'_>) {
    // Hitstun countdown
    if fighter.mugen.hitstun_remaining > 0 {
        fighter.mugen.hitstun_remaining -= 1;
        if fighter.mugen.hitstun_remaining == 0 {
            // Hitstun expired - return to stand and restore control
            if fighter.position.pos.y > GROUND_Y {
                // Still airborne - let physics handle landing
            } else {
                fighter.fighter_state.change_state(STATE_STAND);
                fighter.mugen.ctrl = true;
            }
        }
    }

    // Blockstun countdown
    if fighter.mugen.blockstun_remaining > 0 {
        fighter.mugen.blockstun_remaining -= 1;
        if fighter.mugen.blockstun_remaining == 0 {
            // Blockstun expired - return to stand guard end and restore control
            fighter.fighter_state.change_state(STATE_GUARD_END);
            fighter.mugen.ctrl = true;
        }
    }

    // NotHitBy countdown
    if let Some((_, ref mut ticks)) = fighter.mugen.not_hit_by {
        *ticks -= 1;
        if *ticks <= 0 {
            fighter.mugen.not_hit_by = None;
        }
    }

    // HitBy countdown
    if let Some((_, ref mut ticks)) = fighter.mugen.hit_by {
        *ticks -= 1;
        if *ticks <= 0 {
            fighter.mugen.hit_by = None;
        }
    }

    // Reset combo counter when opponent exits hitstun (not in a hit state)
    if !is_hit_state(fighter.fighter_state.state_num) {
        // The combo counter is on the attacker, so we don't reset here.
        // Instead, we detect this on the attacker side.
    }
}

/// Populate P2 trigger variables in a TriggerContext from the opponent's state.
/// Call this to create a fully-populated context before evaluating controllers.
pub fn populate_p2_context(
    ctx: &mut crate::trigger::TriggerContext,
    opponent_state: &FighterState,
    opponent_mugen: &MugenFighterState,
    opponent_pos: &Position,
    opponent_vel: &Velocity,
    opponent_health: &Health,
    fighter_pos: &Position,
) {
    ctx.p2_stateno = opponent_state.state_num;
    ctx.p2_life = opponent_health.current;
    ctx.p2_bodydist_x = (fighter_pos.pos.x - opponent_pos.pos.x).abs();
    ctx.p2_bodydist_y = (fighter_pos.pos.y - opponent_pos.pos.y).abs();
    ctx.p2_statetype = if is_aerial_state(opponent_state.state_num) {
        2 // Aerial
    } else if is_crouching_state(opponent_state.state_num) {
        1 // Crouching
    } else {
        0 // Standing
    };
    ctx.p2_movetype = if is_hit_state(opponent_state.state_num) {
        2 // Being hit
    } else if is_attack_state(opponent_state.state_num) {
        1 // Attacking
    } else {
        0 // Idle
    };
    ctx.p2_ctrl = opponent_mugen.ctrl;
    ctx.p2_vel_x = opponent_vel.vel.x;
    ctx.p2_vel_y = opponent_vel.vel.y;
}

/// Run the MUGEN controller system for one fighter with P2 context.
/// This is a higher-level wrapper that populates P2 trigger variables
/// before running the standard mugen_tick.
#[allow(clippy::too_many_arguments)]
pub fn mugen_tick_with_p2(
    cns: &Cns,
    fighter_state: &mut FighterState,
    mugen: &mut MugenFighterState,
    position: &mut Position,
    velocity: &mut Velocity,
    health: &mut Health,
    power: &mut PowerGauge,
    opponent_state: &FighterState,
    opponent_mugen: &MugenFighterState,
    opponent_pos: &Position,
    opponent_vel: &Velocity,
    opponent_health: &Health,
) {
    // Clear per-tick sound events so both phases accumulate fresh events
    mugen.pending_sounds.clear();

    let yaccel = cns.movement.yaccel;

    // Phase 1: Statedef -1 (global controllers) with P2 context
    {
        let mut ctx = build_trigger_context(cns, fighter_state, mugen, position, velocity, health, power);
        populate_p2_context(&mut ctx, opponent_state, opponent_mugen, opponent_pos, opponent_vel, opponent_health, position);

        let mut effects = ControllerEffects::default();
        process_controllers(&cns.global_state_controllers, &ctx, &mut effects, yaccel, &mut mugen.rng_state);
        apply_effects(&effects, fighter_state, mugen, position, velocity, cns);
    }

    // Phase 2: Current state controllers with P2 context
    {
        mugen.assert_flags.clear();

        let mut ctx = build_trigger_context(cns, fighter_state, mugen, position, velocity, health, power);
        populate_p2_context(&mut ctx, opponent_state, opponent_mugen, opponent_pos, opponent_vel, opponent_health, position);

        let mut effects = ControllerEffects::default();
        if let Some(statedef) = cns.get_state(fighter_state.state_num) {
            process_controllers(&statedef.controllers, &ctx, &mut effects, yaccel, &mut mugen.rng_state);
        }
        apply_effects(&effects, fighter_state, mugen, position, velocity, cns);

        // Apply statedef physics
        apply_statedef_physics(cns, fighter_state, position, velocity);

        // Advance state frame
        fighter_state.advance_frame();
        mugen.anim_time += 1;
    }
}

/// Reset combo counter for an attacker when their opponent leaves hitstun.
pub fn reset_combo_if_recovered(attacker_mugen: &mut MugenFighterState, defender_state: &FighterState) {
    if !is_hit_state(defender_state.state_num) {
        attacker_mugen.combo_count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cns::*;
    use crate::trigger::TriggerExpr;
    use std::collections::HashMap;
    use tickle_core::math::LogicVec2;

    /// Build a minimal Cns with only the fields we need for testing.
    fn minimal_cns() -> Cns {
        Cns {
            data: CnsData {
                life: 1000,
                attack: 100,
                defence: 100,
                fall_defence_mul: 100,
                liedown_time: 60,
                airjuggle: 15,
                sparkno: 0,
                guard_sparkno: 0,
            },
            size: CnsSize {
                xscale: 1.0,
                yscale: 1.0,
                ground_back: 15,
                ground_front: 15,
                air_back: 12,
                air_front: 12,
                height: 60,
                attack_dist: 160,
                proj_attack_dist: 90,
            },
            velocity: CnsVelocity {
                walk_fwd: Vec2::new(2.4, 0.0),
                walk_back: Vec2::new(-2.2, 0.0),
                run_fwd: Vec2::new(4.6, 0.0),
                run_back: Vec2::new(-4.5, -3.5),
                jump_neu: Vec2::new(0.0, -8.4),
                jump_back: Vec2::new(-2.55, -8.4),
                jump_fwd: Vec2::new(2.5, -8.4),
                runjump_back: Vec2::new(-2.55, -8.4),
                runjump_fwd: Vec2::new(4.0, -8.4),
                airjump_neu: Vec2::new(0.0, -8.4),
                airjump_back: Vec2::new(-2.55, -8.4),
                airjump_fwd: Vec2::new(2.5, -8.4),
            },
            movement: CnsMovement {
                airjump_num: 1,
                airjump_height: 35.0,
                yaccel: 0.44,
                stand_friction: 0.85,
                crouch_friction: 0.82,
                stand_friction_threshold: None,
                crouch_friction_threshold: None,
                air_gethit_groundlevel: None,
                air_gethit_groundrecover_ground_threshold: None,
                air_gethit_airrecover_threshold: None,
                air_gethit_airrecover_yaccel: None,
            },
            statedefs: HashMap::new(),
            global_state_controllers: Vec::new(),
        }
    }

    fn default_fighter() -> (FighterState, MugenFighterState, Position, Velocity, Health, PowerGauge)
    {
        (
            FighterState::new(),
            MugenFighterState::default(),
            Position {
                pos: LogicVec2::new(0, 0),
            },
            Velocity {
                vel: LogicVec2::new(0, 0),
            },
            Health::new(1000),
            PowerGauge::new(),
        )
    }

    #[test]
    fn test_change_state_controller() {
        let mut cns = minimal_cns();
        // State 0 with a controller: if Time >= 5, go to state 10
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Ge(
                    Box::new(TriggerExpr::Variable("time".to_string())),
                    Box::new(TriggerExpr::Int(5)),
                )]],
                controller: Controller::ChangeState {
                    value: TriggerExpr::Int(10),
                    ctrl: None,
                },
            }],
        };
        let state10 = StateDef {
            state_num: 10,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(10),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        };
        cns.statedefs.insert(0, state0);
        cns.statedefs.insert(10, state10);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();

        // Run 4 frames: should NOT transition yet (time 0..4)
        for _ in 0..4 {
            mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);
        }
        assert_eq!(fs.state_num, 0);

        // Frame 5: time = 4 at start of tick, but after 4 advance_frame calls, state_frame = 4
        // Actually: after frame 0,1,2,3 (4 calls), state_frame is now 4
        // On tick 5 (5th call), state_frame=4, Time >= 5 is false. Need one more.
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);
        // state_frame was 4 at start of this tick, which is < 5. So no transition.
        // After advance, state_frame = 5.

        // Tick 6: state_frame=5 at entry, Time >= 5 is true, transition fires
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);
        assert_eq!(fs.state_num, 10);
        assert_eq!(fs.state_frame, 1); // entered state 10, advanced once
        assert_eq!(ms.prev_state_num, 0);
    }

    #[test]
    fn test_velset_controller() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::VelSet {
                    x: Some(TriggerExpr::Int(5)),
                    y: Some(TriggerExpr::Int(-8)),
                },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert_eq!(vel.vel.x, 500); // 5 * 100
        assert_eq!(vel.vel.y, -800); // -8 * 100
    }

    #[test]
    fn test_veladd_controller() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::VelAdd {
                    x: Some(TriggerExpr::Int(3)),
                    y: None,
                },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        vel.vel.x = 200; // Start with some velocity

        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert_eq!(vel.vel.x, 500); // 200 + 300
    }

    #[test]
    fn test_posadd_controller() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::PosAdd {
                    x: Some(TriggerExpr::Int(10)),
                    y: None,
                },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert_eq!(pos.pos.x, 1000); // 10 * 100
    }

    #[test]
    fn test_posset_controller() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::PosSet {
                    x: Some(TriggerExpr::Int(50)),
                    y: Some(TriggerExpr::Int(0)),
                },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        pos.pos.x = 9999;
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert_eq!(pos.pos.x, 5000); // 50 * 100
        assert_eq!(pos.pos.y, 0);
    }

    #[test]
    fn test_varset_controller() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::VarSet {
                    var_num: 5,
                    value: TriggerExpr::Int(42),
                },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert_eq!(ms.vars[5], 42);
    }

    #[test]
    fn test_varadd_controller() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::VarAdd {
                    var_num: 3,
                    value: TriggerExpr::Int(10),
                },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        ms.vars[3] = 5;
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert_eq!(ms.vars[3], 15);
    }

    #[test]
    fn test_ctrlset_controller() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::CtrlSet {
                    value: TriggerExpr::Int(0),
                },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        assert!(ms.ctrl);
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);
        assert!(!ms.ctrl);
    }

    #[test]
    fn test_changeanim_controller() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::ChangeAnim {
                    value: TriggerExpr::Int(200),
                },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert_eq!(ms.anim_num, 200);
        assert_eq!(ms.anim_elem, 1);
        assert_eq!(ms.anim_time, 1); // anim_time advances at end of tick
    }

    #[test]
    fn test_gravity_controller() {
        let mut cns = minimal_cns();
        // yaccel = 0.44, so gravity per frame = 44 logic units
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::Gravity,
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        // 0.44 * 100 = 44
        assert_eq!(vel.vel.y, 44);
    }

    #[test]
    fn test_assert_special_controller() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::AssertSpecial {
                    flags: vec!["noshadow".to_string(), "unguardable".to_string()],
                },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert!(ms.assert_flags.contains(&"noshadow".to_string()));
        assert!(ms.assert_flags.contains(&"unguardable".to_string()));
    }

    #[test]
    fn test_triggerall_blocks_execution() {
        let mut cns = minimal_cns();
        // Controller with triggerall = ctrl (which is 1/true)
        // But triggerall requires stateno = 999, which is false
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: vec![TriggerExpr::Eq(
                    Box::new(TriggerExpr::Variable("stateno".to_string())),
                    Box::new(TriggerExpr::Int(999)),
                )],
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::VarSet {
                    var_num: 0,
                    value: TriggerExpr::Int(99),
                },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        // var(0) should NOT be set because triggerall failed
        assert_eq!(ms.vars[0], 0);
    }

    #[test]
    fn test_state_entry_applies_velset() {
        let mut cns = minimal_cns();
        // State 0: change to state 40 immediately
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::ChangeState {
                    value: TriggerExpr::Int(40),
                    ctrl: None,
                },
            }],
        };
        // State 40: has velset
        let state40 = StateDef {
            state_num: 40,
            state_type: StateTypeValue::Aerial,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Aerial),
            anim: Some(40),
            velset: Some(Vec2::new(0.0, -8.4)),
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        };
        cns.statedefs.insert(0, state0);
        cns.statedefs.insert(40, state40);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert_eq!(fs.state_num, 40);
        // velset sets (-8.4 * 100) as i32 = -839 (f32 truncation), then
        // aerial physics adds (0.44 * 100) as i32 = 44, total = -795
        let velset_y = (-8.4_f32 * 100.0) as i32;
        let gravity_y = (0.44_f32 * 100.0) as i32;
        assert_eq!(vel.vel.y, velset_y + gravity_y);
        assert!(!ms.ctrl); // ctrl=0 from statedef
    }

    #[test]
    fn test_aerial_physics_applies_gravity() {
        let mut cns = minimal_cns();
        let state41 = StateDef {
            state_num: 41,
            state_type: StateTypeValue::Aerial,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Aerial),
            anim: Some(41),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        };
        cns.statedefs.insert(41, state41);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        fs.change_state(41);
        vel.vel.y = -840; // Initial jump velocity

        // Run one frame of aerial physics
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        // yaccel = 0.44 -> 44 logic units per frame
        assert_eq!(vel.vel.y, -840 + 44);
    }

    #[test]
    fn test_standing_physics_applies_friction() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        vel.vel.x = 500;

        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        // stand_friction = 0.85 -> 85 logic units per frame of deceleration
        assert_eq!(vel.vel.x, 500 - 85);
    }

    #[test]
    fn test_global_controller_system() {
        let mut cns = minimal_cns();
        // Global controller: if command = "QCF_a" and ctrl, go to state 200
        cns.global_state_controllers = vec![StateController {
            triggerall: vec![TriggerExpr::Variable("ctrl".to_string())],
            triggers: vec![vec![TriggerExpr::Eq(
                Box::new(TriggerExpr::Variable("command".to_string())),
                Box::new(TriggerExpr::String("QCF_a".to_string())),
            )]],
            controller: Controller::ChangeState {
                value: TriggerExpr::Int(200),
                ctrl: Some(TriggerExpr::Int(0)),
            },
        }];
        let state200 = StateDef {
            state_num: 200,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Attack),
            physics: Some(Physics::Standing),
            anim: Some(200),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        };
        cns.statedefs.insert(200, state200);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        ms.active_commands = vec!["QCF_a".to_string()];
        ms.ctrl = true;

        mugen_global_controller_system(
            &cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg,
        );

        assert_eq!(fs.state_num, 200);
        assert!(!ms.ctrl);
    }

    #[test]
    fn test_no_controllers_just_advances_frame() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        assert_eq!(fs.state_frame, 0);

        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);
        assert_eq!(fs.state_frame, 1);

        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);
        assert_eq!(fs.state_frame, 2);
    }

    #[test]
    fn test_unknown_state_still_advances() {
        // If fighter is in a state not defined in CNS, the system should
        // still advance the frame counter without crashing.
        let cns = minimal_cns();
        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        fs.change_state(9999);

        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);
        assert_eq!(fs.state_num, 9999);
        assert_eq!(fs.state_frame, 1);
    }

    #[test]
    fn test_multiple_controllers_execute_in_order() {
        let mut cns = minimal_cns();
        // Two controllers: first sets var(0)=1, second sets var(1)=2
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![
                StateController {
                    triggerall: Vec::new(),
                    triggers: vec![vec![TriggerExpr::Int(1)]],
                    controller: Controller::VarSet {
                        var_num: 0,
                        value: TriggerExpr::Int(1),
                    },
                },
                StateController {
                    triggerall: Vec::new(),
                    triggers: vec![vec![TriggerExpr::Int(1)]],
                    controller: Controller::VarSet {
                        var_num: 1,
                        value: TriggerExpr::Int(2),
                    },
                },
            ],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert_eq!(ms.vars[0], 1);
        assert_eq!(ms.vars[1], 2);
    }

    #[test]
    fn test_changestate_stops_further_controllers() {
        let mut cns = minimal_cns();
        // ChangeState first, then VarSet. VarSet should NOT execute.
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![
                StateController {
                    triggerall: Vec::new(),
                    triggers: vec![vec![TriggerExpr::Int(1)]],
                    controller: Controller::ChangeState {
                        value: TriggerExpr::Int(10),
                        ctrl: None,
                    },
                },
                StateController {
                    triggerall: Vec::new(),
                    triggers: vec![vec![TriggerExpr::Int(1)]],
                    controller: Controller::VarSet {
                        var_num: 0,
                        value: TriggerExpr::Int(99),
                    },
                },
            ],
        };
        let state10 = StateDef {
            state_num: 10,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(10),
            velset: None,
            ctrl: None,
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        };
        cns.statedefs.insert(0, state0);
        cns.statedefs.insert(10, state10);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert_eq!(fs.state_num, 10);
        assert_eq!(ms.vars[0], 0); // VarSet should NOT have executed
    }

    #[test]
    fn test_mugen_tick_global_before_current() {
        let mut cns = minimal_cns();
        // Global controller: if command = "QCF_a" and ctrl, go to state 1000
        cns.global_state_controllers = vec![StateController {
            triggerall: vec![TriggerExpr::Variable("ctrl".to_string())],
            triggers: vec![vec![TriggerExpr::Eq(
                Box::new(TriggerExpr::Variable("command".to_string())),
                Box::new(TriggerExpr::String("QCF_a".to_string())),
            )]],
            controller: Controller::ChangeState {
                value: TriggerExpr::Int(1000),
                ctrl: Some(TriggerExpr::Int(0)),
            },
        }];
        // State 0: has a VarSet controller (should still run after global transition)
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        };
        let state1000 = StateDef {
            state_num: 1000,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Attack),
            physics: Some(Physics::None),
            anim: Some(1000),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        };
        cns.statedefs.insert(0, state0);
        cns.statedefs.insert(1000, state1000);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        ms.active_commands = vec!["QCF_a".to_string()];
        ms.ctrl = true;

        // Use combined tick: global controllers run first
        mugen_tick(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        // Global transition should have happened
        assert_eq!(fs.state_num, 1000);
        assert!(!ms.ctrl);
        assert_eq!(ms.prev_state_num, 0);
    }

    #[test]
    fn test_statedef_neg1_no_command_no_transition() {
        let mut cns = minimal_cns();
        // Global controller requires command = "QCF_a"
        cns.global_state_controllers = vec![StateController {
            triggerall: vec![TriggerExpr::Variable("ctrl".to_string())],
            triggers: vec![vec![TriggerExpr::Eq(
                Box::new(TriggerExpr::Variable("command".to_string())),
                Box::new(TriggerExpr::String("QCF_a".to_string())),
            )]],
            controller: Controller::ChangeState {
                value: TriggerExpr::Int(1000),
                ctrl: Some(TriggerExpr::Int(0)),
            },
        }];
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        ms.ctrl = true;
        // No active commands - should NOT transition

        mugen_tick(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert_eq!(fs.state_num, 0); // stays in state 0
    }

    #[test]
    fn test_statedef_neg1_ctrl_false_blocks() {
        let mut cns = minimal_cns();
        // Global controller requires ctrl
        cns.global_state_controllers = vec![StateController {
            triggerall: vec![TriggerExpr::Variable("ctrl".to_string())],
            triggers: vec![vec![TriggerExpr::Eq(
                Box::new(TriggerExpr::Variable("command".to_string())),
                Box::new(TriggerExpr::String("QCF_a".to_string())),
            )]],
            controller: Controller::ChangeState {
                value: TriggerExpr::Int(1000),
                ctrl: Some(TriggerExpr::Int(0)),
            },
        }];
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        ms.active_commands = vec!["QCF_a".to_string()];
        ms.ctrl = false; // ctrl is false - triggerall should fail

        mugen_tick(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert_eq!(fs.state_num, 0); // stays in state 0 because ctrl=false
    }

    #[test]
    fn test_statedef_neg1_multiple_commands() {
        let mut cns = minimal_cns();
        // Two global controllers: QCF_a -> state 1000, DP_a -> state 1100
        // QCF comes first, should take priority
        cns.global_state_controllers = vec![
            StateController {
                triggerall: vec![TriggerExpr::Variable("ctrl".to_string())],
                triggers: vec![vec![TriggerExpr::Eq(
                    Box::new(TriggerExpr::Variable("command".to_string())),
                    Box::new(TriggerExpr::String("QCF_a".to_string())),
                )]],
                controller: Controller::ChangeState {
                    value: TriggerExpr::Int(1000),
                    ctrl: Some(TriggerExpr::Int(0)),
                },
            },
            StateController {
                triggerall: vec![TriggerExpr::Variable("ctrl".to_string())],
                triggers: vec![vec![TriggerExpr::Eq(
                    Box::new(TriggerExpr::Variable("command".to_string())),
                    Box::new(TriggerExpr::String("DP_a".to_string())),
                )]],
                controller: Controller::ChangeState {
                    value: TriggerExpr::Int(1100),
                    ctrl: Some(TriggerExpr::Int(0)),
                },
            },
        ];
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        // Both commands active: QCF_a should win since it's first in the list
        ms.active_commands = vec!["QCF_a".to_string(), "DP_a".to_string()];
        ms.ctrl = true;

        mugen_tick(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert_eq!(fs.state_num, 1000); // QCF takes priority
    }

    #[test]
    fn test_playsnd_controller_emits_event() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::PlaySnd { group: 5, sound: 2 },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert_eq!(ms.pending_sounds.len(), 1);
        assert_eq!(ms.pending_sounds[0].group, 5);
        assert_eq!(ms.pending_sounds[0].sound, 2);
    }

    #[test]
    fn test_playsnd_multiple_sounds_per_frame() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![
                StateController {
                    triggerall: Vec::new(),
                    triggers: vec![vec![TriggerExpr::Int(1)]],
                    controller: Controller::PlaySnd { group: 0, sound: 0 },
                },
                StateController {
                    triggerall: Vec::new(),
                    triggers: vec![vec![TriggerExpr::Int(1)]],
                    controller: Controller::PlaySnd { group: 1, sound: 3 },
                },
            ],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert_eq!(ms.pending_sounds.len(), 2);
        assert_eq!(ms.pending_sounds[0], MugenSoundEvent { group: 0, sound: 0 });
        assert_eq!(ms.pending_sounds[1], MugenSoundEvent { group: 1, sound: 3 });
    }

    #[test]
    fn test_playsnd_cleared_on_mugen_tick() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::PlaySnd { group: 5, sound: 0 },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();

        // First tick: sound should be queued
        mugen_tick(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);
        assert_eq!(ms.pending_sounds.len(), 1);

        // Second tick: pending_sounds cleared at start, then re-queued
        mugen_tick(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);
        assert_eq!(ms.pending_sounds.len(), 1);
        assert_eq!(ms.pending_sounds[0].group, 5);
    }

    #[test]
    fn test_playsnd_from_global_controllers() {
        let mut cns = minimal_cns();
        // Add a global controller that plays a sound
        cns.global_state_controllers.push(StateController {
            triggerall: Vec::new(),
            triggers: vec![vec![TriggerExpr::Int(1)]],
            controller: Controller::PlaySnd { group: 100, sound: 1 },
        });

        // Also add a current-state controller that plays a different sound
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::PlaySnd { group: 200, sound: 2 },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        mugen_tick(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        // Both sounds should be collected: global first, then current state
        assert_eq!(ms.pending_sounds.len(), 2);
        assert_eq!(ms.pending_sounds[0], MugenSoundEvent { group: 100, sound: 1 });
        assert_eq!(ms.pending_sounds[1], MugenSoundEvent { group: 200, sound: 2 });
    }

    #[test]
    fn test_playsnd_trigger_conditional() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![
                // This PlaySnd has trigger1 = time = 0 (true on first frame)
                StateController {
                    triggerall: Vec::new(),
                    triggers: vec![vec![TriggerExpr::Eq(
                        Box::new(TriggerExpr::Variable("time".to_string())),
                        Box::new(TriggerExpr::Int(0)),
                    )]],
                    controller: Controller::PlaySnd { group: 10, sound: 0 },
                },
                // This PlaySnd has trigger1 = time = 5 (false on first frame)
                StateController {
                    triggerall: Vec::new(),
                    triggers: vec![vec![TriggerExpr::Eq(
                        Box::new(TriggerExpr::Variable("time".to_string())),
                        Box::new(TriggerExpr::Int(5)),
                    )]],
                    controller: Controller::PlaySnd { group: 20, sound: 0 },
                },
            ],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        // Only the first PlaySnd fires (time == 0)
        assert_eq!(ms.pending_sounds.len(), 1);
        assert_eq!(ms.pending_sounds[0].group, 10);
    }

    #[test]
    fn test_hitdef_controller_sets_active_hitdef() {
        let mut cns = minimal_cns();
        let state200 = StateDef {
            state_num: 200,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Attack),
            physics: Some(Physics::None),
            anim: Some(200),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::HitDef {
                    attr: "S.NA".to_string(),
                    hitflag: "MAF".to_string(),
                    guardflag: "MA".to_string(),
                    animtype: "light".to_string(),
                    air_type: String::new(),
                    ground_type: "high".to_string(),
                    damage: 50,
                    guard_damage: 0,
                    pausetime_p1: 8,
                    pausetime_p2: 8,
                    sparkno: None,
                    guard_sparkno: None,
                    spark_xy: (0, 0),
                    hit_sound: None,
                    guard_sound: None,
                    ground_slidetime: 5,
                    guard_slidetime: 5,
                    ground_hittime: 12,
                    guard_hittime: 10,
                    air_hittime: 18,
                    ground_velocity_x: -3.0,
                    ground_velocity_y: 0.0,
                    air_velocity_x: -2.5,
                    air_velocity_y: -4.0,
                    guard_velocity_x: -5.0,
                    down_velocity_x: 0.0,
                    down_velocity_y: 0.0,
                    yaccel: 0.0,
                    getpower_hit: 30,
                    getpower_guard: 10,
                    fall: false,
                    fall_recover: true,
                    priority: 4,
                    p1stateno: None,
                    p2stateno: None,
                },
            }],
        };
        cns.statedefs.insert(200, state200);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        fs.change_state(200);

        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        let hitdef = ms.active_hitdef.as_ref().expect("HitDef should be active");
        assert_eq!(hitdef.damage, 50);
        assert_eq!(hitdef.ground_velocity_x, -300);
        assert_eq!(hitdef.ground_hittime, 12);
        assert_eq!(hitdef.air_hittime, 18);
        assert_eq!(hitdef.getpower_hit, 30);
        assert!(!hitdef.fall);
        assert!(!hitdef.has_hit);
    }

    #[test]
    fn test_hitdef_clears_move_flags() {
        let mut cns = minimal_cns();
        let state200 = StateDef {
            state_num: 200,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Attack),
            physics: Some(Physics::None),
            anim: Some(200),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::HitDef {
                    attr: "S.NA".to_string(),
                    hitflag: "MAF".to_string(),
                    guardflag: "MA".to_string(),
                    animtype: "light".to_string(),
                    air_type: String::new(),
                    ground_type: "high".to_string(),
                    damage: 50,
                    guard_damage: 0,
                    pausetime_p1: 0,
                    pausetime_p2: 0,
                    sparkno: None,
                    guard_sparkno: None,
                    spark_xy: (0, 0),
                    hit_sound: None,
                    guard_sound: None,
                    ground_slidetime: 0,
                    guard_slidetime: 0,
                    ground_hittime: 12,
                    guard_hittime: 10,
                    air_hittime: 18,
                    ground_velocity_x: -3.0,
                    ground_velocity_y: 0.0,
                    air_velocity_x: -2.5,
                    air_velocity_y: -4.0,
                    guard_velocity_x: -5.0,
                    down_velocity_x: 0.0,
                    down_velocity_y: 0.0,
                    yaccel: 0.0,
                    getpower_hit: 20,
                    getpower_guard: 5,
                    fall: false,
                    fall_recover: true,
                    priority: 4,
                    p1stateno: None,
                    p2stateno: None,
                },
            }],
        };
        cns.statedefs.insert(200, state200);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        fs.change_state(200);
        ms.move_hit = true;
        ms.move_contact = true;

        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        assert!(!ms.move_hit);
        assert!(!ms.move_contact);
        assert!(!ms.move_guarded);
    }

    #[test]
    fn test_hitdef_collision_distance_based() {
        let (mut fs_a, mut ms_a, _, mut vel_a, mut hp_a, mut pg_a) = default_fighter();
        let (mut fs_d, mut ms_d, _, mut vel_d, mut hp_d, mut pg_d) = default_fighter();
        let facing_a = Facing { dir: Facing::RIGHT };
        let facing_d = Facing { dir: Facing::LEFT };

        let pos_a = Position {
            pos: LogicVec2::new(0, 0),
        };
        let pos_d = Position {
            pos: LogicVec2::new(15000, 0),
        };

        fs_a.change_state(200);
        ms_a.anim_num = 200;
        ms_a.active_hitdef = Some(ActiveHitDef {
            damage: 50,
            ground_velocity_x: -300,
            ground_hittime: 12,
            getpower_hit: 20,
            ..ActiveHitDef::default()
        });

        let result = mugen_hitdef_collision(
            &mut MugenCollisionFighter {
                position: &pos_a,
                velocity: &mut vel_a,
                facing: &facing_a,
                fighter_state: &mut fs_a,
                mugen: &mut ms_a,
                health: &mut hp_a,
                power: &mut pg_a,
            },
            &mut MugenCollisionFighter {
                position: &pos_d,
                velocity: &mut vel_d,
                facing: &facing_d,
                fighter_state: &mut fs_d,
                mugen: &mut ms_d,
                health: &mut hp_d,
                power: &mut pg_d,
            },
            None,
            160,
        );

        let hit = result.expect("Hit should connect");
        assert_eq!(hit.damage, 50);
        assert_eq!(hp_d.current, 1000 - 50);
        assert!(ms_a.move_hit);
        assert!(!ms_d.ctrl);
        assert_eq!(vel_d.vel.x, -300);
        assert_eq!(fs_d.state_num, STATE_HIT_STAND_LIGHT);
        assert_eq!(pg_a.current, 20);
    }

    #[test]
    fn test_hitdef_collision_too_far() {
        let (mut fs_a, mut ms_a, _, mut vel_a, mut hp_a, mut pg_a) = default_fighter();
        let (mut fs_d, mut ms_d, _, mut vel_d, mut hp_d, mut pg_d) = default_fighter();
        let facing_a = Facing { dir: Facing::RIGHT };
        let facing_d = Facing { dir: Facing::LEFT };

        let pos_a = Position {
            pos: LogicVec2::new(0, 0),
        };
        let pos_d = Position {
            pos: LogicVec2::new(20000, 0),
        };

        ms_a.active_hitdef = Some(ActiveHitDef {
            damage: 50,
            ..ActiveHitDef::default()
        });

        let result = mugen_hitdef_collision(
            &mut MugenCollisionFighter {
                position: &pos_a,
                velocity: &mut vel_a,
                facing: &facing_a,
                fighter_state: &mut fs_a,
                mugen: &mut ms_a,
                health: &mut hp_a,
                power: &mut pg_a,
            },
            &mut MugenCollisionFighter {
                position: &pos_d,
                velocity: &mut vel_d,
                facing: &facing_d,
                fighter_state: &mut fs_d,
                mugen: &mut ms_d,
                health: &mut hp_d,
                power: &mut pg_d,
            },
            None,
            160,
        );

        assert!(result.is_none());
        assert!(!ms_a.move_hit);
    }

    #[test]
    fn test_hitdef_no_multi_hit() {
        let (mut fs_a, mut ms_a, _, mut vel_a, mut hp_a, mut pg_a) = default_fighter();
        let (mut fs_d, mut ms_d, _, mut vel_d, mut hp_d, mut pg_d) = default_fighter();
        let facing_a = Facing { dir: Facing::RIGHT };
        let facing_d = Facing { dir: Facing::LEFT };

        let pos_a = Position {
            pos: LogicVec2::new(0, 0),
        };
        let pos_d = Position {
            pos: LogicVec2::new(10000, 0),
        };

        ms_a.active_hitdef = Some(ActiveHitDef {
            damage: 50,
            has_hit: true,
            ..ActiveHitDef::default()
        });

        let result = mugen_hitdef_collision(
            &mut MugenCollisionFighter {
                position: &pos_a,
                velocity: &mut vel_a,
                facing: &facing_a,
                fighter_state: &mut fs_a,
                mugen: &mut ms_a,
                health: &mut hp_a,
                power: &mut pg_a,
            },
            &mut MugenCollisionFighter {
                position: &pos_d,
                velocity: &mut vel_d,
                facing: &facing_d,
                fighter_state: &mut fs_d,
                mugen: &mut ms_d,
                health: &mut hp_d,
                power: &mut pg_d,
            },
            None,
            160,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_hitdef_fall_launches() {
        let (mut fs_a, mut ms_a, _, mut vel_a, mut hp_a, mut pg_a) = default_fighter();
        let (mut fs_d, mut ms_d, _, mut vel_d, mut hp_d, mut pg_d) = default_fighter();
        let facing_a = Facing { dir: Facing::RIGHT };
        let facing_d = Facing { dir: Facing::LEFT };

        let pos_a = Position {
            pos: LogicVec2::new(0, 0),
        };
        let pos_d = Position {
            pos: LogicVec2::new(10000, 0),
        };

        ms_a.active_hitdef = Some(ActiveHitDef {
            damage: 100,
            ground_velocity_x: -200,
            ground_velocity_y: -800,
            air_hittime: 30,
            fall: true,
            ..ActiveHitDef::default()
        });

        let result = mugen_hitdef_collision(
            &mut MugenCollisionFighter {
                position: &pos_a,
                velocity: &mut vel_a,
                facing: &facing_a,
                fighter_state: &mut fs_a,
                mugen: &mut ms_a,
                health: &mut hp_a,
                power: &mut pg_a,
            },
            &mut MugenCollisionFighter {
                position: &pos_d,
                velocity: &mut vel_d,
                facing: &facing_d,
                fighter_state: &mut fs_d,
                mugen: &mut ms_d,
                health: &mut hp_d,
                power: &mut pg_d,
            },
            None,
            160,
        );

        let hit = result.expect("Hit should connect");
        assert_eq!(hit.damage, 100);
        assert_eq!(fs_d.state_num, STATE_HIT_AIR_LIGHT);
        assert_eq!(vel_d.vel.y, -800);
    }

    #[test]
    fn test_hitdef_no_active_hitdef_returns_none() {
        let (mut fs_a, mut ms_a, _, mut vel_a, mut hp_a, mut pg_a) = default_fighter();
        let (mut fs_d, mut ms_d, _, mut vel_d, mut hp_d, mut pg_d) = default_fighter();
        let facing_a = Facing { dir: Facing::RIGHT };
        let facing_d = Facing { dir: Facing::LEFT };

        let pos_a = Position {
            pos: LogicVec2::new(0, 0),
        };
        let pos_d = Position {
            pos: LogicVec2::new(10000, 0),
        };

        assert!(ms_a.active_hitdef.is_none());

        let result = mugen_hitdef_collision(
            &mut MugenCollisionFighter {
                position: &pos_a,
                velocity: &mut vel_a,
                facing: &facing_a,
                fighter_state: &mut fs_a,
                mugen: &mut ms_a,
                health: &mut hp_a,
                power: &mut pg_a,
            },
            &mut MugenCollisionFighter {
                position: &pos_d,
                velocity: &mut vel_d,
                facing: &facing_d,
                fighter_state: &mut fs_d,
                mugen: &mut ms_d,
                health: &mut hp_d,
                power: &mut pg_d,
            },
            None,
            160,
        );

        assert!(result.is_none());
    }

    #[test]
    fn test_hitdef_to_hitbox_conversion() {
        let hitdef = ActiveHitDef {
            damage: 80,
            ground_velocity_x: -400,
            ground_velocity_y: -200,
            ground_hittime: 15,
            guard_hittime: 10,
            attr: "S.NA".to_string(),
            ..ActiveHitDef::default()
        };

        let rect = LogicRect::new(0, -5000, 4000, 3000);
        let hitbox = hitdef_to_hitbox(&hitdef, rect);

        assert_eq!(hitbox.damage, 80);
        assert_eq!(hitbox.hitstun, 15);
        assert_eq!(hitbox.blockstun, 10);
        assert_eq!(hitbox.knockback, LogicVec2::new(-400, -200));
        assert_eq!(hitbox.rect, rect);
    }

    #[test]
    fn test_varrandom_stores_value_in_var() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::VarRandom {
                    var_num: 5,
                    range_min: 0,
                    range_max: 999,
                },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        assert_eq!(ms.vars[5], 0);
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);

        // After execution, var(5) should be set to a value in [0, 999]
        assert!(ms.vars[5] >= 0 && ms.vars[5] <= 999,
            "var(5) = {} should be in [0, 999]", ms.vars[5]);
    }

    #[test]
    fn test_varrandom_respects_range() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::VarRandom {
                    var_num: 3,
                    range_min: 10,
                    range_max: 20,
                },
            }],
        };
        cns.statedefs.insert(0, state0);

        // Run multiple ticks and check all values fall in [10, 20]
        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        for _ in 0..50 {
            ms.vars[3] = -1; // reset to detect change
            mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);
            assert!(ms.vars[3] >= 10 && ms.vars[3] <= 20,
                "var(3) = {} should be in [10, 20]", ms.vars[3]);
        }
    }

    #[test]
    fn test_varrandom_deterministic_with_same_seed() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::VarRandom {
                    var_num: 0,
                    range_min: 0,
                    range_max: 999,
                },
            }],
        };
        cns.statedefs.insert(0, state0);

        // Two fighters with the same initial rng_state should produce the same sequence
        let (mut fs1, mut ms1, mut pos1, mut vel1, mut hp1, mut pg1) = default_fighter();
        let (mut fs2, mut ms2, mut pos2, mut vel2, mut hp2, mut pg2) = default_fighter();

        let mut seq1 = Vec::new();
        let mut seq2 = Vec::new();
        for _ in 0..20 {
            mugen_controller_system(&cns, &mut fs1, &mut ms1, &mut pos1, &mut vel1, &mut hp1, &mut pg1);
            seq1.push(ms1.vars[0]);
            mugen_controller_system(&cns, &mut fs2, &mut ms2, &mut pos2, &mut vel2, &mut hp2, &mut pg2);
            seq2.push(ms2.vars[0]);
        }
        assert_eq!(seq1, seq2, "Same seed should produce identical sequences");
    }

    #[test]
    fn test_varrandom_different_seeds_differ() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::VarRandom {
                    var_num: 0,
                    range_min: 0,
                    range_max: 999,
                },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs1, mut ms1, mut pos1, mut vel1, mut hp1, mut pg1) = default_fighter();
        let (mut fs2, mut ms2, mut pos2, mut vel2, mut hp2, mut pg2) = default_fighter();
        ms2.rng_state = 12345; // different seed

        let mut seq1 = Vec::new();
        let mut seq2 = Vec::new();
        for _ in 0..20 {
            mugen_controller_system(&cns, &mut fs1, &mut ms1, &mut pos1, &mut vel1, &mut hp1, &mut pg1);
            seq1.push(ms1.vars[0]);
            mugen_controller_system(&cns, &mut fs2, &mut ms2, &mut pos2, &mut vel2, &mut hp2, &mut pg2);
            seq2.push(ms2.vars[0]);
        }
        assert_ne!(seq1, seq2, "Different seeds should produce different sequences");
    }

    // ── Combat Integration Tests ─────────────────────────────────────────────

    fn make_collision_fighter<'a>(
        pos: &'a Position,
        vel: &'a mut Velocity,
        facing: &'a Facing,
        fs: &'a mut FighterState,
        ms: &'a mut MugenFighterState,
        hp: &'a mut Health,
        pg: &'a mut PowerGauge,
    ) -> MugenCollisionFighter<'a> {
        MugenCollisionFighter {
            position: pos,
            velocity: vel,
            facing,
            fighter_state: fs,
            mugen: ms,
            health: hp,
            power: pg,
        }
    }

    #[test]
    fn test_combat_frame_hit_deals_damage() {
        let pos1 = Position { pos: LogicVec2::new(0, 0) };
        let mut vel1 = Velocity { vel: LogicVec2::ZERO };
        let facing1 = Facing { dir: Facing::RIGHT };
        let mut fs1 = FighterState::new();
        fs1.change_state(STATE_ATTACK_LIGHT);
        let mut ms1 = MugenFighterState::default();
        ms1.active_hitdef = Some(ActiveHitDef {
            damage: 100,
            ground_velocity_x: -500,
            ground_hittime: 12,
            ..ActiveHitDef::default()
        });
        let mut hp1 = Health::new(10000);
        let mut pg1 = PowerGauge::new();

        let pos2 = Position { pos: LogicVec2::new(5000, 0) };
        let mut vel2 = Velocity { vel: LogicVec2::ZERO };
        let facing2 = Facing { dir: Facing::LEFT };
        let mut fs2 = FighterState::new();
        let mut ms2 = MugenFighterState::default();
        let mut hp2 = Health::new(10000);
        let mut pg2 = PowerGauge::new();

        let mut p1 = make_collision_fighter(&pos1, &mut vel1, &facing1, &mut fs1, &mut ms1, &mut hp1, &mut pg1);
        let mut p2 = make_collision_fighter(&pos2, &mut vel2, &facing2, &mut fs2, &mut ms2, &mut hp2, &mut pg2);

        let result = mugen_combat_frame(&mut p1, &mut p2, None, 160);

        assert!(result.p1_hit.is_some(), "P1 should have hit P2");
        assert_eq!(hp2.current, 9900); // 100 damage
        assert!(is_hit_state(fs2.state_num), "P2 should be in hitstun");
        assert!(ms1.move_hit, "P1 move_hit should be set");
        assert_eq!(ms2.hitstun_remaining, 12);
    }

    #[test]
    fn test_combat_frame_guard_reduces_damage() {
        let pos1 = Position { pos: LogicVec2::new(0, 0) };
        let mut vel1 = Velocity { vel: LogicVec2::ZERO };
        let facing1 = Facing { dir: Facing::RIGHT };
        let mut fs1 = FighterState::new();
        fs1.change_state(STATE_ATTACK_LIGHT);
        let mut ms1 = MugenFighterState::default();
        ms1.active_hitdef = Some(ActiveHitDef {
            damage: 100,
            guard_damage: 10,
            guard_velocity_x: -200,
            guard_hittime: 8,
            ground_hittime: 15,
            ground_velocity_x: -500,
            ..ActiveHitDef::default()
        });
        let mut hp1 = Health::new(10000);
        let mut pg1 = PowerGauge::new();

        let pos2 = Position { pos: LogicVec2::new(5000, 0) };
        let mut vel2 = Velocity { vel: LogicVec2::ZERO };
        let facing2 = Facing { dir: Facing::LEFT };
        let mut fs2 = FighterState::new();
        fs2.change_state(STATE_GUARD_STAND); // Defender is blocking
        let mut ms2 = MugenFighterState::default();
        let mut hp2 = Health::new(10000);
        let mut pg2 = PowerGauge::new();

        let mut p1 = make_collision_fighter(&pos1, &mut vel1, &facing1, &mut fs1, &mut ms1, &mut hp1, &mut pg1);
        let mut p2 = make_collision_fighter(&pos2, &mut vel2, &facing2, &mut fs2, &mut ms2, &mut hp2, &mut pg2);

        let result = mugen_combat_frame(&mut p1, &mut p2, None, 160);

        assert!(result.p1_hit.is_some());
        // Guard damage should be 10 (not 100)
        assert_eq!(hp2.current, 9990);
        // Attacker should have move_guarded set, not move_hit
        assert!(ms1.move_guarded);
        assert!(!ms1.move_hit);
        assert!(ms1.move_contact);
        // Blockstun should be set
        assert_eq!(ms2.blockstun_remaining, 8);
    }

    #[test]
    fn test_combat_frame_combo_scaling() {
        let pos1 = Position { pos: LogicVec2::new(0, 0) };
        let mut vel1 = Velocity { vel: LogicVec2::ZERO };
        let facing1 = Facing { dir: Facing::RIGHT };
        let mut fs1 = FighterState::new();
        fs1.change_state(STATE_ATTACK_LIGHT);
        let mut ms1 = MugenFighterState::default();
        ms1.combo_count = 4; // 4th hit -> 70% scaling
        ms1.active_hitdef = Some(ActiveHitDef {
            damage: 1000,
            ground_velocity_x: -500,
            ground_hittime: 12,
            ..ActiveHitDef::default()
        });
        let mut hp1 = Health::new(10000);
        let mut pg1 = PowerGauge::new();

        let pos2 = Position { pos: LogicVec2::new(5000, 0) };
        let mut vel2 = Velocity { vel: LogicVec2::ZERO };
        let facing2 = Facing { dir: Facing::LEFT };
        let mut fs2 = FighterState::new();
        let mut ms2 = MugenFighterState::default();
        let mut hp2 = Health::new(10000);
        let mut pg2 = PowerGauge::new();

        let mut p1 = make_collision_fighter(&pos1, &mut vel1, &facing1, &mut fs1, &mut ms1, &mut hp1, &mut pg1);
        let mut p2 = make_collision_fighter(&pos2, &mut vel2, &facing2, &mut fs2, &mut ms2, &mut hp2, &mut pg2);

        mugen_combat_frame(&mut p1, &mut p2, None, 160);

        // Full damage is 1000 but combo_count=4 gives 70% scaling
        // mugen_hitdef_collision applies 1000 first, then combat_frame heals back 300
        assert_eq!(hp2.current, 9300); // 10000 - 700
        assert_eq!(ms1.combo_count, 5); // Incremented from 4 to 5
    }

    #[test]
    fn test_hitstun_countdown_recovery() {
        let pos = Position { pos: LogicVec2::new(0, 0) };
        let mut vel = Velocity { vel: LogicVec2::ZERO };
        let facing = Facing { dir: Facing::RIGHT };
        let mut fs = FighterState::new();
        fs.change_state(STATE_HIT_STAND_LIGHT);
        let mut ms = MugenFighterState::default();
        ms.hitstun_remaining = 3;
        ms.ctrl = false;
        let mut hp = Health::new(10000);
        let mut pg = PowerGauge::new();

        // Tick 1: hitstun 3 -> 2
        {
            let mut fighter = make_collision_fighter(&pos, &mut vel, &facing, &mut fs, &mut ms, &mut hp, &mut pg);
            tick_stun_timers(&mut fighter);
        }
        assert_eq!(ms.hitstun_remaining, 2);
        assert!(is_hit_state(fs.state_num));

        // Tick 2: hitstun 2 -> 1
        {
            let mut fighter = make_collision_fighter(&pos, &mut vel, &facing, &mut fs, &mut ms, &mut hp, &mut pg);
            tick_stun_timers(&mut fighter);
        }
        assert_eq!(ms.hitstun_remaining, 1);

        // Tick 3: hitstun 1 -> 0, recover to stand
        {
            let mut fighter = make_collision_fighter(&pos, &mut vel, &facing, &mut fs, &mut ms, &mut hp, &mut pg);
            tick_stun_timers(&mut fighter);
        }
        assert_eq!(ms.hitstun_remaining, 0);
        assert_eq!(fs.state_num, STATE_STAND);
        assert!(ms.ctrl);
    }

    #[test]
    fn test_blockstun_countdown_recovery() {
        let pos = Position { pos: LogicVec2::new(0, 0) };
        let mut vel = Velocity { vel: LogicVec2::ZERO };
        let facing = Facing { dir: Facing::RIGHT };
        let mut fs = FighterState::new();
        fs.change_state(STATE_GUARD_STAND);
        let mut ms = MugenFighterState::default();
        ms.blockstun_remaining = 2;
        ms.ctrl = false;
        let mut hp = Health::new(10000);
        let mut pg = PowerGauge::new();

        // Tick 1: blockstun 2 -> 1
        {
            let mut fighter = make_collision_fighter(&pos, &mut vel, &facing, &mut fs, &mut ms, &mut hp, &mut pg);
            tick_stun_timers(&mut fighter);
        }
        assert_eq!(ms.blockstun_remaining, 1);

        // Tick 2: blockstun 1 -> 0, recover to guard end
        {
            let mut fighter = make_collision_fighter(&pos, &mut vel, &facing, &mut fs, &mut ms, &mut hp, &mut pg);
            tick_stun_timers(&mut fighter);
        }
        assert_eq!(ms.blockstun_remaining, 0);
        assert_eq!(fs.state_num, STATE_GUARD_END);
        assert!(ms.ctrl);
    }

    #[test]
    fn test_combo_counter_reset_on_recovery() {
        let mut ms = MugenFighterState::default();
        ms.combo_count = 5;

        // Defender still in hit state - don't reset
        let defender_state = FighterState { state_num: STATE_HIT_STAND_LIGHT, state_frame: 0 };
        reset_combo_if_recovered(&mut ms, &defender_state);
        assert_eq!(ms.combo_count, 5);

        // Defender recovered to stand - reset
        let defender_state = FighterState { state_num: STATE_STAND, state_frame: 0 };
        reset_combo_if_recovered(&mut ms, &defender_state);
        assert_eq!(ms.combo_count, 0);
    }

    #[test]
    fn test_populate_p2_context() {
        let (fs, ms, pos, vel, hp, _pg) = default_fighter();
        let fighter_pos = Position { pos: LogicVec2::new(10000, 0) };

        let mut ctx = TriggerContext::default();
        populate_p2_context(
            &mut ctx,
            &fs, &ms, &pos, &vel, &hp, &fighter_pos,
        );

        assert_eq!(ctx.p2_stateno, 0); // STATE_STAND
        assert_eq!(ctx.p2_life, 1000);
        assert_eq!(ctx.p2_bodydist_x, 10000); // |10000 - 0|
        assert_eq!(ctx.p2_statetype, 0); // Standing
        assert_eq!(ctx.p2_movetype, 0); // Idle
        assert!(ctx.p2_ctrl);
    }

    #[test]
    fn test_no_hit_when_no_active_hitdef() {
        let pos1 = Position { pos: LogicVec2::new(0, 0) };
        let mut vel1 = Velocity { vel: LogicVec2::ZERO };
        let facing1 = Facing { dir: Facing::RIGHT };
        let mut fs1 = FighterState::new();
        let mut ms1 = MugenFighterState::default();
        // No active_hitdef
        let mut hp1 = Health::new(10000);
        let mut pg1 = PowerGauge::new();

        let pos2 = Position { pos: LogicVec2::new(5000, 0) };
        let mut vel2 = Velocity { vel: LogicVec2::ZERO };
        let facing2 = Facing { dir: Facing::LEFT };
        let mut fs2 = FighterState::new();
        let mut ms2 = MugenFighterState::default();
        let mut hp2 = Health::new(10000);
        let mut pg2 = PowerGauge::new();

        let mut p1 = make_collision_fighter(&pos1, &mut vel1, &facing1, &mut fs1, &mut ms1, &mut hp1, &mut pg1);
        let mut p2 = make_collision_fighter(&pos2, &mut vel2, &facing2, &mut fs2, &mut ms2, &mut hp2, &mut pg2);

        let result = mugen_combat_frame(&mut p1, &mut p2, None, 160);

        assert!(result.p1_hit.is_none());
        assert!(result.p2_hit.is_none());
        assert_eq!(hp2.current, 10000); // No damage
    }

    // ── attack_attr_matches unit tests ──────────────────────────────────────

    #[test]
    fn test_attr_matches_exact() {
        assert!(attack_attr_matches("S,NA", "S,NA"));
        assert!(attack_attr_matches("C,SA", "C,SA"));
        assert!(attack_attr_matches("A,HA", "A,HA"));
    }

    #[test]
    fn test_attr_matches_multi_state() {
        // "SCA,NA" should match standing, crouching, and aerial normal attacks
        assert!(attack_attr_matches("SCA,NA", "S,NA"));
        assert!(attack_attr_matches("SCA,NA", "C,NA"));
        assert!(attack_attr_matches("SCA,NA", "A,NA"));
        // But not special attacks
        assert!(!attack_attr_matches("SCA,NA", "S,SA"));
    }

    #[test]
    fn test_attr_matches_multi_attack() {
        // "S,NA SA" should match normal or special standing attacks
        assert!(attack_attr_matches("S,NA SA", "S,NA"));
        assert!(attack_attr_matches("S,NA SA", "S,SA"));
        // But not hyper or aerial
        assert!(!attack_attr_matches("S,NA SA", "S,HA"));
        assert!(!attack_attr_matches("S,NA SA", "A,NA"));
    }

    #[test]
    fn test_attr_no_match_state_mismatch() {
        assert!(!attack_attr_matches("S,NA", "C,NA"));
        assert!(!attack_attr_matches("S,NA", "A,NA"));
    }

    #[test]
    fn test_attr_no_match_attack_mismatch() {
        assert!(!attack_attr_matches("S,NA", "S,SA"));
        assert!(!attack_attr_matches("S,NA", "S,HA"));
    }

    #[test]
    fn test_attr_invalid_format_returns_false() {
        assert!(!attack_attr_matches("SNA", "S,NA")); // no comma in filter
        assert!(!attack_attr_matches("S,NA", "SNA")); // no comma in attr
        assert!(!attack_attr_matches("", "S,NA"));
        assert!(!attack_attr_matches("S,NA", ""));
    }

    #[test]
    fn test_attr_case_insensitive() {
        assert!(attack_attr_matches("s,na", "S,NA"));
        assert!(attack_attr_matches("S,NA", "s,na"));
        assert!(attack_attr_matches("sca,na", "A,NA"));
    }

    // ── NotHitBy/HitBy controller + invincibility tests ──────────────────────

    #[test]
    fn test_not_hit_by_blocks_matching_attack() {
        let pos1 = Position { pos: LogicVec2::new(0, 0) };
        let mut vel1 = Velocity { vel: LogicVec2::ZERO };
        let facing1 = Facing { dir: Facing::RIGHT };
        let mut fs1 = FighterState::new();
        let mut ms1 = MugenFighterState::default();
        ms1.active_hitdef = Some(ActiveHitDef {
            damage: 100,
            attr: "S,NA".to_string(),
            ground_hittime: 12,
            ..ActiveHitDef::default()
        });
        let mut hp1 = Health::new(10000);
        let mut pg1 = PowerGauge::new();

        let pos2 = Position { pos: LogicVec2::new(5000, 0) };
        let mut vel2 = Velocity { vel: LogicVec2::ZERO };
        let facing2 = Facing { dir: Facing::LEFT };
        let mut fs2 = FighterState::new();
        let mut ms2 = MugenFighterState::default();
        // Defender is invincible to standing normal attacks
        ms2.not_hit_by = Some(("S,NA".to_string(), 10));
        let mut hp2 = Health::new(10000);
        let mut pg2 = PowerGauge::new();

        let mut p1 = make_collision_fighter(&pos1, &mut vel1, &facing1, &mut fs1, &mut ms1, &mut hp1, &mut pg1);
        let mut p2 = make_collision_fighter(&pos2, &mut vel2, &facing2, &mut fs2, &mut ms2, &mut hp2, &mut pg2);

        let result = mugen_hitdef_collision(&mut p1, &mut p2, None, 160);

        assert!(result.is_none(), "NotHitBy should block the hit");
        assert_eq!(hp2.current, 10000, "No damage should be applied");
    }

    #[test]
    fn test_not_hit_by_allows_non_matching_attack() {
        let pos1 = Position { pos: LogicVec2::new(0, 0) };
        let mut vel1 = Velocity { vel: LogicVec2::ZERO };
        let facing1 = Facing { dir: Facing::RIGHT };
        let mut fs1 = FighterState::new();
        let mut ms1 = MugenFighterState::default();
        ms1.active_hitdef = Some(ActiveHitDef {
            damage: 100,
            attr: "S,SA".to_string(), // special attack
            ground_hittime: 12,
            ..ActiveHitDef::default()
        });
        let mut hp1 = Health::new(10000);
        let mut pg1 = PowerGauge::new();

        let pos2 = Position { pos: LogicVec2::new(5000, 0) };
        let mut vel2 = Velocity { vel: LogicVec2::ZERO };
        let facing2 = Facing { dir: Facing::LEFT };
        let mut fs2 = FighterState::new();
        let mut ms2 = MugenFighterState::default();
        // Invincible only to normal attacks, NOT to special attacks
        ms2.not_hit_by = Some(("S,NA".to_string(), 10));
        let mut hp2 = Health::new(10000);
        let mut pg2 = PowerGauge::new();

        let mut p1 = make_collision_fighter(&pos1, &mut vel1, &facing1, &mut fs1, &mut ms1, &mut hp1, &mut pg1);
        let mut p2 = make_collision_fighter(&pos2, &mut vel2, &facing2, &mut fs2, &mut ms2, &mut hp2, &mut pg2);

        let result = mugen_hitdef_collision(&mut p1, &mut p2, None, 160);

        assert!(result.is_some(), "Special attack should pass through normal-attack invincibility");
        assert!(hp2.current < 10000, "Damage should be applied");
    }

    #[test]
    fn test_hit_by_allows_matching_attack() {
        let pos1 = Position { pos: LogicVec2::new(0, 0) };
        let mut vel1 = Velocity { vel: LogicVec2::ZERO };
        let facing1 = Facing { dir: Facing::RIGHT };
        let mut fs1 = FighterState::new();
        let mut ms1 = MugenFighterState::default();
        ms1.active_hitdef = Some(ActiveHitDef {
            damage: 100,
            attr: "S,SA".to_string(), // special attack
            ground_hittime: 12,
            ..ActiveHitDef::default()
        });
        let mut hp1 = Health::new(10000);
        let mut pg1 = PowerGauge::new();

        let pos2 = Position { pos: LogicVec2::new(5000, 0) };
        let mut vel2 = Velocity { vel: LogicVec2::ZERO };
        let facing2 = Facing { dir: Facing::LEFT };
        let mut fs2 = FighterState::new();
        let mut ms2 = MugenFighterState::default();
        // Can only be hit by special attacks
        ms2.hit_by = Some(("S,SA".to_string(), 10));
        let mut hp2 = Health::new(10000);
        let mut pg2 = PowerGauge::new();

        let mut p1 = make_collision_fighter(&pos1, &mut vel1, &facing1, &mut fs1, &mut ms1, &mut hp1, &mut pg1);
        let mut p2 = make_collision_fighter(&pos2, &mut vel2, &facing2, &mut fs2, &mut ms2, &mut hp2, &mut pg2);

        let result = mugen_hitdef_collision(&mut p1, &mut p2, None, 160);

        assert!(result.is_some(), "Matching HitBy should allow the hit");
        assert!(hp2.current < 10000);
    }

    #[test]
    fn test_hit_by_blocks_non_matching_attack() {
        let pos1 = Position { pos: LogicVec2::new(0, 0) };
        let mut vel1 = Velocity { vel: LogicVec2::ZERO };
        let facing1 = Facing { dir: Facing::RIGHT };
        let mut fs1 = FighterState::new();
        let mut ms1 = MugenFighterState::default();
        ms1.active_hitdef = Some(ActiveHitDef {
            damage: 100,
            attr: "S,NA".to_string(), // normal attack
            ground_hittime: 12,
            ..ActiveHitDef::default()
        });
        let mut hp1 = Health::new(10000);
        let mut pg1 = PowerGauge::new();

        let pos2 = Position { pos: LogicVec2::new(5000, 0) };
        let mut vel2 = Velocity { vel: LogicVec2::ZERO };
        let facing2 = Facing { dir: Facing::LEFT };
        let mut fs2 = FighterState::new();
        let mut ms2 = MugenFighterState::default();
        // Can only be hit by special attacks — normal attack should not connect
        ms2.hit_by = Some(("S,SA".to_string(), 10));
        let mut hp2 = Health::new(10000);
        let mut pg2 = PowerGauge::new();

        let mut p1 = make_collision_fighter(&pos1, &mut vel1, &facing1, &mut fs1, &mut ms1, &mut hp1, &mut pg1);
        let mut p2 = make_collision_fighter(&pos2, &mut vel2, &facing2, &mut fs2, &mut ms2, &mut hp2, &mut pg2);

        let result = mugen_hitdef_collision(&mut p1, &mut p2, None, 160);

        assert!(result.is_none(), "Non-matching HitBy should block the hit");
        assert_eq!(hp2.current, 10000, "No damage should be applied");
    }

    #[test]
    fn test_not_hit_by_controller_sets_field() {
        let mut cns = minimal_cns();
        let state0 = StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::None),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: vec![StateController {
                triggerall: Vec::new(),
                triggers: vec![vec![TriggerExpr::Int(1)]],
                controller: Controller::NotHitBy {
                    attr: "SCA,NA".to_string(),
                    time: 5,
                },
            }],
        };
        cns.statedefs.insert(0, state0);

        let (mut fs, mut ms, mut pos, mut vel, mut hp, mut pg) = default_fighter();
        assert!(ms.not_hit_by.is_none());
        mugen_controller_system(&cns, &mut fs, &mut ms, &mut pos, &mut vel, &mut hp, &mut pg);
        assert!(ms.not_hit_by.is_some());
        let (attr, ticks) = ms.not_hit_by.as_ref().unwrap();
        assert_eq!(attr, "SCA,NA");
        assert_eq!(*ticks, 5);
    }

    #[test]
    fn test_not_hit_by_timer_expires() {
        let pos = Position { pos: LogicVec2::new(0, 0) };
        let mut vel = Velocity { vel: LogicVec2::ZERO };
        let facing = Facing { dir: Facing::RIGHT };
        let mut fs = FighterState::new();
        let mut ms = MugenFighterState::default();
        ms.not_hit_by = Some(("S,NA".to_string(), 2));
        let mut hp = Health::new(10000);
        let mut pg = PowerGauge::new();

        let mut fighter = make_collision_fighter(&pos, &mut vel, &facing, &mut fs, &mut ms, &mut hp, &mut pg);
        tick_stun_timers(&mut fighter);
        assert!(fighter.mugen.not_hit_by.is_some()); // 1 tick remaining

        tick_stun_timers(&mut fighter);
        assert!(fighter.mugen.not_hit_by.is_none()); // expired
    }
}
