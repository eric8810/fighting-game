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

## Needs Design First (blockers)

These architectural questions must be answered before implementation:

### 1. State Machine Architecture

**Problem**: tickle_core has its own `StateType` enum. MUGEN uses integer state numbers. Two options:

- **Option A - Replace**: Remove `StateType` enum, drive everything from CNS state numbers. Clean but requires rewriting tickle_core systems.
- **Option B - Bridge**: Keep `StateType` enum, map CNS states onto it. Preserves existing systems but limits MUGEN compatibility.

**Recommendation**: Option A. The existing `StateType` enum is too limited for real character logic anyway. CNS state numbers are more expressive and already encode the same information.

### 2. Trigger Expression Evaluator

**Problem**: CNS triggers are a full expression language:
```
ifelse(var(8)=0, const(velocity.jump.neu.x), ifelse(var(8)>0, const(velocity.jump.fwd.x), const(velocity.jump.back.x)))
```

- **Interpreted AST**: Parse to AST at load time, evaluate each tick. Simpler to implement, slower.
- **Compiled closures**: Compile to Rust closures at load time. Faster, harder to implement.

**Recommendation**: Interpreted AST. Performance is not a concern for the subset we support (KOF characters have ~200 statedefs, each with ~5 controllers = ~1000 evaluations per tick, trivial at 60 FPS).

### 3. common1.cns

MUGEN's standard states (0=stand, 20=walk, 40=jump, 5000=hitstun) are defined in `common1.cns`, which is part of the MUGEN engine, not character packages.

**Options**:
- Bundle a compatible `common1.cns` (legal gray area)
- Reimplement standard states in Rust, expose as built-in states

**Recommendation**: Reimplement in Rust. We only need the ~20 standard states relevant to KOF-style characters. This also gives us full control over physics parameters.

### 4. Rollback Compatibility

MUGEN state that must be serialized for rollback:
- State number + state frame counter
- var(0..59) integer variables
- fvar(0..39) float variables (may skip for now)
- Velocity, position, animation frame
- Hit flags (MoveHit, MoveContact, MoveGuarded)
- Control flag

The existing `FighterSnapshot` in tickle_network must be extended.

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

**Prerequisite**: Design decisions 1, 2, 3 above must be resolved.

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
