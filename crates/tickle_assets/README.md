# tickle_assets

Asset management crate for the Tickle Fighting Engine. Provides a generic, type-safe asset loader with caching, using RON (Rusty Object Notation) as the data format.

## Key Types

### `AssetId`

String-based unique identifier for assets. Used as cache keys.

```rust
let id = AssetId::new("ryu_hadoken");
```

### `AssetManager<T>`

Generic asset loader that reads RON files from disk, deserializes them into `T`, and caches results with `Arc<T>` for zero-copy sharing.

- `new(base_path)` -- create a manager rooted at a directory
- `load(id, relative_path)` -- load, parse, cache, and return an `Arc<T>`
- `get(id)` -- retrieve a previously loaded asset from cache

### `AssetError`

Error types: `NotFound`, `ReadError`, `ParseError` (with path context).

## RON File Format

Assets are defined as RON files. Any struct implementing `serde::Deserialize` can be loaded:

```ron
// data/characters/ryu.ron
(
    id: "ryu",
    name: "Ryu",
    health: 10000,
    walk_speed: 500,
)
```

## Usage

```rust
use tickle_assets::{AssetManager, AssetId};

#[derive(serde::Deserialize)]
struct CharacterData {
    id: String,
    name: String,
    health: i32,
    walk_speed: i32,
}

let mut manager = AssetManager::<CharacterData>::new("data/characters");
let id = AssetId::new("ryu");
let character = manager.load(&id, "ryu.ron")?;
println!("{}: {} HP", character.name, character.health);
```

See [docs/09-engine-technical-specs.md](../../docs/09-engine-technical-specs.md) for full specifications.
