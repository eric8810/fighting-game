use crate::error::SffError;
use std::path::Path;

/// Metadata for a single sound in the SND file.
/// The actual audio bytes live at `data_offset` in the file.
#[derive(Debug, Clone)]
pub struct SoundEntry {
    /// Sound group number (e.g. 0 = common, 5000 = common hit sounds).
    pub group: u16,
    /// Sound index within the group.
    pub sound: u16,
    /// Byte offset in the file where raw audio data starts.
    pub data_offset: u64,
    /// Length of audio data in bytes.
    pub data_length: u32,
    /// Loop flag (non-zero means the sound loops).
    pub loopflag: u8,
}

/// Parsed SND file containing sound metadata.
///
/// This parser reads only the index (header + subfile headers).
/// It does not decode or buffer the raw PCM/WAV data.
#[derive(Debug)]
pub struct Snd {
    pub entries: Vec<SoundEntry>,
}

impl Snd {
    /// Look up a sound entry by group and sound number.
    pub fn get_sound(&self, group: u16, sound: u16) -> Option<&SoundEntry> {
        self.entries.iter().find(|e| e.group == group && e.sound == sound)
    }

    /// Total number of sounds in the file.
    pub fn sound_count(&self) -> usize {
        self.entries.len()
    }
}

pub struct SndParser;

impl SndParser {
    /// Parse a SND file from the filesystem.
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Snd, SffError> {
        let data = std::fs::read(path)?;
        Self::parse_bytes(&data)
    }

    /// Parse SND content from raw bytes.
    ///
    /// MUGEN SND v1 layout:
    /// ```text
    /// Bytes  0-11  "ElecbyteSnd\0"  (12-byte signature)
    /// Bytes 12-15  Version           (u32 big-endian, 0x01000001 = v1.0)
    /// Bytes 16-19  Number of sounds  (u32 little-endian)
    /// Bytes 20-23  First subfile offset (u32 little-endian)
    ///
    /// Each subfile entry (at its offset):
    ///   0- 3  Offset to NEXT subfile (u32 LE, absolute from file start; 0 = this is last)
    ///   4- 7  Length of audio data   (u32 LE)
    ///   8- 9  Sound group            (u16 LE)
    ///  10-11  Sound number           (u16 LE)
    ///  12     Loop flag              (u8)
    ///  13-15  Reserved
    ///  16+    Name string (null-terminated, up to 16 bytes)
    /// After the 32-byte entry header: raw audio data
    /// ```
    pub fn parse_bytes(data: &[u8]) -> Result<Snd, SffError> {
        if data.len() < 24 {
            return Err(SffError::InvalidSignature([0u8; 12]));
        }

        // Validate signature.
        let sig: [u8; 12] = data[0..12].try_into().unwrap();
        if &sig != b"ElecbyteSnd\0" {
            return Err(SffError::InvalidSignature(sig));
        }

        // Version (bytes 12-15, big-endian). Accept both 1.0 and 1.1.
        let _version = u32::from_be_bytes([data[12], data[13], data[14], data[15]]);

        // Number of sounds (bytes 16-19, little-endian).
        let num_sounds = u32::from_le_bytes([data[16], data[17], data[18], data[19]]) as usize;

        // Offset to the first subfile header (bytes 20-23, little-endian).
        let first_offset = u32::from_le_bytes([data[20], data[21], data[22], data[23]]) as usize;

        let mut entries = Vec::with_capacity(num_sounds);
        let mut current_offset = first_offset;

        for _ in 0..num_sounds {
            // Each subfile header is 32 bytes minimum.
            if current_offset + 16 > data.len() {
                break;
            }

            // Offset to the next subfile (absolute from file start).
            let next_offset = u32::from_le_bytes([
                data[current_offset],
                data[current_offset + 1],
                data[current_offset + 2],
                data[current_offset + 3],
            ]) as usize;

            // Length of raw audio data.
            let data_length = u32::from_le_bytes([
                data[current_offset + 4],
                data[current_offset + 5],
                data[current_offset + 6],
                data[current_offset + 7],
            ]);

            // Sound group and number.
            let group = u16::from_le_bytes([data[current_offset + 8], data[current_offset + 9]]);
            let sound = u16::from_le_bytes([data[current_offset + 10], data[current_offset + 11]]);

            // Loop flag (byte 12 of the subfile header).
            let loopflag = if current_offset + 12 < data.len() {
                data[current_offset + 12]
            } else {
                0
            };

            // Audio data follows the 32-byte subfile header.
            let data_offset = (current_offset + 32) as u64;

            entries.push(SoundEntry {
                group,
                sound,
                data_offset,
                data_length,
                loopflag,
            });

            if next_offset == 0 {
                break;
            }
            current_offset = next_offset;
        }

        Ok(Snd { entries })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_signature() {
        let bad_data = b"NotAnSndFile\0\0\0\0\0\0\0\0\0\0\0\0";
        let result = SndParser::parse_bytes(bad_data);
        assert!(result.is_err(), "Should fail with invalid signature");
    }

    #[test]
    fn test_too_short() {
        let short_data = b"ElecbyteSnd";
        let result = SndParser::parse_bytes(short_data);
        assert!(result.is_err(), "Should fail on truncated data");
    }

    #[test]
    fn test_parse_kyo_snd() {
        let path = "../../mugen_resources/KyoKusanagi[SuzukiInoue]/kyo.snd";
        if !std::path::Path::new(path).exists() {
            eprintln!("Skipping test: {} not found", path);
            return;
        }
        let snd = SndParser::parse(path).expect("Failed to parse kyo.snd");
        assert!(!snd.entries.is_empty(), "kyo.snd should have sound entries");
        println!("kyo.snd: loaded {} sounds", snd.sound_count());
        // Print first few entries for debugging.
        for entry in snd.entries.iter().take(5) {
            println!(
                "  group={} sound={} offset={} len={} loop={}",
                entry.group, entry.sound, entry.data_offset, entry.data_length, entry.loopflag
            );
        }
    }

    #[test]
    fn test_get_sound_lookup() {
        let path = "../../mugen_resources/KyoKusanagi[SuzukiInoue]/kyo.snd";
        if !std::path::Path::new(path).exists() {
            return;
        }
        let snd = SndParser::parse(path).expect("Failed to parse kyo.snd");
        // The Snd should have a lookup method.
        // We can't assert a specific group/sound without knowing the file contents,
        // but we verify the lookup returns something for any existing entry.
        if let Some(first) = snd.entries.first() {
            let g = first.group;
            let s = first.sound;
            let found = snd.get_sound(g, s);
            assert!(found.is_some(), "get_sound should find the first entry");
        }
    }
}
