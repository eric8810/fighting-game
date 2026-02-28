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

Current status: SFF and AIR parsing complete. CNS/CMD/SND/ACT not yet implemented.

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

## Phase 1 - CNS Parser: StateDef + anim mapping

**Goal**: Read state → anim mapping from `.cns` file, replace hardcoded action_num mapping in `main.rs`.

### Tasks

- [ ] `crates/tickle_mugen/src/cns.rs`: Parse `[Statedef N]` blocks
  - Extract `anim` (action number to play)
  - Extract `type` (S=Stand, C=Crouch, A=Air)
  - Extract `physics` (S/C/A/N)
  - Extract `velset` (initial velocity on state entry)
  - Extract `ctrl` (control flag)
  - Extract `movetype` (A=Attack, I=Idle, H=Hit)
- [ ] `crates/tickle_mugen/src/lib.rs`: Export `Cns` struct with `get_anim(state_num)` API
- [ ] `game/src/main.rs`: Load `kyo.cns`, replace hardcoded `action_num` match with CNS lookup

### Key CNS format

```ini
[Statedef 0]       ; state number
type    = S        ; S=stand, C=crouch, A=air
physics = S
anim = 0           ; AIR action number to play
ctrl = 1           ; player has control
velset = 0,0       ; set velocity on entry
movetype = I       ; I=idle, A=attack, H=hit
```

---

## Phase 2 - State Controller Executor

**Goal**: Execute `[State N, ...]` controllers so the state machine can auto-transition and respond to input/physics.

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

### Trigger expression syntax

- Comparisons: `=`, `!=`, `>`, `<`, `>=`, `<=`, `= [lo, hi]`
- Logic: `triggerall` (AND), `trigger1..N` (OR between numbered triggers)
- Arithmetic: `+`, `-`, `*`, `/`
- Functions: `ifelse(cond, a, b)`, `const(velocity.jump.y)`
- Boolean operators: `&&`, `||`, `!`

### Tasks

- [ ] Parse `[State N, label]` blocks with all parameters
- [ ] Implement trigger expression evaluator (variables + operators above)
- [ ] Implement all controllers listed above
- [ ] Integrate with `tickle_core` game loop (run controllers each logic tick)
- [ ] Pass input state, physics state, animation state as trigger context

---

## Phase 3 - Full MUGEN State Machine

**Goal**: Full MUGEN compatibility including CMD input system, combat controllers, visual effects, and common1.cns.

### CMD file support

The `.cmd` file contains two sections:
1. `[Command]` blocks - define named input sequences (e.g. `qcf_x = ~D, DF, F, x`)
2. `[Statedef -1]` - global state checked every tick for input-driven transitions

```ini
[Command]
name = "qcf_x"
command = ~D, DF, F, x   ; ~ = release, $ = hold, + = simultaneous
time = 15                 ; input window in frames

[Statedef -1]             ; always-active state
[State -1, 1]
type = ChangeState
value = 1000
trigger1 = command = "qcf_x"
trigger1 = ctrl = 1
```

#### CMD trigger variables (from kyo.cmd)

| Variable | Count | Description |
|---------|-------|-------------|
| `stateno` | 396 | Current state number |
| `MoveContact` | 374 | Move connected |
| `command` | 86 | Named command active |
| `ctrl` | 49 | Player has control |
| `statetype` | 47 | S/C/A state type |
| `power` | 15 | Power gauge value |
| `P2BodyDist X` | 15 | Distance to opponent |
| `Time` | 7 | Ticks in state |
| `PalNo` | 3 | Palette number |
| `Vel X` | 2 | Velocity X |
| `NumProjID` | 2 | Active projectile count |
| `Life` | 1 | Current life |

#### CMD command syntax

| Token | Meaning |
|-------|---------|
| `F/B/U/D` | Directional inputs |
| `DF/DB/UF/UB` | Diagonal inputs |
| `x/y/z/a/b/c/s` | Button inputs |
| `~` prefix | Release input |
| `$` prefix | Hold input |
| `+` | Simultaneous inputs |
| `30$B` | Hold B for 30 frames (charge) |

### Combat controllers (from kyo.cns)

| Controller | Count | Description |
|-----------|-------|-------------|
| `HitDef` | 113 | Define attack hitbox and properties |
| `Explod` | 122 | Spawn visual effect entity |
| `Projectile` | 56 | Spawn projectile entity |
| `NotHitBy` | 43 | Invincibility frames |
| `SuperPause` | 13 | Freeze game for super flash |
| `EnvShake` | 16 | Screen shake |
| `BgPalFX` | 11 | Background palette effect |
| `ReversalDef` | 8 | Counter-attack definition |
| `ChangeAnim2` | 7 | Change opponent's animation |
| `AfterImage` | 7 | Motion trail effect |
| `TargetBind` | 7 | Bind position to target |
| `RemoveExplod` | 7 | Remove visual effect |
| `SelfState` | 6 | Change own state (from projectile) |
| `PalFX` | 4 | Character palette effect |
| `StopSnd` | 4 | Stop playing sound |
| `Width` | 7 | Set pushbox width |
| `LifeAdd` | 3 | Add/subtract life |
| `GameMakeAnim` | 3 | Spawn stage animation |
| `EnvColor` | 3 | Full-screen color flash |
| `PlayerPush` | 2 | Enable/disable pushbox |
| `MoveHitReset` | 2 | Reset move hit flag |
| `HitBy` | 2 | Vulnerability definition |
| `TargetState` | 1 | Change target's state |
| `TargetPowerAdd` | 1 | Add power to target |
| `Pause` | 1 | Pause game |
| `LifeSet` | 1 | Set life to value |

### Additional tasks

- [ ] Complete trigger expression parser (all operators, all system variables)
- [ ] CMD parser: `[Command]` blocks with full input sequence syntax
- [ ] CMD `[Statedef -1]` execution (checked every tick before normal state)
- [ ] `common1.cns` support (standard MUGEN states 0-199, 5000-5120)
- [ ] Variable system: `var(0..59)`, `fvar(0..39)`, persistent vars, rollback-compatible
- [ ] ACT palette loading and runtime palette switching (6 palettes per character)
- [ ] SND file parser and playback integration with `tickle_audio`
- [ ] All combat controllers listed above
- [ ] All visual effect controllers (Explod, AfterImage, PalFX, EnvColor, etc.)
- [ ] Projectile entity system
- [ ] HitDef / NotHitBy / HitBy combat system integration with `tickle_core`
- [ ] Replace `tickle_core` StateType enum with CNS state numbers
- [ ] P2 trigger variables (`P2BodyDist`, `P2name`, `P2statetype`, etc.)
