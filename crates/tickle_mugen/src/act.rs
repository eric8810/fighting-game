use std::fs::File;
use std::io::{Read, BufReader};
use std::path::Path;
use crate::Result;

/// RGB color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// ACT palette containing 256 RGB colors
#[derive(Debug, Clone)]
pub struct ActPalette {
    pub colors: Vec<Rgb>,
}

impl ActPalette {
    /// Parse an ACT palette file
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // ACT files contain exactly 768 bytes (256 colors * 3 bytes RGB)
        let mut buffer = vec![0u8; 768];
        reader.read_exact(&mut buffer)?;

        // Parse RGB triplets
        let mut colors = Vec::with_capacity(256);
        for chunk in buffer.chunks_exact(3) {
            colors.push(Rgb {
                r: chunk[0],
                g: chunk[1],
                b: chunk[2],
            });
        }

        Ok(ActPalette { colors })
    }

    /// Convert palette to GPU-compatible RGBA format (u8 array)
    pub fn to_rgba(&self) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(256 * 4);
        for color in &self.colors {
            rgba.push(color.r);
            rgba.push(color.g);
            rgba.push(color.b);
            rgba.push(255); // Alpha channel (fully opaque)
        }
        rgba
    }
}

/// Parser for ACT palette files
pub struct ActParser;

impl ActParser {
    /// Parse an ACT palette file
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<ActPalette> {
        ActPalette::parse(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_act_parser() {
        // Path is relative to crate directory when running cargo test
        let palette = ActParser::parse("../../assets/mugen/kyo/kyo01.act").unwrap();
        assert_eq!(palette.colors.len(), 256);
        // First color is usually black in MUGEN palettes
        assert_eq!(palette.colors[0], Rgb { r: 0, g: 0, b: 0 });
    }

    #[test]
    fn test_palette_to_rgba() {
        let palette = ActParser::parse("../../assets/mugen/kyo/kyo01.act").unwrap();
        let rgba = palette.to_rgba();

        // Should have 256 colors * 4 channels (RGBA)
        assert_eq!(rgba.len(), 256 * 4);

        // First color should be black with full alpha
        assert_eq!(rgba[0], 0);   // R
        assert_eq!(rgba[1], 0);   // G
        assert_eq!(rgba[2], 0);   // B
        assert_eq!(rgba[3], 255); // A
    }

    #[test]
    fn test_multiple_palettes() {
        // Test that we can load multiple palette files (pal1-6)
        let pal1 = ActParser::parse("../../assets/mugen/kyo/kyo01.act").unwrap();
        let pal2 = ActParser::parse("../../assets/mugen/kyo/kyo02.act").unwrap();

        assert_eq!(pal1.colors.len(), 256);
        assert_eq!(pal2.colors.len(), 256);

        // Palettes should be different (different color schemes)
        assert_ne!(pal1.colors, pal2.colors);
    }
}
