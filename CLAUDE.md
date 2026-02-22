# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tickle is a 2D fighting game engine built with Rust + wgpu, inspired by KOF 2000. It features rollback netcode (GGRS), deterministic integer-based physics, and high refresh rate support (60/120/144/240 Hz).

## Common Commands

### Development
```bash
# Run the game (debug build)
cargo run

# Run with release optimizations
cargo run --release

# Build without running
cargo build
cargo build --release
```

### Testing
```bash
# Run all tests (175 tests across workspace)
cargo test --workspace

# Run tests for specific crate
cargo test -p tickle_core
cargo test -p game

# Run performance benchmarks
cargo test -p game --release --test perf_test -- --nocapture
cargo test -p tickle_core --release --test core_bench -- --nocapture

# Run determinism tests (networking)
cargo test -p tickle_network --test determinism_test
```

### Code Quality
```bash
# Lint (should have zero warnings)
cargo clippy --workspace

# Format check
cargo fmt --all -- --check

# Apply formatting
cargo fmt --all
```

## Architecture

### Workspace Structure

The project uses a Cargo workspace with 6 crates:

- **tickle_core**: Core game logic (ECS components, input system, state machine, physics/collision/combat systems)
- **tickle_render**: Rendering layer (wgpu context, sprite batch renderer, texture atlas, debug renderer)
- **tickle_audio**: Audio system (kira-based SFX + music with volume control)
- **tickle_network**: Networking (GGRS rollback types, deterministic RNG, game state snapshots)
- **tickle_assets**: Asset management (RON file loading, Arc-based caching)
- **game**: Main binary (game loop, menu system, stage, UI, application entry point)

### Core Design Principles

1. **Deterministic Integer Coordinates**: All game logic uses `i32` fixed-point coordinates (1/100 pixel precision) to ensure determinism for rollback netcode. No floating-point math in logic layer.

2. **Logic/Render Separation**:
   - Logic runs at fixed 60 FPS
   - Rendering runs at variable rate (60/120/144/240 Hz)
   - Render interpolation between logic frames for smooth high-refresh display

3. **ECS Architecture**: Uses `hecs` for entity-component-system with archetype storage. Core components in `tickle_core/src/components.rs`:
   - Position, Velocity, PreviousPosition (for interpolation)
   - Health, PowerGauge, Facing, Direction
   - StateMachine, Hitbox, InputState, AnimationState

4. **Systems Pipeline** (in `tickle_core/src/systems/`):
   - `physics.rs`: Gravity, ground detection, friction, stage bounds
   - `collision.rs`: Hitbox vs hurtbox AABB detection, pushbox separation
   - `combat.rs`: Damage calculation, combo scaling, hitstun, knockback
   - `animation.rs`: Frame-based sprite animation
   - `audio_events.rs`: Convert game events to audio triggers

### State Machine

The fighter state machine (`tickle_core/src/state_machine.rs`) implements:
- Hierarchical states (Ground/Air/Special)
- Cancel windows for combo system
- Auto-transitions based on frame count
- State types: Idle, Walk, Crouch, Jump, Attack, Block, Hitstun, Knockdown, etc.

### Input System

Command recognition (`tickle_core/src/input.rs`) supports:
- Motion inputs: QCF (236), QCB (214), DP (623), HCF (41236), HCB (63214)
- Dash inputs (66/44)
- 16-frame input buffer with lenient timing windows
- Button encoding: BUTTON_A/B/C/D constants

### Rendering

The rendering layer (`tickle_render/`) uses:
- wgpu (WebGPU API) with instanced quad rendering
- Sprite batch renderer (up to 4096 sprites per frame)
- Texture atlas support for efficient sprite sheets
- Debug renderer for hitbox visualization (toggle with F1)

### Networking

GGRS rollback netcode foundation (`tickle_network/`):
- Deterministic RNG with save/restore for rollback
- Game state snapshots (serialize all game-affecting state)
- Network input encoding (u16 bitfield)
- All game logic must be deterministic (no floats, no system time, no random without seeded RNG)

### Asset Loading

Character data is defined in RON files (`assets/characters/*.ron`):
- Move definitions with frame data (startup, active, recovery)
- Hitbox/hurtbox definitions per animation frame
- Damage, hitstun, knockback values
- Cancel windows for combo system

## Key Files

- `game/src/main.rs`: Application entry point, window management, event loop
- `game/src/game_loop.rs`: Fixed timestep game loop with render interpolation
- `game/src/menu.rs`: Menu system (main menu, pause, round/match end)
- `game/src/ui.rs`: HUD rendering (health bars, power gauge, timer, combo counter)
- `game/src/stage.rs`: Stage system with parallax backgrounds
- `tickle_core/src/state_machine.rs`: Fighter state machine implementation
- `tickle_core/src/input.rs`: Input buffer and command recognition

## Testing Strategy

- Unit tests in each crate for isolated logic
- Integration tests in `game/tests/` for full system behavior
- Performance benchmarks to ensure <8ms logic frame budget
- Determinism tests to verify rollback netcode compatibility

## Performance Targets

- Logic frame time: <8ms (60 FPS budget)
- Current performance: ~0.13μs mean (60,000x headroom)
- Release binary: ~7.8MB (with LTO + strip)
- Zero allocations in hot paths (physics/collision/combat systems)

## Coordinate System

- Origin at ground level, center of stage
- +X = right, +Y = up
- All logic coordinates are `i32` (1 pixel precision)
- Example: `LogicVec2::new(400, 0)` = 4 pixels right of origin
- Render layer converts to `f32` for GPU

## Documentation

Technical design docs in `docs/`:
- `03-technical-architecture.md`: System architecture overview
- `04-network-architecture.md`: Rollback netcode design
- `09-engine-technical-specs.md`: Detailed engine specifications
- `11-high-refresh-rate-support.md`: Render interpolation design
