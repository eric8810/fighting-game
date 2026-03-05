# MUGEN Support Implementation Task List

**Total Tasks**: 22 | **Status**: 22/22 COMPLETE ✅

**Last Updated**: 2026-03-04

---

## Summary

| Phase | Tasks | Status |
|-------|-------|--------|
| **Phase 1** Parsers | #3 #4 #5 #6 | ✅ All complete |
| **Phase 2.1** State Machine | #7 #8 #9 #10 #11 #17 | ✅ All complete |
| **Phase 2.2** Specials | #12 #13 #14 #15 #16 | ✅ All complete |
| **Phase 3** Combat | #18 #19 #20 #21 #22 #24 | ✅ All complete |
| **Phase 3** Invincibility | #23 | ✅ All complete |

---

## Phase 1: Parsers ✅ COMPLETE

### #3 - DEF Parser ✅
**File**: `crates/tickle_mugen/src/def.rs`

### #4 - CNS Parser ✅
**File**: `crates/tickle_mugen/src/cns.rs`
- Parses [Data] [Size] [Velocity] [Movement] sections
- Parses [StateDef N] and [State N] blocks with all controllers
- Controller types: ChangeState, VelSet, VelAdd, PosAdd, PosSet, VarSet, VarAdd, CtrlSet, ChangeAnim, PlaySnd, AssertSpecial, Gravity, HitDef, NotHitBy, HitBy, VarRandom, Unknown

### #5 - ACT Palette Parser ✅
**File**: `crates/tickle_mugen/src/act.rs`

### #6 - Integration ✅
**File**: `game/src/main.rs`
- Loads character from `.def` file at startup
- Applies CNS physics params (walk speed, jump velocity, gravity, friction)
- Animation lookup via `CharacterData::anim_for_state()` from CNS statedefs
- Loads `assets/mugen/common1.cns` (user-provided) and merges with character states

---

## Phase 2: State Machine ✅ COMPLETE

### #7 - Replace StateType ✅
**Files**: `tickle_core/src/components.rs`, `tickle_core/src/state_constants.rs`
- `StateType` enum replaced with `state_num: i32` + `state_frame: i32`
- MUGEN state number constants in `state_constants.rs`

### #8 - Trigger AST Parser ✅
**File**: `crates/tickle_mugen/src/trigger.rs`

### #9 - Trigger Evaluator ✅
**File**: `crates/tickle_mugen/src/trigger.rs`
- Full expression language: operators, functions (var(), ifelse(), floor, abs, command...), type coercion
- 32+ unit tests

### #10 - Controller Parser ✅
**File**: `crates/tickle_mugen/src/cns.rs`
- All controller types parsed including HitDef (26 fields), NotHitBy, HitBy, VarRandom

### #17 - Common States ✅ (architecture changed)
**Original plan**: Implement ~20 states in Rust (`common_states.rs`)
**Actual implementation**: Load `assets/mugen/common1.cns` from user-provided file at runtime.
- `common_states.rs` removed from module system
- `merge_statedefs(common, character)` in `cns.rs` merges the two
- User must supply `assets/mugen/common1.cns` (e.g. from MUGEN installation)
- Graceful fallback (warn + empty) if file missing

### #11 - Controller Executor ✅
**File**: `crates/tickle_mugen/src/mugen_controller.rs` (~3400 lines)
- `mugen_tick()`: executes statedef -1 first, then current state (correct MUGEN order)
- All 13+ controller types execute real logic
- `MugenFighterState`: 17 fields (vars[60], ctrl, rng_state, hitstun_remaining, etc.)
- Deterministic LCG for VarRandom (rollback-safe)

### #12 - CMD Parser ✅
**File**: `crates/tickle_mugen/src/cmd.rs`

### #13 - Command Recognition ✅
**File**: `crates/tickle_mugen/src/command_recognizer.rs`
- QCF/QCB/DP/HCF/HCB motions, facing-aware F/B mapping
- Release (~), hold ($), simultaneous (+) modifiers
- Time window enforcement
- 18 unit tests

### #14 - Statedef -1 ✅
**File**: `crates/tickle_mugen/src/mugen_controller.rs`
- `mugen_global_controller_system()` executes statedef -1 every tick
- `mugen_tick_with_p2()` passes P2 context to both global and current-state controllers
- Command triggers wired via `MugenFighterState.active_commands`

### #15 - SND Parser ✅
**File**: `crates/tickle_mugen/src/snd.rs`

### #16 - PlaySnd Controller ✅
**File**: `crates/tickle_mugen/src/mugen_controller.rs`
- PlaySnd records `MugenSoundEvent` into `MugenFighterState.pending_sounds`
- Game loop drains `pending_sounds` each frame and dispatches to `tickle_audio`

---

## Phase 3: Combat — 6/7 COMPLETE

### #18 - HitDef Parser ✅
**File**: `crates/tickle_mugen/src/cns.rs`
- `Controller::HitDef` with 26 fields (damage, guard_damage, hitflag, animtype, velocities, fall, priority, p1/p2stateno, etc.)
- `NotHitBy` and `HitBy` parsed with attr + time fields

### #19 - FighterSnapshot Extension ✅
**File**: `tickle_network/src/snapshot.rs`
- `MugenState` struct: vars_lo[30]/vars_hi[30], ctrl, anim_num/elem/time, move_hit/contact/guarded, prev_state_num, hitstun_remaining, blockstun_remaining
- `get_var()` / `set_var()` accessors with bounds checking

### #20 - HitDef Executor ✅
**File**: `crates/tickle_mugen/src/mugen_controller.rs`
- `ActiveHitDef` struct (16 fields) stored on `MugenFighterState.active_hitdef`
- `mugen_hitdef_collision()`: Clsn1 vs Clsn2 collision with distance-based fallback
- Applies damage, direction-aware knockback, hitstun state, move_hit flag, power gain
- `has_hit` flag prevents multi-hit per HitDef activation

### #22 - P2 Trigger Variables ✅
**File**: `crates/tickle_mugen/src/trigger.rs`
- 9 P2 fields in `TriggerContext`: p2_stateno, p2_life, p2_bodydist_x/y, p2_statetype, p2_movetype, p2_ctrl, p2_vel_x/y
- Case-insensitive matching
- 8 unit tests

### #21 - Combat Integration ✅
**File**: `crates/tickle_mugen/src/mugen_controller.rs`
- `mugen_combat_frame()`: full two-fighter combat resolution per frame
- `resolve_attack()`: damage, hitstun, knockback, state transition
- `resolve_guarded_attack()`: chip damage, blockstun
- `tick_stun_timers()`: hitstun/blockstun countdown with auto-recovery
- Combo scaling: 100/100/80/70/60/50%

### #23 - NotHitBy/HitBy ✅
**File**: `crates/tickle_mugen/src/mugen_controller.rs`

- `MugenFighterState.not_hit_by: Option<(String, i32)>` and `hit_by: Option<(String, i32)>` track active invincibility windows
- `attack_attr_matches(filter, attr)`: MUGEN attr format parser supporting multi-state ("SCA") and multi-attack ("NA SA") filters, case-insensitive
- `Controller::NotHitBy/HitBy` match arms now populate `ControllerEffects`, applied by `apply_effects()`
- `tick_stun_timers()`: decrements NotHitBy/HitBy counters; clears to `None` on expiry
- `mugen_hitdef_collision()`: invincibility check before hit application — NotHitBy blocks matching attacks, HitBy blocks non-matching attacks
- `MugenState` (snapshot) extended with `not_hit_by_ticks/attr` and `hit_by_ticks/attr` for rollback
- 13 new unit tests covering all invincibility scenarios

### #24 - Rollback Test ✅
**File**: `tickle_network/tests/determinism_test.rs`
- `test_mugen_rollback_determinism`: 60-frame simulation, save at 30, restore, re-simulate, assert bit-identical
- Covers vars_lo/hi, ctrl, anim state, move flags, prev_state_num, position, velocity, health

---

## Test Counts (at completion)

- `tickle_mugen`: 154 tests (134 lib + 20 integration)
- `tickle_core`: 97 tests
- `tickle_network`: 19 tests
- **Total: 270 tests, all passing**

---

## Remaining Work

None. All 22 tasks complete.
