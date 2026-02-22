# Changelog

## v0.1.0-alpha (2026-02-22)

Initial alpha release of the Tickle Fighting Engine.

### Core Engine
- Fixed 60 FPS logic timestep with variable render rate support (60/120/144/240 Hz)
- Render interpolation (alpha blending between logic frames)
- Deterministic integer coordinate system (i32 fixed-point, 1/100 pixel precision)
- ECS architecture using hecs with archetype storage
- Death spiral prevention (frame time clamping at 250ms)

### Input System
- Two-player local input (WASD + Space / Arrow Keys + Enter)
- Input buffer with 60-frame history
- Command recognition: QCF, QCB, DP, HCF, HCB, Dash Forward, Dash Backward
- Configurable input window tolerance

### State Machine
- Fighter states: Idle, WalkForward, WalkBackward, Run, Crouch, Jump, Attack, Hitstun, Blockstun, Knockdown
- Cancel windows with target filtering (Normal, Special, Super, Any)
- Auto-transitions (attack completion, stun recovery, landing)
- Frame-accurate state tracking

### Physics
- Gravity system (airborne entities only)
- Ground detection with landing transitions
- Ground friction with directional deceleration
- Stage boundary clamping
- Velocity integration

### Combat
- Hitbox vs hurtbox AABB collision detection
- Pushbox separation system
- Damage with combo scaling (100/100/80/70/60/50% floor)
- Hitstun and blockstun application
- Knockback vectors
- Power gauge (3 stocks, 1000 points each)
- Attacker/defender gauge gain on hit

### Animation
- Frame-based sprite animation system
- Looping and one-shot playback modes
- Frame timer with configurable duration
- Animation reset support

### Audio
- kira 0.9 integration for low-latency playback
- Sound effect caching with HashMap lookup
- Background music with looping and stop (default tween)
- Three-channel volume: master, SFX, music (0.0-1.0, clamped)
- Event-driven audio dispatch (hit sounds by damage strength, state change sounds)
- Graceful fallback (game runs without audio device)
- .ogg and .wav format support

### Rendering
- wgpu (WebGPU) backend
- Instanced quad renderer (up to 4096 instances per draw call)
- WGSL vertex/fragment shaders for colored quads
- Parallax scrolling stage backgrounds (multi-layer, depth-sorted)
- Camera tracking (follows midpoint between fighters)
- Debug renderer for hitbox visualization

### UI / HUD
- Health bars with damage flash effect and smooth drain animation
- Power gauge display (3 stock indicators)
- Round timer (99 seconds, counts down)
- Combo counter with hit tracking
- Per-player UI state management

### Menu System
- Main menu (VS Mode, Training, Quit)
- Pause menu (Resume, Quit to Menu)
- Round end screen with automatic progression
- Match end screen with return to menu
- Best-of-3 round tracking with win counters
- Game state machine (MainMenu, Fighting, Paused, RoundEnd, MatchEnd)

### Stage System
- RON-based stage definitions
- Parallax background layers with configurable scroll speeds
- Camera limits derived from stage width
- Default "Dojo" stage with 3 parallax layers
- Stage boundary enforcement for fighter movement

### Character System
- RON-based character data loading
- Character properties (health, walk/jump speeds, pushbox, hurtboxes)
- Move data loading with frame data (startup, active, recovery)
- Cancel chain definitions
- Ryu character with light punch, hadoken, and shoryuken moves

### Networking Foundation
- GGRS rollback networking types and input encoding
- Deterministic RNG (PCG-based) with save/restore
- Game state snapshot system (fighter state serialization)
- Snapshot comparison for desync detection
- Network input struct with direction + buttons

### Asset Management
- Generic AssetManager with Arc caching
- RON file loading for characters, moves, and stages
- Automatic deduplication via content-addressed cache

### Performance
- Release profile: opt-level 3, LTO fat, codegen-units 1, strip
- Dev profile: opt-level 1 for faster iteration
- Logic frame: ~0.13us mean (8ms budget = ~60,000x headroom)
- 10,000 frame stability: no degradation
- 175 tests, 0 failures
- Zero clippy warnings, fully formatted (rustfmt)
