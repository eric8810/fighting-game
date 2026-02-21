# tickle_core

Core game logic crate for the Tickle Fighting Engine. Contains deterministic, integer-based types and systems designed for rollback netcode compatibility.

## Modules

### `math`

Fixed-point coordinate primitives. All positions use `i32` at 1/100 pixel precision (100 = 1 pixel) to guarantee determinism across platforms.

- `LogicCoord` (i32) -- scalar coordinate type
- `LogicVec2` -- 2D vector with arithmetic ops, magnitude, normalize, dot product
- `LogicRect` -- AABB rectangle with intersection, containment, translate, and flip

### `components`

ECS component structs used by the game simulation:

- `Position`, `PreviousPosition`, `Velocity` -- spatial components
- `Facing` -- direction the fighter faces (1 = right, -1 = left)
- `Health` -- HP with damage/heal and alive check
- `PowerGauge` -- super meter (0-3000, 3 stocks of 1000)
- `FighterState` / `StateType` -- state machine (Idle, Jump, Attack, Hitstun, etc.)
- `Hitbox`, `Hurtbox`, `Pushbox` -- collision primitives
- `HitboxManager` -- manages per-entity collision boxes with world-space transforms

### `input`

Input handling for fighting game commands:

- `InputState` -- per-frame button + direction snapshot
- `InputBuffer` -- ring buffer of recent inputs (16 frames)
- `Direction` -- 9-way directional input
- `CommandRecognizer` -- detects motion inputs (QCF, QCB, DP, dash)

### `systems`

Game logic systems that operate on component data each tick:

- **physics** -- `velocity_system`, `gravity_system`, `ground_detection_system`, `friction_system`
- **collision** -- `collision_system` (hitbox vs hurtbox AABB), `pushbox_separation_system`, `HitEvent`
- **combat** -- `hit_resolution_system` with combo damage scaling, `apply_blockstun`, `update_power_gauge`
- **animation** -- `animation_system` with frame timer, loop/one-shot modes, `SpriteAnimation`, `SpriteFrame`

## Usage

```rust
use tickle_core::{Position, Velocity, LogicVec2};
use tickle_core::systems::physics::{velocity_system, gravity_system, DEFAULT_GRAVITY};

// Create entity data
let mut entities = [(
    Position { pos: LogicVec2::new(10000, 5000) },
    Velocity { vel: LogicVec2::new(500, 0) },
)];

// Run systems each tick
velocity_system(&mut entities);
gravity_system(&mut entities, DEFAULT_GRAVITY);
```

## Design Principles

- All logic uses `i32` -- no floats in simulation code
- Systems are pure functions over slices, not tied to a specific ECS framework
- Fully deterministic for rollback network compatibility

See [docs/09-engine-technical-specs.md](../../docs/09-engine-technical-specs.md) for full specifications.
