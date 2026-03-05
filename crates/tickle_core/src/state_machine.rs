use crate::components::FighterState;
use crate::input::{InputState, BUTTON_A, BUTTON_B, BUTTON_C};
use crate::state_constants::*;
use serde::{Deserialize, Serialize};

/// Cancel window: a frame range during which an attack can be cancelled into
/// another state category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelWindow {
    pub start_frame: i32,
    pub end_frame: i32,
    pub allowed: CancelTarget,
}

/// What an attack can cancel into during a cancel window.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CancelTarget {
    Normal,  // cancel into normal attacks
    Special, // cancel into special moves
    Super,   // cancel into supers
    Any,     // cancel into anything
}

/// Configuration for an attack state: total duration and cancel windows.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttackData {
    pub total_frames: i32,
    pub cancel_windows: Vec<CancelWindow>,
}

/// Duration-based state config for hitstun / blockstun / knockdown.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StunConfig {
    pub duration: i32,
}

/// StateMachine wraps FighterState and enforces legal transitions.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateMachine {
    pub state: FighterState,
    /// If the current state is an attack, its data lives here.
    current_attack: Option<AttackData>,
    /// Duration for hitstun/blockstun/knockdown (set on entry).
    stun_duration: i32,
}

impl StateMachine {
    pub fn new() -> Self {
        Self {
            state: FighterState::new(),
            current_attack: None,
            stun_duration: 0,
        }
    }

    pub fn current_state(&self) -> i32 {
        self.state.state_num
    }

    pub fn state_frame(&self) -> i32 {
        self.state.state_frame
    }

    /// Advance the frame counter and handle automatic transitions
    /// (attack completion, stun recovery). Returns the new state if
    /// an automatic transition occurred.
    pub fn update(&mut self) -> Option<i32> {
        self.state.advance_frame();

        // Check if current state is an attack state
        if is_attack_state(self.state.state_num) {
            if let Some(ref atk) = self.current_attack {
                if self.state.state_frame >= atk.total_frames as i32 {
                    self.enter_state(STATE_STAND);
                    return Some(STATE_STAND);
                }
            }
            None
        } else if is_hit_state(self.state.state_num) || is_guard_state(self.state.state_num) {
            // Hitstun/blockstun recovery
            if self.stun_duration > 0 && self.state.state_frame >= self.stun_duration as i32 {
                self.enter_state(STATE_STAND);
                return Some(STATE_STAND);
            }
            None
        } else {
            None
        }
    }

    /// Try to transition based on player input. Returns `Some(new_state)` if
    /// the transition is valid, `None` if blocked.
    pub fn try_transition(&mut self, input: &InputState) -> Option<i32> {
        let desired = if self.state.state_num == STATE_STAND {
            self.transitions_from_idle(input)
        } else if self.state.state_num == STATE_WALK_FORWARD || self.state.state_num == STATE_WALK_BACKWARD {
            self.transitions_from_walk(input)
        } else if self.state.state_num == STATE_RUN_FORWARD {
            self.transitions_from_run(input)
        } else if self.state.state_num == STATE_CROUCH {
            self.transitions_from_crouch(input)
        } else if is_aerial_state(self.state.state_num) {
            self.transitions_from_jump(input)
        } else if is_attack_state(self.state.state_num) {
            self.transitions_from_attack(input)
        } else if is_hit_state(self.state.state_num) || is_guard_state(self.state.state_num) {
            None
        } else {
            None
        };

        if let Some(new_state) = desired {
            if is_attack_state(new_state) {
                // Attack animation: 5 frames × 4 duration = 20 total frames.
                // Cancel window opens at frame 8 (after startup) through frame 16.
                self.enter_attack(new_state, AttackData {
                    total_frames: 20,
                    cancel_windows: vec![CancelWindow {
                        start_frame: 8,
                        end_frame: 16,
                        allowed: CancelTarget::Any,
                    }],
                });
            } else {
                self.enter_state(new_state);
            }
        }
        desired
    }

    /// Force-enter a state (used by combat system for hitstun/blockstun).
    pub fn force_enter(&mut self, new_state: i32, duration: i32) {
        self.stun_duration = duration;
        self.current_attack = None;
        self.state.change_state(new_state);
    }

    /// Enter an attack state with associated frame data.
    pub fn enter_attack(&mut self, attack_state: i32, data: AttackData) {
        self.current_attack = Some(data);
        self.stun_duration = 0;
        self.state.change_state(attack_state);
    }

    /// Check if the current attack is in a cancel window that allows the
    /// given target category.
    pub fn in_cancel_window(&self, target: CancelTarget) -> bool {
        if let Some(ref atk) = self.current_attack {
            let frame = self.state.state_frame;
            atk.cancel_windows.iter().any(|w| {
                frame >= w.start_frame
                    && frame <= w.end_frame
                    && (w.allowed == target || w.allowed == CancelTarget::Any)
            })
        } else {
            false
        }
    }

    /// Check if the current attack is in any cancel window at the current frame.
    pub fn in_any_cancel_window(&self) -> bool {
        if let Some(ref atk) = self.current_attack {
            let frame = self.state.state_frame;
            atk.cancel_windows
                .iter()
                .any(|w| frame >= w.start_frame && frame <= w.end_frame)
        } else {
            false
        }
    }

    /// Get the current attack's allowed cancel targets at the current frame.
    pub fn current_cancel_targets(&self) -> Vec<CancelTarget> {
        if let Some(ref atk) = self.current_attack {
            let frame = self.state.state_frame;
            atk.cancel_windows
                .iter()
                .filter(|w| frame >= w.start_frame && frame <= w.end_frame)
                .map(|w| w.allowed)
                .collect()
        } else {
            Vec::new()
        }
    }

    // -- private helpers --

    fn enter_state(&mut self, new_state: i32) {
        self.current_attack = None;
        self.stun_duration = 0;
        self.state.change_state(new_state);
    }

    fn transitions_from_idle(&self, input: &InputState) -> Option<i32> {
        if input.is_pressed(BUTTON_A) || input.is_pressed(BUTTON_B) || input.is_pressed(BUTTON_C) {
            return Some(STATE_ATTACK_LIGHT);
        }
        if input.direction.is_up() {
            return Some(STATE_JUMP_UP);
        }
        if input.direction.is_down() {
            return Some(STATE_CROUCH);
        }
        if input.direction.is_right() {
            return Some(STATE_WALK_FORWARD);
        }
        if input.direction.is_left() {
            return Some(STATE_WALK_BACKWARD);
        }
        None
    }

    fn transitions_from_walk(&self, input: &InputState) -> Option<i32> {
        if input.is_pressed(BUTTON_A) || input.is_pressed(BUTTON_B) || input.is_pressed(BUTTON_C) {
            return Some(STATE_ATTACK_LIGHT);
        }
        if input.direction.is_up() {
            return Some(STATE_JUMP_UP);
        }
        if input.direction == crate::input::Direction::Neutral {
            return Some(STATE_STAND);
        }
        if input.direction.is_down() {
            return Some(STATE_CROUCH);
        }
        None
    }

    fn transitions_from_run(&self, input: &InputState) -> Option<i32> {
        if input.is_pressed(BUTTON_A) || input.is_pressed(BUTTON_B) || input.is_pressed(BUTTON_C) {
            return Some(STATE_ATTACK_LIGHT);
        }
        if input.direction.is_up() {
            return Some(STATE_JUMP_UP);
        }
        if input.direction == crate::input::Direction::Neutral {
            return Some(STATE_STAND);
        }
        None
    }

    fn transitions_from_crouch(&self, input: &InputState) -> Option<i32> {
        if input.is_pressed(BUTTON_A) || input.is_pressed(BUTTON_B) || input.is_pressed(BUTTON_C) {
            return Some(STATE_ATTACK_LIGHT);
        }
        if !input.direction.is_down() {
            return Some(STATE_STAND);
        }
        None
    }

    fn transitions_from_jump(&self, input: &InputState) -> Option<i32> {
        // Only air attacks allowed during jump
        if input.is_pressed(BUTTON_A) || input.is_pressed(BUTTON_B) || input.is_pressed(BUTTON_C) {
            return Some(STATE_ATTACK_LIGHT);
        }
        None
    }

    fn transitions_from_attack(&self, input: &InputState) -> Option<i32> {
        // Can only cancel during cancel windows
        if (self.in_cancel_window(CancelTarget::Normal) || self.in_cancel_window(CancelTarget::Any))
            && (input.is_pressed(BUTTON_A)
                || input.is_pressed(BUTTON_B)
                || input.is_pressed(BUTTON_C))
        {
            return Some(STATE_ATTACK_LIGHT);
        }
        None
    }

    /// Land from a jump (called externally when ground is detected).
    pub fn land(&mut self) {
        if is_aerial_state(self.state.state_num) || is_attack_state(self.state.state_num) {
            self.enter_state(STATE_STAND);
        }
    }
}

impl Default for StateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Direction;

    fn neutral() -> InputState {
        InputState::new(0, Direction::Neutral)
    }

    fn press_a() -> InputState {
        InputState::new(BUTTON_A, Direction::Neutral)
    }

    fn dir_up() -> InputState {
        InputState::new(0, Direction::Up)
    }

    fn dir_down() -> InputState {
        InputState::new(0, Direction::Down)
    }

    fn dir_right() -> InputState {
        InputState::new(0, Direction::Right)
    }

    fn dir_left() -> InputState {
        InputState::new(0, Direction::Left)
    }

    #[test]
    fn test_initial_state_is_idle() {
        let sm = StateMachine::new();
        assert_eq!(sm.current_state(), STATE_STAND);
        assert_eq!(sm.state_frame(), 0);
    }

    #[test]
    fn test_idle_to_walk_forward() {
        let mut sm = StateMachine::new();
        let result = sm.try_transition(&dir_right());
        assert_eq!(result, Some(STATE_WALK_FORWARD));
        assert_eq!(sm.current_state(), STATE_WALK_FORWARD);
        assert_eq!(sm.state_frame(), 0);
    }

    #[test]
    fn test_idle_to_walk_backward() {
        let mut sm = StateMachine::new();
        let result = sm.try_transition(&dir_left());
        assert_eq!(result, Some(STATE_WALK_BACKWARD));
        assert_eq!(sm.current_state(), STATE_WALK_BACKWARD);
    }

    #[test]
    fn test_idle_to_jump() {
        let mut sm = StateMachine::new();
        let result = sm.try_transition(&dir_up());
        assert_eq!(result, Some(STATE_JUMP_UP));
        assert_eq!(sm.current_state(), STATE_JUMP_UP);
    }

    #[test]
    fn test_idle_to_crouch() {
        let mut sm = StateMachine::new();
        let result = sm.try_transition(&dir_down());
        assert_eq!(result, Some(STATE_CROUCH));
        assert_eq!(sm.current_state(), STATE_CROUCH);
    }

    #[test]
    fn test_idle_to_attack() {
        let mut sm = StateMachine::new();
        let result = sm.try_transition(&press_a());
        assert_eq!(result, Some(STATE_ATTACK_LIGHT));
        assert_eq!(sm.current_state(), STATE_ATTACK_LIGHT);
    }

    #[test]
    fn test_walk_to_idle_on_neutral() {
        let mut sm = StateMachine::new();
        sm.try_transition(&dir_right());
        let result = sm.try_transition(&neutral());
        assert_eq!(result, Some(STATE_STAND));
        assert_eq!(sm.current_state(), STATE_STAND);
    }

    #[test]
    fn test_walk_to_jump() {
        let mut sm = StateMachine::new();
        sm.try_transition(&dir_right());
        let result = sm.try_transition(&dir_up());
        assert_eq!(result, Some(STATE_JUMP_UP));
    }

    #[test]
    fn test_jump_blocks_walk() {
        let mut sm = StateMachine::new();
        sm.try_transition(&dir_up()); // enter jump
        let result = sm.try_transition(&dir_right());
        assert_eq!(result, None); // can't walk in air
        assert_eq!(sm.current_state(), STATE_JUMP_UP);
    }

    #[test]
    fn test_jump_allows_air_attack() {
        let mut sm = StateMachine::new();
        sm.try_transition(&dir_up());
        let result = sm.try_transition(&press_a());
        assert_eq!(result, Some(STATE_ATTACK_LIGHT));
    }

    #[test]
    fn test_hitstun_blocks_all_input() {
        let mut sm = StateMachine::new();
        sm.force_enter(STATE_HIT_STAND_LIGHT, 20);
        assert_eq!(sm.current_state(), STATE_HIT_STAND_LIGHT);
        // All inputs should be blocked
        assert_eq!(sm.try_transition(&press_a()), None);
        assert_eq!(sm.try_transition(&dir_right()), None);
        assert_eq!(sm.try_transition(&dir_up()), None);
    }

    #[test]
    fn test_blockstun_blocks_all_input() {
        let mut sm = StateMachine::new();
        sm.force_enter(STATE_GUARD_STAND, 10);
        assert_eq!(sm.try_transition(&press_a()), None);
        assert_eq!(sm.try_transition(&dir_right()), None);
    }

    #[test]
    fn test_hitstun_recovery_to_idle() {
        let mut sm = StateMachine::new();
        sm.force_enter(STATE_HIT_STAND_LIGHT, 5);
        // Advance 5 frames
        for _ in 0..4 {
            let r = sm.update();
            assert_eq!(r, None);
        }
        // 5th frame triggers recovery
        let r = sm.update();
        assert_eq!(r, Some(STATE_STAND));
        assert_eq!(sm.current_state(), STATE_STAND);
    }

    #[test]
    fn test_attack_completion_to_idle() {
        let mut sm = StateMachine::new();
        let data = AttackData {
            total_frames: 10,
            cancel_windows: vec![],
        };
        sm.enter_attack(STATE_ATTACK_LIGHT, data);
        assert_eq!(sm.current_state(), STATE_ATTACK_LIGHT);
        // Advance 9 frames (no transition yet)
        for _ in 0..9 {
            assert_eq!(sm.update(), None);
        }
        // 10th frame completes the attack
        assert_eq!(sm.update(), Some(STATE_STAND));
        assert_eq!(sm.current_state(), STATE_STAND);
    }

    #[test]
    fn test_cancel_window() {
        let mut sm = StateMachine::new();
        let data = AttackData {
            total_frames: 20,
            cancel_windows: vec![CancelWindow {
                start_frame: 5,
                end_frame: 10,
                allowed: CancelTarget::Normal,
            }],
        };
        sm.enter_attack(STATE_ATTACK_MEDIUM, data);
        // Advance to frame 4 -- not in window yet
        for _ in 0..4 {
            sm.update();
        }
        assert_eq!(sm.try_transition(&press_a()), None);
        // Advance to frame 5 -- now in window
        sm.update();
        let result = sm.try_transition(&press_a());
        assert_eq!(result, Some(STATE_ATTACK_LIGHT));
    }

    #[test]
    fn test_attack_blocks_movement() {
        let mut sm = StateMachine::new();
        let data = AttackData {
            total_frames: 20,
            cancel_windows: vec![],
        };
        sm.enter_attack(STATE_ATTACK_LIGHT, data);
        assert_eq!(sm.try_transition(&dir_right()), None);
        assert_eq!(sm.try_transition(&dir_up()), None);
    }

    #[test]
    fn test_state_frame_counting() {
        let mut sm = StateMachine::new();
        assert_eq!(sm.state_frame(), 0);
        sm.update();
        assert_eq!(sm.state_frame(), 1);
        sm.update();
        assert_eq!(sm.state_frame(), 2);
        // Transition resets frame
        sm.try_transition(&dir_right());
        assert_eq!(sm.state_frame(), 0);
    }

    #[test]
    fn test_land_from_jump() {
        let mut sm = StateMachine::new();
        sm.try_transition(&dir_up());
        assert_eq!(sm.current_state(), STATE_JUMP_UP);
        sm.land();
        assert_eq!(sm.current_state(), STATE_STAND);
    }

    #[test]
    fn test_crouch_to_idle_on_release() {
        let mut sm = StateMachine::new();
        sm.try_transition(&dir_down());
        assert_eq!(sm.current_state(), STATE_CROUCH);
        let result = sm.try_transition(&neutral());
        assert_eq!(result, Some(STATE_STAND));
    }
}
