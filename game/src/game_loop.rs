use std::time::{Duration, Instant};

/// Fixed logic update rate: 60 FPS (standard for fighting games).
const LOGIC_HZ: f64 = 60.0;

/// Duration of one logic tick.
const LOGIC_DT: f64 = 1.0 / LOGIC_HZ;

/// Maximum frame time cap to prevent the death spiral.
/// If a single frame takes longer than this, we clamp it so the
/// accumulator doesn't queue an unbounded number of logic updates.
const MAX_FRAME_TIME: f64 = 0.25;

/// Supported render refresh rates.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshRate {
    Hz60,
    Hz120,
    Hz144,
    Hz240,
}

impl RefreshRate {
    /// Returns the target render interval as a `Duration`.
    pub fn interval(self) -> Duration {
        let hz: f64 = match self {
            Self::Hz60 => 60.0,
            Self::Hz120 => 120.0,
            Self::Hz144 => 144.0,
            Self::Hz240 => 240.0,
        };
        Duration::from_secs_f64(1.0 / hz)
    }

    /// Returns the refresh rate in Hz.
    pub fn as_hz(self) -> u32 {
        match self {
            Self::Hz60 => 60,
            Self::Hz120 => 120,
            Self::Hz144 => 144,
            Self::Hz240 => 240,
        }
    }
}

/// Tracks rendered frames and computes FPS once per second.
pub struct FrameCounter {
    frames: u32,
    last_second: Instant,
    current_fps: u32,
}

impl FrameCounter {
    pub fn new() -> Self {
        Self {
            frames: 0,
            last_second: Instant::now(),
            current_fps: 0,
        }
    }

    /// Call once per rendered frame. Returns `Some(fps)` when a full
    /// second has elapsed, otherwise `None`.
    pub fn tick(&mut self) -> Option<u32> {
        self.frames += 1;
        let now = Instant::now();
        if now.duration_since(self.last_second).as_secs() >= 1 {
            self.current_fps = self.frames;
            self.frames = 0;
            self.last_second = now;
            return Some(self.current_fps);
        }
        None
    }

    /// Returns the most recently measured FPS value.
    pub fn current_fps(&self) -> u32 {
        self.current_fps
    }
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a single [`GameLoop::tick`] call.
pub struct TickResult<T = ()> {
    /// Number of logic updates that were executed this tick.
    pub logic_updates: u32,
    /// Interpolation alpha in `[0.0, 1.0)` representing how far we are
    /// between the last logic frame and the next. Renderers should use
    /// this to interpolate visual positions for smooth display.
    pub alpha: f32,
    /// Collected results from all logic updates this tick.
    pub results: Vec<T>,
}

/// Fixed-timestep game loop with accumulator pattern.
///
/// Logic always advances at [`LOGIC_HZ`] (60 FPS). The render layer
/// runs at whatever rate the display supports; [`GameLoop::tick`]
/// returns an interpolation alpha so the renderer can blend between
/// the previous and current logic states.
pub struct GameLoop {
    accumulator: f64,
    previous_time: Instant,
    frame_counter: FrameCounter,
}

impl GameLoop {
    pub fn new() -> Self {
        Self {
            accumulator: 0.0,
            previous_time: Instant::now(),
            frame_counter: FrameCounter::new(),
        }
    }

    /// Returns the fixed logic timestep duration.
    pub fn logic_dt(&self) -> f64 {
        LOGIC_DT
    }

    /// Returns the fixed logic update rate in Hz.
    pub fn logic_hz(&self) -> f64 {
        LOGIC_HZ
    }

    /// Advance the game loop by the real elapsed time since the last
    /// call. Invokes `logic_update` zero or more times (at the fixed
    /// 60 FPS cadence) and returns a [`TickResult`] with the
    /// interpolation alpha for rendering.
    pub fn tick<T>(&mut self, mut logic_update: impl FnMut() -> T) -> TickResult<T> {
        let now = Instant::now();
        let frame_time = (now - self.previous_time).as_secs_f64();
        self.previous_time = now;

        // Clamp to prevent death spiral.
        let frame_time = frame_time.min(MAX_FRAME_TIME);
        self.accumulator += frame_time;

        let mut logic_updates: u32 = 0;
        let mut results = Vec::new();
        while self.accumulator >= LOGIC_DT {
            results.push(logic_update());
            self.accumulator -= LOGIC_DT;
            logic_updates += 1;
        }

        // Tick the FPS counter (once per rendered frame).
        self.frame_counter.tick();

        // Clamp to [0.0, 1.0) -- floating-point drift can push the
        // ratio to exactly 1.0 even though the while-loop condition
        // should have consumed it.
        let alpha = ((self.accumulator / LOGIC_DT) as f32).clamp(0.0, 1.0 - f32::EPSILON);

        TickResult {
            logic_updates,
            alpha,
            results,
        }
    }

    /// Variant of [`tick`](Self::tick) that accepts an explicit
    /// `frame_time` instead of measuring wall-clock time. Useful for
    /// deterministic testing and replay.
    pub fn tick_with_dt<T>(&mut self, frame_time: f64, mut logic_update: impl FnMut() -> T) -> TickResult<T> {
        let frame_time = frame_time.min(MAX_FRAME_TIME);
        self.accumulator += frame_time;

        let mut logic_updates: u32 = 0;
        let mut results = Vec::new();
        while self.accumulator >= LOGIC_DT {
            results.push(logic_update());
            self.accumulator -= LOGIC_DT;
            logic_updates += 1;
        }

        self.frame_counter.tick();

        let alpha = ((self.accumulator / LOGIC_DT) as f32).clamp(0.0, 1.0 - f32::EPSILON);

        TickResult {
            logic_updates,
            alpha,
            results,
        }
    }

    /// Access the embedded frame counter.
    pub fn frame_counter(&self) -> &FrameCounter {
        &self.frame_counter
    }

    /// Mutable access to the embedded frame counter.
    pub fn frame_counter_mut(&mut self) -> &mut FrameCounter {
        &mut self.frame_counter
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- Fixed timestep tests ------------------------------------------------

    #[test]
    fn fixed_timestep_60fps_logic() {
        let mut game_loop = GameLoop::new();
        let mut logic_count: u32 = 0;

        // Simulate 10 seconds at 144 Hz render rate.
        let render_dt = 1.0 / 144.0;
        let total_render_frames = 144 * 10; // 1440 frames

        for _ in 0..total_render_frames {
            let result = game_loop.tick_with_dt(render_dt, || {
                logic_count += 1;
            });
            // Alpha must always be in [0.0, 1.0).
            assert!(
                result.alpha >= 0.0 && result.alpha < 1.0,
                "alpha out of range: {}",
                result.alpha
            );
        }

        // 10 seconds at 60 FPS logic = 600 logic frames.
        // Allow a tolerance of 1 due to floating-point accumulation.
        assert!(
            (logic_count as i64 - 600).unsigned_abs() <= 1,
            "expected ~600 logic updates, got {logic_count}"
        );
    }

    #[test]
    fn fixed_timestep_at_60hz_render() {
        let mut game_loop = GameLoop::new();
        let mut logic_count: u32 = 0;

        // When render rate == logic rate, each tick should produce
        // exactly 1 logic update (modulo float drift).
        let render_dt = 1.0 / 60.0;
        for _ in 0..600 {
            game_loop.tick_with_dt(render_dt, || {
                logic_count += 1;
            });
        }

        assert!(
            (logic_count as i64 - 600).unsigned_abs() <= 1,
            "expected ~600 logic updates, got {logic_count}"
        );
    }

    #[test]
    fn fixed_timestep_at_240hz_render() {
        let mut game_loop = GameLoop::new();
        let mut logic_count: u32 = 0;

        let render_dt = 1.0 / 240.0;
        let total_render_frames = 240 * 5; // 5 seconds

        for _ in 0..total_render_frames {
            game_loop.tick_with_dt(render_dt, || {
                logic_count += 1;
            });
        }

        // 5 seconds * 60 FPS = 300 logic frames.
        assert!(
            (logic_count as i64 - 300).unsigned_abs() <= 1,
            "expected ~300 logic updates, got {logic_count}"
        );
    }

    // -- Interpolation alpha tests -------------------------------------------

    #[test]
    fn alpha_zero_right_after_logic_update() {
        let mut game_loop = GameLoop::new();

        // Feed exactly one logic tick worth of time.
        let result = game_loop.tick_with_dt(LOGIC_DT, || {});
        assert_eq!(result.logic_updates, 1);
        // Accumulator should be ~0, so alpha ~0.
        assert!(
            result.alpha < 0.01,
            "alpha should be ~0, got {}",
            result.alpha
        );
    }

    #[test]
    fn alpha_increases_between_logic_frames() {
        let mut game_loop = GameLoop::new();

        // Feed half a logic tick.
        let result = game_loop.tick_with_dt(LOGIC_DT * 0.5, || {});
        assert_eq!(result.logic_updates, 0);
        assert!(
            (result.alpha - 0.5).abs() < 0.01,
            "expected alpha ~0.5, got {}",
            result.alpha
        );
    }

    #[test]
    fn alpha_stays_in_range_over_many_frames() {
        let mut game_loop = GameLoop::new();
        let render_dt = 1.0 / 144.0;

        for _ in 0..10_000 {
            let result = game_loop.tick_with_dt(render_dt, || {});
            assert!(
                result.alpha >= 0.0 && result.alpha < 1.0,
                "alpha out of range: {}",
                result.alpha
            );
        }
    }

    // -- Accumulator / death spiral tests ------------------------------------

    #[test]
    fn death_spiral_prevention() {
        let mut game_loop = GameLoop::new();
        let mut logic_count: u32 = 0;

        // Simulate a massive lag spike of 2 seconds.
        let result = game_loop.tick_with_dt(2.0, || {
            logic_count += 1;
        });

        // Should be capped at MAX_FRAME_TIME (0.25s) worth of updates.
        // 0.25 / (1/60) = 15 logic updates.
        assert_eq!(
            logic_count, 15,
            "death spiral cap should limit to 15 updates"
        );
        assert!(result.alpha >= 0.0 && result.alpha < 1.0);
    }

    #[test]
    fn zero_dt_produces_no_updates() {
        let mut game_loop = GameLoop::new();
        let mut logic_count: u32 = 0;

        let result = game_loop.tick_with_dt(0.0, || {
            logic_count += 1;
        });

        assert_eq!(logic_count, 0);
        assert_eq!(result.logic_updates, 0);
        assert!(result.alpha.abs() < f32::EPSILON);
    }

    #[test]
    fn accumulator_carries_remainder() {
        let mut game_loop = GameLoop::new();
        let mut total_logic: u32 = 0;

        // Feed 1.5 logic ticks.
        let r1 = game_loop.tick_with_dt(LOGIC_DT * 1.5, || {
            total_logic += 1;
        });
        assert_eq!(r1.logic_updates, 1);
        // Remaining 0.5 tick in accumulator.
        assert!((r1.alpha - 0.5).abs() < 0.01);

        // Feed another 0.5 logic tick -- accumulator should now hit 1.0.
        let r2 = game_loop.tick_with_dt(LOGIC_DT * 0.5, || {
            total_logic += 1;
        });
        assert_eq!(r2.logic_updates, 1);
        assert!(r2.alpha < 0.01);

        assert_eq!(total_logic, 2);
    }

    // -- FrameCounter tests --------------------------------------------------

    #[test]
    fn frame_counter_initial_state() {
        let counter = FrameCounter::new();
        assert_eq!(counter.current_fps(), 0);
    }

    // -- RefreshRate tests ---------------------------------------------------

    #[test]
    fn refresh_rate_hz_values() {
        assert_eq!(RefreshRate::Hz60.as_hz(), 60);
        assert_eq!(RefreshRate::Hz120.as_hz(), 120);
        assert_eq!(RefreshRate::Hz144.as_hz(), 144);
        assert_eq!(RefreshRate::Hz240.as_hz(), 240);
    }

    #[test]
    fn refresh_rate_intervals() {
        let eps = Duration::from_micros(1);
        let check = |rate: RefreshRate, expected_us: u64| {
            let diff = if rate.interval() > Duration::from_micros(expected_us) {
                rate.interval() - Duration::from_micros(expected_us)
            } else {
                Duration::from_micros(expected_us) - rate.interval()
            };
            assert!(diff < eps, "{:?} interval off: {:?}", rate, diff);
        };
        check(RefreshRate::Hz60, 16_667);
        check(RefreshRate::Hz120, 8_333);
        check(RefreshRate::Hz144, 6_944);
        check(RefreshRate::Hz240, 4_167);
    }
}
