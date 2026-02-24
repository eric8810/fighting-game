use crate::{Result, SffError};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// AIR animation file
pub struct Air {
    /// Actions indexed by action number
    actions: HashMap<u32, Action>,
}

/// Single animation action
#[derive(Clone, Debug)]
pub struct Action {
    /// Action number (e.g., 0 = standing idle)
    pub number: u32,
    /// Animation frames
    pub frames: Vec<Frame>,
    /// Default Clsn2 (hurtbox) for all frames
    pub clsn2_default: Vec<Clsn>,
    /// Default Clsn1 (hitbox) for all frames
    pub clsn1_default: Vec<Clsn>,
}

/// Single animation frame
#[derive(Clone, Debug)]
pub struct Frame {
    /// Sprite group number
    pub group: u16,
    /// Sprite image number
    pub image: u16,
    /// X offset from character position
    pub x_offset: i16,
    /// Y offset from character position
    pub y_offset: i16,
    /// Duration in ticks (60 ticks = 1 second)
    pub duration: i32,
    /// Flip flags
    pub flip: FlipFlags,
    /// Frame-specific Clsn2 (hurtbox) - overrides default
    pub clsn2: Option<Vec<Clsn>>,
    /// Frame-specific Clsn1 (hitbox) - overrides default
    pub clsn1: Option<Vec<Clsn>>,
}

/// Flip flags
#[derive(Clone, Debug, Default)]
pub struct FlipFlags {
    pub horizontal: bool,
    pub vertical: bool,
}

/// Collision box (AABB)
#[derive(Clone, Debug)]
pub struct Clsn {
    /// Left edge (X coordinate)
    pub left: i16,
    /// Top edge (Y coordinate, negative = up)
    pub top: i16,
    /// Right edge (X coordinate)
    pub right: i16,
    /// Bottom edge (Y coordinate, negative = up)
    pub bottom: i16,
}

impl Air {
    /// Load AIR file from path
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        Self::parse(&content)
    }

    /// Parse AIR from string
    fn parse(content: &str) -> Result<Self> {
        let mut actions = HashMap::new();
        let mut current_action: Option<Action> = None;
        let mut current_clsn2_default: Vec<Clsn> = Vec::new();
        let mut current_clsn1_default: Vec<Clsn> = Vec::new();
        let mut pending_clsn2: Option<Vec<Clsn>> = None;
        let mut pending_clsn1: Option<Vec<Clsn>> = None;

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            // Begin Action
            if line.starts_with("[Begin Action ") {
                // Save previous action
                if let Some(mut action) = current_action.take() {
                    action.clsn2_default = current_clsn2_default.clone();
                    action.clsn1_default = current_clsn1_default.clone();
                    actions.insert(action.number, action);
                }

                // Parse action number
                let num_str = line.trim_start_matches("[Begin Action ")
                    .trim_end_matches(']')
                    .trim();
                let number = num_str.parse::<u32>()
                    .map_err(|_| SffError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Invalid action number: {}", num_str)
                    )))?;

                current_action = Some(Action {
                    number,
                    frames: Vec::new(),
                    clsn2_default: Vec::new(),
                    clsn1_default: Vec::new(),
                });
                current_clsn2_default.clear();
                current_clsn1_default.clear();
                pending_clsn2 = None;
                pending_clsn1 = None;
                continue;
            }

            // Clsn2Default
            if line.starts_with("Clsn2Default:") {
                let count_str = line.trim_start_matches("Clsn2Default:").trim();
                let _count = count_str.parse::<usize>().unwrap_or(0);
                current_clsn2_default.clear();
                continue;
            }

            // Clsn1Default
            if line.starts_with("Clsn1Default:") {
                let count_str = line.trim_start_matches("Clsn1Default:").trim();
                let _count = count_str.parse::<usize>().unwrap_or(0);
                current_clsn1_default.clear();
                continue;
            }

            // Clsn2 (frame-specific)
            if line.starts_with("Clsn2:") {
                let count_str = line.trim_start_matches("Clsn2:").trim();
                let _count = count_str.parse::<usize>().unwrap_or(0);
                pending_clsn2 = Some(Vec::new());
                continue;
            }

            // Clsn1 (frame-specific)
            if line.starts_with("Clsn1:") {
                let count_str = line.trim_start_matches("Clsn1:").trim();
                let _count = count_str.parse::<usize>().unwrap_or(0);
                pending_clsn1 = Some(Vec::new());
                continue;
            }

            // Clsn2[n] = left, top, right, bottom
            if line.contains("Clsn2[") {
                if let Some(clsn) = Self::parse_clsn_line(line) {
                    if let Some(ref mut pending) = pending_clsn2 {
                        pending.push(clsn);
                    } else {
                        current_clsn2_default.push(clsn);
                    }
                }
                continue;
            }

            // Clsn1[n] = left, top, right, bottom
            if line.contains("Clsn1[") {
                if let Some(clsn) = Self::parse_clsn_line(line) {
                    if let Some(ref mut pending) = pending_clsn1 {
                        pending.push(clsn);
                    } else {
                        current_clsn1_default.push(clsn);
                    }
                }
                continue;
            }

            // Loopstart marker
            if line == "Loopstart" {
                // TODO: handle loop start marker
                continue;
            }

            // Frame line: group, image, x, y, duration [, flip] [, ...]
            if let Some(frame) = Self::parse_frame_line(line) {
                if let Some(ref mut action) = current_action {
                    let mut frame = frame;
                    frame.clsn2 = pending_clsn2.take();
                    frame.clsn1 = pending_clsn1.take();
                    action.frames.push(frame);
                }
            }
        }

        // Save last action
        if let Some(mut action) = current_action {
            action.clsn2_default = current_clsn2_default;
            action.clsn1_default = current_clsn1_default;
            actions.insert(action.number, action);
        }

        log::info!("Loaded {} actions from AIR", actions.len());
        Ok(Self { actions })
    }

    /// Parse a Clsn line: "Clsn2[0] = -10, 0, 10, -79"
    fn parse_clsn_line(line: &str) -> Option<Clsn> {
        let parts: Vec<&str> = line.split('=').collect();
        if parts.len() != 2 {
            return None;
        }

        let coords: Vec<&str> = parts[1].split(',').map(|s| s.trim()).collect();
        if coords.len() < 4 {
            return None;
        }

        Some(Clsn {
            left: coords[0].parse().ok()?,
            top: coords[1].parse().ok()?,
            right: coords[2].parse().ok()?,
            bottom: coords[3].parse().ok()?,
        })
    }

    /// Parse a frame line: "0,0, 0,0, 10" or "5,0, 0,0, 4, H"
    fn parse_frame_line(line: &str) -> Option<Frame> {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() < 5 {
            return None;
        }

        let group = parts[0].parse().ok()?;
        let image = parts[1].parse().ok()?;
        let x_offset = parts[2].parse().ok()?;
        let y_offset = parts[3].parse().ok()?;
        let duration = parts[4].parse().ok()?;

        let mut flip = FlipFlags::default();
        if parts.len() > 5 {
            let flip_str = parts[5].to_uppercase();
            flip.horizontal = flip_str.contains('H');
            flip.vertical = flip_str.contains('V');
        }

        Some(Frame {
            group,
            image,
            x_offset,
            y_offset,
            duration,
            flip,
            clsn2: None,
            clsn1: None,
        })
    }

    /// Get action by number
    pub fn get_action(&self, number: u32) -> Option<&Action> {
        self.actions.get(&number)
    }

    /// Get all action numbers
    pub fn action_numbers(&self) -> Vec<u32> {
        let mut nums: Vec<u32> = self.actions.keys().copied().collect();
        nums.sort();
        nums
    }

    /// Number of actions loaded
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_kfm_air() {
        let path = "../../assets/mugen/kfm/kfm.air";
        if std::path::Path::new(path).exists() {
            let air = Air::load(path).expect("Failed to load kfm.air");
            assert!(air.action_count() > 0, "Should load at least one action");

            println!("Loaded {} actions", air.action_count());

            // Check Action 0 (Standing Idle)
            if let Some(action) = air.get_action(0) {
                println!("Action 0 (Standing): {} frames", action.frames.len());
                println!("  Clsn2Default: {} boxes", action.clsn2_default.len());

                for (i, frame) in action.frames.iter().take(5).enumerate() {
                    println!("  Frame {}: ({},{}) offset=({},{}) dur={}",
                        i, frame.group, frame.image, frame.x_offset, frame.y_offset, frame.duration);
                }
            }

            // List first 10 actions
            let nums = air.action_numbers();
            println!("First 10 actions:");
            for num in nums.iter().take(10) {
                if let Some(action) = air.get_action(*num) {
                    println!("  Action {}: {} frames", num, action.frames.len());
                }
            }
        } else {
            println!("Skipping test: kfm.air not found at {}", path);
        }
    }
}
