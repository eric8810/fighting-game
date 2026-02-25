use crate::sff_v1::{SffV1, SpriteData};
use std::collections::HashMap;

/// A packed texture atlas built from SFF sprite data.
/// All sprites are laid out in a single RGBA image for GPU upload.
pub struct SpriteAtlas {
    /// RGBA pixel data (width * height * 4 bytes)
    pub rgba: Vec<u8>,
    /// Atlas width in pixels
    pub width: u32,
    /// Atlas height in pixels
    pub height: u32,
    /// UV rect for each sprite: (u, v, uw, vh) in normalized [0,1] coords
    pub uvs: HashMap<(u16, u16), [f32; 4]>,
    /// Sprite dimensions and axis offsets
    pub sprites: HashMap<(u16, u16), SpriteInfo>,
}

/// Per-sprite metadata needed for rendering
#[derive(Clone, Copy, Debug)]
pub struct SpriteInfo {
    pub width: u16,
    pub height: u16,
    pub axis_x: i16,
    pub axis_y: i16,
}

impl SpriteAtlas {
    /// Build an atlas from all sprites in an SFF file.
    /// Sprites are packed in rows, sorted by group/image.
    pub fn build(sff: &SffV1) -> Self {
        let mut keys: Vec<(u16, u16)> = sff.sprite_keys();
        keys.sort();

        // Compute atlas dimensions: pack sprites in rows of up to MAX_ROW_W pixels
        const MAX_ROW_W: u32 = 2048;
        let mut row_x = 0u32;
        let mut row_y = 0u32;
        let mut row_h = 0u32;
        let mut atlas_w = 0u32;

        // First pass: compute layout
        let mut layout: HashMap<(u16, u16), (u32, u32)> = HashMap::new();
        for &key in &keys {
            if let Some(s) = sff.get_sprite(key.0, key.1) {
                let sw = s.width as u32;
                let sh = s.height as u32;
                if sw == 0 || sh == 0 {
                    continue;
                }
                if row_x + sw > MAX_ROW_W && row_x > 0 {
                    row_y += row_h;
                    row_x = 0;
                    row_h = 0;
                }
                layout.insert(key, (row_x, row_y));
                row_x += sw;
                row_h = row_h.max(sh);
                atlas_w = atlas_w.max(row_x);
            }
        }
        let atlas_h = row_y + row_h;

        // Ensure non-zero dimensions
        let atlas_w = atlas_w.max(1);
        let atlas_h = atlas_h.max(1);

        let mut rgba = vec![0u8; (atlas_w * atlas_h * 4) as usize];
        let mut uvs = HashMap::new();
        let mut sprites = HashMap::new();

        // Second pass: blit sprites into atlas
        for &key in &keys {
            if let (Some(s), Some(&(ox, oy))) = (sff.get_sprite(key.0, key.1), layout.get(&key)) {
                let sw = s.width as u32;
                let sh = s.height as u32;
                if sw == 0 || sh == 0 {
                    continue;
                }

                blit_indexed(&mut rgba, atlas_w, s, ox, oy);

                let u  = ox as f32 / atlas_w as f32;
                let v  = oy as f32 / atlas_h as f32;
                let uw = sw as f32 / atlas_w as f32;
                let vh = sh as f32 / atlas_h as f32;
                uvs.insert(key, [u, v, uw, vh]);
                sprites.insert(key, SpriteInfo {
                    width: s.width,
                    height: s.height,
                    axis_x: s.axis_x,
                    axis_y: s.axis_y,
                });
            }
        }

        Self { rgba, width: atlas_w, height: atlas_h, uvs, sprites }
    }

    /// Get UV rect for a sprite (normalized [0,1])
    pub fn get_uv(&self, group: u16, image: u16) -> Option<[f32; 4]> {
        self.uvs.get(&(group, image)).copied()
    }

    /// Get sprite metadata
    pub fn get_info(&self, group: u16, image: u16) -> Option<SpriteInfo> {
        self.sprites.get(&(group, image)).copied()
    }
}

/// Blit an 8-bit indexed sprite into an RGBA atlas buffer.
/// Palette index 0 is treated as transparent.
fn blit_indexed(rgba: &mut [u8], atlas_w: u32, sprite: &SpriteData, ox: u32, oy: u32) {
    let sw = sprite.width as u32;
    let sh = sprite.height as u32;
    let palette = &sprite.palette;

    for py in 0..sh {
        for px in 0..sw {
            let src_idx = (py * sw + px) as usize;
            let color_idx = sprite.pixels.get(src_idx).copied().unwrap_or(0) as usize;

            let dst = ((oy + py) * atlas_w + (ox + px)) as usize * 4;
            if dst + 3 >= rgba.len() {
                continue;
            }

            if color_idx == 0 {
                // Index 0 = transparent
                rgba[dst]     = 0;
                rgba[dst + 1] = 0;
                rgba[dst + 2] = 0;
                rgba[dst + 3] = 0;
            } else {
                let pal_off = color_idx * 3;
                rgba[dst]     = palette.get(pal_off).copied().unwrap_or(0);
                rgba[dst + 1] = palette.get(pal_off + 1).copied().unwrap_or(0);
                rgba[dst + 2] = palette.get(pal_off + 2).copied().unwrap_or(0);
                rgba[dst + 3] = 255;
            }
        }
    }
}
