use crate::{Result, SffError};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// SFF sprite file (supports both v1 and v2)
pub struct SffV1 {
    /// Sprite data indexed by (group, image)
    sprites: HashMap<(u16, u16), SpriteData>,
}

/// Individual sprite data
#[derive(Clone, Debug)]
pub struct SpriteData {
    /// Sprite width in pixels
    pub width: u16,
    /// Sprite height in pixels
    pub height: u16,
    /// X axis offset (relative to upper-left corner)
    pub axis_x: i16,
    /// Y axis offset (relative to upper-left corner)
    pub axis_y: i16,
    /// Raw pixel data (8-bit indexed color)
    pub pixels: Vec<u8>,
    /// Palette (256 RGB triplets, 768 bytes total)
    pub palette: Vec<u8>,
}

impl SffV1 {
    /// Load SFF file from path (supports v1 and v2)
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let data = fs::read(path)?;
        Self::parse(&data)
    }

    /// Parse SFF from bytes (auto-detects v1 or v2)
    fn parse(data: &[u8]) -> Result<Self> {
        // Check signature
        if data.len() < 16 {
            return Err(SffError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "File too small",
            )));
        }

        // Read version bytes at offset 12-15
        let version = [data[12], data[13], data[14], data[15]];

        // Version format: [verlo3, verlo2, verlo, verhi]
        // v1: [0x01, 0x01, 0x00, 0x00] or similar
        // v2: [0x00, 0x01, 0x00, 0x02] = v2.01
        let ver_hi = version[3];

        if ver_hi == 1 {
            // SFF v1 - use mugen-sff crate
            Self::parse_v1(data)
        } else if ver_hi == 2 {
            // SFF v2 - use our own parser
            Self::parse_v2(data)
        } else {
            Err(SffError::UnsupportedVersion(ver_hi, version[2]))
        }
    }

    /// Parse SFF v1 using mugen-sff crate
    fn parse_v1(data: &[u8]) -> Result<Self> {
        let decoder = mugen_sff::Decoder::decode(data)
            .map_err(|e| SffError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

        let mut sprites = HashMap::new();

        for sprite in decoder.sprites() {
            let id = sprite.id();
            let coords = sprite.coordinates();

            // Decode PCX data to get width, height, and pixels
            let (width, height, pixels) = Self::decode_pcx(sprite.raw_data())?;

            let sprite_data = SpriteData {
                width,
                height,
                axis_x: coords.x,
                axis_y: coords.y,
                pixels,
                palette: sprite.palette().to_vec(),
            };

            sprites.insert((id.group, id.image), sprite_data);
        }

        log::info!("Loaded {} sprites from SFF v1", sprites.len());

        Ok(Self { sprites })
    }

    /// Parse SFF v2 format
    fn parse_v2(data: &[u8]) -> Result<Self> {
        if data.len() < 64 {
            return Err(SffError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "SFF v2 header too small",
            )));
        }

        // SFF v2 header layout (from Ikemen-GO source):
        // offset 36: FirstSpriteHeaderOffset
        // offset 40: NumberOfSprites
        // offset 44: FirstPaletteHeaderOffset
        // offset 48: NumberOfPalettes
        // offset 52: lofs (literal data section base offset)
        // offset 56: dummy
        // offset 60: tofs (translated data section base offset)
        let first_sprite_offset = u32::from_le_bytes([data[36], data[37], data[38], data[39]]) as usize;
        let num_sprites         = u32::from_le_bytes([data[40], data[41], data[42], data[43]]) as usize;
        let first_pal_offset    = u32::from_le_bytes([data[44], data[45], data[46], data[47]]) as usize;
        let num_palettes        = u32::from_le_bytes([data[48], data[49], data[50], data[51]]) as usize;
        let lofs                = u32::from_le_bytes([data[52], data[53], data[54], data[55]]) as usize;
        let tofs                = u32::from_le_bytes([data[60], data[61], data[62], data[63]]) as usize;

        log::debug!(
            "SFF v2: sprites={} at {}, palettes={} at {}, lofs={}, tofs={}",
            num_sprites, first_sprite_offset,
            num_palettes, first_pal_offset,
            lofs, tofs
        );

        // Parse palette nodes at first_pal_offset (16 bytes each):
        // [0-1]  group (i16)
        // [2-3]  item  (i16)
        // [4-5]  numcols (i16)
        // [6-7]  link (u16)
        // [8-11] ofs (u32) — relative to lofs
        // [12-15] siz (u32)
        let mut palettes: Vec<Vec<u8>> = Vec::new();
        for p in 0..num_palettes {
            let base = first_pal_offset + p * 16;
            if base + 16 > data.len() { break; }
            let numcols = i16::from_le_bytes([data[base + 4], data[base + 5]]) as usize;
            let pal_ofs = u32::from_le_bytes([data[base + 8], data[base + 9], data[base + 10], data[base + 11]]) as usize;
            let pal_siz = u32::from_le_bytes([data[base + 12], data[base + 13], data[base + 14], data[base + 15]]) as usize;

            let abs_pal = lofs + pal_ofs;
            let mut palette = vec![0u8; 768]; // 256 RGB triplets
            if abs_pal + pal_siz <= data.len() {
                // Palette data is RGBA (4 bytes per color); extract RGB
                let cols = numcols.min(256);
                for c in 0..cols {
                    let src = abs_pal + c * 4;
                    if src + 3 <= data.len() {
                        palette[c * 3]     = data[src];
                        palette[c * 3 + 1] = data[src + 1];
                        palette[c * 3 + 2] = data[src + 2];
                    }
                }
            }
            palettes.push(palette);
        }
        log::debug!("Loaded {} palettes", palettes.len());

        // Parse sprite nodes at first_sprite_offset (28 bytes each):
        // [0-1]   group (u16)
        // [2-3]   item  (u16)
        // [4-5]   width (u16)
        // [6-7]   height (u16)
        // [8-9]   axis_x (i16)
        // [10-11] axis_y (i16)
        // [12-13] link (u16)   ← NOT padding
        // [14]    format (u8): 0=raw, 2=rle8, 3=rle5, 4=lz5
        // [15]    coldepth (u8)
        // [16-19] ofs (u32)
        // [20-23] size (u32)
        // [24-25] palidx (u16)
        // [26-27] flags (u16): bit0 → 0=lofs, 1=tofs
        let mut sprites = HashMap::new();
        for s in 0..num_sprites {
            let base = first_sprite_offset + s * 28;
            if base + 28 > data.len() { break; }

            let group  = u16::from_le_bytes([data[base],     data[base + 1]]);
            let item   = u16::from_le_bytes([data[base + 2], data[base + 3]]);
            let width  = u16::from_le_bytes([data[base + 4], data[base + 5]]);
            let height = u16::from_le_bytes([data[base + 6], data[base + 7]]);
            let axis_x = i16::from_le_bytes([data[base + 8], data[base + 9]]);
            let axis_y = i16::from_le_bytes([data[base + 10], data[base + 11]]);
            let link   = u16::from_le_bytes([data[base + 12], data[base + 13]]);
            let format = data[base + 14];
            let _depth = data[base + 15];
            let ofs    = u32::from_le_bytes([data[base + 16], data[base + 17], data[base + 18], data[base + 19]]) as usize;
            let size   = u32::from_le_bytes([data[base + 20], data[base + 21], data[base + 22], data[base + 23]]) as usize;
            let palidx = u16::from_le_bytes([data[base + 24], data[base + 25]]) as usize;
            let flags  = u16::from_le_bytes([data[base + 26], data[base + 27]]);

            // Linked sprite: reuse pixel data from an earlier sprite
            if format == 1 {
                let link_base = first_sprite_offset + (link as usize) * 28;
                if link_base + 28 <= data.len() {
                    let src_group = u16::from_le_bytes([data[link_base], data[link_base + 1]]);
                    let src_item  = u16::from_le_bytes([data[link_base + 2], data[link_base + 3]]);
                    if let Some(src) = sprites.get(&(src_group, src_item)).cloned() {
                        sprites.insert((group, item), SpriteData { axis_x, axis_y, ..src });
                    }
                }
                continue;
            }

            let abs_ofs = if flags & 1 != 0 { tofs + ofs } else { lofs + ofs };

            let pixels = if size == 0 || abs_ofs + size > data.len() {
                log::warn!("SFF v2: invalid data for sprite ({},{}): abs_ofs={} size={}", group, item, abs_ofs, size);
                vec![0u8; (width as usize) * (height as usize)]
            } else {
                let src = &data[abs_ofs..abs_ofs + size];
                match format {
                    0 => src.to_vec(),
                    2 => Self::decode_rle8(src, width as usize, height as usize),
                    3 => Self::decode_rle5(src, width as usize, height as usize),
                    4 => Self::decode_lz5(src, width as usize, height as usize),
                    _ => {
                        log::warn!("SFF v2: unsupported format {} for ({},{})", format, group, item);
                        vec![0u8; (width as usize) * (height as usize)]
                    }
                }
            };

            let palette = palettes.get(palidx)
                .or_else(|| palettes.first())
                .cloned()
                .unwrap_or_else(|| vec![0u8; 768]);

            sprites.insert((group, item), SpriteData { width, height, axis_x, axis_y, pixels, palette });
        }

        log::info!("Loaded {} sprites from SFF v2", sprites.len());
        Ok(Self { sprites })
    }

    /// RLE8 decode (format 2) — ported from Ikemen-GO Rle8Decode
    fn decode_rle8(data: &[u8], width: usize, height: usize) -> Vec<u8> {
        let total = width * height;
        let mut p = vec![0u8; total];
        let mut i = 0usize;
        let mut j = 0usize;
        while j < total {
            if i >= data.len() { break; }
            let mut n = 1usize;
            let mut d = data[i];
            if i < data.len() - 1 { i += 1; }
            if d & 0xc0 == 0x40 {
                n = (d & 0x3f) as usize;
                d = data[i];
                if i < data.len() - 1 { i += 1; }
            }
            while n > 0 && j < total {
                p[j] = d;
                j += 1;
                n -= 1;
            }
        }
        p
    }

    /// RLE5 decode (format 3) — ported from Ikemen-GO Rle5Decode
    fn decode_rle5(data: &[u8], width: usize, height: usize) -> Vec<u8> {
        let total = width * height;
        let mut p = vec![0u8; total];
        let mut i = 0usize;
        let mut j = 0usize;
        while j < total {
            if i >= data.len() { break; }
            let mut rl = data[i] as i32;
            if i < data.len() - 1 { i += 1; }
            if i >= data.len() { break; }
            let dl_byte = data[i];
            let mut dl = (dl_byte & 0x7f) as i32;
            let mut c = 0u8;
            if dl_byte >> 7 != 0 {
                if i < data.len() - 1 { i += 1; }
                if i >= data.len() { break; }
                c = data[i];
            }
            if i < data.len() - 1 { i += 1; }
            loop {
                if j < total { p[j] = c; j += 1; }
                rl -= 1;
                if rl < 0 {
                    dl -= 1;
                    if dl < 0 { break; }
                    if i >= data.len() { break; }
                    c  = data[i] & 0x1f;
                    rl = (data[i] >> 5) as i32;
                    if i < data.len() - 1 { i += 1; }
                }
            }
        }
        p
    }

    /// LZ5 decode (format 4) — ported from Ikemen-GO Lz5Decode
    fn decode_lz5(data: &[u8], width: usize, height: usize) -> Vec<u8> {
        let total = width * height;
        let mut p = vec![0u8; total];
        if data.is_empty() { return p; }

        let mut i = 0usize;
        let mut j = 0usize;
        let mut ct  = data[i];
        let mut cts = 0u32;
        let mut rb  = 0u8;
        let mut rbc = 0u32;
        if i < data.len() - 1 { i += 1; }

        while j < total {
            if i >= data.len() { break; }
            let mut d = data[i] as i32;
            if i < data.len() - 1 { i += 1; }

            if ct & (1u8 << cts) != 0 {
                // back-reference
                let mut n: i32;
                if d & 0x3f == 0 {
                    // long back-reference
                    if i >= data.len() { break; }
                    d = (d << 2 | data[i] as i32) + 1;
                    if i < data.len() - 1 { i += 1; }
                    if i >= data.len() { break; }
                    n = data[i] as i32 + 2;
                    if i < data.len() - 1 { i += 1; }
                } else {
                    // short back-reference: accumulate 2 high bits of d into rb
                    rb |= (d & (0xc0i32 >> rbc)) as u8;
                    rbc += 2;
                    n = d & 0x3f;
                    if rbc < 8 {
                        if i >= data.len() { break; }
                        d = data[i] as i32 + 1;
                        if i < data.len() - 1 { i += 1; }
                    } else {
                        d = rb as i32 + 1;
                        rb  = 0;
                        rbc = 0;
                    }
                }
                // copy n+1 bytes from p[j - d]
                loop {
                    if j < total && j >= d as usize {
                        p[j] = p[j - d as usize];
                        j += 1;
                    }
                    n -= 1;
                    if n < 0 { break; }
                }
            } else {
                // literal run
                let n: i32;
                if d & 0xe0 == 0 {
                    // long literal
                    if i >= data.len() { break; }
                    n = data[i] as i32 + 8;
                    if i < data.len() - 1 { i += 1; }
                } else {
                    n = d >> 5;
                    d &= 0x1f;
                }
                let mut rem = n;
                while rem > 0 && j < total {
                    p[j] = d as u8;
                    j += 1;
                    rem -= 1;
                }
            }

            cts += 1;
            if cts >= 8 {
                if i >= data.len() { break; }
                ct  = data[i];
                cts = 0;
                if i < data.len() - 1 { i += 1; }
            }
        }
        p
    }

    /// Decode PCX data to extract width, height, and pixels
    fn decode_pcx(data: &[u8]) -> Result<(u16, u16, Vec<u8>)> {
        if data.len() < 128 {
            return Err(SffError::InvalidPcx(0));
        }

        // PCX header (128 bytes)
        let manufacturer = data[0];
        if manufacturer != 10 {
            return Err(SffError::InvalidPcx(0));
        }

        let xmin = u16::from_le_bytes([data[4], data[5]]);
        let ymin = u16::from_le_bytes([data[6], data[7]]);
        let xmax = u16::from_le_bytes([data[8], data[9]]);
        let ymax = u16::from_le_bytes([data[10], data[11]]);

        let width = xmax - xmin + 1;
        let height = ymax - ymin + 1;

        // Decode RLE-compressed pixel data
        let mut pixels = Vec::with_capacity((width as usize) * (height as usize));
        let mut i = 128; // Start after header

        while pixels.len() < (width as usize) * (height as usize) && i < data.len() {
            let byte = data[i];
            i += 1;

            if byte >= 0xC0 {
                // RLE run
                let count = (byte & 0x3F) as usize;
                if i >= data.len() {
                    break;
                }
                let value = data[i];
                i += 1;
                for _ in 0..count {
                    pixels.push(value);
                    if pixels.len() >= (width as usize) * (height as usize) {
                        break;
                    }
                }
            } else {
                // Literal byte
                pixels.push(byte);
            }
        }

        // Trim to exact size
        pixels.truncate((width as usize) * (height as usize));

        Ok((width, height, pixels))
    }

    /// Get sprite data by group and image number
    pub fn get_sprite(&self, group: u16, image: u16) -> Option<&SpriteData> {
        self.sprites.get(&(group, image))
    }

    /// Get all sprite keys (group, image)
    pub fn sprite_keys(&self) -> Vec<(u16, u16)> {
        self.sprites.keys().copied().collect()
    }

    /// Number of sprites loaded
    pub fn sprite_count(&self) -> usize {
        self.sprites.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_kfm_sff() {
        // This test requires the KFM file to be present
        let path = "../../assets/mugen/kfm/kfm.sff";
        if std::path::Path::new(path).exists() {
            let sff = SffV1::load(path).expect("Failed to load kfm.sff");
            assert!(sff.sprite_count() > 0, "Should load at least one sprite");

            println!("Loaded {} sprites", sff.sprite_count());

            // Check for Action 0 (Idle) sprites
            let sprite_0_0 = sff.get_sprite(0, 0);
            if let Some(s) = sprite_0_0 {
                println!("Sprite (0,0): {}x{}, axis: ({}, {})", s.width, s.height, s.axis_x, s.axis_y);
                assert!(s.width > 0 && s.height > 0, "Sprite should have valid dimensions");
            }

            // List first 10 sprites and verify pixel data
            let mut keys = sff.sprite_keys();
            keys.sort();
            println!("First 10 sprites:");
            for (g, i) in keys.iter().take(10) {
                if let Some(s) = sff.get_sprite(*g, *i) {
                    let non_zero = s.pixels.iter().filter(|&&b| b != 0).count();
                    println!("  ({}, {}): {}x{}, axis: ({}, {}), pixels={}, non_zero={}",
                        g, i, s.width, s.height, s.axis_x, s.axis_y, s.pixels.len(), non_zero);
                    assert_eq!(s.pixels.len(), s.width as usize * s.height as usize,
                        "Pixel buffer size should match w*h for ({},{})", g, i);
                }
            }
            assert_eq!(sff.sprite_count(), 281, "KFM should have 281 sprites");
        } else {
            println!("Skipping test: kfm.sff not found at {}", path);
        }
    }
}
