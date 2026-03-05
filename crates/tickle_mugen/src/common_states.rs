use crate::cns::{MoveType, Physics, StateDef, StateTypeValue};
use std::collections::HashMap;

// Note: controllers will be added when controller parsing is implemented

/// Returns the built-in common state definitions.
/// These correspond to MUGEN's common1.cns states.
/// Character-specific states from their .cns file take precedence.
pub fn common_states() -> HashMap<i32, StateDef> {
    let mut states = HashMap::new();

    // State 0 - Stand
    states.insert(
        0,
        StateDef {
            state_num: 0,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(0),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: Some(0),
            facep2: Some(1),
            hitdefpersist: Some(0),
            movehitpersist: Some(0),
            hitcountpersist: Some(0),
            sprpriority: Some(0),
            controllers: Vec::new(),
        },
    );

    // State 10 - Stand to Crouch
    states.insert(
        10,
        StateDef {
            state_num: 10,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(10),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 11 - Crouch
    states.insert(
        11,
        StateDef {
            state_num: 11,
            state_type: StateTypeValue::Crouching,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Crouching),
            anim: Some(11),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: Some(0),
            facep2: Some(1),
            hitdefpersist: Some(0),
            movehitpersist: Some(0),
            hitcountpersist: Some(0),
            sprpriority: Some(0),
            controllers: Vec::new(),
        },
    );

    // State 12 - Crouch to Stand
    states.insert(
        12,
        StateDef {
            state_num: 12,
            state_type: StateTypeValue::Crouching,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(12),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 20 - Walk Forward
    states.insert(
        20,
        StateDef {
            state_num: 20,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(20),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: Some(1),
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 21 - Walk Backward
    states.insert(
        21,
        StateDef {
            state_num: 21,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(21),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: None,
            facep2: Some(1),
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 40 - Jump Start
    states.insert(
        40,
        StateDef {
            state_num: 40,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(40),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: Some(1),
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 41 - Jump Up
    states.insert(
        41,
        StateDef {
            state_num: 41,
            state_type: StateTypeValue::Aerial,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Aerial),
            anim: Some(41),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: Some(0),
            facep2: Some(1),
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 42 - Jump Down (same anim as 41 but falling)
    states.insert(
        42,
        StateDef {
            state_num: 42,
            state_type: StateTypeValue::Aerial,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Aerial),
            anim: Some(41),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: Some(0),
            facep2: Some(1),
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 43 - Jump Land
    states.insert(
        43,
        StateDef {
            state_num: 43,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(43),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 45 - Air Jump Start
    states.insert(
        45,
        StateDef {
            state_num: 45,
            state_type: StateTypeValue::Aerial,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Aerial),
            anim: Some(45),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 46 - Air Jump
    states.insert(
        46,
        StateDef {
            state_num: 46,
            state_type: StateTypeValue::Aerial,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Aerial),
            anim: Some(46),
            velset: None,
            ctrl: Some(1),
            poweradd: None,
            juggle: Some(0),
            facep2: Some(1),
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 100 - Run Forward
    states.insert(
        100,
        StateDef {
            state_num: 100,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(100),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 105 - Run Backward
    states.insert(
        105,
        StateDef {
            state_num: 105,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(105),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 120 - Guard Start
    states.insert(
        120,
        StateDef {
            state_num: 120,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(120),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 130 - Guard Stand
    states.insert(
        130,
        StateDef {
            state_num: 130,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(130),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 131 - Guard Crouch
    states.insert(
        131,
        StateDef {
            state_num: 131,
            state_type: StateTypeValue::Crouching,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Crouching),
            anim: Some(131),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 132 - Guard Air
    states.insert(
        132,
        StateDef {
            state_num: 132,
            state_type: StateTypeValue::Aerial,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Aerial),
            anim: Some(132),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 140 - Guard End
    states.insert(
        140,
        StateDef {
            state_num: 140,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(140),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 5000 - Hitstun Stand Light
    states.insert(
        5000,
        StateDef {
            state_num: 5000,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::BeingHit),
            physics: Some(Physics::None),
            anim: Some(5000),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 5001 - Hitstun Stand Medium
    states.insert(
        5001,
        StateDef {
            state_num: 5001,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::BeingHit),
            physics: Some(Physics::None),
            anim: Some(5000),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 5002 - Hitstun Stand Heavy
    states.insert(
        5002,
        StateDef {
            state_num: 5002,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::BeingHit),
            physics: Some(Physics::None),
            anim: Some(5000),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 5010 - Hitstun Crouch Light
    states.insert(
        5010,
        StateDef {
            state_num: 5010,
            state_type: StateTypeValue::Crouching,
            movetype: Some(MoveType::BeingHit),
            physics: Some(Physics::None),
            anim: Some(5010),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 5020 - Hitstun Air Light
    states.insert(
        5020,
        StateDef {
            state_num: 5020,
            state_type: StateTypeValue::Aerial,
            movetype: Some(MoveType::BeingHit),
            physics: Some(Physics::Aerial),
            anim: Some(5020),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 5070 - Trip
    states.insert(
        5070,
        StateDef {
            state_num: 5070,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::BeingHit),
            physics: Some(Physics::None),
            anim: Some(5070),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 5080 - Liedown
    states.insert(
        5080,
        StateDef {
            state_num: 5080,
            state_type: StateTypeValue::Lying,
            movetype: Some(MoveType::BeingHit),
            physics: Some(Physics::None),
            anim: Some(5080),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    // State 5100 - Getup
    states.insert(
        5100,
        StateDef {
            state_num: 5100,
            state_type: StateTypeValue::Standing,
            movetype: Some(MoveType::Idle),
            physics: Some(Physics::Standing),
            anim: Some(5100),
            velset: None,
            ctrl: Some(0),
            poweradd: None,
            juggle: None,
            facep2: None,
            hitdefpersist: None,
            movehitpersist: None,
            hitcountpersist: None,
            sprpriority: None,
            controllers: Vec::new(),
        },
    );

    states
}

/// Merge common states with character-specific states.
/// Character states take priority over common states.
pub fn merge_with_common(char_states: HashMap<i32, StateDef>) -> HashMap<i32, StateDef> {
    let mut result = common_states();
    for (num, state) in char_states {
        result.insert(num, state);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_states_returns_expected_states() {
        let states = common_states();
        assert!(states.contains_key(&0)); // Stand
        assert!(states.contains_key(&11)); // Crouch
        assert!(states.contains_key(&41)); // Jump Up
        assert!(states.contains_key(&5000)); // Hitstun
        assert!(states.len() >= 20);
    }

    #[test]
    fn test_stand_state_properties() {
        let states = common_states();
        let stand = states.get(&0).unwrap();
        assert_eq!(stand.state_type, StateTypeValue::Standing);
        assert_eq!(stand.anim, Some(0));
        assert_eq!(stand.ctrl, Some(1));
    }

    #[test]
    fn test_crouch_state_properties() {
        let states = common_states();
        let crouch = states.get(&11).unwrap();
        assert_eq!(crouch.state_type, StateTypeValue::Crouching);
        assert_eq!(crouch.physics, Some(Physics::Crouching));
        assert_eq!(crouch.ctrl, Some(1));
    }

    #[test]
    fn test_jump_up_state_properties() {
        let states = common_states();
        let jump_up = states.get(&41).unwrap();
        assert_eq!(jump_up.state_type, StateTypeValue::Aerial);
        assert_eq!(jump_up.physics, Some(Physics::Aerial));
        assert_eq!(jump_up.ctrl, Some(1));
    }

    #[test]
    fn test_hitstun_states_have_being_hit_movetype() {
        let states = common_states();
        for &state_num in &[5000, 5001, 5002, 5010, 5020, 5070, 5080] {
            let state = states.get(&state_num).unwrap();
            assert_eq!(
                state.movetype,
                Some(MoveType::BeingHit),
                "State {} should have BeingHit movetype",
                state_num
            );
            assert_eq!(
                state.ctrl,
                Some(0),
                "State {} should have ctrl=0",
                state_num
            );
        }
    }

    #[test]
    fn test_lying_state() {
        let states = common_states();
        let liedown = states.get(&5080).unwrap();
        assert_eq!(liedown.state_type, StateTypeValue::Lying);
    }

    #[test]
    fn test_merge_character_overrides_common() {
        let mut char_states = std::collections::HashMap::new();
        char_states.insert(
            0,
            StateDef {
                state_num: 0,
                state_type: StateTypeValue::Standing,
                movetype: None,
                physics: None,
                anim: Some(999), // character override
                velset: None,
                ctrl: Some(1),
                poweradd: None,
                juggle: None,
                facep2: None,
                hitdefpersist: None,
                movehitpersist: None,
                hitcountpersist: None,
                sprpriority: None,
                controllers: Vec::new(),
            },
        );
        let merged = merge_with_common(char_states);
        assert_eq!(merged.get(&0).unwrap().anim, Some(999));
    }

    #[test]
    fn test_merge_preserves_common_states_not_overridden() {
        let char_states = std::collections::HashMap::new();
        let merged = merge_with_common(char_states);
        // All common states should still be present
        assert!(merged.contains_key(&0));
        assert!(merged.contains_key(&11));
        assert!(merged.contains_key(&41));
        assert!(merged.contains_key(&5000));
    }

    #[test]
    fn test_merge_adds_character_specific_states() {
        let mut char_states = std::collections::HashMap::new();
        char_states.insert(
            200,
            StateDef {
                state_num: 200,
                state_type: StateTypeValue::Standing,
                movetype: Some(MoveType::Attack),
                physics: Some(Physics::Standing),
                anim: Some(200),
                velset: None,
                ctrl: Some(0),
                poweradd: Some(30),
                juggle: None,
                facep2: None,
                hitdefpersist: None,
                movehitpersist: None,
                hitcountpersist: None,
                sprpriority: None,
                controllers: Vec::new(),
            },
        );
        let merged = merge_with_common(char_states);
        assert!(merged.contains_key(&200));
        assert_eq!(merged.get(&200).unwrap().movetype, Some(MoveType::Attack));
    }
}
