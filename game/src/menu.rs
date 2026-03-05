use hecs::World;
use tickle_audio;
use tickle_core::{
    FighterState, Health, InputBuffer, LogicVec2, Position, PowerGauge, PreviousPosition, Velocity,
};
use tickle_mugen::MugenFighterState;

use crate::quad_renderer::QuadInstance;
use crate::text_renderer::TextArea;
use crate::ui::UIRenderer;
use crate::{Player1, Player2};

// ---------------------------------------------------------------------------
// Game state
// ---------------------------------------------------------------------------

/// Top-level game state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    MainMenu,
    RoundIntro, // NEW: "ROUND X / FIGHT!" animation before fighting
    Fighting,
    Paused,
    RoundEnd,
    MatchEnd,
}

/// Which game mode was selected from the main menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameMode {
    Vs,
    Training,
}

/// Main menu items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MainMenuItem {
    VsMode,
    Training,
    Quit,
}

const MAIN_MENU_ITEMS: [MainMenuItem; 3] = [
    MainMenuItem::VsMode,
    MainMenuItem::Training,
    MainMenuItem::Quit,
];

/// Pause menu items.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PauseMenuItem {
    Resume,
    Quit,
}

const PAUSE_MENU_ITEMS: [PauseMenuItem; 2] = [PauseMenuItem::Resume, PauseMenuItem::Quit];

// ---------------------------------------------------------------------------
// Menu input (abstracted from raw keys)
// ---------------------------------------------------------------------------

/// Simplified input events consumed by the menu system.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuInput {
    Up,
    Down,
    Confirm,
    Back,
    Pause,
    None,
}

// ---------------------------------------------------------------------------
// Round / match tracking
// ---------------------------------------------------------------------------

const ROUNDS_TO_WIN: u32 = 2;
const ROUND_END_FREEZE_FRAMES: u32 = 120; // 2 seconds at 60 FPS
const ROUND_INTRO_FRAMES: u32 = 150;      // 2.5 seconds for ROUND X + FIGHT!

// ---------------------------------------------------------------------------
// MenuSystem
// ---------------------------------------------------------------------------

#[allow(dead_code)]
pub struct MenuSystem {
    pub game_state: GameState,
    pub game_mode: GameMode,
    // Main menu
    main_cursor: usize,
    // Pause menu
    pause_cursor: usize,
    // Round tracking
    p1_wins: u32,
    p2_wins: u32,
    current_round: u32,
    round_end_timer: u32,
    round_intro_timer: u32,
    round_winner: Option<u8>, // 1 or 2
    match_winner: Option<u8>,
    // Training mode
    pub training_infinite_hp: bool,
    pub training_show_hitboxes: bool,
    pub training_show_framedata: bool,
    // Request to quit the application
    pub quit_requested: bool,
}

#[allow(dead_code)]
impl MenuSystem {
    pub fn new() -> Self {
        Self {
            game_state: GameState::MainMenu,
            game_mode: GameMode::Vs,
            main_cursor: 0,
            pause_cursor: 0,
            p1_wins: 0,
            p2_wins: 0,
            current_round: 1,
            round_end_timer: 0,
            round_intro_timer: 0,
            round_winner: None,
            match_winner: None,
            training_infinite_hp: true,
            training_show_hitboxes: true,
            training_show_framedata: false,
            quit_requested: false,
        }
    }

    pub fn p1_wins(&self) -> u32 {
        self.p1_wins
    }

    pub fn p2_wins(&self) -> u32 {
        self.p2_wins
    }

    pub fn current_round(&self) -> u32 {
        self.current_round
    }

    pub fn round_winner(&self) -> Option<u8> {
        self.round_winner
    }

    pub fn match_winner(&self) -> Option<u8> {
        self.match_winner
    }

    /// Returns true if the game logic should run this frame.
    pub fn should_run_logic(&self) -> bool {
        self.game_state == GameState::Fighting || self.game_state == GameState::RoundIntro
    }

    /// Process a menu input event. Returns true if the world needs resetting.
    pub fn handle_input(&mut self, input: MenuInput) -> bool {
        match self.game_state {
            GameState::MainMenu => {
                self.handle_main_menu(input);
                // Signal world reset if we just transitioned into RoundIntro.
                self.game_state == GameState::RoundIntro
            }
            GameState::RoundIntro => {
                // Input ignored during intro animation
                false
            }
            GameState::Fighting => {
                if input == MenuInput::Pause {
                    self.pause_cursor = 0;
                    self.game_state = GameState::Paused;
                }
                false
            }
            GameState::Paused => self.handle_pause_menu(input),
            GameState::RoundEnd => {
                // Input ignored during round-end freeze
                false
            }
            GameState::MatchEnd => {
                if input == MenuInput::Confirm {
                    self.reset_match();
                    self.game_state = GameState::MainMenu;
                }
                false
            }
        }
    }

    fn handle_main_menu(&mut self, input: MenuInput) {
        match input {
            MenuInput::Up => {
                if self.main_cursor > 0 {
                    self.main_cursor -= 1;
                }
            }
            MenuInput::Down => {
                if self.main_cursor < MAIN_MENU_ITEMS.len() - 1 {
                    self.main_cursor += 1;
                }
            }
            MenuInput::Confirm => match MAIN_MENU_ITEMS[self.main_cursor] {
                MainMenuItem::VsMode => {
                    self.game_mode = GameMode::Vs;
                    self.reset_match();
                    self.start_round_intro();
                }
                MainMenuItem::Training => {
                    self.game_mode = GameMode::Training;
                    self.reset_match();
                    self.start_round_intro();
                }
                MainMenuItem::Quit => {
                    self.quit_requested = true;
                }
            },
            _ => {}
        }
    }

    /// Returns true if the world needs resetting (quit to menu).
    fn handle_pause_menu(&mut self, input: MenuInput) -> bool {
        match input {
            MenuInput::Up => {
                if self.pause_cursor > 0 {
                    self.pause_cursor -= 1;
                }
                false
            }
            MenuInput::Down => {
                if self.pause_cursor < PAUSE_MENU_ITEMS.len() - 1 {
                    self.pause_cursor += 1;
                }
                false
            }
            MenuInput::Confirm => match PAUSE_MENU_ITEMS[self.pause_cursor] {
                PauseMenuItem::Resume => {
                    self.game_state = GameState::Fighting;
                    false
                }
                PauseMenuItem::Quit => {
                    self.reset_match();
                    self.game_state = GameState::MainMenu;
                    true
                }
            },
            MenuInput::Back | MenuInput::Pause => {
                self.game_state = GameState::Fighting;
                false
            }
            _ => false,
        }
    }

    /// Called each logic frame. Returns true if the world should be reset for a new round.
    pub fn update_round(&mut self, world: &World, ui: &UIRenderer) -> (bool, Option<tickle_audio::AudioEvent>) {
        // Handle round intro animation
        if self.game_state == GameState::RoundIntro {
            return (self.tick_round_intro(), None);
        }

        if self.game_state == GameState::RoundEnd {
            return (self.tick_round_end(), None);
        }
        if self.game_state != GameState::Fighting {
            return (false, None);
        }

        // Training mode: refill health each frame if infinite HP is on.
        if self.game_mode == GameMode::Training && self.training_infinite_hp {
            for (_, (_, hp)) in world.query::<(&Player1, &mut Health)>().iter() {
                hp.current = hp.max;
            }
            for (_, (_, hp)) in world.query::<(&Player2, &mut Health)>().iter() {
                hp.current = hp.max;
            }
            return (false, None);
        }

        // Check KO.
        let mut p1_hp = 0i32;
        let mut p2_hp = 0i32;
        for (_, (_, hp)) in world.query::<(&Player1, &Health)>().iter() {
            p1_hp = hp.current;
        }
        for (_, (_, hp)) in world.query::<(&Player2, &Health)>().iter() {
            p2_hp = hp.current;
        }

        let ko = p1_hp <= 0 || p2_hp <= 0;
        let time_up = ui.round_timer() == 0;

        if ko || time_up {
            // Determine round winner.
            let winner = if p1_hp > p2_hp {
                1
            } else if p2_hp > p1_hp {
                2
            } else {
                0 // draw
            };
            self.round_winner = if winner > 0 { Some(winner) } else { None };
            self.round_end_timer = ROUND_END_FREEZE_FRAMES;
            self.game_state = GameState::RoundEnd;
            return (false, Some(tickle_audio::AudioEvent::PlayKOSound));
        }
        (false, None)
    }

    fn start_round_intro(&mut self) {
        self.round_intro_timer = ROUND_INTRO_FRAMES;
        self.game_state = GameState::RoundIntro;
    }

    fn tick_round_intro(&mut self) -> bool {
        if self.round_intro_timer > 0 {
            self.round_intro_timer -= 1;
            return false;
        }
        // Intro finished, start fighting
        self.game_state = GameState::Fighting;
        false
    }

    fn tick_round_end(&mut self) -> bool {
        if self.round_end_timer > 0 {
            self.round_end_timer -= 1;
            return false;
        }
        // Award win.
        match self.round_winner {
            Some(1) => self.p1_wins += 1,
            Some(2) => self.p2_wins += 1,
            _ => {} // draw: no one gets a point
        }
        // Check match winner.
        if self.p1_wins >= ROUNDS_TO_WIN {
            self.match_winner = Some(1);
            self.game_state = GameState::MatchEnd;
            return false;
        }
        if self.p2_wins >= ROUNDS_TO_WIN {
            self.match_winner = Some(2);
            self.game_state = GameState::MatchEnd;
            return false;
        }
        // Next round.
        self.current_round += 1;
        self.round_winner = None;
        self.start_round_intro();
        true // signal: reset world for new round
    }

    fn reset_match(&mut self) {
        self.p1_wins = 0;
        self.p2_wins = 0;
        self.current_round = 1;
        self.round_winner = None;
        self.match_winner = None;
        self.round_end_timer = 0;
    }

    /// Reset fighter positions and state for a new round.
    pub fn reset_fighters(world: &mut World) {
        for (_, (_, pos, prev, vel, fs, mugen, ib, hp, gauge)) in world.query_mut::<(
            &Player1,
            &mut Position,
            &mut PreviousPosition,
            &mut Velocity,
            &mut FighterState,
            &mut MugenFighterState,
            &mut InputBuffer,
            &mut Health,
            &mut PowerGauge,
        )>() {
            pos.pos = LogicVec2::from_pixels(200, 0);
            prev.pos = pos.pos;
            vel.vel = LogicVec2::ZERO;
            *fs = FighterState::new();
            *mugen = MugenFighterState::default();
            *ib = InputBuffer::new();
            hp.current = hp.max;
            gauge.current = 0;
        }
        for (_, (_, pos, prev, vel, fs, mugen, ib, hp, gauge)) in world.query_mut::<(
            &Player2,
            &mut Position,
            &mut PreviousPosition,
            &mut Velocity,
            &mut FighterState,
            &mut MugenFighterState,
            &mut InputBuffer,
            &mut Health,
            &mut PowerGauge,
        )>() {
            pos.pos = LogicVec2::from_pixels(600, 0);
            prev.pos = pos.pos;
            vel.vel = LogicVec2::ZERO;
            *fs = FighterState::new();
            *mugen = MugenFighterState::default();
            *ib = InputBuffer::new();
            hp.current = hp.max;
            gauge.current = 0;
        }
    }

    /// Render menu overlays as quad instances.
    pub fn render(&self, screen_w: f32, screen_h: f32) -> Vec<QuadInstance> {
        match self.game_state {
            GameState::MainMenu => self.render_main_menu(screen_w, screen_h),
            GameState::RoundIntro => self.render_round_intro(screen_w, screen_h),
            GameState::Paused => self.render_pause_menu(screen_w, screen_h),
            GameState::RoundEnd => self.render_round_end(screen_w, screen_h),
            GameState::MatchEnd => self.render_match_end(screen_w, screen_h),
            GameState::Fighting => vec![],
        }
    }

    fn render_round_intro(&self, screen_w: f32, screen_h: f32) -> Vec<QuadInstance> {
        // Dark overlay during intro (text rendered separately in render_text)
        vec![QuadInstance {
            rect: [0.0, 0.0, screen_w, screen_h],
            color: [0.0, 0.0, 0.0, 0.3],
            ..Default::default()
        }]
    }

    fn render_main_menu(&self, screen_w: f32, screen_h: f32) -> Vec<QuadInstance> {
        let mut quads = Vec::with_capacity(20);

        // Full-screen dark blue-black background.
        quads.push(QuadInstance {
            rect: [0.0, 0.0, screen_w, screen_h],
            color: [0.03, 0.03, 0.06, 1.0],
            uv: [0.0, 0.0, 1.0, 1.0],
        });

        // Subtle diagonal line pattern (simulated with thin quads)
        for i in 0..20 {
            let x = i as f32 * 60.0 - 400.0;
            quads.push(QuadInstance {
                rect: [x, 0.0, 1.0, screen_h * 2.0],
                color: [1.0, 1.0, 1.0, 0.02],
                ..Default::default()
            });
        }

        // Title bar (gold gradient bar)
        let title_w = 420.0;
        let title_h = 44.0;
        let title_x = (screen_w - title_w) / 2.0;
        let title_y = screen_h * 0.17;
        // Gold border
        quads.push(QuadInstance {
            rect: [title_x - 2.0, title_y - 2.0, title_w + 4.0, title_h + 4.0],
            color: [0.78, 0.63, 0.0, 1.0],
            ..Default::default()
        });
        // Dark fill
        quads.push(QuadInstance {
            rect: [title_x, title_y, title_w, title_h],
            color: [0.78, 0.63, 0.0, 1.0],
            ..Default::default()
        });

        // Menu items.
        let item_w = 220.0;
        let item_h = 36.0;
        let start_y = screen_h * 0.42;
        let gap = 8.0;
        for (i, _item) in MAIN_MENU_ITEMS.iter().enumerate() {
            let x = (screen_w - item_w) / 2.0;
            let y = start_y + (item_h + gap) * i as f32;

            // Gold border for selected item
            if i == self.main_cursor {
                quads.push(QuadInstance {
                    rect: [x - 2.0, y - 2.0, item_w + 4.0, item_h + 4.0],
                    color: [0.78, 0.63, 0.0, 1.0],
                    ..Default::default()
                });
            }

            // Item background
            let bg_color = if i == self.main_cursor {
                [0.12, 0.12, 0.15, 0.95]
            } else {
                [0.08, 0.08, 0.10, 0.8]
            };
            quads.push(QuadInstance {
                rect: [x, y, item_w, item_h],
                color: bg_color,
            ..Default::default()
            });
        }
        quads
    }

    fn render_pause_menu(&self, screen_w: f32, screen_h: f32) -> Vec<QuadInstance> {
        let mut quads = Vec::with_capacity(8);
        // Semi-transparent overlay.
        quads.push(QuadInstance {
            rect: [0.0, 0.0, screen_w, screen_h],
            color: [0.0, 0.0, 0.0, 0.6],
            ..Default::default()
        });
        // Pause box.
        let box_w = 200.0;
        let box_h = 120.0;
        let box_x = (screen_w - box_w) / 2.0;
        let box_y = (screen_h - box_h) / 2.0;
        quads.push(QuadInstance {
            rect: [box_x, box_y, box_w, box_h],
            color: [0.15, 0.15, 0.2, 0.9],
            ..Default::default()
        });
        // Pause items.
        let item_w = 160.0;
        let item_h = 28.0;
        let start_y = box_y + 20.0;
        let gap = 10.0;
        for (i, _item) in PAUSE_MENU_ITEMS.iter().enumerate() {
            let x = (screen_w - item_w) / 2.0;
            let y = start_y + (item_h + gap) * i as f32;
            let color = if i == self.pause_cursor {
                [0.3, 0.6, 0.9, 1.0]
            } else {
                [0.25, 0.25, 0.3, 1.0]
            };
            quads.push(QuadInstance {
                rect: [x, y, item_w, item_h],
                color,
                ..Default::default()
            });
        }
        quads
    }

    fn render_round_end(&self, screen_w: f32, screen_h: f32) -> Vec<QuadInstance> {
        let mut quads = Vec::with_capacity(4);
        // Semi-transparent overlay.
        quads.push(QuadInstance {
            rect: [0.0, 0.0, screen_w, screen_h],
            color: [0.0, 0.0, 0.0, 0.4],
            ..Default::default()
        });
        // Winner banner.
        let banner_w = 300.0;
        let banner_h = 50.0;
        let x = (screen_w - banner_w) / 2.0;
        let y = (screen_h - banner_h) / 2.0;
        let color = match self.round_winner {
            Some(1) => [0.2, 0.4, 0.9, 1.0],
            Some(2) => [0.9, 0.2, 0.2, 1.0],
            _ => [0.5, 0.5, 0.5, 1.0], // draw
        };
        quads.push(QuadInstance {
            rect: [x, y, banner_w, banner_h],
            color,
            ..Default::default()
        });
        quads
    }

    fn render_match_end(&self, screen_w: f32, screen_h: f32) -> Vec<QuadInstance> {
        let mut quads = Vec::with_capacity(4);
        quads.push(QuadInstance {
            rect: [0.0, 0.0, screen_w, screen_h],
            color: [0.0, 0.0, 0.0, 0.7],
            ..Default::default()
        });
        let banner_w = 350.0;
        let banner_h = 60.0;
        let x = (screen_w - banner_w) / 2.0;
        let y = (screen_h - banner_h) / 2.0;
        let color = match self.match_winner {
            Some(1) => [0.2, 0.4, 0.9, 1.0],
            Some(2) => [0.9, 0.2, 0.2, 1.0],
            _ => [0.5, 0.5, 0.5, 1.0],
        };
        quads.push(QuadInstance {
            rect: [x, y, banner_w, banner_h],
            color,
            ..Default::default()
        });
        quads
    }

    /// Generate text areas for the current menu state.
    pub fn render_text(&self, screen_w: f32, screen_h: f32) -> Vec<TextArea> {
        match self.game_state {
            GameState::MainMenu => self.render_text_main_menu(screen_w, screen_h),
            GameState::RoundIntro => self.render_text_round_intro(screen_w, screen_h),
            GameState::Paused => self.render_text_pause_menu(screen_w, screen_h),
            GameState::RoundEnd => self.render_text_round_end(screen_w, screen_h),
            GameState::MatchEnd => self.render_text_match_end(screen_w, screen_h),
            GameState::Fighting => vec![],
        }
    }

    fn render_text_main_menu(&self, screen_w: f32, screen_h: f32) -> Vec<TextArea> {
        let mut areas = Vec::new();

        // Title bar (gold background)
        let title_bar_y = screen_h * 0.18;
        areas.push(TextArea {
            text: "TICKLE FIGHTING ENGINE".to_string(),
            x: screen_w * 0.5 - 180.0,
            y: title_bar_y + 6.0,
            size: 20.0,
            color: [0.05, 0.05, 0.08, 1.0], // dark text on gold bar
            bounds: None,
        });

        // Menu items
        let labels = ["VS MODE", "TRAINING", "QUIT"];
        let item_h = 36.0;
        let start_y = screen_h * 0.42;
        let gap = 8.0;
        let item_size = 14.0;
        for (i, label) in labels.iter().enumerate() {
            let label_w = label.len() as f32 * item_size * 0.6;
            let y = start_y + (item_h + gap) * i as f32 + (item_h - item_size) / 2.0;
            let color = if i == self.main_cursor {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.5, 0.5, 0.5, 1.0]
            };
            areas.push(TextArea {
                text: label.to_string(),
                x: (screen_w - label_w) / 2.0,
                y,
                size: item_size,
                color,
                bounds: None,
            });
        }
        areas
    }

    fn render_text_round_intro(&self, screen_w: f32, screen_h: f32) -> Vec<TextArea> {
        let mut areas = Vec::new();

        // Animation phases:
        // 0-30: "ROUND X" scaling in
        // 30-90: "ROUND X" displayed
        // 90-120: "ROUND X" fades, "FIGHT!" appears
        // 120-150: "FIGHT!" displayed
        let elapsed = ROUND_INTRO_FRAMES - self.round_intro_timer;

        let center_y = screen_h * 0.4;

        if elapsed < 90 {
            // ROUND X
            let text = format!("ROUND {}", self.current_round);
            let size = 28.0;
            let text_w = text.len() as f32 * size * 0.6;
            areas.push(TextArea {
                text,
                x: (screen_w - text_w) / 2.0,
                y: center_y,
                size,
                color: [0.91, 0.75, 0.0, 1.0], // gold
                bounds: None,
            });
        }

        if elapsed >= 90 {
            // FIGHT!
            let text = "FIGHT!";
            let size = 32.0;
            let text_w = text.len() as f32 * size * 0.6;
            areas.push(TextArea {
                text: text.to_string(),
                x: (screen_w - text_w) / 2.0,
                y: center_y,
                size,
                color: [1.0, 1.0, 1.0, 1.0],
                bounds: None,
            });
        }

        areas
    }

    fn render_text_pause_menu(&self, screen_w: f32, screen_h: f32) -> Vec<TextArea> {
        let mut areas = Vec::new();
        let labels = ["RESUME", "QUIT TO MENU"];
        let item_h = 28.0;
        let box_h = 120.0;
        let start_y = (screen_h - box_h) / 2.0 + 20.0;
        let gap = 10.0;
        let item_size = 16.0;
        for (i, label) in labels.iter().enumerate() {
            let label_w = label.len() as f32 * item_size * 0.6;
            let y = start_y + (item_h + gap) * i as f32 + (item_h - item_size) / 2.0;
            let color = if i == self.pause_cursor {
                [1.0, 1.0, 1.0, 1.0]
            } else {
                [0.8, 0.8, 0.8, 1.0]
            };
            areas.push(TextArea {
                text: label.to_string(),
                x: (screen_w - label_w) / 2.0,
                y,
                size: item_size,
                color,
                bounds: None,
            });
        }
        areas
    }

    fn render_text_round_end(&self, screen_w: f32, screen_h: f32) -> Vec<TextArea> {
        let text = match self.round_winner {
            Some(1) => "PLAYER 1 WINS!",
            Some(2) => "PLAYER 2 WINS!",
            _ => "DRAW",
        };
        let size = 28.0;
        let text_w = text.len() as f32 * size * 0.6;
        let banner_h = 50.0;
        vec![TextArea {
            text: text.to_string(),
            x: (screen_w - text_w) / 2.0,
            y: (screen_h - banner_h) / 2.0 + (banner_h - size) / 2.0,
            size,
            color: [1.0, 1.0, 1.0, 1.0],
            bounds: None,
        }]
    }

    fn render_text_match_end(&self, screen_w: f32, screen_h: f32) -> Vec<TextArea> {
        let text = match self.match_winner {
            Some(1) => "PLAYER 1 WINS THE MATCH!",
            Some(2) => "PLAYER 2 WINS THE MATCH!",
            _ => "DRAW",
        };
        let size = 26.0;
        let text_w = text.len() as f32 * size * 0.6;
        let banner_h = 60.0;
        vec![TextArea {
            text: text.to_string(),
            x: (screen_w - text_w) / 2.0,
            y: (screen_h - banner_h) / 2.0 + (banner_h - size) / 2.0,
            size,
            color: [1.0, 1.0, 1.0, 1.0],
            bounds: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_main_menu() {
        let ms = MenuSystem::new();
        assert_eq!(ms.game_state, GameState::MainMenu);
        assert!(!ms.should_run_logic());
    }

    #[test]
    fn test_main_menu_navigation() {
        let mut ms = MenuSystem::new();
        assert_eq!(ms.main_cursor, 0);
        ms.handle_input(MenuInput::Down);
        assert_eq!(ms.main_cursor, 1);
        ms.handle_input(MenuInput::Down);
        assert_eq!(ms.main_cursor, 2);
        // Can't go past end.
        ms.handle_input(MenuInput::Down);
        assert_eq!(ms.main_cursor, 2);
        ms.handle_input(MenuInput::Up);
        assert_eq!(ms.main_cursor, 1);
    }

    #[test]
    fn test_select_vs_mode() {
        let mut ms = MenuSystem::new();
        // Cursor at 0 = VsMode
        ms.handle_input(MenuInput::Confirm);
        assert_eq!(ms.game_state, GameState::RoundIntro);
        assert_eq!(ms.game_mode, GameMode::Vs);
        assert!(ms.should_run_logic());
    }

    #[test]
    fn test_select_training_mode() {
        let mut ms = MenuSystem::new();
        ms.handle_input(MenuInput::Down); // Training
        ms.handle_input(MenuInput::Confirm);
        assert_eq!(ms.game_state, GameState::RoundIntro);
        assert_eq!(ms.game_mode, GameMode::Training);
    }

    #[test]
    fn test_select_quit() {
        let mut ms = MenuSystem::new();
        ms.handle_input(MenuInput::Down);
        ms.handle_input(MenuInput::Down); // Quit
        ms.handle_input(MenuInput::Confirm);
        assert!(ms.quit_requested);
    }

    #[test]
    fn test_pause_and_resume() {
        let mut ms = MenuSystem::new();
        ms.handle_input(MenuInput::Confirm); // enter RoundIntro
        // Skip intro to fighting
        ms.round_intro_timer = 0;
        let world = World::new();
        let ui = UIRenderer::new();
        ms.update_round(&world, &ui);
        assert_eq!(ms.game_state, GameState::Fighting);
        ms.handle_input(MenuInput::Pause);
        assert_eq!(ms.game_state, GameState::Paused);
        assert!(!ms.should_run_logic());
        // Resume
        ms.handle_input(MenuInput::Confirm); // cursor at Resume
        assert_eq!(ms.game_state, GameState::Fighting);
    }

    #[test]
    fn test_pause_back_resumes() {
        let mut ms = MenuSystem::new();
        ms.handle_input(MenuInput::Confirm);
        // Skip intro to fighting
        ms.round_intro_timer = 0;
        let world = World::new();
        let ui = UIRenderer::new();
        ms.update_round(&world, &ui);
        ms.handle_input(MenuInput::Pause);
        assert_eq!(ms.game_state, GameState::Paused);
        ms.handle_input(MenuInput::Back);
        assert_eq!(ms.game_state, GameState::Fighting);
    }

    #[test]
    fn test_pause_quit_to_menu() {
        let mut ms = MenuSystem::new();
        ms.handle_input(MenuInput::Confirm); // enter RoundIntro
        // Skip intro to fighting
        ms.round_intro_timer = 0;
        let world = World::new();
        let ui = UIRenderer::new();
        ms.update_round(&world, &ui);
        ms.handle_input(MenuInput::Pause);
        ms.handle_input(MenuInput::Down); // Quit
        let needs_reset = ms.handle_input(MenuInput::Confirm);
        assert!(needs_reset);
        assert_eq!(ms.game_state, GameState::MainMenu);
        assert_eq!(ms.p1_wins, 0);
        assert_eq!(ms.p2_wins, 0);
    }

    #[test]
    fn test_round_end_ko_p1_wins() {
        let mut ms = MenuSystem::new();
        ms.game_state = GameState::Fighting;
        ms.game_mode = GameMode::Vs;

        // Simulate: P2 KO'd
        let mut world = World::new();
        world.spawn((Player1, Health::new(5000)));
        world.spawn((
            Player2,
            Health {
                current: 0,
                max: 10000,
            },
        ));
        let ui = UIRenderer::new();

        ms.update_round(&world, &ui);
        assert_eq!(ms.game_state, GameState::RoundEnd);
        assert_eq!(ms.round_winner, Some(1));
    }

    #[test]
    fn test_round_end_timer_then_next_round() {
        let mut ms = MenuSystem::new();
        ms.game_state = GameState::RoundEnd;
        ms.round_winner = Some(1);
        ms.round_end_timer = 2;

        let world = World::new();
        let ui = UIRenderer::new();

        // Tick down the freeze timer.
        assert!(!ms.update_round(&world, &ui).0); // timer 2->1
        assert!(!ms.update_round(&world, &ui).0); // timer 1->0
                                                // Timer hits 0, awards win and starts next round.
        let (reset, _) = ms.update_round(&world, &ui);
        assert!(reset);
        assert_eq!(ms.p1_wins, 1);
        assert_eq!(ms.current_round, 2);
        assert_eq!(ms.game_state, GameState::RoundIntro); // now goes through intro
    }

    #[test]
    fn test_match_winner_after_2_rounds() {
        let mut ms = MenuSystem::new();
        ms.game_state = GameState::Fighting;
        ms.game_mode = GameMode::Vs;
        ms.p1_wins = 1; // already won 1 round

        // P2 KO'd again.
        let mut world = World::new();
        world.spawn((Player1, Health::new(5000)));
        world.spawn((
            Player2,
            Health {
                current: 0,
                max: 10000,
            },
        ));
        let ui = UIRenderer::new();

        ms.update_round(&world, &ui); // enters RoundEnd
                                      // Burn through freeze timer.
        ms.round_end_timer = 0;
        ms.update_round(&world, &ui); // awards win, checks match
        assert_eq!(ms.p1_wins, 2);
        assert_eq!(ms.match_winner, Some(1));
        assert_eq!(ms.game_state, GameState::MatchEnd);
    }

    #[test]
    fn test_match_end_confirm_returns_to_menu() {
        let mut ms = MenuSystem::new();
        ms.game_state = GameState::MatchEnd;
        ms.match_winner = Some(1);
        ms.p1_wins = 2;
        ms.handle_input(MenuInput::Confirm);
        assert_eq!(ms.game_state, GameState::MainMenu);
        assert_eq!(ms.p1_wins, 0);
    }

    #[test]
    fn test_training_mode_no_ko() {
        let mut ms = MenuSystem::new();
        ms.game_state = GameState::Fighting;
        ms.game_mode = GameMode::Training;
        ms.training_infinite_hp = true;

        let mut world = World::new();
        world.spawn((
            Player1,
            Health {
                current: 0,
                max: 10000,
            },
        ));
        world.spawn((
            Player2,
            Health {
                current: 0,
                max: 10000,
            },
        ));
        let ui = UIRenderer::new();

        ms.update_round(&world, &ui);
        // Should still be fighting (infinite HP refills).
        assert_eq!(ms.game_state, GameState::Fighting);
        // Health should be refilled.
        for (_, (_, hp)) in world.query::<(&Player1, &Health)>().iter() {
            assert_eq!(hp.current, 10000);
        }
    }

    #[test]
    fn test_render_main_menu_produces_quads() {
        let ms = MenuSystem::new();
        let quads = ms.render(800.0, 600.0);
        // Background + diagonal lines (20) + title border + title + menu item borders (3) + menu items (3) = 28
        assert!(quads.len() >= 5);
    }

    #[test]
    fn test_render_fighting_produces_no_overlay() {
        let mut ms = MenuSystem::new();
        ms.game_state = GameState::Fighting;
        let quads = ms.render(800.0, 600.0);
        assert!(quads.is_empty());
    }

    #[test]
    fn test_render_pause_produces_quads() {
        let mut ms = MenuSystem::new();
        ms.game_state = GameState::Paused;
        let quads = ms.render(800.0, 600.0);
        // Overlay + box + 2 items = 4 quads.
        assert_eq!(quads.len(), 4);
    }
}
