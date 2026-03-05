use crate::error::SffError;
use crate::trigger::{TriggerExpr, TriggerParser};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct CnsData {
    pub life: i32,
    pub attack: i32,
    pub defence: i32,
    pub fall_defence_mul: i32,
    pub liedown_time: i32,
    pub airjuggle: i32,
    pub sparkno: i32,
    pub guard_sparkno: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CnsSize {
    pub xscale: f32,
    pub yscale: f32,
    pub ground_back: i32,
    pub ground_front: i32,
    pub air_back: i32,
    pub air_front: i32,
    pub height: i32,
    pub attack_dist: i32,
    pub proj_attack_dist: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CnsVelocity {
    pub walk_fwd: Vec2,
    pub walk_back: Vec2,
    pub run_fwd: Vec2,
    pub run_back: Vec2,
    pub jump_neu: Vec2,
    pub jump_back: Vec2,
    pub jump_fwd: Vec2,
    pub runjump_back: Vec2,
    pub runjump_fwd: Vec2,
    pub airjump_neu: Vec2,
    pub airjump_back: Vec2,
    pub airjump_fwd: Vec2,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CnsMovement {
    pub airjump_num: i32,
    pub airjump_height: f32,
    pub yaccel: f32,
    pub stand_friction: f32,
    pub crouch_friction: f32,
    pub stand_friction_threshold: Option<f32>,
    pub crouch_friction_threshold: Option<f32>,
    pub air_gethit_groundlevel: Option<f32>,
    pub air_gethit_groundrecover_ground_threshold: Option<f32>,
    pub air_gethit_airrecover_threshold: Option<f32>,
    pub air_gethit_airrecover_yaccel: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateTypeValue {
    Standing,
    Crouching,
    Aerial,
    Lying,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MoveType {
    Idle,
    Attack,
    BeingHit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Physics {
    Standing,
    Crouching,
    Aerial,
    None,
}

// ─── Controller types ────────────────────────────────────────────────────────

/// A single parsed controller from a [State N, Label] section.
#[derive(Debug, Clone, PartialEq)]
pub struct StateController {
    /// All triggerall conditions – every one must be true.
    pub triggerall: Vec<TriggerExpr>,
    /// Each element corresponds to a trigger-group (trigger1, trigger2, …).
    /// Within a group every condition must be true (AND).
    /// The controller fires if ANY group is satisfied (OR between groups).
    pub triggers: Vec<Vec<TriggerExpr>>,
    /// The action to perform.
    pub controller: Controller,
}

impl StateController {
    /// Returns true when the controller should fire given the context.
    pub fn should_fire(&self, ctx: &crate::trigger::TriggerContext) -> bool {
        // All triggerall conditions must pass.
        if !self.triggerall.iter().all(|t| t.evaluate_bool(ctx)) {
            return false;
        }
        // At least one trigger group must pass.
        if self.triggers.is_empty() {
            return false;
        }
        self.triggers
            .iter()
            .any(|group| group.iter().all(|t| t.evaluate_bool(ctx)))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Controller {
    ChangeState {
        value: TriggerExpr,
        ctrl: Option<TriggerExpr>,
    },
    VelSet {
        x: Option<TriggerExpr>,
        y: Option<TriggerExpr>,
    },
    VelAdd {
        x: Option<TriggerExpr>,
        y: Option<TriggerExpr>,
    },
    PosAdd {
        x: Option<TriggerExpr>,
        y: Option<TriggerExpr>,
    },
    PosSet {
        x: Option<TriggerExpr>,
        y: Option<TriggerExpr>,
    },
    VarSet {
        /// The variable index extracted from e.g. `var(8) = value`
        var_num: i32,
        value: TriggerExpr,
    },
    VarAdd {
        var_num: i32,
        value: TriggerExpr,
    },
    CtrlSet {
        value: TriggerExpr,
    },
    ChangeAnim {
        value: TriggerExpr,
    },
    PlaySnd {
        group: i32,
        sound: i32,
    },
    AssertSpecial {
        flags: Vec<String>,
    },
    HitDef {
        attr: String,
        hitflag: String,
        guardflag: String,
        animtype: String,
        air_type: String,
        ground_type: String,
        damage: i32,
        guard_damage: i32,
        pausetime_p1: i32,
        pausetime_p2: i32,
        sparkno: Option<i32>,
        guard_sparkno: Option<i32>,
        spark_xy: (i32, i32),
        hit_sound: Option<(i32, i32)>,
        guard_sound: Option<(i32, i32)>,
        ground_slidetime: i32,
        guard_slidetime: i32,
        ground_hittime: i32,
        guard_hittime: i32,
        air_hittime: i32,
        ground_velocity_x: f32,
        ground_velocity_y: f32,
        air_velocity_x: f32,
        air_velocity_y: f32,
        guard_velocity_x: f32,
        down_velocity_x: f32,
        down_velocity_y: f32,
        yaccel: f32,
        getpower_hit: i32,
        getpower_guard: i32,
        fall: bool,
        fall_recover: bool,
        priority: i32,
        p1stateno: Option<i32>,
        p2stateno: Option<i32>,
    },
    NotHitBy {
        attr: String,
        time: i32,
    },
    HitBy {
        attr: String,
        time: i32,
    },
    VarRandom {
        /// Variable index to store the random value in.
        var_num: i32,
        /// Minimum value (inclusive). Default 0.
        range_min: i32,
        /// Maximum value (inclusive). Default 999.
        range_max: i32,
    },
    Gravity,
    Unknown(String),
}

// ─── StateDef ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct StateDef {
    pub state_num: i32,
    pub state_type: StateTypeValue,
    pub movetype: Option<MoveType>,
    pub physics: Option<Physics>,
    pub anim: Option<i32>,
    pub velset: Option<Vec2>,
    pub ctrl: Option<i32>,
    pub poweradd: Option<i32>,
    pub juggle: Option<i32>,
    pub facep2: Option<i32>,
    pub hitdefpersist: Option<i32>,
    pub movehitpersist: Option<i32>,
    pub hitcountpersist: Option<i32>,
    pub sprpriority: Option<i32>,
    /// Controllers belonging to this statedef (from [State N, …] sections).
    pub controllers: Vec<StateController>,
}

// ─── Cns ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Cns {
    pub data: CnsData,
    pub size: CnsSize,
    pub velocity: CnsVelocity,
    pub movement: CnsMovement,
    pub statedefs: HashMap<i32, StateDef>,
    /// Controllers from [State -1, …] sections (global state, runs every tick).
    pub global_state_controllers: Vec<StateController>,
}

impl Cns {
    pub fn get_state(&self, state_num: i32) -> Option<&StateDef> {
        self.statedefs.get(&state_num)
    }
}

/// Merge two statedef maps. Character-specific states override common states.
pub fn merge_statedefs(
    common: HashMap<i32, StateDef>,
    character: HashMap<i32, StateDef>,
) -> HashMap<i32, StateDef> {
    let mut merged = common;
    merged.extend(character);
    merged
}

// ─────────────────────────────────────────────────────────────────────────────
// Parsing helpers that work on Vec<(String,String)>
// ─────────────────────────────────────────────────────────────────────────────

/// Look up the last value for a key (case-insensitive on the stored key).
fn kv_get<'a>(data: &'a [(String, String)], key: &str) -> Option<&'a str> {
    let key_lower = key.to_lowercase();
    data.iter()
        .rev()
        .find(|(k, _)| k.to_lowercase() == key_lower)
        .map(|(_, v)| v.as_str())
}

/// Collect all values for a key (preserving order).
fn kv_get_all<'a>(data: &'a [(String, String)], key: &str) -> Vec<&'a str> {
    let key_lower = key.to_lowercase();
    data.iter()
        .filter(|(k, _)| k.to_lowercase() == key_lower)
        .map(|(_, v)| v.as_str())
        .collect()
}

fn kv_int(data: &[(String, String)], key: &str) -> Result<i32, SffError> {
    kv_get(data, key)
        .ok_or_else(|| SffError::MissingField(key.to_string()))?
        .parse()
        .map_err(|_| SffError::DefParse(format!("Invalid integer for {}", key)))
}

fn kv_int_or(data: &[(String, String)], key: &str, default: i32) -> i32 {
    kv_get(data, key)
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn kv_int_opt(data: &[(String, String)], key: &str) -> Option<i32> {
    kv_get(data, key).and_then(|v| v.parse().ok())
}

fn kv_float(data: &[(String, String)], key: &str) -> Result<f32, SffError> {
    kv_get(data, key)
        .ok_or_else(|| SffError::MissingField(key.to_string()))?
        .parse()
        .map_err(|_| SffError::DefParse(format!("Invalid float for {}", key)))
}

fn kv_float_opt(data: &[(String, String)], key: &str) -> Option<f32> {
    kv_get(data, key).and_then(|v| v.parse().ok())
}

fn kv_parse_vec2_from_str(value: &str, key: &str) -> Result<Vec2, SffError> {
    let parts: Vec<&str> = value.split(',').collect();
    if parts.len() != 2 {
        return Err(SffError::DefParse(format!("Invalid Vec2 for {}: {}", key, value)));
    }
    let x = parts[0].trim().parse::<f32>()
        .map_err(|_| SffError::DefParse(format!("Invalid x component for {}: {}", key, parts[0])))?;
    let y = parts[1].trim().parse::<f32>()
        .map_err(|_| SffError::DefParse(format!("Invalid y component for {}: {}", key, parts[1])))?;
    Ok(Vec2::new(x, y))
}

fn kv_vec2_pair(data: &[(String, String)], key: &str) -> Result<Vec2, SffError> {
    let value = kv_get(data, key)
        .ok_or_else(|| SffError::MissingField(key.to_string()))?;
    kv_parse_vec2_from_str(value, key)
}

fn kv_vec2_single_or_pair(data: &[(String, String)], key: &str) -> Result<Vec2, SffError> {
    let value = kv_get(data, key)
        .ok_or_else(|| SffError::MissingField(key.to_string()))?;
    if value.contains(',') {
        kv_parse_vec2_from_str(value, key)
    } else {
        let x = value.parse::<f32>()
            .map_err(|_| SffError::DefParse(format!("Invalid float for {}: {}", key, value)))?;
        Ok(Vec2::new(x, 0.0))
    }
}

fn kv_vec2_pair_opt(data: &[(String, String)], key: &str) -> Option<Vec2> {
    kv_get(data, key).and_then(|v| kv_parse_vec2_from_str(v, key).ok())
}

// ─── HitDef parsing helpers ──────────────────────────────────────────────────

/// Parse a comma-separated pair of integers (e.g., "40,10" or just "40").
fn parse_int_pair(value: &str, default_second: i32) -> (i32, i32) {
    let parts: Vec<&str> = value.split(',').collect();
    let first = parts[0].trim().parse::<i32>().unwrap_or(0);
    let second = if parts.len() > 1 {
        parts[1].trim().parse::<i32>().unwrap_or(default_second)
    } else {
        default_second
    };
    (first, second)
}

/// Parse a comma-separated pair of floats (e.g., "-3.5,-6" or just "-3.5").
fn parse_float_pair(value: &str, default_second: f32) -> (f32, f32) {
    let parts: Vec<&str> = value.split(',').collect();
    let first = parts[0].trim().parse::<f32>().unwrap_or(0.0);
    let second = if parts.len() > 1 {
        parts[1].trim().parse::<f32>().unwrap_or(default_second)
    } else {
        default_second
    };
    (first, second)
}

/// Parse a sound value like "S200,0" or "200,0" into (group, sound).
/// The "S" prefix indicates a character-specific sound.
fn parse_sound_value(value: &str) -> Option<(i32, i32)> {
    let v = value.trim().strip_prefix('S').unwrap_or(value.trim());
    let parts: Vec<&str> = v.split(',').collect();
    if parts.len() >= 2 {
        let group = parts[0].trim().parse::<i32>().ok()?;
        let sound = parts[1].trim().parse::<i32>().ok()?;
        Some((group, sound))
    } else {
        parts[0].trim().parse::<i32>().ok().map(|g| (g, 0))
    }
}

// ─── Trigger parsing helpers ──────────────────────────────────────────────────

/// Parse a TriggerExpr from a string, stripping inline comments (`;`).
fn parse_trigger_str(s: &str) -> Result<TriggerExpr, SffError> {
    // Strip inline comments
    let s = if let Some(pos) = s.find(';') {
        s[..pos].trim()
    } else {
        s.trim()
    };
    TriggerParser::parse(s)
}

/// Parse a required TriggerExpr param.
fn parse_trigger_param(data: &[(String, String)], key: &str) -> Result<TriggerExpr, SffError> {
    let s = kv_get(data, key)
        .ok_or_else(|| SffError::MissingField(key.to_string()))?;
    parse_trigger_str(s)
}

/// Parse an optional TriggerExpr param.
fn parse_trigger_param_opt(data: &[(String, String)], key: &str) -> Option<TriggerExpr> {
    kv_get(data, key).and_then(|s| parse_trigger_str(s).ok())
}

// ─────────────────────────────────────────────────────────────────────────────
// CnsParser
// ─────────────────────────────────────────────────────────────────────────────

pub struct CnsParser;

impl CnsParser {
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Cns, SffError> {
        let content = fs::read_to_string(path)?;
        Self::parse_content(&content)
    }

    pub fn parse_content(content: &str) -> Result<Cns, SffError> {
        let mut data: Option<CnsData> = None;
        let mut size: Option<CnsSize> = None;
        let mut velocity: Option<CnsVelocity> = None;
        let mut movement: Option<CnsMovement> = None;
        let mut statedefs: HashMap<i32, StateDef> = HashMap::new();
        // state_num → list of controllers (collected before merging into statedefs)
        let mut state_controllers: HashMap<i32, Vec<StateController>> = HashMap::new();

        let mut current_section = String::new();
        // Use Vec<(String,String)> to preserve duplicate keys (trigger1, triggerall, etc.)
        let mut section_data: Vec<(String, String)> = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and full-line comments
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            // Section header
            if line.starts_with('[') {
                if let Some(close_bracket) = line.find(']') {
                    // Process previous section before moving on
                    if !current_section.is_empty() {
                        Self::process_section(
                            &current_section,
                            &section_data,
                            &mut data,
                            &mut size,
                            &mut velocity,
                            &mut movement,
                            &mut statedefs,
                            &mut state_controllers,
                        )?;
                    }

                    current_section = line[1..close_bracket].trim().to_string();
                    section_data.clear();
                    continue;
                }
            }

            // Parse key=value pairs (strip inline comments after the value)
            if let Some(eq_pos) = line.find('=') {
                let raw_key = line[..eq_pos].trim();
                let raw_value = line[eq_pos + 1..].trim();

                // Strip inline comment from value
                let value = if let Some(sc) = raw_value.find(';') {
                    raw_value[..sc].trim().to_string()
                } else {
                    raw_value.to_string()
                };

                // Store with lowercase key for consistent lookup
                section_data.push((raw_key.to_lowercase(), value));
            }
        }

        // Process the last section
        if !current_section.is_empty() {
            Self::process_section(
                &current_section,
                &section_data,
                &mut data,
                &mut size,
                &mut velocity,
                &mut movement,
                &mut statedefs,
                &mut state_controllers,
            )?;
        }

        // Collect global state controllers (State -1) before iterating
        let global_state_controllers = state_controllers
            .remove(&-1)
            .unwrap_or_default();

        // Merge state controllers into their statedefs
        for (state_num, controllers) in state_controllers {
            if let Some(statedef) = statedefs.get_mut(&state_num) {
                statedef.controllers.extend(controllers);
            }
            // Controllers for state numbers without a matching Statedef are
            // silently discarded (common with helper/OtherPlayer state blocks).
        }

        Ok(Cns {
            data: data.ok_or_else(|| SffError::MissingField("Data section".to_string()))?,
            size: size.ok_or_else(|| SffError::MissingField("Size section".to_string()))?,
            velocity: velocity.ok_or_else(|| SffError::MissingField("Velocity section".to_string()))?,
            movement: movement.ok_or_else(|| SffError::MissingField("Movement section".to_string()))?,
            statedefs,
            global_state_controllers,
        })
    }

    /// Parse only Statedef and State sections from a CNS-format string.
    /// Unlike `parse_content`, this does not require [Data]/[Size]/[Velocity]/[Movement] sections.
    /// Returns `(statedefs, global_controllers)` where global_controllers are from [Statedef -1].
    pub fn parse_statedefs_only(content: &str) -> (HashMap<i32, StateDef>, Vec<StateController>) {
        let mut statedefs: HashMap<i32, StateDef> = HashMap::new();
        let mut state_controllers: HashMap<i32, Vec<StateController>> = HashMap::new();
        let mut current_section = String::new();
        let mut section_data: Vec<(String, String)> = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if line.starts_with('[') {
                if let Some(close_bracket) = line.find(']') {
                    if !current_section.is_empty() {
                        Self::collect_statedef_section(
                            &current_section, &section_data,
                            &mut statedefs, &mut state_controllers,
                        );
                    }
                    current_section = line[1..close_bracket].trim().to_string();
                    section_data.clear();
                    continue;
                }
            }
            if let Some(eq_pos) = line.find('=') {
                let raw_key = line[..eq_pos].trim();
                let raw_value = line[eq_pos + 1..].trim();
                let value = if let Some(sc) = raw_value.find(';') {
                    raw_value[..sc].trim().to_string()
                } else {
                    raw_value.to_string()
                };
                section_data.push((raw_key.to_lowercase(), value));
            }
        }
        if !current_section.is_empty() {
            Self::collect_statedef_section(
                &current_section, &section_data,
                &mut statedefs, &mut state_controllers,
            );
        }

        // Extract statedef -1 global controllers (same as parse_content does).
        let global_controllers = state_controllers.remove(&-1).unwrap_or_default();

        for (state_num, controllers) in state_controllers {
            if let Some(sd) = statedefs.get_mut(&state_num) {
                sd.controllers.extend(controllers);
            }
        }
        (statedefs, global_controllers)
    }

    fn collect_statedef_section(
        section: &str,
        data: &[(String, String)],
        statedefs: &mut HashMap<i32, StateDef>,
        state_controllers: &mut HashMap<i32, Vec<StateController>>,
    ) {
        let lower = section.to_lowercase();
        if lower.starts_with("statedef ") || lower.starts_with("statedef-") {
            if let Ok(Some(sd)) = Self::parse_statedef_section(section, data) {
                statedefs.insert(sd.state_num, sd);
            }
        } else if lower.starts_with("state ") || lower.starts_with("state\t") {
            if let Some(state_num) = Self::extract_state_number(section) {
                if let Some(controller) = Self::parse_state_controller(data) {
                    state_controllers.entry(state_num).or_default().push(controller);
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn process_section(
        section: &str,
        data: &[(String, String)],
        cns_data: &mut Option<CnsData>,
        cns_size: &mut Option<CnsSize>,
        cns_velocity: &mut Option<CnsVelocity>,
        cns_movement: &mut Option<CnsMovement>,
        statedefs: &mut HashMap<i32, StateDef>,
        state_controllers: &mut HashMap<i32, Vec<StateController>>,
    ) -> Result<(), SffError> {
        let lower = section.to_lowercase();
        match lower.as_str() {
            "data" => {
                *cns_data = Some(Self::parse_data_section(data)?);
            }
            "size" => {
                *cns_size = Some(Self::parse_size_section(data)?);
            }
            "velocity" => {
                *cns_velocity = Some(Self::parse_velocity_section(data)?);
            }
            "movement" => {
                *cns_movement = Some(Self::parse_movement_section(data)?);
            }
            _ if lower.starts_with("statedef ") || lower.starts_with("statedef-") => {
                if let Some(statedef) = Self::parse_statedef_section(section, data)? {
                    statedefs.insert(statedef.state_num, statedef);
                }
            }
            _ if lower.starts_with("state ") || lower.starts_with("state\t") => {
                // [State N] or [State N, Label]
                if let Some(state_num) = Self::extract_state_number(section) {
                    if let Some(controller) = Self::parse_state_controller(data) {
                        state_controllers
                            .entry(state_num)
                            .or_default()
                            .push(controller);
                    }
                }
            }
            _ => {
                // Ignore unknown sections
            }
        }
        Ok(())
    }

    /// Extract the state number from a section name like "State 200, VelSet" or "State -1".
    fn extract_state_number(section: &str) -> Option<i32> {
        // section starts with "State " (case preserved from original)
        let rest = section.trim();
        // skip the "State" prefix (5 chars) and any following whitespace
        let after_state = rest.get(5..)?.trim_start();
        // The number ends at ',' or end-of-string
        let num_str = if let Some(comma) = after_state.find(',') {
            &after_state[..comma]
        } else {
            after_state
        };
        num_str.trim().parse::<i32>().ok()
    }

    /// Parse a [State N, …] section into a StateController.
    fn parse_state_controller(data: &[(String, String)]) -> Option<StateController> {
        // Determine controller type
        let type_str = kv_get(data, "type")?.trim().to_lowercase();

        // Collect triggerall conditions (there may be multiple)
        let triggerall: Vec<TriggerExpr> = kv_get_all(data, "triggerall")
            .iter()
            .filter_map(|s| parse_trigger_str(s).ok())
            .collect();

        // Collect per-trigger groups (trigger1, trigger2, …)
        // Multiple lines with the same key (e.g., two `trigger1 = …`) form an AND-group.
        let mut trigger_groups: HashMap<u32, Vec<TriggerExpr>> = HashMap::new();
        for (key, value) in data {
            let key_lower = key.to_lowercase();
            if let Some(rest) = key_lower.strip_prefix("trigger") {
                // rest is a number like "1", "2", …
                if let Ok(idx) = rest.parse::<u32>() {
                    if idx == 0 {
                        // trigger0 is not standard; skip
                        continue;
                    }
                    if let Ok(expr) = parse_trigger_str(value) {
                        trigger_groups.entry(idx).or_default().push(expr);
                    }
                }
            }
        }

        // Order groups by index and collect into the triggers Vec
        let mut group_indices: Vec<u32> = trigger_groups.keys().copied().collect();
        group_indices.sort_unstable();
        let triggers: Vec<Vec<TriggerExpr>> = group_indices
            .iter()
            .filter_map(|idx| trigger_groups.remove(idx))
            .collect();

        // Parse the controller action
        let controller = Self::parse_controller_action(&type_str, data);

        Some(StateController {
            triggerall,
            triggers,
            controller,
        })
    }

    fn parse_controller_action(type_str: &str, data: &[(String, String)]) -> Controller {
        match type_str {
            "changestate" => {
                let value = match parse_trigger_param(data, "value") {
                    Ok(e) => e,
                    Err(_) => return Controller::Unknown("changestate:missing_value".to_string()),
                };
                Controller::ChangeState {
                    value,
                    ctrl: parse_trigger_param_opt(data, "ctrl"),
                }
            }
            "velset" => Controller::VelSet {
                x: parse_trigger_param_opt(data, "x"),
                y: parse_trigger_param_opt(data, "y"),
            },
            "veladd" => Controller::VelAdd {
                x: parse_trigger_param_opt(data, "x"),
                y: parse_trigger_param_opt(data, "y"),
            },
            "posadd" => Controller::PosAdd {
                x: parse_trigger_param_opt(data, "x"),
                y: parse_trigger_param_opt(data, "y"),
            },
            "posset" => Controller::PosSet {
                x: parse_trigger_param_opt(data, "x"),
                y: parse_trigger_param_opt(data, "y"),
            },
            "varset" => {
                // The assignment looks like: var(8) = 0
                // stored as key="var(8)", value="0"
                // We search for a key that matches the pattern var(\d+)
                let (var_num, value) = Self::find_varset_assignment(data);
                Controller::VarSet { var_num, value }
            }
            "varadd" => {
                let (var_num, value) = Self::find_varset_assignment(data);
                Controller::VarAdd { var_num, value }
            }
            "ctrlset" => {
                let value = match parse_trigger_param(data, "value") {
                    Ok(e) => e,
                    Err(_) => return Controller::Unknown("ctrlset:missing_value".to_string()),
                };
                Controller::CtrlSet { value }
            }
            "changeanim" => {
                let value = match parse_trigger_param(data, "value") {
                    Ok(e) => e,
                    Err(_) => return Controller::Unknown("changeanim:missing_value".to_string()),
                };
                Controller::ChangeAnim { value }
            }
            "playsnd" => {
                // value = group, sound   e.g. "30,0" or "30, 0"
                let (group, sound) = kv_get(data, "value")
                    .and_then(|v| {
                        let parts: Vec<&str> = v.split(',').collect();
                        if parts.len() >= 2 {
                            let g = parts[0].trim().parse::<i32>().ok()?;
                            let s = parts[1].trim().parse::<i32>().ok()?;
                            Some((g, s))
                        } else {
                            parts[0].trim().parse::<i32>().ok().map(|g| (g, 0))
                        }
                    })
                    .unwrap_or((0, 0));
                Controller::PlaySnd { group, sound }
            }
            "assertspecial" => {
                // Collect all `flag` values
                let flags: Vec<String> = kv_get_all(data, "flag")
                    .iter()
                    .map(|s| s.trim().to_string())
                    .collect();
                Controller::AssertSpecial { flags }
            }
            "hitdef" => Self::parse_hitdef(data),
            "nothitby" => {
                let attr = kv_get(data, "value")
                    .or_else(|| kv_get(data, "attr"))
                    .unwrap_or("")
                    .to_string();
                let time = kv_int_or(data, "time", 1);
                Controller::NotHitBy { attr, time }
            }
            "hitby" => {
                let attr = kv_get(data, "value")
                    .or_else(|| kv_get(data, "attr"))
                    .unwrap_or("")
                    .to_string();
                let time = kv_int_or(data, "time", 1);
                Controller::HitBy { attr, time }
            }
            "varrandom" => {
                // v = N (variable index) or var(N) style
                let var_num = if let Some(v_str) = kv_get(data, "v") {
                    v_str.parse::<i32>().unwrap_or(0)
                } else {
                    // Search for var(N) style key
                    data.iter()
                        .find(|(k, _)| {
                            let kl = k.to_lowercase();
                            kl.starts_with("var(") && kl.ends_with(')')
                        })
                        .and_then(|(k, _)| {
                            let kl = k.to_lowercase();
                            kl[4..kl.len() - 1].parse::<i32>().ok()
                        })
                        .unwrap_or(0)
                };
                // range = min, max  (default 0, 999)
                let (range_min, range_max) = kv_get(data, "range")
                    .and_then(|v| {
                        let parts: Vec<&str> = v.split(',').collect();
                        if parts.len() == 2 {
                            let lo = parts[0].trim().parse::<i32>().ok()?;
                            let hi = parts[1].trim().parse::<i32>().ok()?;
                            Some((lo, hi))
                        } else if parts.len() == 1 {
                            // Single value means 0..value
                            let hi = parts[0].trim().parse::<i32>().ok()?;
                            Some((0, hi))
                        } else {
                            None
                        }
                    })
                    .unwrap_or((0, 999));
                Controller::VarRandom {
                    var_num,
                    range_min,
                    range_max,
                }
            }
            "gravity" => Controller::Gravity,
            other => Controller::Unknown(other.to_string()),
        }
    }

    /// Search data for a key matching `var(\d+)` and parse its value as a TriggerExpr.
    fn find_varset_assignment(data: &[(String, String)]) -> (i32, TriggerExpr) {
        for (key, value) in data {
            let key_lower = key.to_lowercase();
            if key_lower.starts_with("var(") && key_lower.ends_with(')') {
                let inner = &key_lower[4..key_lower.len() - 1];
                if let Ok(idx) = inner.parse::<i32>() {
                    if let Ok(expr) = parse_trigger_str(value) {
                        return (idx, expr);
                    }
                }
            }
        }
        // Fallback: v = N  style (less common)
        if let Some(v_str) = kv_get(data, "v") {
            if let Ok(idx) = v_str.parse::<i32>() {
                if let Some(val) = parse_trigger_param_opt(data, "value") {
                    return (idx, val);
                }
            }
        }
        (0, TriggerExpr::Int(0))
    }

    /// Parse a HitDef controller from section data.
    fn parse_hitdef(data: &[(String, String)]) -> Controller {
        let attr = kv_get(data, "attr").unwrap_or("S, NA").to_string();
        let hitflag = kv_get(data, "hitflag").unwrap_or("MAF").to_string();
        let guardflag = kv_get(data, "guardflag").unwrap_or("MA").to_string();
        let animtype = kv_get(data, "animtype").unwrap_or("Light").to_string();
        let air_type = kv_get(data, "air.type").unwrap_or("High").to_string();
        let ground_type = kv_get(data, "ground.type").unwrap_or("High").to_string();

        // Damage: "40" or "40,10" (hit_damage, guard_damage)
        let (damage, guard_damage) = kv_get(data, "damage")
            .map(|v| parse_int_pair(v, 0))
            .unwrap_or((0, 0));

        // Pausetime: "10,10" (p1_shake, p2_shake)
        let (pausetime_p1, pausetime_p2) = kv_get(data, "pausetime")
            .map(|v| parse_int_pair(v, 0))
            .unwrap_or((0, 0));

        // Spark
        let sparkno = kv_int_opt(data, "sparkno");
        let guard_sparkno = kv_int_opt(data, "guard.sparkno");
        let spark_xy = kv_get(data, "sparkxy")
            .map(|v| parse_int_pair(v, 0))
            .unwrap_or((0, 0));

        // Sounds: "S200,0" or "200,0"
        let hit_sound = kv_get(data, "hitsound").and_then(parse_sound_value);
        let guard_sound = kv_get(data, "guardsound").and_then(parse_sound_value);

        // Ground timing
        let ground_slidetime = kv_int_or(data, "ground.slidetime", 0);
        let guard_slidetime = kv_int_or(data, "guard.slidetime", 0);
        let ground_hittime = kv_int_or(data, "ground.hittime", 0);
        let guard_hittime = kv_int_or(data, "guard.hittime", 0);
        let air_hittime = kv_int_or(data, "air.hittime", 20);

        // Velocities: "-3.5" or "-3.5, -6"
        let (ground_velocity_x, ground_velocity_y) = kv_get(data, "ground.velocity")
            .map(|v| parse_float_pair(v, 0.0))
            .unwrap_or((0.0, 0.0));
        let (air_velocity_x, air_velocity_y) = kv_get(data, "air.velocity")
            .map(|v| parse_float_pair(v, 0.0))
            .unwrap_or((0.0, 0.0));
        let guard_velocity_x = kv_get(data, "guard.velocity")
            .and_then(|v| v.trim().parse::<f32>().ok())
            .unwrap_or(0.0);
        let (down_velocity_x, down_velocity_y) = kv_get(data, "down.velocity")
            .map(|v| parse_float_pair(v, 0.0))
            .unwrap_or((0.0, 0.0));

        let yaccel = kv_float_opt(data, "yaccel").unwrap_or(0.35);

        // Power
        let (getpower_hit, getpower_guard) = kv_get(data, "getpower")
            .map(|v| parse_int_pair(v, 0))
            .unwrap_or((0, 0));

        // Knockdown
        let fall = kv_int_or(data, "fall", 0) != 0;
        let fall_recover = kv_int_or(data, "fall.recover", 1) != 0;

        let priority = kv_int_or(data, "priority", 4);
        let p1stateno = kv_int_opt(data, "p1stateno");
        let p2stateno = kv_int_opt(data, "p2stateno");

        Controller::HitDef {
            attr,
            hitflag,
            guardflag,
            animtype,
            air_type,
            ground_type,
            damage,
            guard_damage,
            pausetime_p1,
            pausetime_p2,
            sparkno,
            guard_sparkno,
            spark_xy,
            hit_sound,
            guard_sound,
            ground_slidetime,
            guard_slidetime,
            ground_hittime,
            guard_hittime,
            air_hittime,
            ground_velocity_x,
            ground_velocity_y,
            air_velocity_x,
            air_velocity_y,
            guard_velocity_x,
            down_velocity_x,
            down_velocity_y,
            yaccel,
            getpower_hit,
            getpower_guard,
            fall,
            fall_recover,
            priority,
            p1stateno,
            p2stateno,
        }
    }

    // ─── Section parsers ───────────────────────────────────────────────────────

    fn parse_data_section(data: &[(String, String)]) -> Result<CnsData, SffError> {
        Ok(CnsData {
            life: kv_int(data, "life")?,
            attack: kv_int(data, "attack")?,
            defence: kv_int(data, "defence")?,
            fall_defence_mul: kv_int_or(data, "fall.defence_up", 100),
            liedown_time: kv_int(data, "liedown.time")?,
            airjuggle: kv_int(data, "airjuggle")?,
            sparkno: kv_int(data, "sparkno")?,
            guard_sparkno: kv_int(data, "guard.sparkno")?,
        })
    }

    fn parse_size_section(data: &[(String, String)]) -> Result<CnsSize, SffError> {
        Ok(CnsSize {
            xscale: kv_float(data, "xscale")?,
            yscale: kv_float(data, "yscale")?,
            ground_back: kv_int(data, "ground.back")?,
            ground_front: kv_int(data, "ground.front")?,
            air_back: kv_int(data, "air.back")?,
            air_front: kv_int(data, "air.front")?,
            height: kv_int(data, "height")?,
            attack_dist: kv_int(data, "attack.dist")?,
            proj_attack_dist: kv_int(data, "proj.attack.dist")?,
        })
    }

    fn parse_velocity_section(data: &[(String, String)]) -> Result<CnsVelocity, SffError> {
        Ok(CnsVelocity {
            walk_fwd: kv_vec2_single_or_pair(data, "walk.fwd")?,
            walk_back: kv_vec2_single_or_pair(data, "walk.back")?,
            run_fwd: kv_vec2_pair(data, "run.fwd")?,
            run_back: kv_vec2_pair(data, "run.back")?,
            jump_neu: kv_vec2_pair(data, "jump.neu")?,
            jump_back: kv_vec2_pair(data, "jump.back")?,
            jump_fwd: kv_vec2_pair(data, "jump.fwd")?,
            runjump_back: kv_vec2_pair(data, "runjump.back")?,
            runjump_fwd: kv_vec2_pair(data, "runjump.fwd")?,
            airjump_neu: kv_vec2_pair(data, "airjump.neu")?,
            airjump_back: kv_vec2_pair(data, "airjump.back")?,
            airjump_fwd: kv_vec2_pair(data, "airjump.fwd")?,
        })
    }

    fn parse_movement_section(data: &[(String, String)]) -> Result<CnsMovement, SffError> {
        Ok(CnsMovement {
            airjump_num: kv_int(data, "airjump.num")?,
            airjump_height: kv_float(data, "airjump.height")?,
            yaccel: kv_float(data, "yaccel")?,
            stand_friction: kv_float(data, "stand.friction")?,
            crouch_friction: kv_float(data, "crouch.friction")?,
            stand_friction_threshold: kv_float_opt(data, "stand.friction.threshold"),
            crouch_friction_threshold: kv_float_opt(data, "crouch.friction.threshold"),
            air_gethit_groundlevel: kv_float_opt(data, "air.gethit.groundlevel"),
            air_gethit_groundrecover_ground_threshold: kv_float_opt(data, "air.gethit.groundrecover.ground.threshold"),
            air_gethit_airrecover_threshold: kv_float_opt(data, "air.gethit.airrecover.threshold"),
            air_gethit_airrecover_yaccel: kv_float_opt(data, "air.gethit.airrecover.yaccel"),
        })
    }

    fn parse_statedef_section(section: &str, data: &[(String, String)]) -> Result<Option<StateDef>, SffError> {
        // Extract state number from "Statedef N" (may have negative: "Statedef -1")
        let parts: Vec<&str> = section.split_whitespace().collect();
        if parts.len() < 2 {
            return Ok(None);
        }

        let state_num = parts[1].parse::<i32>()
            .map_err(|_| SffError::DefParse(format!("Invalid state number: {}", parts[1])))?;

        let state_type = Self::parse_state_type_opt(kv_get(data, "type"));

        Ok(Some(StateDef {
            state_num,
            state_type,
            movetype: Self::parse_movetype_opt(kv_get(data, "movetype")),
            physics: Self::parse_physics_opt(kv_get(data, "physics")),
            anim: kv_int_opt(data, "anim"),
            velset: kv_vec2_pair_opt(data, "velset"),
            ctrl: kv_int_opt(data, "ctrl"),
            poweradd: kv_int_opt(data, "poweradd"),
            juggle: kv_int_opt(data, "juggle"),
            facep2: kv_int_opt(data, "facep2"),
            hitdefpersist: kv_int_opt(data, "hitdefpersist"),
            movehitpersist: kv_int_opt(data, "movehitpersist"),
            hitcountpersist: kv_int_opt(data, "hitcountpersist"),
            sprpriority: kv_int_opt(data, "sprpriority"),
            controllers: Vec::new(),
        }))
    }

    // ─── Value parsers ────────────────────────────────────────────────────────

    fn parse_state_type_opt(value: Option<&str>) -> StateTypeValue {
        let upper = value.map(|s| s.trim().to_uppercase());
        match upper.as_deref() {
            Some("S") => StateTypeValue::Standing,
            Some("C") => StateTypeValue::Crouching,
            Some("A") => StateTypeValue::Aerial,
            Some("L") => StateTypeValue::Lying,
            _ => StateTypeValue::Standing,
        }
    }

    fn parse_movetype_opt(value: Option<&str>) -> Option<MoveType> {
        value.and_then(|s| {
            let upper = s.trim().to_uppercase();
            match upper.as_str() {
                "I" => Some(MoveType::Idle),
                "A" => Some(MoveType::Attack),
                "H" => Some(MoveType::BeingHit),
                _ => None,
            }
        })
    }

    fn parse_physics_opt(value: Option<&str>) -> Option<Physics> {
        value.and_then(|s| {
            let upper = s.trim().to_uppercase();
            match upper.as_str() {
                "S" => Some(Physics::Standing),
                "C" => Some(Physics::Crouching),
                "A" => Some(Physics::Aerial),
                "N" => Some(Physics::None),
                _ => None,
            }
        })
    }
}
