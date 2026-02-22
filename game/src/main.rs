pub mod game_loop;
mod menu;
mod quad_renderer;
mod stage;
#[allow(dead_code)]
mod ui;

use std::sync::Arc;

use hecs::World;
use tickle_audio::{
    load_default_music, load_default_sounds, process_audio_events, AudioEvent, AudioSystem,
    HitStrength,
};
use tickle_core::systems::audio_events::{
    audio_events_from_hits, GameAudioEvent, HitSoundStrength,
};
use tickle_core::systems::collision::HitEvent;
use tickle_core::{
    Direction, Facing, Health, HitType, Hitbox, InputState, LogicRect, LogicVec2, Position,
    PowerGauge, PreviousPosition, StateMachine, StateType, Velocity, BUTTON_A,
};
use tickle_render::RenderContext;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use game_loop::GameLoop;
use menu::{GameState, MenuInput, MenuSystem};
use quad_renderer::{QuadInstance, QuadRenderer};
use stage::Stage;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const GROUND_Y: i32 = 0;
const GRAVITY: i32 = -80;
const MOVE_SPEED: i32 = 400; // 4 px/frame
const JUMP_VEL: i32 = 1800; // 18 px/frame upward
const FRICTION: i32 = 50;

/// Fighter visual size in pixels for rendering.
const FIGHTER_W: f32 = 60.0;
const FIGHTER_H: f32 = 100.0;

// ---------------------------------------------------------------------------
// Player tag components
// ---------------------------------------------------------------------------

pub(crate) struct Player1;
pub(crate) struct Player2;

// ---------------------------------------------------------------------------
// Input tracking
// ---------------------------------------------------------------------------

#[derive(Default)]
struct RawInput {
    left: bool,
    right: bool,
    up: bool,
    down: bool,
    attack: bool,
}

impl RawInput {
    fn to_input_state(&self) -> InputState {
        let x: i8 = match (self.left, self.right) {
            (true, false) => -1,
            (false, true) => 1,
            _ => 0,
        };
        let y: i8 = match (self.up, self.down) {
            (true, false) => 1,
            (false, true) => -1,
            _ => 0,
        };
        let buttons = if self.attack { BUTTON_A } else { 0 };
        InputState::new(buttons, Direction::from_xy(x, y))
    }
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

struct App {
    render_ctx: Option<RenderContext>,
    quad_renderer: Option<QuadRenderer>,
    window: Option<Arc<Window>>,
    world: World,
    game_loop: GameLoop,
    p1_input: RawInput,
    p2_input: RawInput,
    ui_renderer: ui::UIRenderer,
    audio: Option<AudioSystem>,
    stage: Stage,
    menu: MenuSystem,
    /// Buffered menu input (consumed once per frame from key-down events).
    pending_menu_input: MenuInput,
    /// Whether background music has been started for the current round.
    music_started: bool,
    /// Previous frame state types for detecting state changes (P1, P2).
    prev_states: (StateType, StateType),
}

impl App {
    fn new() -> Self {
        let mut world = World::new();
        spawn_fighters(&mut world);

        // Initialize audio (non-fatal if it fails -- game runs without sound)
        let audio = match AudioSystem::new("./assets") {
            Ok(mut sys) => {
                if let Err(e) = load_default_sounds(&mut sys) {
                    log::warn!("Failed to load default sounds: {}", e);
                }
                if let Err(e) = load_default_music(&mut sys) {
                    log::warn!("Failed to load default music: {}", e);
                }
                log::info!("Audio system ready");
                Some(sys)
            }
            Err(e) => {
                log::warn!("Audio system unavailable: {}", e);
                None
            }
        };

        // Load stage (try from file, fall back to built-in default).
        let stage = match Stage::load_from_file("./assets/stages/dojo.ron") {
            Ok(s) => {
                log::info!("Loaded stage: {}", s.data.name);
                s
            }
            Err(e) => {
                log::warn!("Failed to load stage file: {}; using default dojo", e);
                Stage::default_dojo()
            }
        };

        Self {
            render_ctx: None,
            quad_renderer: None,
            window: None,
            world,
            game_loop: GameLoop::new(),
            p1_input: RawInput::default(),
            p2_input: RawInput::default(),
            ui_renderer: ui::UIRenderer::new(),
            audio,
            stage,
            menu: MenuSystem::new(),
            pending_menu_input: MenuInput::None,
            music_started: false,
            prev_states: (StateType::Idle, StateType::Idle),
        }
    }
}

fn spawn_fighters(world: &mut World) {
    // Player 1 -- left side (blue)
    world.spawn((
        Player1,
        Position {
            pos: LogicVec2::from_pixels(200, 0),
        },
        PreviousPosition {
            pos: LogicVec2::from_pixels(200, 0),
        },
        Velocity {
            vel: LogicVec2::ZERO,
        },
        Facing { dir: Facing::RIGHT },
        StateMachine::new(),
        Health::new(10_000),
        PowerGauge::new(),
        FighterColor([0.2, 0.4, 0.9, 1.0]),
    ));
    // Player 2 -- right side (red)
    world.spawn((
        Player2,
        Position {
            pos: LogicVec2::from_pixels(600, 0),
        },
        PreviousPosition {
            pos: LogicVec2::from_pixels(600, 0),
        },
        Velocity {
            vel: LogicVec2::ZERO,
        },
        Facing { dir: Facing::LEFT },
        StateMachine::new(),
        Health::new(10_000),
        PowerGauge::new(),
        FighterColor([0.9, 0.2, 0.2, 1.0]),
    ));
}

// ---------------------------------------------------------------------------
// ApplicationHandler
// ---------------------------------------------------------------------------

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("Tickle Fighting Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());

        let ctx = pollster::block_on(RenderContext::new(window)).unwrap();
        let qr = QuadRenderer::new(&ctx.device, ctx.surface_format());
        self.quad_renderer = Some(qr);
        self.render_ctx = Some(ctx);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(new_size) => {
                if let Some(ctx) = self.render_ctx.as_mut() {
                    ctx.resize(new_size);
                }
            }

            WindowEvent::KeyboardInput { event, .. } => {
                let pressed = event.state == ElementState::Pressed;
                if let PhysicalKey::Code(key) = event.physical_key {
                    // Menu input (only on key-down, not repeat).
                    if pressed && !event.repeat {
                        let mi = key_to_menu_input(key, &self.menu);
                        if mi != MenuInput::None {
                            self.pending_menu_input = mi;
                        }
                    }
                    // Fighter input only during gameplay.
                    if self.menu.should_run_logic() {
                        handle_key(&mut self.p1_input, &mut self.p2_input, key, pressed);
                    }
                }
            }

            WindowEvent::RedrawRequested => {
                // Handle quit request from menu.
                if self.menu.quit_requested {
                    event_loop.exit();
                    return;
                }
                self.tick_and_render();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(win) = self.window.as_ref() {
            win.request_redraw();
        }
    }
}

// ---------------------------------------------------------------------------
// Input mapping
// ---------------------------------------------------------------------------

fn handle_key(p1: &mut RawInput, p2: &mut RawInput, key: KeyCode, pressed: bool) {
    // Player 1: WASD + Space
    match key {
        KeyCode::KeyA => p1.left = pressed,
        KeyCode::KeyD => p1.right = pressed,
        KeyCode::KeyW => p1.up = pressed,
        KeyCode::KeyS => p1.down = pressed,
        KeyCode::Space => p1.attack = pressed,
        // Player 2: Arrow keys + Enter
        KeyCode::ArrowLeft => p2.left = pressed,
        KeyCode::ArrowRight => p2.right = pressed,
        KeyCode::ArrowUp => p2.up = pressed,
        KeyCode::ArrowDown => p2.down = pressed,
        KeyCode::Enter => p2.attack = pressed,
        _ => {}
    }
}

/// Map a key press to a menu input based on current game state.
fn key_to_menu_input(key: KeyCode, menu: &MenuSystem) -> MenuInput {
    match menu.game_state {
        GameState::MainMenu => match key {
            KeyCode::KeyW | KeyCode::ArrowUp => MenuInput::Up,
            KeyCode::KeyS | KeyCode::ArrowDown => MenuInput::Down,
            KeyCode::Enter | KeyCode::Space => MenuInput::Confirm,
            KeyCode::Escape => MenuInput::Back,
            _ => MenuInput::None,
        },
        GameState::Fighting => match key {
            KeyCode::Escape => MenuInput::Pause,
            _ => MenuInput::None,
        },
        GameState::Paused => match key {
            KeyCode::KeyW | KeyCode::ArrowUp => MenuInput::Up,
            KeyCode::KeyS | KeyCode::ArrowDown => MenuInput::Down,
            KeyCode::Enter | KeyCode::Space => MenuInput::Confirm,
            KeyCode::Escape => MenuInput::Back,
            _ => MenuInput::None,
        },
        GameState::RoundEnd => MenuInput::None,
        GameState::MatchEnd => match key {
            KeyCode::Enter | KeyCode::Space => MenuInput::Confirm,
            _ => MenuInput::None,
        },
    }
}

// ---------------------------------------------------------------------------
// Game logic (runs at fixed 60 FPS)
// ---------------------------------------------------------------------------

fn logic_update(
    world: &mut World,
    p1_input: &InputState,
    p2_input: &InputState,
    stage: &stage::Stage,
) -> Vec<HitEvent> {
    // Save previous positions for interpolation.
    for (_, (pos, prev)) in world.query_mut::<(&Position, &mut PreviousPosition)>() {
        prev.pos = pos.pos;
    }

    // Apply input to Player 1.
    for (_, (_, vel, sm)) in world.query_mut::<(&Player1, &mut Velocity, &mut StateMachine)>() {
        apply_input(vel, sm, p1_input);
    }
    // Apply input to Player 2.
    for (_, (_, vel, sm)) in world.query_mut::<(&Player2, &mut Velocity, &mut StateMachine)>() {
        apply_input(vel, sm, p2_input);
    }

    // Physics: gravity + velocity integration + ground detection.
    for (_, (pos, vel, sm)) in
        world.query_mut::<(&mut Position, &mut Velocity, &mut StateMachine)>()
    {
        // Gravity (only when airborne).
        if pos.pos.y > GROUND_Y || vel.vel.y > 0 {
            vel.vel.y += GRAVITY;
        }
        // Ground friction.
        if pos.pos.y <= GROUND_Y && sm.current_state() != StateType::Jump {
            if vel.vel.x > 0 {
                vel.vel.x = (vel.vel.x - FRICTION).max(0);
            } else if vel.vel.x < 0 {
                vel.vel.x = (vel.vel.x + FRICTION).min(0);
            }
        }
        // Integrate velocity.
        pos.pos.x += vel.vel.x;
        pos.pos.y += vel.vel.y;
        // Ground clamp.
        if pos.pos.y < GROUND_Y {
            pos.pos.y = GROUND_Y;
            vel.vel.y = 0;
            sm.land();
        }
        // Stage bounds.
        pos.pos.x = stage.clamp_x(pos.pos.x);
        // Advance state frame and handle auto-transitions.
        sm.update();
    }

    // Simple proximity-based hit detection for attack states.
    let mut p1_data: Option<(Position, StateMachine)> = None;
    let mut p2_data: Option<(Position, StateMachine)> = None;
    for (_, (_, pos, sm)) in world.query::<(&Player1, &Position, &StateMachine)>().iter() {
        p1_data = Some((*pos, sm.clone()));
    }
    for (_, (_, pos, sm)) in world.query::<(&Player2, &Position, &StateMachine)>().iter() {
        p2_data = Some((*pos, sm.clone()));
    }

    let mut hit_events = Vec::new();
    if let (Some((p1_pos, p1_sm)), Some((p2_pos, p2_sm))) = (p1_data, p2_data) {
        let distance = (p1_pos.pos.x - p2_pos.pos.x).abs();
        let hit_range = 8000; // ~80 pixels in logic coords

        // P1 attacking P2: generate hit on the first active frame.
        if let StateType::Attack(id) = p1_sm.current_state() {
            if p1_sm.state.state_frame == 1 && distance < hit_range {
                let damage = 500 + (id as i32) * 200;
                hit_events.push(HitEvent {
                    attacker: 0,
                    defender: 1,
                    hitbox: Hitbox {
                        rect: LogicRect::new(0, 0, 100, 100),
                        damage,
                        hitstun: 15,
                        blockstun: 8,
                        knockback: LogicVec2::new(300, 0),
                        hit_type: HitType::Mid,
                    },
                });
            }
        }

        // P2 attacking P1.
        if let StateType::Attack(id) = p2_sm.current_state() {
            if p2_sm.state.state_frame == 1 && distance < hit_range {
                let damage = 500 + (id as i32) * 200;
                hit_events.push(HitEvent {
                    attacker: 1,
                    defender: 0,
                    hitbox: Hitbox {
                        rect: LogicRect::new(0, 0, 100, 100),
                        damage,
                        hitstun: 15,
                        blockstun: 8,
                        knockback: LogicVec2::new(-300, 0),
                        hit_type: HitType::Mid,
                    },
                });
            }
        }

        // Apply hit damage to defenders.
        for hit in &hit_events {
            if hit.defender == 1 {
                for (_, (_, hp)) in world.query_mut::<(&Player2, &mut Health)>() {
                    hp.take_damage(hit.hitbox.damage);
                }
            } else {
                for (_, (_, hp)) in world.query_mut::<(&Player1, &mut Health)>() {
                    hp.take_damage(hit.hitbox.damage);
                }
            }
        }
    }

    hit_events
}

fn apply_input(vel: &mut Velocity, sm: &mut StateMachine, input: &InputState) {
    let was_jumping = sm.current_state() == StateType::Jump;
    // Let the state machine decide the transition.
    sm.try_transition(input);

    match sm.current_state() {
        StateType::WalkForward => vel.vel.x = MOVE_SPEED,
        StateType::WalkBackward => vel.vel.x = -MOVE_SPEED,
        StateType::Jump if !was_jumping => {
            // Just entered jump this frame
            vel.vel.y = JUMP_VEL;
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Tick + Render
// ---------------------------------------------------------------------------

impl App {
    fn tick_and_render(&mut self) {
        let ctx = match self.render_ctx.as_ref() {
            Some(c) => c,
            None => return,
        };
        let qr = match self.quad_renderer.as_ref() {
            Some(q) => q,
            None => return,
        };

        // Process buffered menu input.
        let mi = std::mem::replace(&mut self.pending_menu_input, MenuInput::None);
        if mi != MenuInput::None {
            let needs_reset = self.menu.handle_input(mi);
            if needs_reset {
                MenuSystem::reset_fighters(&mut self.world);
                self.ui_renderer.reset();
                self.p1_input = RawInput::default();
                self.p2_input = RawInput::default();
                self.music_started = false;
            }
        }

        let p1_input = self.p1_input.to_input_state();
        let p2_input = self.p2_input.to_input_state();
        let world = &mut self.world;
        let ui = &mut self.ui_renderer;
        let stage = &self.stage;
        let menu = &mut self.menu;
        let prev_states = &mut self.prev_states;

        let result = self.game_loop.tick(|| {
            if menu.should_run_logic() {
                // Capture pre-update states for change detection.
                let mut p1_pre = StateType::Idle;
                let mut p2_pre = StateType::Idle;
                for (_, (_, sm)) in world.query::<(&Player1, &StateMachine)>().iter() {
                    p1_pre = sm.current_state();
                }
                for (_, (_, sm)) in world.query::<(&Player2, &StateMachine)>().iter() {
                    p2_pre = sm.current_state();
                }

                let hit_events = logic_update(world, &p1_input, &p2_input, stage);

                // Register hits for combo counter.
                for hit in &hit_events {
                    ui.register_hit(hit.attacker == 0);
                }

                ui.update(world);

                // Capture post-update states.
                let mut p1_post = StateType::Idle;
                let mut p2_post = StateType::Idle;
                for (_, (_, sm)) in world.query::<(&Player1, &StateMachine)>().iter() {
                    p1_post = sm.current_state();
                }
                for (_, (_, sm)) in world.query::<(&Player2, &StateMachine)>().iter() {
                    p2_post = sm.current_state();
                }

                let state_changes = (
                    if p1_pre != p1_post {
                        Some(p1_post)
                    } else {
                        None
                    },
                    if p2_pre != p2_post {
                        Some(p2_post)
                    } else {
                        None
                    },
                );
                *prev_states = (p1_post, p2_post);

                let new_round = menu.update_round(world, ui);
                if new_round {
                    MenuSystem::reset_fighters(world);
                    ui.reset();
                }
                (hit_events, state_changes, new_round)
            } else {
                (Vec::new(), (None, None), false)
            }
        });

        // Process audio events from all logic updates.
        if let Some(audio) = self.audio.as_mut() {
            // Start background music when entering fighting state.
            if self.menu.game_state == GameState::Fighting && !self.music_started {
                let _ = audio.play_music("stage_theme", true);
                self.music_started = true;
            }
            // Stop music when leaving fighting state (except pause).
            if self.menu.game_state != GameState::Fighting
                && self.menu.game_state != GameState::Paused
                && self.music_started
            {
                audio.stop_music();
                self.music_started = false;
            }

            for (hit_events, state_changes, _new_round) in &result.results {
                // Generate and play audio events from hits.
                let game_audio = audio_events_from_hits(hit_events);
                let hit_audio: Vec<AudioEvent> = game_audio
                    .iter()
                    .map(|ge| match ge {
                        GameAudioEvent::HitSound { strength } => {
                            let hs = match strength {
                                HitSoundStrength::Light => HitStrength::Light,
                                HitSoundStrength::Medium => HitStrength::Medium,
                                HitSoundStrength::Heavy => HitStrength::Heavy,
                            };
                            AudioEvent::PlayHitSound { strength: hs }
                        }
                        _ => AudioEvent::StopMusic, // unreachable for hit-derived events
                    })
                    .collect();
                process_audio_events(audio, &hit_audio);

                // Play sounds for state changes (hitstun/blockstun).
                let mut state_audio = Vec::new();
                if let Some(new_state) = state_changes.0 {
                    if matches!(new_state, StateType::Hitstun | StateType::Blockstun) {
                        state_audio.push(AudioEvent::PlayActionSound {
                            id: "hit_light".to_string(),
                        });
                    }
                }
                if let Some(new_state) = state_changes.1 {
                    if matches!(new_state, StateType::Hitstun | StateType::Blockstun) {
                        state_audio.push(AudioEvent::PlayActionSound {
                            id: "hit_light".to_string(),
                        });
                    }
                }
                if !state_audio.is_empty() {
                    process_audio_events(audio, &state_audio);
                }
            }
        }

        // Update FPS in window title.
        if let Some(fps) = self.game_loop.frame_counter_mut().tick() {
            if let Some(win) = self.window.as_ref() {
                win.set_title(&format!(
                    "Tickle Fighting Engine | FPS: {} | Logic updates: {}",
                    fps, result.logic_updates
                ));
            }
        }

        // Build render instances with interpolation.
        let alpha = result.alpha;
        let screen_w = ctx.size.width as f32;
        let screen_h = ctx.size.height as f32;
        let mut instances = Vec::new();

        let ground_screen_y = screen_h - 100.0; // ground at 100px from bottom

        // Update camera to track fighters.
        {
            let mut p1_x = 0.0_f32;
            let mut p2_x = 0.0_f32;
            for (_, (_, pos)) in self.world.query::<(&Player1, &Position)>().iter() {
                p1_x = pos.pos.x as f32 / 100.0;
            }
            for (_, (_, pos)) in self.world.query::<(&Player2, &Position)>().iter() {
                p2_x = pos.pos.x as f32 / 100.0;
            }
            self.stage.update_camera(p1_x, p2_x, screen_w);
        }
        let camera_x = self.stage.camera_x;

        // Stage background layers (rendered behind everything).
        instances.extend(
            self.stage
                .render_layers(screen_w, screen_h, ground_screen_y),
        );

        // Ground line (scrolls with camera).
        instances.push(QuadInstance {
            rect: [-camera_x, ground_screen_y, screen_w + camera_x * 2.0, 4.0],
            color: [0.3, 0.3, 0.3, 1.0],
        });

        // Fighters.
        for (_, (pos, prev, _facing, _hp, color)) in self
            .world
            .query::<(
                &Position,
                &PreviousPosition,
                &Facing,
                &Health,
                &FighterColor,
            )>()
            .iter()
        {
            let prev_render = prev.pos.to_render();
            let cur_render = pos.pos.to_render();
            let x = prev_render[0] + (cur_render[0] - prev_render[0]) * alpha;
            let y = prev_render[1] + (cur_render[1] - prev_render[1]) * alpha;

            // Convert logic coords to screen coords, applying camera offset.
            let screen_x = x - FIGHTER_W / 2.0 - camera_x;
            let screen_y = ground_screen_y - y - FIGHTER_H;

            instances.push(QuadInstance {
                rect: [screen_x, screen_y, FIGHTER_W, FIGHTER_H],
                color: color.0,
            });
        }

        // UI overlay (health bars, gauges, timer, combo).
        instances.extend(self.ui_renderer.render(&self.world, screen_w));

        // Menu overlay (main menu, pause, round/match end).
        instances.extend(self.menu.render(screen_w, screen_h));

        // Render.
        let output = match ctx.surface.get_current_texture() {
            Ok(t) => t,
            Err(_) => return,
        };
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("frame_encoder"),
            });

        qr.draw(
            &mut encoder,
            &view,
            &ctx.queue,
            ctx.size.width as f32,
            ctx.size.height as f32,
            &instances,
        );

        ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }
}

// ---------------------------------------------------------------------------
// Fighter color tag (used to distinguish P1 / P2 visually)
// ---------------------------------------------------------------------------

struct FighterColor([f32; 4]);

// ---------------------------------------------------------------------------
// Network mode
// ---------------------------------------------------------------------------

/// Game network mode, selected at startup.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkMode {
    /// Local versus (two players, one machine).
    Local,
    /// Online via GGRS rollback networking.
    Online,
}

impl NetworkMode {
    /// Parse from CLI args. Defaults to Local.
    pub fn from_args() -> Self {
        let args: Vec<String> = std::env::args().collect();
        if args.iter().any(|a| a == "--online") {
            Self::Online
        } else {
            Self::Local
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    env_logger::init();
    let network_mode = NetworkMode::from_args();
    log::info!("Starting Tickle Fighting Engine in {:?} mode", network_mode);
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
