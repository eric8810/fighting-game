# MUGEN Support Implementation Plan

## Scope Definition

MUGEN support is a **content pipeline tool**, not a compatibility target. The goal is to load MUGEN character packages for rapid prototyping and development validation, not to run arbitrary MUGEN content correctly.

**We support**: The subset of MUGEN needed to run KOF-style characters (movement, attacks, basic specials, hit reactions).

**We do not support**: Visual effects (Explod, AfterImage, PalFX), complex projectile state machines, intro/win animations, palette effects, or obscure edge cases.

---

## File Format Reference (Kyo Kusanagi)

| File | Lines | Status |
|------|-------|--------|
| kyo.sff | 2.5 MB | ✓ Complete |
| kyo.air | 4,380 | ✓ Complete |
| kyo.cns | 11,523 | ✗ Not started |
| kyo.cmd | 1,890 | ✗ Not started |
| kyo.snd | 1.6 MB | ✗ Not started |
| kyo*.act | 768 B each | ✗ Not started |
| KyoKusanagi.def | 369 B | ✗ Not started |

---

## Design Decisions (finalized)

These architectural decisions have been finalized and will guide Phase 2-3 implementation:

### 1. State Machine Architecture

**Decision**: Option A - Replace `StateType` enum with integer state numbers.

**Rationale**:
- Current `StateType` enum only has 9 values (Idle, Walk, Crouch, Jump, Attack, Block, Hitstun, Knockdown, Special), insufficient for real character logic
- CNS state numbers are more expressive (Kyo has 233 statedefs) and already encode the same information
- Type safety can be maintained through constants and helper functions:
  ```rust
  const STATE_STAND: i32 = 0;
  const STATE_WALK_FWD: i32 = 20;
  const STATE_CROUCH: i32 = 10;
  // etc.
  ```
- Cleaner integration with MUGEN's state transition system

**Implementation**: Replace `StateType` field in components with `state_num: i32` and `state_frame: i32`.

### 2. Trigger Expression Evaluator

**Decision**: Interpreted AST approach.

**Rationale**:
- Performance is not a bottleneck: typical KOF character has ~200 statedefs × ~5 controllers = ~1000 trigger evaluations per frame
- Each evaluation takes <1 microsecond, well within 16.67ms frame budget (60 FPS)
- Simpler to implement and debug
- Easier to extend with new trigger variables and operators
- No need for complex compilation infrastructure

**Implementation**: Parse trigger expressions to AST at load time, evaluate during state controller execution.

### 3. common1.cns

**Decision**: Reimplement standard states in Rust as built-in states.

**Rationale**:
- Avoids legal gray area of bundling MUGEN's common1.cns
- Only need ~20 standard states for KOF-style characters:
  - 0 = Stand
  - 10 = Crouch Start
  - 11 = Crouching
  - 12 = Crouch End
  - 20 = Walk Forward
  - 21 = Walk Back
  - 40 = Jump Start
  - 41 = Jump Neutral
  - 42 = Jump Forward
  - 43 = Jump Back
  - 45 = Air Jump Start
  - 46 = Air Jump
  - 50 = Guard Start
  - 51 = Guard Mid
  - 52 = Guard End
  - 100 = Run Forward
  - 105 = Run Back
  - 5000 = Hitstun (standing)
  - 5010 = Hitstun (crouching)
  - 5020 = Hitstun (air)
  - 5100 = Knockdown
- Full control over physics parameters and integration with tickle_core systems

**Implementation**: Create `crates/tickle_mugen/src/common_states.rs` with Rust implementations of standard states.

### 4. Rollback Compatibility

**Decision**: Extend `FighterSnapshot` to include MUGEN-specific state.

**Required additions**:
```rust
pub struct FighterSnapshot {
    // Existing fields
    pub entity: Entity,
    pub position: LogicVec2,
    pub velocity: LogicVec2,
    pub health: i32,
    pub power_gauge: i32,
    pub facing: Facing,

    // New MUGEN fields
    pub state_num: i32,           // Current CNS state number
    pub state_frame: i32,         // Ticks since state entry
    pub vars: [i32; 60],          // var(0..59) integer variables
    pub ctrl: bool,               // Control flag
    pub anim_num: i32,            // Current animation number
    pub anim_elem: i32,           // Current animation element
    pub anim_time: i32,           // Ticks in current element
    pub move_hit: bool,           // MoveHit flag
    pub move_contact: bool,       // MoveContact flag
    pub move_guarded: bool,       // MoveGuarded flag
    pub prev_state_num: i32,      // Previous state number
}
```

**Note**: Float variables (fvar) can be skipped initially as KOF characters rarely use them.

**Implementation**: Update `tickle_network/src/snapshot.rs` and ensure all MUGEN state is properly saved/restored during rollback.

---

## Phase 1 - Parsers + anim mapping

**Goal**: Load character from `.def`, read state → anim mapping from `.cns`, replace hardcoded mappings in `main.rs`.

**Complexity**: Low. Pure parsing, no architectural changes needed.

**Prerequisite**: None.

### Tasks

- [ ] `crates/tickle_mugen/src/def.rs`
  - Parse character name, file references (sff, air, cns, cmd, snd, pal1..6)
- [ ] `crates/tickle_mugen/src/cns.rs` (parser only)
  - Parse `[Data]` section: life, attack, defence
  - Parse `[Size]` section: ground.back, ground.front, height
  - Parse `[Velocity]` section: walk.fwd, walk.back, jump.neu/fwd/back, run.fwd/back
  - Parse `[Movement]` section: yaccel, stand.friction, airjump.num
  - Parse `[Statedef N]` blocks: anim, type (S/C/A), physics, velset, ctrl, movetype
- [ ] `crates/tickle_mugen/src/act.rs`
  - Load 256 RGB entries from `.act` file
  - Support palette index selection (pal1..6)
- [ ] `crates/tickle_mugen/src/lib.rs`
  - Export `CharacterDef`, `Cns`, `Act` structs
- [ ] `game/src/main.rs`
  - Load character via `.def` file
  - Replace hardcoded `action_num` match with `cns.get_anim(state_num)`
  - Apply character physics params from CNS `[Velocity]` / `[Movement]`

---

## Phase 2 - State Controller Executor

**Goal**: Execute CNS controllers so the character can animate, move, and transition states automatically.

**Complexity**: Medium.

**Prerequisite**: Design decisions finalized (see above).

### Controllers to implement (KOF-relevant subset)

| Controller | Priority | Description |
|-----------|----------|-------------|
| `ChangeState` | Critical | State transitions |
| `VelSet` | Critical | Set velocity |
| `VelAdd` | Critical | Add to velocity |
| `PosAdd` | Critical | Add to position |
| `PosSet` | High | Set absolute position |
| `VarSet` | High | Set var(N) |
| `VarAdd` | High | Add to var(N) |
| `CtrlSet` | High | Set control flag |
| `ChangeAnim` | High | Switch animation |
| `PlaySnd` | Medium | Play sound via tickle_audio |
| `AssertSpecial` | Medium | Special flags (NoWalk, etc.) |
| `VarRandom` | Low | Random variable |
| `Gravity` | Low | Apply gravity |

### Trigger variables to implement

| Variable | Priority | Description |
|---------|----------|-------------|
| `Time` | Critical | Ticks since state entry |
| `AnimElem` | Critical | Current animation element |
| `AnimTime` | Critical | Ticks until/since anim element |
| `Vel X/Y` | Critical | Current velocity |
| `Pos Y` | Critical | Y position (ground detection) |
| `var(N)` | Critical | Integer variable |
| `command` | Critical | Named command active |
| `ctrl` | Critical | Control flag |
| `stateno` | Critical | Current state number |
| `statetype` | High | S/C/A state type |
| `MoveHit` | High | Move connected |
| `MoveContact` | High | Move contact |
| `MoveGuarded` | High | Move was guarded |
| `Random` | Medium | Random 0-999 |
| `Anim` | Medium | Current animation number |
| `PrevStateNo` | Medium | Previous state number |
| `Life` | Low | Current life |
| `Power` | Low | Power gauge |
| `FrontEdgeDist` | Low | Distance to stage edge |

### CMD support

- [ ] Parse `[Command]` blocks: name, input sequence, time window
- [ ] Implement command input syntax: directions (F/B/U/D/DF/etc.), buttons (x/y/z/a/b/c/s), modifiers (~release, $hold, +simultaneous, charge)
- [ ] Parse and execute `[Statedef -1]` (global state, checked every tick)
- [ ] Integrate with existing input buffer in `tickle_core/src/input.rs`

### SND support

- [ ] `crates/tickle_mugen/src/snd.rs`: Parse SND file header, extract sample list
- [ ] Decode audio samples (typically PCM or MP3)
- [ ] Integrate with `tickle_audio` for PlaySnd controller

---

## Phase 3 - Combat Integration

**Goal**: Connect MUGEN HitDef to tickle_core combat system. Characters can actually fight.

**Complexity**: High.

**Prerequisite**: Phase 2 complete, rollback snapshot design resolved.

### Tasks

- [ ] `HitDef` controller: parse and apply attack properties
  - damage, guard damage, attack type (normal/special/hyper)
  - hitflag (H/L/A - high/low/air), guardflag
  - animtype (light/medium/hard/back/up/diagup)
  - hitstun, blockstun, knockback (xvel, yvel)
  - sparkno (hit spark animation)
  - priority, p1/p2 state on hit
- [ ] `NotHitBy` / `HitBy`: vulnerability windows (invincibility frames)
- [ ] `ReversalDef`: counter-attack definition
- [ ] Connect HitDef to existing tickle_core collision system
- [ ] Extend `FighterSnapshot` for full MUGEN state rollback
- [ ] P2 trigger variables: `P2BodyDist X/Y`, `P2statetype`, `P2stateno`, `P2life`

### Out of scope for Phase 3

- `Projectile` controller (complex, separate entity system needed)
- `Explod` / visual effects
- `SuperPause` / `EnvShake`
- `TargetBind` / `TargetState`
- `ChangeAnim2` (change opponent animation)
