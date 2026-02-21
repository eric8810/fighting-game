# tickle_audio

Audio crate for the Tickle Fighting Engine.

> This crate is a placeholder for the future audio system. Implementation is planned to use [kira](https://crates.io/crates/kira) for low-latency sound effects and music playback.

## Planned Features

- Sound effect playback with pooling (hit sounds, block sounds, UI)
- Background music with crossfade
- Per-channel volume control
- Deterministic audio triggers from game events (compatible with rollback)

## Planned API

```rust
pub struct AudioSystem {
    // kira AudioManager + sound cache
}

impl AudioSystem {
    pub fn play_sound(&mut self, id: &str) -> Result<()>;
    pub fn play_music(&mut self, id: &str, looping: bool);
    pub fn stop_music(&mut self);
}
```

See [docs/09-engine-technical-specs.md](../../docs/09-engine-technical-specs.md) for full specifications.
