# Tickle Fighting Engine

A 2D fighting game engine built with Rust + wgpu, inspired by KOF 2000.

## Quick Start

### Run the Game

```bash
cargo run
```

### Controls

**Player 1 (Blue):**
- WASD: Move (W=Jump, S=Crouch, A=Left, D=Right)
- Space: Attack

**Player 2 (Red):**
- Arrow Keys: Move
- Enter: Attack

**Menu / System:**
- Escape: Pause (during fight) / Back (in menus)
- Enter/Space: Confirm menu selection
- W/S or Up/Down: Navigate menus

### Run Tests

```bash
# Run all tests (175 tests)
cargo test --workspace

# Run performance benchmarks
cargo test -p game --release --test perf_test -- --nocapture

# Run core system benchmarks
cargo test -p tickle_core --release --test core_bench -- --nocapture
```

### Code Quality

```bash
cargo clippy --workspace   # Zero warnings
cargo fmt --all -- --check # Fully formatted
```

## Project Structure

```
tickle/
├── crates/
│   ├── tickle_core/      # Core logic (math, ECS components, input, state machine, systems)
│   ├── tickle_render/    # Rendering (wgpu context, sprite batch, texture atlas, debug)
│   ├── tickle_audio/     # Audio (kira-based SFX + music with volume control)
│   ├── tickle_network/   # Networking (GGRS rollback, deterministic RNG, snapshots)
│   └── tickle_assets/    # Asset management (RON loader, Arc caching)
├── game/                 # Main game binary
│   ├── src/main.rs       # App, game loop, rendering
│   ├── src/menu.rs       # Menu system (main menu, pause, round/match end)
│   ├── src/stage.rs      # Stage system with parallax backgrounds
│   ├── src/ui.rs         # HUD (health bars, power gauge, combo counter, timer)
│   └── tests/            # Performance tests
├── assets/               # Game assets (sprites, sounds, music, character data)
└── docs/                 # Technical documentation
```

## Implemented Systems

### Core Engine
- Fixed 60 FPS logic + variable render rate (60/120/144/240 Hz)
- Render interpolation for smooth high-refresh-rate display
- Deterministic integer coordinate system (i32 fixed-point, 1/100 pixel)
- ECS architecture (hecs) with archetype storage
- Input system with command recognition (QCF, QCB, DP, HCF, HCB, Dash)
- State machine with cancel windows and auto-transitions

### Audio
- kira 0.9 integration for low-latency sound effects
- Background music with looping and stop
- Three-channel volume control (master, SFX, music)
- Event-driven audio (hit sounds, state change sounds, BGM triggers)

### Rendering
- wgpu (WebGPU) backend with instanced quad rendering
- Sprite batch renderer (up to 4096 sprites)
- Texture atlas support with PNG loading
- Debug renderer for hitbox visualization
- Parallax scrolling stage backgrounds with camera tracking

### Game Logic
- Physics (gravity, ground detection, friction, stage bounds)
- Collision detection (hitbox vs hurtbox AABB)
- Combat system (damage, combo scaling, hitstun, knockback, power gauge)
- Animation system (frame-based, looping/one-shot)
- Character data loading from RON files

### Networking
- GGRS rollback networking foundation
- Deterministic RNG with save/restore
- Game state snapshots for rollback
- Network input encoding

### UI
- Health bars with damage flash and drain animation
- Power gauge (3 stocks)
- Round timer
- Combo counter
- Menu system (main menu, pause, round end, match end)
- VS and Training game modes

## Performance

Logic frame time: ~0.13us (budget: 8ms, headroom: ~60,000x)
- 10,000 frame stability test: no degradation (late/early ratio: 0.98x)
- ECS scales linearly: 2 entities 0.03us, 100 entities 0.16us
- Release binary: 7.7MB (LTO + strip)

## Build

```bash
# Debug build
cargo build

# Release build (optimized, LTO, stripped)
cargo build --release
```

## System Requirements

- Rust 1.75+ (2021 edition)
- Windows 10/11, Linux, or macOS
- GPU with Vulkan, DirectX 12, or Metal support

## License

MIT License
