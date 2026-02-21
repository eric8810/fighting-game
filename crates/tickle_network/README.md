# tickle_network

Rollback networking crate for the Tickle Fighting Engine.

> This crate is a placeholder for the future network system. Implementation is planned to use [GGRS](https://crates.io/crates/ggrs) for peer-to-peer rollback netcode.

## Planned Features

- GGRS integration for peer-to-peer rollback networking
- Game state snapshot save/load for rollback frames
- Input synchronization between players
- Spectator support

## Planned API

```rust
pub struct GameSnapshot {
    pub entities: Vec<EntitySnapshot>,
    pub frame: u32,
    pub rng_state: u64,
}

// GGRS GameState implementation for save/load
impl GameState for GameSnapshot { ... }
```

## Determinism Requirements

The network layer depends on fully deterministic simulation in `tickle_core`:

- All logic uses `i32` (no floats)
- Fixed-seed LCG for random numbers
- Frame-count based timing (no wall clock)
- Synchronous execution only

See [docs/09-engine-technical-specs.md](../../docs/09-engine-technical-specs.md) for full specifications.
