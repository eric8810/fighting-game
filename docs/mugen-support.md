# MUGEN Support Implementation Plan

## Overview

MUGEN characters are defined by several configuration files:
- `.def` - Character definition (references all other files)
- `.sff` - Sprite data (indexed color, palette-based)
- `.air` - Animation data (frame sequences)
- `.cns` - State definitions and controllers (state machine logic)
- `.cmd` - Command definitions (input mappings)
- `.snd` - Sound data

Current status: SFF and AIR parsing complete. CNS/CMD not yet implemented.

---

## Phase 1 - CNS Parser: StateDef + anim mapping

**Goal**: Read state → anim mapping from `.cns` file, replace hardcoded action_num mapping in `main.rs`.

### Tasks

- [ ] `crates/tickle_mugen/src/cns.rs`: Parse `[Statedef N]` blocks
  - Extract `anim` (action number)
  - Extract `type` (S=Stand, C=Crouch, A=Air)
  - Extract `physics` (S/C/A/N)
  - Extract `velset` (initial velocity on state entry)
  - Extract `ctrl` (control flag)
- [ ] `crates/tickle_mugen/src/lib.rs`: Export `Cns` struct
- [ ] `game/src/main.rs`: Load `kyo.cns`, replace hardcoded `action_num` match with `cns.get_anim(state_num)`

### Key CNS format

```ini
[Statedef 0]       ; state number
type    = S        ; S=stand, C=crouch, A=air
physics = S
anim = 0           ; AIR action number to play
ctrl = 1           ; player has control
velset = 0,0       ; set velocity on entry
```

---

## Phase 2 - State Controller Executor

**Goal**: Execute `[State N, ...]` controllers so the state machine can auto-transition and respond to input/physics.

### Tasks

- [ ] Parse `[State N, label]` blocks
  - `type` - controller type
  - `triggerall` / `trigger1..N` - conditions
  - Controller-specific parameters (`value`, `x`, `y`, etc.)
- [ ] Implement basic trigger expression evaluator
  - Literals: `Time = 0`, `AnimTime = 0`, `1` (always true)
  - Comparisons: `=`, `!=`, `>`, `<`, `>=`, `<=`, `= [lo, hi]`
  - Variables: `Vel X`, `Vel Y`, `Pos Y`, `var(N)`
  - Commands: `command = "holdfwd"`, `command != "holdup"`
  - Functions: `ifelse(cond, a, b)`, `const(velocity.jump.y)`
  - Boolean: `triggerall` (must pass) + any `trigger1..N` (OR logic)
- [ ] Implement basic controllers
  - `ChangeState` - transition to new state
  - `VelSet` - set velocity X/Y
  - `VelAdd` - add to velocity X/Y
  - `PosSet` - set position X/Y
  - `ChangeAnim` - switch animation without changing state
  - `VarSet` - set integer variable `var(N)`
  - `PlaySnd` - trigger sound effect
  - `AssertSpecial` - set special flags (NoWalk, RoundNotOver, etc.)
- [ ] Integrate with `tickle_core` game loop
  - Run state controllers each logic tick
  - Pass input state, physics state, animation state as trigger context

---

## Phase 3 - Full MUGEN State Machine

**Goal**: Full MUGEN state machine compatibility for complex character logic.

### Tasks

- [ ] Complete trigger expression parser
  - All arithmetic operators: `+`, `-`, `*`, `/`, `%`, `**`
  - All comparison and logical operators
  - All built-in functions: `abs`, `sin`, `cos`, `floor`, `ceil`, `random`, etc.
  - All system variables: `AnimElem`, `AnimElemTime`, `HitCount`, `MoveContact`, etc.
  - P2 variables: `P2dist X`, `P2dist Y`, `P2name`, `P2statetype`
- [ ] Complete controller set
  - `HitDef` - define attack hitbox and properties
  - `Projectile` - spawn projectile entity
  - `HitOverride` - override hit response
  - `Afterimage` / `AfterimageTime` - motion trail effect
  - `EnvShake` - screen shake
  - `SprPriority` - sprite draw order
  - `SuperPause` - freeze game for super move flash
  - `PalFX` - palette color effects
  - `BGPalFX` - background palette effects
  - `Explod` - spawn visual effect entity
  - `BindToTarget` / `BindToParent` - position binding
  - `DestroySelf` - remove entity (for projectiles)
  - `MakeDust` - dust particle effect
  - `HitFallSet` / `HitFallDamage` - knockdown control
  - `StateTypeSet` - change state type mid-state
  - `AngleDraw` / `AngleSet` / `AngleAdd` - sprite rotation
- [ ] Variable system
  - Integer vars `var(0..59)`, float vars `fvar(0..39)`
  - Persistent vars across states (`IntPersistIndex`, `FloatPersistIndex`)
  - Rollback-compatible save/restore
- [ ] `common1.cns` support
  - Standard MUGEN states 0-199 (idle, walk, jump, crouch)
  - Standard hit states 5000-5120 (hitstun, knockdown, getup)
  - Standard guard states 150-155
- [ ] CMD parser integration
  - Parse `.cmd` command definitions
  - Integrate with existing input buffer in `tickle_core/src/input.rs`
  - Support `command = "QCF_x"` style trigger evaluation
- [ ] Replace `tickle_core` StateType enum
  - State machine driven entirely by CNS state numbers
  - `StateType` enum becomes internal detail or removed
  - Physics driven by CNS `physics` field (S/C/A/N)
