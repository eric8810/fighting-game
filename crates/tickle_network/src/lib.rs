pub mod snapshot;

use bytemuck::{CheckedBitPattern, NoUninit, Zeroable};
use ggrs::{Config, Frame, GameStateCell, GgrsRequest, InputStatus};
use snapshot::GameSnapshot;
use std::net::SocketAddr;
use tickle_core::InputState;

// ---------------------------------------------------------------------------
// GGRS Config
// ---------------------------------------------------------------------------

/// Network input: a compact representation of one player's input for one frame.
/// Must satisfy bytemuck traits for GGRS wire format.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, NoUninit, CheckedBitPattern, Zeroable)]
pub struct NetworkInput {
    /// Packed buttons bitmask.
    pub buttons: u8,
    /// Direction encoded as numpad notation (1-9, 5 = neutral).
    pub direction: u8,
    /// Padding for alignment.
    pub _pad: [u8; 2],
}

impl NetworkInput {
    pub fn from_input_state(input: &InputState) -> Self {
        Self {
            buttons: input.buttons,
            direction: direction_to_numpad(&input.direction),
            _pad: [0; 2],
        }
    }

    pub fn to_input_state(self) -> InputState {
        InputState::new(self.buttons, numpad_to_direction(self.direction))
    }
}

/// Encode Direction as numpad notation for compact wire format.
fn direction_to_numpad(dir: &tickle_core::Direction) -> u8 {
    use tickle_core::Direction;
    match dir {
        Direction::DownLeft => 1,
        Direction::Down => 2,
        Direction::DownRight => 3,
        Direction::Left => 4,
        Direction::Neutral => 5,
        Direction::Right => 6,
        Direction::UpLeft => 7,
        Direction::Up => 8,
        Direction::UpRight => 9,
    }
}

fn numpad_to_direction(numpad: u8) -> tickle_core::Direction {
    use tickle_core::Direction;
    match numpad {
        1 => Direction::DownLeft,
        2 => Direction::Down,
        3 => Direction::DownRight,
        4 => Direction::Left,
        6 => Direction::Right,
        7 => Direction::UpLeft,
        8 => Direction::Up,
        9 => Direction::UpRight,
        _ => Direction::Neutral,
    }
}

/// GGRS session configuration for Tickle Fighting Engine.
pub struct TickleGGRSConfig;

impl Config for TickleGGRSConfig {
    type Input = NetworkInput;
    type State = GameSnapshot;
    type Address = SocketAddr;
}

// ---------------------------------------------------------------------------
// Deterministic RNG (LCG)
// ---------------------------------------------------------------------------

/// Linear congruential generator for deterministic randomness.
/// Both peers must use the same seed to stay in sync.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn state(&self) -> u64 {
        self.state
    }

    pub fn set_state(&mut self, state: u64) {
        self.state = state;
    }

    /// Generate next pseudo-random u32.
    pub fn next(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    /// Generate a random i32 in [min, max).
    pub fn range(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        let range = (max - min) as u32;
        min + (self.next() % range) as i32
    }
}

// ---------------------------------------------------------------------------
// GGRS request handler
// ---------------------------------------------------------------------------

/// Handles a batch of GGRS requests by delegating to caller-provided closures.
///
/// - `save_fn`: called with `(cell, frame)` to persist the current game state.
/// - `load_fn`: called with `(cell, frame)` to restore a previous game state.
/// - `advance_fn`: called with a slice of `(NetworkInput, InputStatus)` to step
///   the simulation forward one frame.
pub fn handle_ggrs_requests<S, L, A>(
    requests: Vec<GgrsRequest<TickleGGRSConfig>>,
    mut save_fn: S,
    mut load_fn: L,
    mut advance_fn: A,
) where
    S: FnMut(GameStateCell<GameSnapshot>, Frame),
    L: FnMut(GameStateCell<GameSnapshot>, Frame),
    A: FnMut(&[(NetworkInput, InputStatus)]),
{
    for request in requests {
        match request {
            GgrsRequest::SaveGameState { cell, frame } => {
                save_fn(cell, frame);
            }
            GgrsRequest::LoadGameState { cell, frame } => {
                load_fn(cell, frame);
            }
            GgrsRequest::AdvanceFrame { inputs } => {
                advance_fn(&inputs);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tickle_core::Direction;

    #[test]
    fn network_input_roundtrip() {
        let input = InputState::new(0b0101, Direction::DownRight);
        let net = NetworkInput::from_input_state(&input);
        let back = net.to_input_state();
        assert_eq!(back.buttons, input.buttons);
        assert_eq!(back.direction, input.direction);
    }

    #[test]
    fn network_input_all_directions() {
        let dirs = [
            Direction::Neutral,
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
            Direction::UpLeft,
            Direction::UpRight,
            Direction::DownLeft,
            Direction::DownRight,
        ];
        for dir in &dirs {
            let input = InputState::new(0, *dir);
            let net = NetworkInput::from_input_state(&input);
            let back = net.to_input_state();
            assert_eq!(back.direction, *dir, "roundtrip failed for {:?}", dir);
        }
    }

    #[test]
    fn network_input_is_zeroable() {
        let zero = NetworkInput::zeroed();
        assert_eq!(zero.buttons, 0);
        assert_eq!(zero.direction, 0);
        // direction 0 maps to Neutral
        assert_eq!(zero.to_input_state().direction, Direction::Neutral);
    }

    #[test]
    fn deterministic_rng_reproducible() {
        let mut rng1 = DeterministicRng::new(42);
        let mut rng2 = DeterministicRng::new(42);
        for _ in 0..1000 {
            assert_eq!(rng1.next(), rng2.next());
        }
    }

    #[test]
    fn deterministic_rng_different_seeds() {
        let mut rng1 = DeterministicRng::new(1);
        let mut rng2 = DeterministicRng::new(2);
        // Very unlikely to produce the same sequence
        let same = (0..100).all(|_| rng1.next() == rng2.next());
        assert!(!same);
    }

    #[test]
    fn deterministic_rng_range() {
        let mut rng = DeterministicRng::new(99);
        for _ in 0..1000 {
            let val = rng.range(-10, 10);
            assert!((-10..10).contains(&val));
        }
    }

    #[test]
    fn deterministic_rng_state_save_restore() {
        let mut rng = DeterministicRng::new(42);
        // Advance a few steps
        for _ in 0..50 {
            rng.next();
        }
        let saved = rng.state();

        // Continue generating
        let mut values_after_save = Vec::new();
        for _ in 0..100 {
            values_after_save.push(rng.next());
        }

        // Restore and verify same sequence
        let mut rng2 = DeterministicRng::new(0);
        rng2.set_state(saved);
        for (i, expected) in values_after_save.iter().enumerate() {
            assert_eq!(rng2.next(), *expected, "mismatch at step {}", i);
        }
    }
}