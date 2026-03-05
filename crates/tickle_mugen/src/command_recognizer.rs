use crate::cmd::{Command, CommandButton, CommandDirection, InputModifier, InputStep};
use tickle_core::input::{Direction, InputBuffer, InputState, BUTTON_A, BUTTON_B, BUTTON_C, BUTTON_D};

/// Maps MUGEN CommandDirection to tickle_core Direction, accounting for facing.
fn resolve_direction(cmd_dir: &CommandDirection, facing_right: bool) -> Direction {
    match cmd_dir {
        CommandDirection::Forward => {
            if facing_right {
                Direction::Right
            } else {
                Direction::Left
            }
        }
        CommandDirection::Back => {
            if facing_right {
                Direction::Left
            } else {
                Direction::Right
            }
        }
        CommandDirection::Up => Direction::Up,
        CommandDirection::Down => Direction::Down,
        CommandDirection::DownForward => {
            if facing_right {
                Direction::DownRight
            } else {
                Direction::DownLeft
            }
        }
        CommandDirection::DownBack => {
            if facing_right {
                Direction::DownLeft
            } else {
                Direction::DownRight
            }
        }
        CommandDirection::UpForward => {
            if facing_right {
                Direction::UpRight
            } else {
                Direction::UpLeft
            }
        }
        CommandDirection::UpBack => {
            if facing_right {
                Direction::UpLeft
            } else {
                Direction::UpRight
            }
        }
    }
}

/// Maps a MUGEN CommandButton to a tickle_core button bitmask.
/// MUGEN has 6 attack buttons (x,y,z,a,b,c) plus start (s).
/// We map them to the 4-button layout: x/y -> A/B, a/b -> C/D.
fn button_mask(btn: &CommandButton) -> u8 {
    match btn {
        CommandButton::X => BUTTON_A,
        CommandButton::Y => BUTTON_B,
        CommandButton::Z => BUTTON_B, // Z maps to same as Y in 4-button
        CommandButton::A => BUTTON_C,
        CommandButton::B => BUTTON_D,
        CommandButton::C => BUTTON_D, // C maps to same as B in 4-button
        CommandButton::S => 0,        // Start not used in commands
    }
}

/// Check if a single InputStep matches the input at a given frame in the buffer.
/// Returns true if the step's direction and buttons match the frame's input.
fn step_matches_frame(
    step: &InputStep,
    current: InputState,
    prev: InputState,
    facing_right: bool,
) -> bool {
    // Check direction requirement
    if let Some(ref cmd_dir) = step.direction {
        let expected_dir = resolve_direction(cmd_dir, facing_right);
        match step.modifier {
            InputModifier::None | InputModifier::Charge => {
                if current.direction != expected_dir {
                    return false;
                }
            }
            InputModifier::Release => {
                // Release: the direction must have been active previously and NOT active now.
                // In MUGEN, ~D means "release down" - was down, now not down.
                let was_active = direction_matches(prev.direction, expected_dir);
                let is_active = direction_matches(current.direction, expected_dir);
                if !was_active || is_active {
                    return false;
                }
            }
            InputModifier::Hold => {
                // Hold: the direction must be currently held
                if !direction_matches(current.direction, expected_dir) {
                    return false;
                }
            }
        }
    }

    // Check button requirements
    if !step.buttons.is_empty() {
        let required_mask: u8 = step.buttons.iter().fold(0u8, |acc, b| acc | button_mask(b));
        if required_mask == 0 {
            return true; // Start button only, always passes
        }
        match step.modifier {
            InputModifier::None | InputModifier::Charge => {
                // Button must be pressed this frame (newly pressed or held)
                if current.buttons & required_mask != required_mask {
                    return false;
                }
            }
            InputModifier::Release => {
                // Button was pressed before, not pressed now
                let was_pressed = prev.buttons & required_mask == required_mask;
                let is_pressed = current.buttons & required_mask == required_mask;
                if !was_pressed || is_pressed {
                    return false;
                }
            }
            InputModifier::Hold => {
                // Button must be currently held
                if current.buttons & required_mask != required_mask {
                    return false;
                }
            }
        }
    }

    true
}

/// Check if the actual direction includes the expected direction component.
/// For example, DownRight includes both Down and Right.
fn direction_matches(actual: Direction, expected: Direction) -> bool {
    if actual == expected {
        return true;
    }
    // Allow diagonal to match cardinal components
    match expected {
        Direction::Down => matches!(actual, Direction::Down | Direction::DownLeft | Direction::DownRight),
        Direction::Up => matches!(actual, Direction::Up | Direction::UpLeft | Direction::UpRight),
        Direction::Left => matches!(actual, Direction::Left | Direction::UpLeft | Direction::DownLeft),
        Direction::Right => matches!(actual, Direction::Right | Direction::UpRight | Direction::DownRight),
        _ => actual == expected,
    }
}

/// Recognizes MUGEN commands from the input buffer.
pub struct MugenCommandRecognizer;

impl MugenCommandRecognizer {
    /// Check all commands against the input buffer and return the names of
    /// commands that are currently active.
    pub fn recognize(
        commands: &[Command],
        buffer: &InputBuffer,
        facing_right: bool,
    ) -> Vec<String> {
        let mut active = Vec::new();
        for cmd in commands {
            if Self::check_command(cmd, buffer, facing_right) {
                active.push(cmd.name.clone());
            }
        }
        active
    }

    /// Check if a single command's input sequence matches the buffer history.
    fn check_command(cmd: &Command, buffer: &InputBuffer, facing_right: bool) -> bool {
        let steps = &cmd.inputs;
        if steps.is_empty() {
            return false;
        }

        let window = cmd.time.min(16) as usize;

        // The last step must match the most recent input.
        // We scan backwards through the buffer trying to match the sequence.
        // step_idx starts at the last step (most recent input).
        let mut step_idx = steps.len() - 1;

        // The last step must match frame 0 (current frame).
        let current = buffer.get(0);
        let prev = buffer.get(1);
        if !step_matches_frame(&steps[step_idx], current, prev, facing_right) {
            return false;
        }

        if step_idx == 0 {
            // Single-step command, already matched
            return true;
        }

        step_idx -= 1;

        // Now scan backwards through the buffer to match remaining steps
        for frames_ago in 1..window {
            let frame_input = buffer.get(frames_ago);
            let frame_prev = buffer.get(frames_ago + 1);

            if step_matches_frame(&steps[step_idx], frame_input, frame_prev, facing_right) {
                if step_idx == 0 {
                    return true; // All steps matched
                }
                step_idx -= 1;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmd::CmdParser;
    use tickle_core::input::{Direction, InputBuffer, InputState, BUTTON_A, BUTTON_B};

    /// Helper: push a direction-only input
    fn push_dir(buf: &mut InputBuffer, dir: Direction) {
        buf.push(InputState::new(0, dir));
    }

    /// Helper: push a button press with direction
    fn push_btn_dir(buf: &mut InputBuffer, btn: u8, dir: Direction) {
        buf.push(InputState::new(btn, dir));
    }

    /// Helper: push a button press with neutral direction
    fn push_btn(buf: &mut InputBuffer, btn: u8) {
        buf.push(InputState::new(btn, Direction::Neutral));
    }

    #[test]
    fn test_qcf_a_recognized() {
        // QCF_a: ~D, DF, F, x
        let content = r#"
[Command]
name = "QCF_a"
command = ~D, DF, F, x
time = 15
"#;
        let cmd = CmdParser::parse_content(content).unwrap();

        let mut buf = InputBuffer::new();
        // Need to simulate release of D: was down, then DF, F, button
        push_dir(&mut buf, Direction::Down); // held down
        push_dir(&mut buf, Direction::Down); // still down
        push_dir(&mut buf, Direction::Neutral); // release down (step 1: ~D matches here)
        push_dir(&mut buf, Direction::DownRight); // DF (step 2)
        push_dir(&mut buf, Direction::Right); // F (step 3)
        push_btn_dir(&mut buf, BUTTON_A, Direction::Right); // x (step 4) with last direction

        let active = MugenCommandRecognizer::recognize(&cmd.commands, &buf, true);
        assert!(
            active.contains(&"QCF_a".to_string()),
            "QCF_a should be recognized, got: {:?}",
            active
        );
    }

    #[test]
    fn test_qcf_not_recognized_wrong_sequence() {
        let content = r#"
[Command]
name = "QCF_a"
command = ~D, DF, F, x
time = 15
"#;
        let cmd = CmdParser::parse_content(content).unwrap();

        let mut buf = InputBuffer::new();
        // Wrong sequence: right, down, left, button
        push_dir(&mut buf, Direction::Right);
        push_dir(&mut buf, Direction::Down);
        push_dir(&mut buf, Direction::Left);
        push_btn(&mut buf, BUTTON_A);

        let active = MugenCommandRecognizer::recognize(&cmd.commands, &buf, true);
        assert!(!active.contains(&"QCF_a".to_string()));
    }

    #[test]
    fn test_single_button_command() {
        let content = r#"
[Command]
name = "a"
command = x
time = 1
"#;
        let cmd = CmdParser::parse_content(content).unwrap();

        let mut buf = InputBuffer::new();
        push_btn(&mut buf, BUTTON_A); // x maps to BUTTON_A

        let active = MugenCommandRecognizer::recognize(&cmd.commands, &buf, true);
        assert!(active.contains(&"a".to_string()));
    }

    #[test]
    fn test_single_button_not_pressed() {
        let content = r#"
[Command]
name = "a"
command = x
time = 1
"#;
        let cmd = CmdParser::parse_content(content).unwrap();

        let mut buf = InputBuffer::new();
        push_dir(&mut buf, Direction::Neutral); // no button

        let active = MugenCommandRecognizer::recognize(&cmd.commands, &buf, true);
        assert!(!active.contains(&"a".to_string()));
    }

    #[test]
    fn test_hold_forward_command() {
        // /F = hold forward
        let content = r#"
[Command]
name = "holdfwd"
command = /F
time = 1
"#;
        let cmd = CmdParser::parse_content(content).unwrap();

        let mut buf = InputBuffer::new();
        push_dir(&mut buf, Direction::Right); // facing right, F = Right

        let active = MugenCommandRecognizer::recognize(&cmd.commands, &buf, true);
        assert!(active.contains(&"holdfwd".to_string()));
    }

    #[test]
    fn test_hold_forward_not_held() {
        let content = r#"
[Command]
name = "holdfwd"
command = /F
time = 1
"#;
        let cmd = CmdParser::parse_content(content).unwrap();

        let mut buf = InputBuffer::new();
        push_dir(&mut buf, Direction::Left); // wrong direction

        let active = MugenCommandRecognizer::recognize(&cmd.commands, &buf, true);
        assert!(!active.contains(&"holdfwd".to_string()));
    }

    #[test]
    fn test_hold_back_facing_left() {
        // When facing left, B = Right
        let content = r#"
[Command]
name = "holdback"
command = /B
time = 1
"#;
        let cmd = CmdParser::parse_content(content).unwrap();

        let mut buf = InputBuffer::new();
        push_dir(&mut buf, Direction::Right); // facing left, B = Right

        let active = MugenCommandRecognizer::recognize(&cmd.commands, &buf, false);
        assert!(active.contains(&"holdback".to_string()));
    }

    #[test]
    fn test_simultaneous_buttons() {
        let content = r#"
[Command]
name = "recovery"
command = x+y
time = 1
"#;
        let cmd = CmdParser::parse_content(content).unwrap();

        let mut buf = InputBuffer::new();
        // x=BUTTON_A, y=BUTTON_B, both pressed simultaneously
        push_btn(&mut buf, BUTTON_A | BUTTON_B);

        let active = MugenCommandRecognizer::recognize(&cmd.commands, &buf, true);
        assert!(active.contains(&"recovery".to_string()));
    }

    #[test]
    fn test_simultaneous_buttons_only_one_pressed() {
        let content = r#"
[Command]
name = "recovery"
command = x+y
time = 1
"#;
        let cmd = CmdParser::parse_content(content).unwrap();

        let mut buf = InputBuffer::new();
        push_btn(&mut buf, BUTTON_A); // only x, not x+y

        let active = MugenCommandRecognizer::recognize(&cmd.commands, &buf, true);
        assert!(!active.contains(&"recovery".to_string()));
    }

    #[test]
    fn test_dash_forward_command() {
        let content = r#"
[Command]
name = "FF"
command = F, F
time = 10
"#;
        let cmd = CmdParser::parse_content(content).unwrap();

        let mut buf = InputBuffer::new();
        push_dir(&mut buf, Direction::Right); // first F
        push_dir(&mut buf, Direction::Neutral); // gap
        push_dir(&mut buf, Direction::Right); // second F

        let active = MugenCommandRecognizer::recognize(&cmd.commands, &buf, true);
        assert!(active.contains(&"FF".to_string()));
    }

    #[test]
    fn test_dp_command() {
        // DP: F, D, DF, x
        let content = r#"
[Command]
name = "DP_a"
command = F, D, DF, x
time = 15
"#;
        let cmd = CmdParser::parse_content(content).unwrap();

        let mut buf = InputBuffer::new();
        push_dir(&mut buf, Direction::Right); // F
        push_dir(&mut buf, Direction::Down); // D
        push_dir(&mut buf, Direction::DownRight); // DF
        push_btn_dir(&mut buf, BUTTON_A, Direction::DownRight); // x at DF

        let active = MugenCommandRecognizer::recognize(&cmd.commands, &buf, true);
        assert!(
            active.contains(&"DP_a".to_string()),
            "DP_a should be recognized, got: {:?}",
            active
        );
    }

    #[test]
    fn test_qcb_command() {
        // QCB: ~D, DB, B, x
        let content = r#"
[Command]
name = "QCB_a"
command = ~D, DB, B, x
time = 15
"#;
        let cmd = CmdParser::parse_content(content).unwrap();

        let mut buf = InputBuffer::new();
        push_dir(&mut buf, Direction::Down); // held down
        push_dir(&mut buf, Direction::Neutral); // release down (~D)
        push_dir(&mut buf, Direction::DownLeft); // DB
        push_dir(&mut buf, Direction::Left); // B
        push_btn_dir(&mut buf, BUTTON_A, Direction::Left); // x

        let active = MugenCommandRecognizer::recognize(&cmd.commands, &buf, true);
        assert!(
            active.contains(&"QCB_a".to_string()),
            "QCB_a should be recognized, got: {:?}",
            active
        );
    }

    #[test]
    fn test_direction_matches_includes_diagonal() {
        // Down should match DownLeft and DownRight
        assert!(direction_matches(Direction::DownRight, Direction::Down));
        assert!(direction_matches(Direction::DownLeft, Direction::Down));
        assert!(direction_matches(Direction::Down, Direction::Down));
        assert!(!direction_matches(Direction::Right, Direction::Down));
        assert!(!direction_matches(Direction::Up, Direction::Down));
    }

    #[test]
    fn test_resolve_direction_facing_right() {
        assert_eq!(
            resolve_direction(&CommandDirection::Forward, true),
            Direction::Right
        );
        assert_eq!(
            resolve_direction(&CommandDirection::Back, true),
            Direction::Left
        );
        assert_eq!(
            resolve_direction(&CommandDirection::DownForward, true),
            Direction::DownRight
        );
    }

    #[test]
    fn test_resolve_direction_facing_left() {
        assert_eq!(
            resolve_direction(&CommandDirection::Forward, false),
            Direction::Left
        );
        assert_eq!(
            resolve_direction(&CommandDirection::Back, false),
            Direction::Right
        );
        assert_eq!(
            resolve_direction(&CommandDirection::DownForward, false),
            Direction::DownLeft
        );
    }

    #[test]
    fn test_multiple_commands_recognized() {
        let content = r#"
[Command]
name = "a"
command = x
time = 1

[Command]
name = "holdfwd"
command = /F
time = 1
"#;
        let cmd = CmdParser::parse_content(content).unwrap();

        let mut buf = InputBuffer::new();
        // Press button A while holding right
        push_btn_dir(&mut buf, BUTTON_A, Direction::Right);

        let active = MugenCommandRecognizer::recognize(&cmd.commands, &buf, true);
        assert!(active.contains(&"a".to_string()));
        assert!(active.contains(&"holdfwd".to_string()));
    }

    #[test]
    fn test_empty_buffer_matches_nothing() {
        let content = r#"
[Command]
name = "a"
command = x
time = 1
"#;
        let cmd = CmdParser::parse_content(content).unwrap();
        let buf = InputBuffer::new();

        let active = MugenCommandRecognizer::recognize(&cmd.commands, &buf, true);
        assert!(active.is_empty());
    }

    #[test]
    fn test_command_outside_time_window() {
        // Command with time=3 should only look back 3 frames
        let content = r#"
[Command]
name = "FF"
command = F, F
time = 3
"#;
        let cmd = CmdParser::parse_content(content).unwrap();

        let mut buf = InputBuffer::new();
        push_dir(&mut buf, Direction::Right); // F at frame -5
        push_dir(&mut buf, Direction::Neutral);
        push_dir(&mut buf, Direction::Neutral);
        push_dir(&mut buf, Direction::Neutral);
        push_dir(&mut buf, Direction::Neutral);
        push_dir(&mut buf, Direction::Right); // F at frame 0

        // The first F is 5 frames ago, but time window is only 3
        let active = MugenCommandRecognizer::recognize(&cmd.commands, &buf, true);
        assert!(!active.contains(&"FF".to_string()));
    }
}
