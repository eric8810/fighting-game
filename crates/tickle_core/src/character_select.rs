use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::math::LogicRect;

/// Move category for cancel chain validation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MoveCategory {
    NormalLight,
    NormalMedium,
    NormalHeavy,
    Special,
    Super,
}

/// Frame range within a move.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FrameRange {
    pub start: u32,
    pub end: u32,
}

/// Hitbox data as stored in RON move files.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveHitbox {
    pub rect: RonRect,
    pub damage: i32,
    pub hitstun: u32,
    pub blockstun: u32,
    pub knockback: RonVec2,
    pub hit_type: String,
}

/// Hurtbox data as stored in RON move files.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveHurtbox {
    pub rect: RonRect,
}

/// Pushbox data as stored in RON move files.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MovePushbox {
    pub rect: RonRect,
}

/// Simple rect for RON deserialization (matches the RON format).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RonRect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl RonRect {
    pub fn to_logic_rect(&self) -> LogicRect {
        LogicRect::new(self.x, self.y, self.w, self.h)
    }
}

/// Simple 2D vector for RON deserialization.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RonVec2 {
    pub x: i32,
    pub y: i32,
}

/// A single frame group within a move (covers a range of frames).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveFrameGroup {
    pub frame_range: FrameRange,
    pub hitboxes: Vec<MoveHitbox>,
    pub hurtboxes: Vec<MoveHurtbox>,
    pub pushbox: MovePushbox,
}

/// Cancel window as stored in RON move files.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoveCancelWindow {
    pub start: u32,
    pub end: u32,
    pub into: Vec<String>,
}

/// Complete move data loaded from a RON file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoveData {
    pub id: String,
    pub name: String,
    pub startup: u32,
    pub active: u32,
    pub recovery: u32,
    pub total_frames: u32,
    pub damage: i32,
    pub stun_damage: i32,
    pub gauge_gain_on_hit: i32,
    pub gauge_gain_on_block: i32,
    pub on_hit_advantage: i32,
    pub on_block_advantage: i32,
    #[serde(default)]
    pub power_cost: i32,
    pub cancel_windows: Vec<MoveCancelWindow>,
    pub frames: Vec<MoveFrameGroup>,
    pub animation: String,
    pub hit_effect: String,
    pub hit_sound: String,
    pub block_sound: String,
}

/// PLACEHOLDER_CANCEL_CHAIN

/// Cancel chain entry: defines what a move can cancel into.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelChainEntry {
    pub from: String,
    pub into: Vec<String>,
}

/// Animation frame reference for RON.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimFrame {
    pub atlas_index: u32,
    pub offset: RonVec2,
}

/// Animation definition for RON.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AnimDef {
    pub texture: String,
    pub frames: Vec<AnimFrame>,
    pub frame_duration: u32,
    pub looping: bool,
}

/// Complete character data loaded from a RON file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacterData {
    pub id: String,
    pub name: String,
    pub health: i32,
    pub walk_speed: i32,
    pub back_walk_speed: i32,
    pub run_speed: i32,
    pub jump_velocity: i32,
    #[serde(default)]
    pub jump_forward_velocity: i32,
    #[serde(default)]
    pub jump_backward_velocity: i32,
    pub gravity: i32,
    pub pushbox_width: i32,
    pub pushbox_height: i32,
    #[serde(default = "default_power_gauge_max")]
    pub power_gauge_max: i32,
    pub standing_hurtbox: RonRect,
    pub crouching_hurtbox: RonRect,
    pub jumping_hurtbox: RonRect,
    pub moves: Vec<String>,
    #[serde(default)]
    pub cancel_chains: Vec<CancelChainEntry>,
    #[serde(default)]
    pub animations: HashMap<String, AnimDef>,
}

fn default_power_gauge_max() -> i32 {
    3000
}

/// PLACEHOLDER_CHAR_PROPS

/// Runtime character properties applied to a fighter entity.
#[derive(Clone, Debug)]
pub struct CharacterProperties {
    pub id: String,
    pub name: String,
    pub health: i32,
    pub walk_speed: i32,
    pub back_walk_speed: i32,
    pub run_speed: i32,
    pub jump_velocity: i32,
    pub jump_forward_velocity: i32,
    pub jump_backward_velocity: i32,
    pub gravity: i32,
    pub pushbox_width: i32,
    pub pushbox_height: i32,
    pub power_gauge_max: i32,
    pub standing_hurtbox: LogicRect,
    pub crouching_hurtbox: LogicRect,
    pub jumping_hurtbox: LogicRect,
    pub move_ids: Vec<String>,
    pub cancel_chains: Vec<CancelChainEntry>,
}

impl CharacterProperties {
    /// Build runtime properties from deserialized character data.
    pub fn from_data(data: &CharacterData) -> Self {
        Self {
            id: data.id.clone(),
            name: data.name.clone(),
            health: data.health,
            walk_speed: data.walk_speed,
            back_walk_speed: data.back_walk_speed,
            run_speed: data.run_speed,
            jump_velocity: data.jump_velocity,
            jump_forward_velocity: data.jump_forward_velocity,
            jump_backward_velocity: data.jump_backward_velocity,
            gravity: data.gravity,
            pushbox_width: data.pushbox_width,
            pushbox_height: data.pushbox_height,
            power_gauge_max: data.power_gauge_max,
            standing_hurtbox: data.standing_hurtbox.to_logic_rect(),
            crouching_hurtbox: data.crouching_hurtbox.to_logic_rect(),
            jumping_hurtbox: data.jumping_hurtbox.to_logic_rect(),
            move_ids: data.moves.clone(),
            cancel_chains: data.cancel_chains.clone(),
        }
    }

    /// Check if a move can cancel into another move based on cancel chains.
    pub fn can_cancel(&self, from_move: &str, into_move: &str) -> bool {
        self.cancel_chains
            .iter()
            .any(|chain| chain.from == from_move && chain.into.contains(&into_move.to_string()))
    }
}

/// PLACEHOLDER_LOADER

/// Load character data from a RON file.
pub fn load_character_data(path: &Path) -> Result<CharacterData, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read character file '{}': {}", path.display(), e))?;
    ron::from_str(&contents)
        .map_err(|e| format!("Failed to parse character file '{}': {}", path.display(), e))
}

/// Load move data from a RON file.
pub fn load_move_data(path: &Path) -> Result<MoveData, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read move file '{}': {}", path.display(), e))?;
    ron::from_str(&contents)
        .map_err(|e| format!("Failed to parse move file '{}': {}", path.display(), e))
}

/// Load a character and all its moves from the assets directory.
pub fn load_character_with_moves(
    characters_dir: &Path,
    moves_dir: &Path,
    character_id: &str,
) -> Result<(CharacterProperties, HashMap<String, MoveData>), String> {
    let char_path = characters_dir.join(format!("{}.ron", character_id));
    let char_data = load_character_data(&char_path)?;
    let props = CharacterProperties::from_data(&char_data);

    let mut moves = HashMap::new();
    for move_id in &char_data.moves {
        let move_path = moves_dir.join(format!("{}.ron", move_id));
        let move_data = load_move_data(&move_path)?;
        moves.insert(move_id.clone(), move_data);
    }

    Ok((props, moves))
}

/// PLACEHOLDER_TESTS

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_test_dirs(test_name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
        let base = std::env::temp_dir().join(format!("tickle_char_test_{}", test_name));
        let _ = fs::remove_dir_all(&base);
        let chars_dir = base.join("characters");
        let moves_dir = base.join("moves");
        fs::create_dir_all(&chars_dir).unwrap();
        fs::create_dir_all(&moves_dir).unwrap();
        (chars_dir, moves_dir)
    }

    fn write_test_character(chars_dir: &Path) {
        let content = r#"(
    id: "test_char",
    name: "Test Fighter",
    health: 10000,
    walk_speed: 500,
    back_walk_speed: 350,
    run_speed: 900,
    jump_velocity: 1800,
    jump_forward_velocity: 600,
    jump_backward_velocity: -500,
    gravity: -80,
    pushbox_width: 3000,
    pushbox_height: 8000,
    power_gauge_max: 3000,
    standing_hurtbox: (x: -2000, y: 0, w: 4000, h: 8000),
    crouching_hurtbox: (x: -2000, y: 0, w: 4500, h: 5000),
    jumping_hurtbox: (x: -1500, y: -1000, w: 3000, h: 6000),
    moves: ["test_move_lp", "test_move_hp"],
    cancel_chains: [
        (from: "test_move_lp", into: ["test_move_hp"]),
    ],
)"#;
        fs::write(chars_dir.join("test_char.ron"), content).unwrap();
    }

    /// PLACEHOLDER_TEST_MOVE

    fn write_test_move(moves_dir: &Path, id: &str, damage: i32) {
        let content = format!(
            r#"(
    id: "{id}",
    name: "Test Move",
    startup: 3,
    active: 3,
    recovery: 6,
    total_frames: 12,
    damage: {damage},
    stun_damage: 30,
    gauge_gain_on_hit: 5,
    gauge_gain_on_block: 3,
    on_hit_advantage: 3,
    on_block_advantage: 1,
    cancel_windows: [
        (start: 4, end: 6, into: ["special"]),
    ],
    frames: [
        (
            frame_range: (start: 1, end: 3),
            hitboxes: [],
            hurtboxes: [(rect: (x: -2000, y: 0, w: 4000, h: 8000))],
            pushbox: (rect: (x: -1500, y: 0, w: 3000, h: 8000)),
        ),
        (
            frame_range: (start: 4, end: 6),
            hitboxes: [(
                rect: (x: 1500, y: -5500, w: 4000, h: 2000),
                damage: {damage},
                hitstun: 10,
                blockstun: 8,
                knockback: (x: 5000, y: 0),
                hit_type: "mid",
            )],
            hurtboxes: [(rect: (x: -2000, y: 0, w: 4000, h: 8000))],
            pushbox: (rect: (x: -1500, y: 0, w: 3000, h: 8000)),
        ),
        (
            frame_range: (start: 7, end: 12),
            hitboxes: [],
            hurtboxes: [(rect: (x: -2000, y: 0, w: 4000, h: 8000))],
            pushbox: (rect: (x: -1500, y: 0, w: 3000, h: 8000)),
        ),
    ],
    animation: "{id}_anim",
    hit_effect: "spark_light",
    hit_sound: "hit_light_01",
    block_sound: "block_01",
)"#
        );
        fs::write(moves_dir.join(format!("{}.ron", id)), content).unwrap();
    }

    #[test]
    fn test_load_character_data() {
        let (chars_dir, _moves_dir) = setup_test_dirs("load_char");
        write_test_character(&chars_dir);

        let data = load_character_data(&chars_dir.join("test_char.ron")).unwrap();
        assert_eq!(data.id, "test_char");
        assert_eq!(data.health, 10000);
        assert_eq!(data.walk_speed, 500);
        assert_eq!(data.moves.len(), 2);
        assert_eq!(data.cancel_chains.len(), 1);
    }

    /// PLACEHOLDER_MORE_TESTS

    #[test]
    fn test_load_move_data() {
        let (_chars_dir, moves_dir) = setup_test_dirs("load_move");
        write_test_move(&moves_dir, "test_move_lp", 400);

        let data = load_move_data(&moves_dir.join("test_move_lp.ron")).unwrap();
        assert_eq!(data.id, "test_move_lp");
        assert_eq!(data.damage, 400);
        assert_eq!(data.startup, 3);
        assert_eq!(data.total_frames, 12);
        assert_eq!(data.frames.len(), 3);
        assert_eq!(data.cancel_windows.len(), 1);
        assert_eq!(data.cancel_windows[0].start, 4);
        assert_eq!(data.cancel_windows[0].end, 6);
    }

    #[test]
    fn test_load_character_with_moves() {
        let (chars_dir, moves_dir) = setup_test_dirs("load_with_moves");
        write_test_character(&chars_dir);
        write_test_move(&moves_dir, "test_move_lp", 400);
        write_test_move(&moves_dir, "test_move_hp", 1000);

        let (props, moves) =
            load_character_with_moves(&chars_dir, &moves_dir, "test_char").unwrap();

        assert_eq!(props.id, "test_char");
        assert_eq!(props.health, 10000);
        assert_eq!(props.move_ids.len(), 2);
        assert_eq!(moves.len(), 2);
        assert_eq!(moves["test_move_lp"].damage, 400);
        assert_eq!(moves["test_move_hp"].damage, 1000);
    }

    #[test]
    fn test_cancel_chain_validation() {
        let (chars_dir, moves_dir) = setup_test_dirs("cancel_chain");
        write_test_character(&chars_dir);
        write_test_move(&moves_dir, "test_move_lp", 400);
        write_test_move(&moves_dir, "test_move_hp", 1000);

        let (props, _) =
            load_character_with_moves(&chars_dir, &moves_dir, "test_char").unwrap();

        // LP can cancel into HP
        assert!(props.can_cancel("test_move_lp", "test_move_hp"));
        // HP cannot cancel into LP (not in chain)
        assert!(!props.can_cancel("test_move_hp", "test_move_lp"));
        // Unknown move returns false
        assert!(!props.can_cancel("nonexistent", "test_move_hp"));
    }

    #[test]
    fn test_super_move_power_consumption() {
        // Verify that a super move's power_cost field is loaded correctly
        let (_, moves_dir) = setup_test_dirs("super_power");
        let content = r#"(
    id: "test_super",
    name: "Test Super",
    startup: 4,
    active: 8,
    recovery: 24,
    total_frames: 36,
    damage: 2800,
    stun_damage: 0,
    gauge_gain_on_hit: 0,
    gauge_gain_on_block: 0,
    on_hit_advantage: -999,
    on_block_advantage: -10,
    power_cost: 1000,
    cancel_windows: [],
    frames: [
        (
            frame_range: (start: 1, end: 36),
            hitboxes: [],
            hurtboxes: [(rect: (x: -2000, y: 0, w: 4000, h: 8000))],
            pushbox: (rect: (x: -1500, y: 0, w: 3000, h: 8000)),
        ),
    ],
    animation: "test_super_anim",
    hit_effect: "spark_super",
    hit_sound: "hit_super_01",
    block_sound: "block_02",
)"#;
        fs::write(moves_dir.join("test_super.ron"), content).unwrap();

        let data = load_move_data(&moves_dir.join("test_super.ron")).unwrap();
        assert_eq!(data.power_cost, 1000);

        // Verify PowerGauge can consume the cost
        let mut gauge = crate::PowerGauge::new();
        gauge.add(1500);
        assert!(gauge.can_use_super());
        assert!(gauge.consume(data.power_cost));
        assert_eq!(gauge.current, 500);

        // Not enough gauge
        assert!(!gauge.consume(data.power_cost));
        assert_eq!(gauge.current, 500); // unchanged
    }

    #[test]
    fn test_character_properties_from_data() {
        let data = CharacterData {
            id: "ryu".to_string(),
            name: "Ryu".to_string(),
            health: 10000,
            walk_speed: 500,
            back_walk_speed: 350,
            run_speed: 900,
            jump_velocity: 1800,
            jump_forward_velocity: 600,
            jump_backward_velocity: -500,
            gravity: -80,
            pushbox_width: 3000,
            pushbox_height: 8000,
            power_gauge_max: 3000,
            standing_hurtbox: RonRect { x: -2000, y: 0, w: 4000, h: 8000 },
            crouching_hurtbox: RonRect { x: -2000, y: 0, w: 4500, h: 5000 },
            jumping_hurtbox: RonRect { x: -1500, y: -1000, w: 3000, h: 6000 },
            moves: vec!["ryu_stand_lp".to_string()],
            cancel_chains: vec![],
            animations: HashMap::new(),
        };

        let props = CharacterProperties::from_data(&data);
        assert_eq!(props.health, 10000);
        assert_eq!(props.standing_hurtbox, LogicRect::new(-2000, 0, 4000, 8000));
        assert_eq!(props.jump_forward_velocity, 600);
    }
}
