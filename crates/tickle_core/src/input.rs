use serde::{Deserialize, Serialize};

/// Direction input
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Neutral,
    Up,
    Down,
    Left,
    Right,
    UpLeft,
    UpRight,
    DownLeft,
    DownRight,
}

impl Direction {
    /// Convert from raw input (x, y) where -1, 0, 1
    pub fn from_xy(x: i8, y: i8) -> Self {
        match (x, y) {
            (0, 0) => Self::Neutral,
            (0, 1) => Self::Up,
            (0, -1) => Self::Down,
            (-1, 0) => Self::Left,
            (1, 0) => Self::Right,
            (-1, 1) => Self::UpLeft,
            (1, 1) => Self::UpRight,
            (-1, -1) => Self::DownLeft,
            (1, -1) => Self::DownRight,
            _ => Self::Neutral,
        }
    }

    pub fn is_down(&self) -> bool {
        matches!(self, Self::Down | Self::DownLeft | Self::DownRight)
    }

    pub fn is_up(&self) -> bool {
        matches!(self, Self::Up | Self::UpLeft | Self::UpRight)
    }

    pub fn is_left(&self) -> bool {
        matches!(self, Self::Left | Self::UpLeft | Self::DownLeft)
    }

    pub fn is_right(&self) -> bool {
        matches!(self, Self::Right | Self::UpRight | Self::DownRight)
    }
}

/// Button bits
pub const BUTTON_A: u8 = 1 << 0;
pub const BUTTON_B: u8 = 1 << 1;
pub const BUTTON_C: u8 = 1 << 2;
pub const BUTTON_D: u8 = 1 << 3;

/// Input state for a single frame
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputState {
    pub buttons: u8,
    pub direction: Direction,
}

impl InputState {
    pub const EMPTY: Self = Self {
        buttons: 0,
        direction: Direction::Neutral,
    };

    pub fn new(buttons: u8, direction: Direction) -> Self {
        Self { buttons, direction }
    }

    pub fn is_pressed(&self, button: u8) -> bool {
        self.buttons & button != 0
    }

    pub fn any_button(&self) -> bool {
        self.buttons != 0
    }
}

/// Input buffer (16 frames history)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InputBuffer {
    history: [InputState; 16],
    head: usize,
}

impl InputBuffer {
    pub fn new() -> Self {
        Self {
            history: [InputState::EMPTY; 16],
            head: 0,
        }
    }

    /// Push new input to buffer
    pub fn push(&mut self, input: InputState) {
        self.head = (self.head + 1) % 16;
        self.history[self.head] = input;
    }

    /// Get input from N frames ago (0 = current frame)
    pub fn get(&self, frames_ago: usize) -> InputState {
        if frames_ago >= 16 {
            return InputState::EMPTY;
        }
        let index = (self.head + 16 - frames_ago) % 16;
        self.history[index]
    }

    /// Get full history (oldest to newest)
    pub fn get_history(&self) -> [InputState; 16] {
        let mut result = [InputState::EMPTY; 16];
        for i in 0..16 {
            result[i] = self.get(15 - i);
        }
        result
    }

    /// Check if button was just pressed (not pressed last frame, pressed this frame)
    pub fn just_pressed(&self, button: u8) -> bool {
        let current = self.get(0);
        let previous = self.get(1);
        current.is_pressed(button) && !previous.is_pressed(button)
    }
}

impl Default for InputBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Special move command types
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    QCF,  // Quarter Circle Forward: ↓↘→
    QCB,  // Quarter Circle Back: ↓↙←
    DP,   // Dragon Punch: →↓↘
    HCF,  // Half Circle Forward: ←↙↓↘→
    HCB,  // Half Circle Back: →↘↓↙←
    DashForward,  // →→
    DashBackward, // ←←
}

/// Command recognizer
pub struct CommandRecognizer {
    tolerance: usize, // Frame tolerance for command input
}

impl CommandRecognizer {
    pub fn new() -> Self {
        Self { tolerance: 16 }
    }

    /// Recognize command from input buffer
    pub fn recognize(&self, buffer: &InputBuffer, facing_right: bool) -> Option<Command> {
        // Check commands in priority order (most specific first)
        if self.check_hcf(buffer, facing_right) {
            return Some(Command::HCF);
        }
        if self.check_hcb(buffer, facing_right) {
            return Some(Command::HCB);
        }
        if self.check_qcf(buffer, facing_right) {
            return Some(Command::QCF);
        }
        if self.check_qcb(buffer, facing_right) {
            return Some(Command::QCB);
        }
        if self.check_dp(buffer, facing_right) {
            return Some(Command::DP);
        }
        if self.check_dash_forward(buffer, facing_right) {
            return Some(Command::DashForward);
        }
        if self.check_dash_backward(buffer, facing_right) {
            return Some(Command::DashBackward);
        }
        None
    }

    /// Check QCF: ↓↘→ (or ↓↙← if facing left)
    fn check_qcf(&self, buffer: &InputBuffer, facing_right: bool) -> bool {
        let (down, diag, forward) = if facing_right {
            (Direction::Down, Direction::DownRight, Direction::Right)
        } else {
            (Direction::Down, Direction::DownLeft, Direction::Left)
        };

        self.check_sequence(buffer, &[down, diag, forward])
    }

    /// Check QCB: ↓↙← (or ↓↘→ if facing left)
    fn check_qcb(&self, buffer: &InputBuffer, facing_right: bool) -> bool {
        let (down, diag, back) = if facing_right {
            (Direction::Down, Direction::DownLeft, Direction::Left)
        } else {
            (Direction::Down, Direction::DownRight, Direction::Right)
        };

        self.check_sequence(buffer, &[down, diag, back])
    }

    /// Check DP: →↓↘ (or ←↓↙ if facing left)
    fn check_dp(&self, buffer: &InputBuffer, facing_right: bool) -> bool {
        let (forward, down, diag) = if facing_right {
            (Direction::Right, Direction::Down, Direction::DownRight)
        } else {
            (Direction::Left, Direction::Down, Direction::DownLeft)
        };

        self.check_sequence(buffer, &[forward, down, diag])
    }

    /// Check HCF: ←↙↓↘→ (or →↘↓↙← if facing left)
    fn check_hcf(&self, buffer: &InputBuffer, facing_right: bool) -> bool {
        let seq = if facing_right {
            vec![
                Direction::Left,
                Direction::DownLeft,
                Direction::Down,
                Direction::DownRight,
                Direction::Right,
            ]
        } else {
            vec![
                Direction::Right,
                Direction::DownRight,
                Direction::Down,
                Direction::DownLeft,
                Direction::Left,
            ]
        };

        self.check_sequence(buffer, &seq)
    }

    /// Check HCB: →↘↓↙← (or ←↙↓↘→ if facing left)
    fn check_hcb(&self, buffer: &InputBuffer, facing_right: bool) -> bool {
        let seq = if facing_right {
            vec![
                Direction::Right,
                Direction::DownRight,
                Direction::Down,
                Direction::DownLeft,
                Direction::Left,
            ]
        } else {
            vec![
                Direction::Left,
                Direction::DownLeft,
                Direction::Down,
                Direction::DownRight,
                Direction::Right,
            ]
        };

        self.check_sequence(buffer, &seq)
    }

    /// Check dash forward: →→
    fn check_dash_forward(&self, buffer: &InputBuffer, facing_right: bool) -> bool {
        let forward = if facing_right {
            Direction::Right
        } else {
            Direction::Left
        };

        // Check for two forward inputs with neutral in between
        for i in 0..8 {
            if buffer.get(i).direction == forward {
                for j in (i + 1)..8 {
                    if buffer.get(j).direction == Direction::Neutral {
                        for k in (j + 1)..8 {
                            if buffer.get(k).direction == forward {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Check dash backward: ←←
    fn check_dash_backward(&self, buffer: &InputBuffer, facing_right: bool) -> bool {
        let back = if facing_right {
            Direction::Left
        } else {
            Direction::Right
        };

        // Check for two back inputs with neutral in between
        for i in 0..8 {
            if buffer.get(i).direction == back {
                for j in (i + 1)..8 {
                    if buffer.get(j).direction == Direction::Neutral {
                        for k in (j + 1)..8 {
                            if buffer.get(k).direction == back {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    /// Check if a sequence of directions exists in buffer (with tolerance)
    fn check_sequence(&self, buffer: &InputBuffer, sequence: &[Direction]) -> bool {
        let mut seq_index = 0;
        let mut frame_count = 0;

        // Check from oldest to newest (reverse order)
        for i in (0..self.tolerance).rev() {
            let input = buffer.get(i);

            if input.direction == sequence[seq_index] {
                seq_index += 1;
                if seq_index >= sequence.len() {
                    return true;
                }
                frame_count = 0;
            } else if input.direction != Direction::Neutral {
                // Allow some frames of other inputs (lenient matching)
                frame_count += 1;
                if frame_count > 3 {
                    // Too many wrong inputs, reset
                    seq_index = 0;
                    frame_count = 0;
                }
            }
        }

        false
    }
}

impl Default for CommandRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_from_xy() {
        assert_eq!(Direction::from_xy(0, 0), Direction::Neutral);
        assert_eq!(Direction::from_xy(0, 1), Direction::Up);
        assert_eq!(Direction::from_xy(0, -1), Direction::Down);
        assert_eq!(Direction::from_xy(-1, 0), Direction::Left);
        assert_eq!(Direction::from_xy(1, 0), Direction::Right);
        assert_eq!(Direction::from_xy(1, -1), Direction::DownRight);
    }

    #[test]
    fn test_input_buffer() {
        let mut buffer = InputBuffer::new();

        buffer.push(InputState::new(BUTTON_A, Direction::Down));
        buffer.push(InputState::new(BUTTON_A, Direction::DownRight));
        buffer.push(InputState::new(BUTTON_A, Direction::Right));

        assert_eq!(buffer.get(0).direction, Direction::Right);
        assert_eq!(buffer.get(1).direction, Direction::DownRight);
        assert_eq!(buffer.get(2).direction, Direction::Down);
    }

    #[test]
    fn test_just_pressed() {
        let mut buffer = InputBuffer::new();

        buffer.push(InputState::new(0, Direction::Neutral));
        assert!(!buffer.just_pressed(BUTTON_A));

        buffer.push(InputState::new(BUTTON_A, Direction::Neutral));
        assert!(buffer.just_pressed(BUTTON_A));

        buffer.push(InputState::new(BUTTON_A, Direction::Neutral));
        assert!(!buffer.just_pressed(BUTTON_A));
    }

    #[test]
    fn test_qcf_recognition() {
        let mut buffer = InputBuffer::new();
        let recognizer = CommandRecognizer::new();

        // Simulate QCF input
        buffer.push(InputState::new(0, Direction::Down));
        buffer.push(InputState::new(0, Direction::DownRight));
        buffer.push(InputState::new(BUTTON_A, Direction::Right));

        assert_eq!(recognizer.recognize(&buffer, true), Some(Command::QCF));
    }

    #[test]
    fn test_dp_recognition() {
        let mut buffer = InputBuffer::new();
        let recognizer = CommandRecognizer::new();

        // Simulate DP input
        buffer.push(InputState::new(0, Direction::Right));
        buffer.push(InputState::new(0, Direction::Down));
        buffer.push(InputState::new(BUTTON_A, Direction::DownRight));

        assert_eq!(recognizer.recognize(&buffer, true), Some(Command::DP));
    }

    #[test]
    fn test_dash_recognition() {
        let mut buffer = InputBuffer::new();
        let recognizer = CommandRecognizer::new();

        // Simulate dash forward
        buffer.push(InputState::new(0, Direction::Right));
        buffer.push(InputState::new(0, Direction::Neutral));
        buffer.push(InputState::new(0, Direction::Right));

        assert_eq!(recognizer.recognize(&buffer, true), Some(Command::DashForward));
    }
}
