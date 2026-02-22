use hecs::World;
use tickle_core::{Health, PowerGauge};

use crate::quad_renderer::QuadInstance;
use crate::{Player1, Player2};

// ---------------------------------------------------------------------------
// Layout constants (all in screen pixels)
// ---------------------------------------------------------------------------

const SCREEN_W: f32 = 800.0;

/// Health bar dimensions and position.
const HEALTH_BAR_W: f32 = 300.0;
const HEALTH_BAR_H: f32 = 20.0;
const HEALTH_BAR_Y: f32 = 20.0;
const HEALTH_BAR_P1_X: f32 = 20.0;
const HEALTH_BAR_P2_X: f32 = SCREEN_W - 20.0 - HEALTH_BAR_W;

/// Delayed (white) health bar drain speed per frame (fraction of bar).
const HEALTH_DRAIN_SPEED: f32 = 0.005;
/// Delay frames before the white bar starts draining.
const HEALTH_DRAIN_DELAY: u32 = 30;

/// Power gauge dimensions (below health bar).
const GAUGE_Y: f32 = 48.0;
const GAUGE_STOCK_W: f32 = 60.0;
const GAUGE_STOCK_H: f32 = 10.0;
const GAUGE_STOCK_GAP: f32 = 4.0;
const GAUGE_P1_X: f32 = 20.0;

/// Round timer position (top center).
const TIMER_Y: f32 = 16.0;
const TIMER_DIGIT_W: f32 = 14.0;
const TIMER_DIGIT_H: f32 = 24.0;
const TIMER_DIGIT_GAP: f32 = 4.0;

/// Combo counter position (near each fighter).
const COMBO_W: f32 = 8.0;
const COMBO_H: f32 = 12.0;
const COMBO_GAP: f32 = 2.0;

// ---------------------------------------------------------------------------
// Colors
// ---------------------------------------------------------------------------

const COLOR_BG: [f32; 4] = [0.15, 0.15, 0.15, 0.8];
const COLOR_HP_P1: [f32; 4] = [0.1, 0.7, 0.2, 1.0];
const COLOR_HP_P2: [f32; 4] = [0.8, 0.2, 0.2, 1.0];
const COLOR_HP_DRAIN: [f32; 4] = [0.9, 0.9, 0.9, 0.8];
const COLOR_HP_LOW: [f32; 4] = [0.9, 0.1, 0.1, 1.0];
const COLOR_GAUGE_FULL: [f32; 4] = [0.2, 0.5, 0.9, 1.0];
const COLOR_GAUGE_PARTIAL: [f32; 4] = [0.1, 0.3, 0.6, 1.0];
const COLOR_GAUGE_EMPTY: [f32; 4] = [0.1, 0.1, 0.1, 0.6];
const COLOR_TIMER: [f32; 4] = [0.9, 0.9, 0.3, 1.0];
const COLOR_TIMER_LOW: [f32; 4] = [0.9, 0.2, 0.2, 1.0];
const COLOR_COMBO: [f32; 4] = [1.0, 0.8, 0.0, 1.0];

/// Low-health threshold (fraction).
const LOW_HP_THRESHOLD: f32 = 0.25;

// ---------------------------------------------------------------------------
// Per-player animated state
// ---------------------------------------------------------------------------

struct PlayerUI {
    /// Displayed health ratio (smoothly animated).
    display_hp: f32,
    /// Delayed "white bar" health ratio.
    drain_hp: f32,
    /// Frames remaining before drain starts.
    drain_delay: u32,
    /// Flash timer (frames remaining).
    flash_timer: u32,
    /// Combo hit count.
    combo_count: u32,
    /// Frames since last combo hit (resets combo when too high).
    combo_timer: u32,
}

impl PlayerUI {
    fn new() -> Self {
        Self {
            display_hp: 1.0,
            drain_hp: 1.0,
            drain_delay: 0,
            flash_timer: 0,
            combo_count: 0,
            combo_timer: 0,
        }
    }

    fn update(&mut self, actual_hp: f32) {
        // Detect damage: actual dropped below displayed.
        if actual_hp < self.display_hp - 0.001 {
            self.drain_delay = HEALTH_DRAIN_DELAY;
            self.flash_timer = 6;
            self.display_hp = actual_hp;
        } else {
            self.display_hp = actual_hp;
        }

        // Drain the white bar after delay.
        if self.drain_hp > self.display_hp {
            if self.drain_delay > 0 {
                self.drain_delay -= 1;
            } else {
                self.drain_hp = (self.drain_hp - HEALTH_DRAIN_SPEED)
                    .max(self.display_hp);
            }
        } else {
            self.drain_hp = self.display_hp;
        }

        // Tick flash.
        if self.flash_timer > 0 {
            self.flash_timer -= 1;
        }

        // Combo timeout.
        if self.combo_count > 0 {
            self.combo_timer += 1;
            if self.combo_timer > 60 {
                self.combo_count = 0;
                self.combo_timer = 0;
            }
        }
    }

    fn register_hit(&mut self) {
        self.combo_count += 1;
        self.combo_timer = 0;
    }
}

// ---------------------------------------------------------------------------
// UIRenderer
// ---------------------------------------------------------------------------

pub struct UIRenderer {
    p1: PlayerUI,
    p2: PlayerUI,
    round_timer: u32,
    timer_tick: u32,
}

impl UIRenderer {
    /// 99-second round timer at 60 FPS.
    const ROUND_SECONDS: u32 = 99;
    const FRAMES_PER_SECOND: u32 = 60;

    pub fn new() -> Self {
        Self {
            p1: PlayerUI::new(),
            p2: PlayerUI::new(),
            round_timer: Self::ROUND_SECONDS,
            timer_tick: 0,
        }
    }

    /// Call once per logic frame to advance timer and animations.
    pub fn update(&mut self, world: &World) {
        // Read health from ECS.
        let mut p1_hp: Option<f32> = None;
        let mut p2_hp: Option<f32> = None;

        for (_, (_, hp)) in world.query::<(&Player1, &Health)>().iter() {
            p1_hp = Some(hp.percentage());
        }
        for (_, (_, hp)) in world.query::<(&Player2, &Health)>().iter() {
            p2_hp = Some(hp.percentage());
        }

        if let Some(hp) = p1_hp {
            self.p1.update(hp);
        }
        if let Some(hp) = p2_hp {
            self.p2.update(hp);
        }

        // Countdown timer.
        if self.round_timer > 0 {
            self.timer_tick += 1;
            if self.timer_tick >= Self::FRAMES_PER_SECOND {
                self.timer_tick = 0;
                self.round_timer -= 1;
            }
        }
    }

    /// Register a combo hit on the given player (0 = P1 attacking, 1 = P2 attacking).
    pub fn register_hit(&mut self, attacker_is_p1: bool) {
        if attacker_is_p1 {
            self.p1.register_hit();
        } else {
            self.p2.register_hit();
        }
    }

    pub fn round_timer(&self) -> u32 {
        self.round_timer
    }

    pub fn reset(&mut self) {
        self.p1 = PlayerUI::new();
        self.p2 = PlayerUI::new();
        self.round_timer = Self::ROUND_SECONDS;
        self.timer_tick = 0;
    }

    /// Generate quad instances for the entire HUD overlay.
    pub fn render(&self, world: &World, screen_w: f32) -> Vec<QuadInstance> {
        let mut quads = Vec::with_capacity(32);

        // Read power gauges.
        let mut p1_gauge: Option<&PowerGauge> = None;
        let mut p2_gauge: Option<&PowerGauge> = None;
        let mut q1 = world.query::<(&Player1, &PowerGauge)>();
        for (_, (_, g)) in q1.iter() {
            p1_gauge = Some(g);
        }
        let mut q2 = world.query::<(&Player2, &PowerGauge)>();
        for (_, (_, g)) in q2.iter() {
            p2_gauge = Some(g);
        }

        // Scale factor for non-800px screens.
        let sx = screen_w / SCREEN_W;

        // --- Health bars ---
        self.render_health_bar(
            &mut quads,
            &self.p1,
            HEALTH_BAR_P1_X * sx,
            HEALTH_BAR_Y,
            HEALTH_BAR_W * sx,
            COLOR_HP_P1,
            false,
        );
        self.render_health_bar(
            &mut quads,
            &self.p2,
            HEALTH_BAR_P2_X * sx,
            HEALTH_BAR_Y,
            HEALTH_BAR_W * sx,
            COLOR_HP_P2,
            true,
        );

        // --- Power gauges ---
        let gauge_stock_w = GAUGE_STOCK_W * sx;
        let gauge_gap = GAUGE_STOCK_GAP * sx;
        if let Some(g) = p1_gauge {
            self.render_power_gauge(
                &mut quads,
                g,
                GAUGE_P1_X * sx,
                GAUGE_Y,
                gauge_stock_w,
                gauge_gap,
            );
        }
        if let Some(g) = p2_gauge {
            let p2_gx = screen_w - 20.0 * sx
                - (gauge_stock_w * 3.0 + gauge_gap * 2.0);
            self.render_power_gauge(
                &mut quads, g, p2_gx, GAUGE_Y, gauge_stock_w, gauge_gap,
            );
        }

        // --- Round timer ---
        self.render_timer(&mut quads, screen_w);

        // --- Combo counters ---
        self.render_combo(&mut quads, &self.p1, 160.0 * sx, 80.0);
        self.render_combo(&mut quads, &self.p2, screen_w - 160.0 * sx, 80.0);

        quads
    }

    fn render_health_bar(
        &self,
        quads: &mut Vec<QuadInstance>,
        player: &PlayerUI,
        x: f32,
        y: f32,
        w: f32,
        color: [f32; 4],
        _mirrored: bool,
    ) {
        // Background.
        quads.push(QuadInstance {
            rect: [x, y, w, HEALTH_BAR_H],
            color: COLOR_BG,
        });

        // Drain bar (white).
        let drain_w = w * player.drain_hp;
        if drain_w > 0.0 {
            quads.push(QuadInstance {
                rect: [x, y, drain_w, HEALTH_BAR_H],
                color: COLOR_HP_DRAIN,
            });
        }

        // Current health bar.
        let hp_w = w * player.display_hp;
        if hp_w > 0.0 {
            let hp_color = if player.display_hp < LOW_HP_THRESHOLD {
                COLOR_HP_LOW
            } else {
                color
            };
            // Flash effect: alternate visibility.
            let visible = player.flash_timer == 0 || player.flash_timer % 2 == 0;
            if visible {
                quads.push(QuadInstance {
                    rect: [x, y, hp_w, HEALTH_BAR_H],
                    color: hp_color,
                });
            }
        }

        // Border (top, bottom, left, right as thin quads).
        let border_color: [f32; 4] = [0.6, 0.6, 0.6, 1.0];
        let b = 2.0;
        // Top
        quads.push(QuadInstance {
            rect: [x, y, w, b],
            color: border_color,
        });
        // Bottom
        quads.push(QuadInstance {
            rect: [x, y + HEALTH_BAR_H - b, w, b],
            color: border_color,
        });
    }

    fn render_power_gauge(
        &self,
        quads: &mut Vec<QuadInstance>,
        gauge: &PowerGauge,
        x: f32,
        y: f32,
        stock_w: f32,
        gap: f32,
    ) {
        let stocks = gauge.stocks();
        let partial = (gauge.current % PowerGauge::STOCK_SIZE) as f32
            / PowerGauge::STOCK_SIZE as f32;

        for i in 0..3 {
            let sx = x + (stock_w + gap) * i as f32;
            // Background.
            quads.push(QuadInstance {
                rect: [sx, y, stock_w, GAUGE_STOCK_H],
                color: COLOR_GAUGE_EMPTY,
            });
            // Fill.
            if i < stocks {
                // Full stock.
                quads.push(QuadInstance {
                    rect: [sx, y, stock_w, GAUGE_STOCK_H],
                    color: COLOR_GAUGE_FULL,
                });
            } else if i == stocks {
                // Partial stock.
                let fill_w = stock_w * partial;
                if fill_w > 0.0 {
                    quads.push(QuadInstance {
                        rect: [sx, y, fill_w, GAUGE_STOCK_H],
                        color: COLOR_GAUGE_PARTIAL,
                    });
                }
            }
        }
    }

    fn render_timer(&self, quads: &mut Vec<QuadInstance>, screen_w: f32) {
        let seconds = self.round_timer;
        let tens = seconds / 10;
        let ones = seconds % 10;
        let color = if seconds <= 10 {
            COLOR_TIMER_LOW
        } else {
            COLOR_TIMER
        };

        let total_w = TIMER_DIGIT_W * 2.0 + TIMER_DIGIT_GAP;
        let start_x = (screen_w - total_w) / 2.0;

        // Render each digit as a filled rectangle (placeholder for real font).
        // Tens digit.
        self.render_digit(quads, tens, start_x, TIMER_Y, color);
        // Ones digit.
        self.render_digit(
            quads,
            ones,
            start_x + TIMER_DIGIT_W + TIMER_DIGIT_GAP,
            TIMER_Y,
            color,
        );
    }

    /// Render a single digit as a simple seven-segment-style display using quads.
    fn render_digit(&self, quads: &mut Vec<QuadInstance>, digit: u32, x: f32, y: f32, color: [f32; 4]) {
        // Segments: top, top-left, top-right, middle, bot-left, bot-right, bottom
        // Each segment is a thin rectangle.
        let w = TIMER_DIGIT_W;
        let h = TIMER_DIGIT_H;
        let seg = 3.0; // segment thickness
        let half = h / 2.0;

        // Segment definitions: (x_off, y_off, w, h)
        let segments: [(f32, f32, f32, f32); 7] = [
            (0.0, 0.0, w, seg),                // 0: top
            (0.0, 0.0, seg, half),              // 1: top-left
            (w - seg, 0.0, seg, half),          // 2: top-right
            (0.0, half - seg / 2.0, w, seg),    // 3: middle
            (0.0, half, seg, half),             // 4: bot-left
            (w - seg, half, seg, half),         // 5: bot-right
            (0.0, h - seg, w, seg),             // 6: bottom
        ];

        // Which segments are on for each digit (0-9).
        let patterns: [u8; 10] = [
            0b1110111, // 0
            0b0010010, // 1
            0b1011101, // 2
            0b1011011, // 3
            0b0111010, // 4
            0b1101011, // 5
            0b1101111, // 6
            0b1010010, // 7
            0b1111111, // 8
            0b1111011, // 9
        ];

        let pattern = patterns[digit as usize % 10];
        for (i, &(sx, sy, sw, sh)) in segments.iter().enumerate() {
            if pattern & (1 << (6 - i)) != 0 {
                quads.push(QuadInstance {
                    rect: [x + sx, y + sy, sw, sh],
                    color,
                });
            }
        }
    }

    fn render_combo(
        &self,
        quads: &mut Vec<QuadInstance>,
        player: &PlayerUI,
        center_x: f32,
        y: f32,
    ) {
        if player.combo_count < 2 {
            return;
        }
        // Show combo count as small blocks (one per hit, max 20 visible).
        let count = player.combo_count.min(20) as usize;
        let total_w = count as f32 * (COMBO_W + COMBO_GAP) - COMBO_GAP;
        let start_x = center_x - total_w / 2.0;
        for i in 0..count {
            quads.push(QuadInstance {
                rect: [
                    start_x + i as f32 * (COMBO_W + COMBO_GAP),
                    y,
                    COMBO_W,
                    COMBO_H,
                ],
                color: COLOR_COMBO,
            });
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_bar_drain_animation() {
        let mut p = PlayerUI::new();
        assert_eq!(p.display_hp, 1.0);
        assert_eq!(p.drain_hp, 1.0);

        // Simulate taking damage to 50%.
        p.update(0.5);
        assert!((p.display_hp - 0.5).abs() < 0.01);
        // Drain should still be at 1.0 (delay not elapsed).
        assert!(p.drain_hp > 0.5);
        assert!(p.drain_delay > 0);

        // Tick through the delay.
        for _ in 0..HEALTH_DRAIN_DELAY {
            p.update(0.5);
        }
        // Now drain should start moving.
        let before = p.drain_hp;
        p.update(0.5);
        assert!(p.drain_hp < before || (p.drain_hp - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_power_gauge_stock_display() {
        let mut g = PowerGauge::new();
        assert_eq!(g.stocks(), 0);

        g.add(1500);
        assert_eq!(g.stocks(), 1);

        g.add(1500);
        assert_eq!(g.stocks(), 3);
    }

    #[test]
    fn test_timer_countdown() {
        let mut ui = UIRenderer::new();
        assert_eq!(ui.round_timer(), 99);

        // Create an empty world for update.
        let world = World::new();

        // Tick 60 frames = 1 second.
        for _ in 0..60 {
            ui.update(&world);
        }
        assert_eq!(ui.round_timer(), 98);
    }

    #[test]
    fn test_combo_counter() {
        let mut p = PlayerUI::new();
        p.register_hit();
        assert_eq!(p.combo_count, 1);
        p.register_hit();
        assert_eq!(p.combo_count, 2);

        // Timeout resets combo.
        for _ in 0..61 {
            p.update(1.0);
        }
        assert_eq!(p.combo_count, 0);
    }

    #[test]
    fn test_flash_on_damage() {
        let mut p = PlayerUI::new();
        p.update(0.8);
        assert!(p.flash_timer > 0);
        // After enough ticks, flash ends.
        for _ in 0..10 {
            p.update(0.8);
        }
        assert_eq!(p.flash_timer, 0);
    }
}
