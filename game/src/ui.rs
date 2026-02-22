use hecs::World;
use tickle_core::{Health, PowerGauge};

use crate::quad_renderer::QuadInstance;
use crate::text_renderer::TextArea;
use crate::{Player1, Player2};

// ---------------------------------------------------------------------------
// Layout constants (all in screen pixels, designed for 800px wide)
// ---------------------------------------------------------------------------

const SCREEN_W: f32 = 800.0;

const HEALTH_BAR_W: f32 = 304.0;
const HEALTH_BAR_H: f32 = 18.0;
const HEALTH_BAR_Y: f32 = 22.0;
const HEALTH_BAR_P1_X: f32 = 20.0;
const HEALTH_BAR_P2_X: f32 = SCREEN_W - 20.0 - HEALTH_BAR_W;

const HEALTH_DRAIN_SPEED: f32 = 0.004;
const HEALTH_DRAIN_DELAY: u32 = 30;

// Win dots row (below health bar)
const WIN_DOT_Y: f32 = HEALTH_BAR_Y + HEALTH_BAR_H + 5.0;
const WIN_DOT_R: f32 = 5.0;
const WIN_DOT_GAP: f32 = 4.0;

// Power gauge (below win dots)
const GAUGE_Y: f32 = WIN_DOT_Y + WIN_DOT_R * 2.0 + 4.0;
const GAUGE_STOCK_W: f32 = 90.0;
const GAUGE_STOCK_H: f32 = 7.0;
const GAUGE_STOCK_GAP: f32 = 3.0;

// Timer (top center)
const TIMER_Y: f32 = 10.0;
const TIMER_BOX_W: f32 = 58.0;
const TIMER_BOX_H: f32 = 36.0;

// ---------------------------------------------------------------------------
// Colors — KOF2000 palette
// ---------------------------------------------------------------------------

const COLOR_BG: [f32; 4] = [0.05, 0.05, 0.07, 0.9];
const COLOR_BORDER: [f32; 4] = [0.78, 0.63, 0.0, 1.0];   // gold #c8a000
const COLOR_HP_GREEN: [f32; 4] = [0.06, 0.75, 0.19, 1.0]; // #10c030
const COLOR_HP_YELLOW: [f32; 4] = [0.82, 0.69, 0.0, 1.0]; // #d0b000
const COLOR_HP_RED: [f32; 4] = [0.82, 0.06, 0.06, 1.0];   // #d01010
const COLOR_HP_DRAIN: [f32; 4] = [0.9, 0.9, 0.9, 0.85];
const COLOR_WIN_DOT: [f32; 4] = [0.78, 0.63, 0.0, 1.0];
const COLOR_WIN_EMPTY: [f32; 4] = [0.2, 0.2, 0.2, 0.8];
const COLOR_GAUGE_FILL: [f32; 4] = [0.1, 0.42, 1.0, 1.0];  // #1a6bff
const COLOR_GAUGE_GLOW: [f32; 4] = [0.25, 0.67, 1.0, 0.5]; // glow
const COLOR_GAUGE_EMPTY: [f32; 4] = [0.04, 0.04, 0.12, 0.8];
const COLOR_TIMER_NORMAL: [f32; 4] = [0.91, 0.75, 0.0, 1.0]; // #e8c000
const COLOR_TIMER_LOW: [f32; 4] = [0.91, 0.13, 0.13, 1.0];   // #e82020
const COLOR_COMBO: [f32; 4] = [1.0, 0.8, 0.0, 1.0];

const LOW_HP_THRESHOLD: f32 = 0.25;
const MID_HP_THRESHOLD: f32 = 0.50;

// ---------------------------------------------------------------------------
// Per-player animated state
// ---------------------------------------------------------------------------

struct PlayerUI {
    display_hp: f32,
    drain_hp: f32,
    drain_delay: u32,
    flash_timer: u32,
    combo_count: u32,
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
        if actual_hp < self.display_hp - 0.001 {
            self.drain_delay = HEALTH_DRAIN_DELAY;
            self.flash_timer = 6;
            self.display_hp = actual_hp;
        } else {
            self.display_hp = actual_hp;
        }

        if self.drain_hp > self.display_hp {
            if self.drain_delay > 0 {
                self.drain_delay -= 1;
            } else {
                self.drain_hp = (self.drain_hp - HEALTH_DRAIN_SPEED).max(self.display_hp);
            }
        } else {
            self.drain_hp = self.display_hp;
        }

        if self.flash_timer > 0 {
            self.flash_timer -= 1;
        }

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

    /// HP color: green → yellow → red
    fn hp_color(&self) -> [f32; 4] {
        if self.display_hp > MID_HP_THRESHOLD {
            COLOR_HP_GREEN
        } else if self.display_hp > LOW_HP_THRESHOLD {
            COLOR_HP_YELLOW
        } else {
            COLOR_HP_RED
        }
    }

    fn is_flashing(&self) -> bool {
        self.display_hp < LOW_HP_THRESHOLD
            && (self.flash_timer > 0 || (self.combo_timer % 6) < 3)
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
    timer_flash: u32,
    p1_name: String,
    p2_name: String,
    p1_wins: u32,
    p2_wins: u32,
}

impl UIRenderer {
    const ROUND_SECONDS: u32 = 99;
    const FRAMES_PER_SECOND: u32 = 60;

    pub fn new() -> Self {
        Self {
            p1: PlayerUI::new(),
            p2: PlayerUI::new(),
            round_timer: Self::ROUND_SECONDS,
            timer_tick: 0,
            timer_flash: 0,
            p1_name: "PLAYER 1".to_string(),
            p2_name: "PLAYER 2".to_string(),
            p1_wins: 0,
            p2_wins: 0,
        }
    }

    pub fn set_names(&mut self, p1: &str, p2: &str) {
        self.p1_name = p1.to_string();
        self.p2_name = p2.to_string();
    }

    pub fn set_wins(&mut self, p1: u32, p2: u32) {
        self.p1_wins = p1;
        self.p2_wins = p2;
    }

    pub fn update(&mut self, world: &World) {
        let mut p1_hp: Option<f32> = None;
        let mut p2_hp: Option<f32> = None;

        for (_, (_, hp)) in world.query::<(&Player1, &Health)>().iter() {
            p1_hp = Some(hp.percentage());
        }
        for (_, (_, hp)) in world.query::<(&Player2, &Health)>().iter() {
            p2_hp = Some(hp.percentage());
        }

        if let Some(hp) = p1_hp { self.p1.update(hp); }
        if let Some(hp) = p2_hp { self.p2.update(hp); }

        if self.round_timer > 0 {
            self.timer_tick += 1;
            if self.timer_tick >= Self::FRAMES_PER_SECOND {
                self.timer_tick = 0;
                self.round_timer -= 1;
            }
        }

        // Timer flash when <= 10
        if self.round_timer <= 10 {
            self.timer_flash = (self.timer_flash + 1) % 60;
        } else {
            self.timer_flash = 0;
        }
    }

    pub fn register_hit(&mut self, attacker_is_p1: bool) {
        if attacker_is_p1 { self.p1.register_hit(); } else { self.p2.register_hit(); }
    }

    pub fn round_timer(&self) -> u32 { self.round_timer }

    pub fn reset(&mut self) {
        self.p1 = PlayerUI::new();
        self.p2 = PlayerUI::new();
        self.round_timer = Self::ROUND_SECONDS;
        self.timer_tick = 0;
        self.timer_flash = 0;
        // wins and names are preserved across rounds
    }

    // -----------------------------------------------------------------------
    // Quad rendering
    // -----------------------------------------------------------------------

    pub fn render(&self, world: &World, screen_w: f32) -> Vec<QuadInstance> {
        let mut quads = Vec::with_capacity(64);
        let sx = screen_w / SCREEN_W;

        let mut p1_gauge: Option<&PowerGauge> = None;
        let mut p2_gauge: Option<&PowerGauge> = None;
        let mut q1 = world.query::<(&Player1, &PowerGauge)>();
        for (_, (_, g)) in q1.iter() { p1_gauge = Some(g); }
        let mut q2 = world.query::<(&Player2, &PowerGauge)>();
        for (_, (_, g)) in q2.iter() { p2_gauge = Some(g); }

        // Health bars
        self.render_health_bar(&mut quads, &self.p1, HEALTH_BAR_P1_X * sx, HEALTH_BAR_Y, HEALTH_BAR_W * sx, false);
        self.render_health_bar(&mut quads, &self.p2, HEALTH_BAR_P2_X * sx, HEALTH_BAR_Y, HEALTH_BAR_W * sx, true);

        // Win dots
        self.render_win_dots(&mut quads, self.p1_wins, HEALTH_BAR_P1_X * sx, WIN_DOT_Y, false);
        self.render_win_dots(&mut quads, self.p2_wins, (HEALTH_BAR_P2_X + HEALTH_BAR_W) * sx, WIN_DOT_Y, true);

        // Power gauges
        let gw = GAUGE_STOCK_W * sx;
        let gg = GAUGE_STOCK_GAP * sx;
        if let Some(g) = p1_gauge {
            self.render_power_gauge(&mut quads, g, HEALTH_BAR_P1_X * sx, GAUGE_Y, gw, gg);
        }
        if let Some(g) = p2_gauge {
            let p2_gx = (HEALTH_BAR_P2_X + HEALTH_BAR_W) * sx - (gw * 3.0 + gg * 2.0);
            self.render_power_gauge(&mut quads, g, p2_gx, GAUGE_Y, gw, gg);
        }

        // Timer background box
        let timer_x = (screen_w - TIMER_BOX_W * sx) / 2.0;
        quads.push(QuadInstance { rect: [timer_x - 2.0, TIMER_Y - 2.0, TIMER_BOX_W * sx + 4.0, TIMER_BOX_H * sx + 4.0], color: COLOR_BORDER, ..Default::default() });
        quads.push(QuadInstance { rect: [timer_x, TIMER_Y, TIMER_BOX_W * sx, TIMER_BOX_H * sx], color: COLOR_BG, ..Default::default() });

        // Combo counters (quad blocks)
        self.render_combo_quads(&mut quads, &self.p1, 160.0 * sx, 80.0);
        self.render_combo_quads(&mut quads, &self.p2, screen_w - 160.0 * sx, 80.0);

        quads
    }

    fn render_health_bar(&self, quads: &mut Vec<QuadInstance>, player: &PlayerUI, x: f32, y: f32, w: f32, mirrored: bool) {
        // Outer gold border
        quads.push(QuadInstance { rect: [x - 1.0, y - 1.0, w + 2.0, HEALTH_BAR_H + 2.0], color: COLOR_BORDER, ..Default::default() });
        // Dark background
        quads.push(QuadInstance { rect: [x, y, w, HEALTH_BAR_H], color: COLOR_BG, ..Default::default() });

        // Drain bar (white)
        let drain_w = w * player.drain_hp;
        if drain_w > 0.0 {
            let drain_x = if mirrored { x + w - drain_w } else { x };
            quads.push(QuadInstance { rect: [drain_x, y, drain_w, HEALTH_BAR_H], color: COLOR_HP_DRAIN, ..Default::default() });
        }

        // Current HP bar
        let hp_w = w * player.display_hp;
        if hp_w > 0.0 && !player.is_flashing() {
            let hp_x = if mirrored { x + w - hp_w } else { x };
            quads.push(QuadInstance { rect: [hp_x, y, hp_w, HEALTH_BAR_H], color: player.hp_color(), ..Default::default() });
        }
    }

    fn render_win_dots(&self, quads: &mut Vec<QuadInstance>, wins: u32, anchor_x: f32, y: f32, right_align: bool) {
        const MAX_ROUNDS: u32 = 2;
        let total_w = MAX_ROUNDS as f32 * (WIN_DOT_R * 2.0) + (MAX_ROUNDS - 1) as f32 * WIN_DOT_GAP;
        let start_x = if right_align { anchor_x - total_w } else { anchor_x };

        for i in 0..MAX_ROUNDS {
            let dot_x = start_x + i as f32 * (WIN_DOT_R * 2.0 + WIN_DOT_GAP);
            let color = if i < wins { COLOR_WIN_DOT } else { COLOR_WIN_EMPTY };
            // Approximate circle with a square (proper circles need more quads)
            quads.push(QuadInstance { rect: [dot_x, y, WIN_DOT_R * 2.0, WIN_DOT_R * 2.0], color, ..Default::default() });
        }
    }

    fn render_power_gauge(&self, quads: &mut Vec<QuadInstance>, gauge: &PowerGauge, x: f32, y: f32, stock_w: f32, gap: f32) {
        let stocks = gauge.stocks();
        let partial = (gauge.current % PowerGauge::STOCK_SIZE) as f32 / PowerGauge::STOCK_SIZE as f32;

        for i in 0..3u32 {
            let sx = x + (stock_w + gap) * i as f32;
            // Border
            quads.push(QuadInstance { rect: [sx - 1.0, y - 1.0, stock_w + 2.0, GAUGE_STOCK_H + 2.0], color: COLOR_BORDER, ..Default::default() });
            // Background
            quads.push(QuadInstance { rect: [sx, y, stock_w, GAUGE_STOCK_H], color: COLOR_GAUGE_EMPTY, ..Default::default() });

            let stocks_u32 = stocks as u32;
            if i < stocks_u32 {
                // Full stock + glow
                quads.push(QuadInstance { rect: [sx - 1.0, y - 1.0, stock_w + 2.0, GAUGE_STOCK_H + 2.0], color: COLOR_GAUGE_GLOW, ..Default::default() });
                quads.push(QuadInstance { rect: [sx, y, stock_w, GAUGE_STOCK_H], color: COLOR_GAUGE_FILL, ..Default::default() });
            } else if i == stocks_u32 && partial > 0.0 {
                quads.push(QuadInstance { rect: [sx, y, stock_w * partial, GAUGE_STOCK_H], color: COLOR_GAUGE_FILL, ..Default::default() });
            }
        }
    }

    fn render_combo_quads(&self, quads: &mut Vec<QuadInstance>, player: &PlayerUI, center_x: f32, y: f32) {
        if player.combo_count < 2 { return; }
        let count = player.combo_count.min(20) as usize;
        let block_w = 7.0;
        let block_h = 10.0;
        let block_gap = 2.0;
        let total_w = count as f32 * (block_w + block_gap) - block_gap;
        let start_x = center_x - total_w / 2.0;
        for i in 0..count {
            quads.push(QuadInstance {
                rect: [start_x + i as f32 * (block_w + block_gap), y, block_w, block_h],
                color: COLOR_COMBO,
            ..Default::default()
            });
        }
    }

    // -----------------------------------------------------------------------
    // Text rendering
    // -----------------------------------------------------------------------

    pub fn render_text(&self, world: &World, screen_w: f32, _screen_h: f32) -> Vec<TextArea> {
        let sx = screen_w / SCREEN_W;
        let mut areas = Vec::new();

        // P1 name (above health bar, left-aligned)
        areas.push(TextArea {
            text: self.p1_name.clone(),
            x: HEALTH_BAR_P1_X * sx,
            y: HEALTH_BAR_Y - 14.0,
            size: 8.0,
            color: [0.9, 0.9, 0.9, 1.0],
            bounds: None,
        });

        // P2 name (above health bar, right-aligned)
        let p2_name_w = self.p2_name.len() as f32 * 8.0 * 0.6;
        areas.push(TextArea {
            text: self.p2_name.clone(),
            x: (HEALTH_BAR_P2_X + HEALTH_BAR_W) * sx - p2_name_w,
            y: HEALTH_BAR_Y - 14.0,
            size: 8.0,
            color: [0.9, 0.9, 0.9, 1.0],
            bounds: None,
        });

        // Timer
        let timer_text = format!("{:02}", self.round_timer);
        let timer_size = 24.0;
        let timer_w = timer_text.len() as f32 * timer_size * 0.6;
        let timer_visible = self.round_timer > 10 || self.timer_flash < 30;
        if timer_visible {
            let timer_color = if self.round_timer <= 10 { COLOR_TIMER_LOW } else { COLOR_TIMER_NORMAL };
            areas.push(TextArea {
                text: timer_text,
                x: (screen_w - timer_w) / 2.0,
                y: TIMER_Y + (TIMER_BOX_H * sx - timer_size) / 2.0,
                size: timer_size,
                color: timer_color,
                bounds: None,
            });
        }

        // Combo text
        if self.p1.combo_count >= 2 {
            let t = format!("{} HIT", self.p1.combo_count);
            let tw = t.len() as f32 * 10.0 * 0.6;
            areas.push(TextArea { text: t, x: 160.0 * sx - tw / 2.0, y: 93.0, size: 10.0, color: COLOR_COMBO, bounds: None });
        }
        if self.p2.combo_count >= 2 {
            let t = format!("{} HIT", self.p2.combo_count);
            let tw = t.len() as f32 * 10.0 * 0.6;
            areas.push(TextArea { text: t, x: (screen_w - 160.0 * sx) - tw / 2.0, y: 93.0, size: 10.0, color: COLOR_COMBO, bounds: None });
        }

        let _ = world;
        areas
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
        p.update(0.5);
        assert!((p.display_hp - 0.5).abs() < 0.01);
        assert!(p.drain_hp > 0.5);
        assert!(p.drain_delay > 0);
        for _ in 0..HEALTH_DRAIN_DELAY {
            p.update(0.5);
        }
        let before = p.drain_hp;
        p.update(0.5);
        assert!(p.drain_hp < before || (p.drain_hp - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_hp_color_thresholds() {
        let mut p = PlayerUI::new();
        p.update(0.8);
        assert_eq!(p.hp_color(), COLOR_HP_GREEN);
        p.display_hp = 0.4;
        assert_eq!(p.hp_color(), COLOR_HP_YELLOW);
        p.display_hp = 0.2;
        assert_eq!(p.hp_color(), COLOR_HP_RED);
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
        let world = World::new();
        for _ in 0..60 { ui.update(&world); }
        assert_eq!(ui.round_timer(), 98);
    }

    #[test]
    fn test_combo_counter() {
        let mut p = PlayerUI::new();
        p.register_hit();
        assert_eq!(p.combo_count, 1);
        p.register_hit();
        assert_eq!(p.combo_count, 2);
        for _ in 0..61 { p.update(1.0); }
        assert_eq!(p.combo_count, 0);
    }

    #[test]
    fn test_win_dots_preserved_on_reset() {
        let mut ui = UIRenderer::new();
        ui.set_wins(1, 0);
        ui.reset();
        assert_eq!(ui.p1_wins, 1); // wins survive round reset
    }
}

