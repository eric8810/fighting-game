# MUGEN Support Implementation Plan

## Overview

MUGEN characters are defined by several configuration files:
- `.def` - Character definition (references all other files)
- `.sff` - Sprite data (indexed color, palette-based)
- `.air` - Animation data (frame sequences)
- `.cns` - State definitions and controllers (state machine logic)
- `.cmd` - Command definitions (input mappings and state -1 transitions)
- `.snd` - Sound data
- `.act` - Palette files (256 RGB entries, 768 bytes each)

---

## File Format Reference (Kyo Kusanagi stats)

| File | Lines | Notes |
|------|-------|-------|
| kyo.cns | 11,523 | 233 statedefs, main state machine |
| kyo.cmd | 1,890 | 69 ChangeState, 10 VarSet in state -1 |
| kyo.air | 4,380 | 233 actions |
| kyo.sff | 2.5 MB | sprite atlas |
| kyo.snd | 1.6 MB | sound samples |
| kyo*.act | 768 B each | 6 palette variants |

---

## Current Engine Status vs MUGEN Requirements

### What's already done ✓

| System | Status | Notes |
|--------|--------|-------|
| SFF v1/v2 parsing | ✓ Complete | Sprite data, palette, PCX decode |
| AIR parsing | ✓ Complete | Frame sequences, CLSN boxes |
| Sprite atlas builder | ✓ Complete | GPU texture upload |
| Sprite rendering | ✓ Complete | Textured quads, UV mapping, facing flip |
| Animation frame advance | ✓ Complete | Per-frame duration, looping |
| Physics (gravity/friction) | ✓ Complete | Integer coords, deterministic |
| Pushbox separation | ✓ Complete | Prevents character overlap |
| Hitbox/hurtbox AABB | ✓ Complete | Per-frame collision detection |
| Combat (damage/hitstun) | ✓ Complete | Combo scaling, knockback |
| Input buffer | ✓ Complete | 16-frame history |
| Motion input recognition | ✓ Complete | QCF/QCB/DP/HCF/HCB/dash |
| Audio system | ✓ Complete | Kira-based, SFX + music |
| Rollback foundation | ✓ Complete | GGRS types, state snapshots |
| UI (health/power/timer) | ✓ Complete | HUD rendering |

### What's missing or needs redesign ✗

| System | Status | Gap |
|--------|--------|-----|
| CNS parser | ✗ Missing | No state machine config loading |
| CMD parser | ✗ Missing | No command definition loading |
| SND parser | ✗ Missing | Can't play character voice/SFX |
| ACT palette loading | ✗ Missing | Palette variants not supported |
| MUGEN state machine | ✗ Missing | State driven by hardcoded enum, not CNS |
| State controller executor | ✗ Missing | No VelSet/ChangeState/HitDef execution |
| Trigger expression evaluator | ✗ Missing | No CNS trigger language support |
| Variable system | ✗ Missing | No var(N)/fvar(N) |
| HitDef integration | ✗ Partial | Combat system exists but not CNS-driven |
| Projectile entities | ✗ Missing | No projectile spawning/tracking |
| Visual effects | ✗ Missing | No Explod/AfterImage/PalFX |
| common1.cns | ✗ Missing | Standard MUGEN states not loaded |
| DEF file loader | ✗ Missing | Character loaded by hardcoded path |
| Palette switching | ✗ Missing | Only palette 0 used |

### Systems that need redesign to support MUGEN

| System | Current Design | Required Change |
|--------|---------------|-----------------|
| State machine | `StateType` enum (Idle/Walk/Jump/etc.) | Replace with integer state numbers driven by CNS |
| Input commands | Hardcoded QCF/QCB/DP patterns | Load from `.cmd` file, match against named commands |
| Move data | RON asset files with frame data | Replaced by CNS HitDef controllers |
| Physics params | Hardcoded constants | Load from CNS `[Velocity]` / `[Movement]` sections |
| Character loading | Hardcoded path in main.rs | Load from `.def` file |

---

## What Needs Detailed Design (not ready to implement)

These areas require architectural decisions before coding:

### 1. MUGEN State Machine vs tickle_core State Machine

Currently `tickle_core` has its own `StateType` enum and state machine. MUGEN's CNS defines a completely different state machine. Two options:

- **Option A**: Replace `tickle_core` state machine entirely with CNS-driven system
- **Option B**: Keep `tickle_core` state machine, map MUGEN states onto it

Option A is cleaner for MUGEN compatibility but breaks the existing deterministic physics/combat systems. Option B preserves existing systems but creates a translation layer that may not cover all MUGEN states.

**Needs decision before Phase 3.**

### 2. Trigger Expression Evaluator Architecture

MUGEN's trigger language is a full expression language with variables, functions, comparisons, and boolean logic. It needs to evaluate expressions like:

```
ifelse(var(8)=0, const(velocity.jump.neu.x), ifelse(var(8)>0, const(velocity.jump.fwd.x), const(velocity.jump.back.x)))
```

Options:
- **Interpreted**: Parse to AST, evaluate at runtime each tick
- **Compiled**: Compile to bytecode or closures at load time

Performance matters here since triggers run every frame for every state controller. **Needs design before Phase 2.**

### 3. Rollback Compatibility of MUGEN State

MUGEN state includes: state number, var(0..59), fvar(0..39), velocity, position, animation frame, hit flags, projectile list, visual effects. All of this must be serializable for rollback. The current `FighterSnapshot` in `tickle_network` only captures basic physics state.

**Needs design before Phase 3.**

### 4. Projectile Entity System

MUGEN projectiles are full entities with their own state machines (CNS statedefs), physics, and hitboxes. They need to be spawned, tracked, and destroyed. The current ECS (hecs) can support this but the integration with rollback snapshots needs design.

**Needs design before Phase 3.**

### 5. common1.cns

MUGEN's `common1.cns` defines the standard states (0=stand, 20=walk, 40=jump, 5000=hitstun, etc.) that all characters inherit. Without it, characters can't transition to basic states. This file is not included in character packages - it's part of the MUGEN engine itself.

We need to either bundle a compatible `common1.cns` or reimplement the standard states ourselves.

**Needs decision before Phase 2.**

---

## Phase 1 - CNS/CMD/DEF Parsers + anim mapping

**Goal**: Load character from `.def` file, read state → anim mapping from `.cns`, replace all hardcoded mappings.

**Complexity**: Low. Pure parsing, no architectural changes.

### Tasks

- [ ] `crates/tickle_mugen/src/def.rs`: Parse `.def` file
  - Extract character name, file references (sff, air, cns, cmd, snd, palN)
- [ ] `crates/tickle_mugen/src/cns.rs`: Parse `[Statedef N]` blocks
  - Extract `anim`, `type` (S/C/A), `physics`, `velset`, `ctrl`, `movetype`
  - Extract `[Velocity]` and `[Movement]` sections (walk speed, jump velocity, gravity, etc.)
  - Extract `[Data]` section (life, attack, defence)
  - Extract `[Size]` section (ground.back, ground.front, height, etc.)
- [ ] `crates/tickle_mugen/src/cmd.rs`: Parse `.cmd` file
  - Parse `[Command]` blocks: name, command sequence, time window
  - Parse `[Statedef -1]` controllers (ChangeState only for now)
- [ ] `crates/tickle_mugen/src/act.rs`: Parse `.act` palette files
  - Load 256 RGB entries, support palette switching at runtime
- [ ] `crates/tickle_mugen/src/snd.rs`: Parse `.snd` file header
  - Extract sound sample list (group, index, offset, size)
  - Decode and play samples via `tickle_audio`
- [ ] `game/src/main.rs`: Load character from `.def`, use CNS anim mapping

---

## Phase 2 - State Controller Executor

**Goal**: Execute `[State N, ...]` controllers so the state machine can auto-transition and respond to input/physics.

**Complexity**: Medium. Requires trigger evaluator design decision first.

### Controller types to implement (from kyo.cns, by frequency)

| Controller | Count | Description |
|-----------|-------|-------------|
| `PlaySnd` | 223 | Play sound effect |
| `ChangeState` | 192 | Transition to new state |
| `PosAdd` | 104 | Add to position X/Y |
| `VelSet` | 74 | Set velocity X/Y |
| `VarSet` | 35 | Set integer variable var(N) |
| `VelAdd` | 27 | Add to velocity X/Y |
| `AssertSpecial` | 26 | Set special flags |
| `SprPriority` | 21 | Sprite draw order |
| `PosSet` | 11 | Set absolute position |
| `VarAdd` | 9 | Add to variable |
| `CtrlSet` | 2 | Set control flag |
| `VarRandom` | 1 | Set variable to random value |
| `Gravity` | 1 | Apply gravity |

### Trigger variables to implement (from kyo.cns, by frequency)

| Variable | Count | Description |
|---------|-------|-------------|
| `Time` | 821 | Ticks since state entry |
| `AnimElem` | 752 | Current animation element index |
| `AnimTime` | 170 | Ticks until/since anim element |
| `var(N)` | 53 | Integer variable |
| `command` | 44 | Input command name |
| `Random` | 27 | Random 0-999 |
| `Vel X/Y` | 23 | Current velocity |
| `Pos X/Y` | 20 | Current position |
| `MoveHit` | 15 | Whether move connected |
| `stateno` | 13 | Current state number |
| `Anim` | 9 | Current animation number |
| `ProjHit` | 6 | Projectile hit flag |
| `MoveContact` | 5 | Move contact flag |
| `HitShakeOver` | 5 | Hit shake finished |
| `MoveGuarded` | 2 | Move was guarded |
| `RoundState` | 1 | Round state (0-4) |
| `PrevStateNo` | 1 | Previous state number |
| `Life` | 1 | Current life |
| `FrontEdgeDist` | 1 | Distance to front edge |

### CMD trigger variables (from kyo.cmd, by frequency)

| Variable | Count | Description |
|---------|-------|-------------|
| `stateno` | 396 | Current state number |
| `MoveContact` | 374 | Move connected |
| `command` | 86 | Named command active |
| `ctrl` | 49 | Player has control |
| `statetype` | 47 | S/C/A state type |
| `power` | 15 | Power gauge value |
| `P2BodyDist X` | 15 | Distance to opponent body |
| `Time` | 7 | Ticks in state |
| `PalNo` | 3 | Palette number |
| `Vel X` | 2 | Velocity X |
| `NumProjID` | 2 | Active projectile count |
| `Life` | 1 | Current life |

### Trigger expression syntax

- Comparisons: `=`, `!=`, `>`, `<`, `>=`, `<=`, `= [lo, hi]`
- Logic: `triggerall` (AND), `trigger1..N` (OR between numbered triggers)
- Arithmetic: `+`, `-`, `*`, `/`
- Functions: `ifelse(cond, a, b)`, `const(velocity.jump.y)`
- Boolean operators: `&&`, `||`, `!`

### Tasks

- [ ] Design and implement trigger expression evaluator (AST interpreter)
- [ ] Parse `[State N, label]` blocks with all parameters
- [ ] Implement all controllers listed above
- [ ] Implement `[Statedef -1]` execution from CMD (checked before normal state each tick)
- [ ] Implement `common1.cns` standard states (bundle or reimplement)
- [ ] Integrate with `tickle_core` game loop
- [ ] Variable system: `var(0..59)`, `fvar(0..39)`, rollback-compatible

---

## Phase 3 - Full MUGEN State Machine

**Goal**: Full MUGEN compatibility. Requires architectural decisions from the "Needs Design" section above.

**Complexity**: High. Requires state machine redesign and rollback integration.

### Tasks

- [ ] Decide and implement state machine architecture (Option A or B from design section)
- [ ] Replace `tickle_core` StateType enum with CNS state numbers
- [ ] Extend `FighterSnapshot` in `tickle_network` to capture full MUGEN state
- [ ] Complete trigger expression parser (all operators, all system variables)
- [ ] Combat controllers: `HitDef`, `NotHitBy`, `HitBy`, `ReversalDef`
- [ ] Projectile entity system with own state machine and rollback support
- [ ] Visual effect controllers: `Explod`, `AfterImage`, `PalFX`, `BGPalFX`, `EnvColor`, `EnvShake`, `SuperPause`
- [ ] Target controllers: `TargetBind`, `TargetState`, `TargetPowerAdd`, `ChangeAnim2`
- [ ] Misc controllers: `Width`, `LifeAdd`, `LifeSet`, `StopSnd`, `GameMakeAnim`, `PlayerPush`, `MoveHitReset`
- [ ] P2 trigger variables: `P2BodyDist`, `P2name`, `P2statetype`, `P2stateno`, etc.
- [ ] `NumProjID` and projectile-related triggers
- [ ] Runtime palette switching via ACT files
- [ ] Full CMD command syntax: `~` (release), `$` (hold), `+` (simultaneous), charge inputs
