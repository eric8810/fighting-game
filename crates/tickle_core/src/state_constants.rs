// MUGEN-compatible state number constants
// Standard states (0-5999) are reserved for common character states
// Custom states (6000+) are character-specific

// Standing states
pub const STATE_STAND: i32 = 0;
pub const STATE_STAND_TO_CROUCH: i32 = 10;

// Crouching states
pub const STATE_CROUCH: i32 = 11;
pub const STATE_CROUCH_TO_STAND: i32 = 12;

// Walking states
pub const STATE_WALK_FORWARD: i32 = 20;
pub const STATE_WALK_BACKWARD: i32 = 21;

// Running states
pub const STATE_RUN_FORWARD: i32 = 100;
pub const STATE_RUN_BACKWARD: i32 = 105;

// Jump states
pub const STATE_JUMP_START: i32 = 40;
pub const STATE_JUMP_UP: i32 = 41;
pub const STATE_JUMP_DOWN: i32 = 42;
pub const STATE_JUMP_LAND: i32 = 43;

// Air states
pub const STATE_AIR_JUMP_START: i32 = 45;
pub const STATE_AIR_JUMP_UP: i32 = 46;
pub const STATE_AIR_JUMP_DOWN: i32 = 47;

// Guard states
pub const STATE_GUARD_START: i32 = 120;
pub const STATE_GUARD_STAND: i32 = 130;
pub const STATE_GUARD_CROUCH: i32 = 131;
pub const STATE_GUARD_AIR: i32 = 132;
pub const STATE_GUARD_END: i32 = 140;

// Hit states (being hit)
pub const STATE_HIT_STAND_LIGHT: i32 = 5000;
pub const STATE_HIT_STAND_MEDIUM: i32 = 5001;
pub const STATE_HIT_STAND_HEAVY: i32 = 5002;
pub const STATE_HIT_CROUCH_LIGHT: i32 = 5010;
pub const STATE_HIT_CROUCH_MEDIUM: i32 = 5011;
pub const STATE_HIT_CROUCH_HEAVY: i32 = 5012;
pub const STATE_HIT_AIR_LIGHT: i32 = 5020;
pub const STATE_HIT_AIR_MEDIUM: i32 = 5021;
pub const STATE_HIT_AIR_HEAVY: i32 = 5022;

// Knockdown states
pub const STATE_HIT_TRIP: i32 = 5070;
pub const STATE_HIT_LIEDOWN: i32 = 5080;
pub const STATE_HIT_GETUP: i32 = 5100;

// Attack states (character-specific, typically 200-999)
pub const STATE_ATTACK_LIGHT: i32 = 200;
pub const STATE_ATTACK_MEDIUM: i32 = 210;
pub const STATE_ATTACK_HEAVY: i32 = 220;
pub const STATE_ATTACK_SPECIAL: i32 = 1000;
pub const STATE_ATTACK_SUPER: i32 = 3000;

/// Helper function to check if a state is a standing state
pub fn is_standing_state(state_num: i32) -> bool {
    matches!(state_num, STATE_STAND | STATE_WALK_FORWARD | STATE_WALK_BACKWARD)
}

/// Helper function to check if a state is a crouching state
pub fn is_crouching_state(state_num: i32) -> bool {
    matches!(state_num, STATE_CROUCH | STATE_STAND_TO_CROUCH | STATE_CROUCH_TO_STAND)
}

/// Helper function to check if a state is an aerial state
pub fn is_aerial_state(state_num: i32) -> bool {
    matches!(
        state_num,
        STATE_JUMP_UP
            | STATE_JUMP_DOWN
            | STATE_AIR_JUMP_UP
            | STATE_AIR_JUMP_DOWN
            | STATE_HIT_AIR_LIGHT
            | STATE_HIT_AIR_MEDIUM
            | STATE_HIT_AIR_HEAVY
    )
}

/// Helper function to check if a state is a hit state
pub fn is_hit_state(state_num: i32) -> bool {
    (5000..=5099).contains(&state_num)
}

/// Helper function to check if a state is a guard state
pub fn is_guard_state(state_num: i32) -> bool {
    (120..=140).contains(&state_num)
}

/// Helper function to check if a state is an attack state
pub fn is_attack_state(state_num: i32) -> bool {
    (200..=999).contains(&state_num) || (1000..=4999).contains(&state_num)
}
