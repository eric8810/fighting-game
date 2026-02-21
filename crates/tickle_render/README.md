# tickle_render

GPU rendering crate for the Tickle Fighting Engine. Built on wgpu for cross-platform support (Windows, Linux, macOS, WebGPU/WASM).

## Modules

### `context` -- `RenderContext`

Initializes wgpu and manages the GPU surface, device, and queue. Handles window resize and provides a simple `render_clear` for screen clearing.

```rust
let ctx = RenderContext::new(window).await?;
ctx.render_clear(wgpu::Color::BLACK)?;
```

### `sprite_batch` -- `SpriteBatchRenderer`

Instanced sprite renderer that batches up to 10,000 sprites per draw call. Uses a unit quad with per-instance position, size, UV, and color data.

- `SpriteInstance` -- per-sprite GPU data (position, size, uv_offset, uv_size, color)
- `CameraUniform` -- orthographic projection matrix
- `SpriteBatchRenderer::draw_sprite()` -- queue a sprite
- `SpriteBatchRenderer::flush()` -- upload and draw all queued sprites

### `texture` -- `Texture`, `TextureAtlas`

GPU texture loading and sprite sheet management.

- `Texture::load_from_bytes()` -- load PNG/image bytes into a GPU texture
- `Texture::white_1x1()` -- fallback solid-color texture
- `TextureAtlas` -- maps named regions to UV coordinates within a sprite sheet
- `TextureAtlas::generate_grid()` -- auto-generate regions for uniform grid sheets
- `TextureAtlas::get_uv()` -- get normalized UV offset/size for a named region

### `debug_renderer` -- `DebugRenderer`

Line-based overlay for visualizing collision boxes at runtime. Toggle with F1.

- `draw_rect()` -- draw a `LogicRect` outline
- `draw_cross()` -- draw a position marker
- Color-coded: red for hitboxes, green for hurtboxes, yellow for pushboxes

## Usage

```rust
use tickle_render::{RenderContext, SpriteBatchRenderer, SpriteInstance, Texture};

// Initialize
let ctx = RenderContext::new(window).await?;
let mut renderer = SpriteBatchRenderer::new(&ctx.device, ctx.surface_format());

// Each frame
renderer.begin();
renderer.draw_sprite(SpriteInstance {
    position: [100.0, 200.0],
    size: [64.0, 64.0],
    uv_offset: [0.0, 0.0],
    uv_size: [1.0, 1.0],
    color: [1.0, 1.0, 1.0, 1.0],
});
// flush inside a render pass...
```

See [docs/09-engine-technical-specs.md](../../docs/09-engine-technical-specs.md) for full specifications.
