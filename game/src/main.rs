pub mod game_loop;
mod menu;
mod quad_renderer;
mod stage;
pub mod text_renderer;
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
    Direction, Facing, FighterState, Health, HitType, Hitbox, InputBuffer, InputState,
    LogicRect, LogicVec2, Position, PowerGauge, PreviousPosition, Velocity, BUTTON_A,
    is_hit_state, is_guard_state,
};
use tickle_render::{RenderContext, Texture};
use tickle_mugen::{
    Air, CharacterDef, Cmd, CmdParser, Cns, CnsParser, MugenCollisionFighter,
    MugenCommandRecognizer, MugenFighterState, SffV1, SpriteAtlas, StateDef, StateController,
    merge_statedefs, mugen_combat_frame, mugen_tick_with_p2, reset_combo_if_recovered,
};
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use game_loop::GameLoop;
use menu::{GameState, MenuInput, MenuSystem};
use quad_renderer::{QuadInstance, QuadRenderer};
use stage::Stage;
use text_renderer::{TextArea, TextRenderer};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

const GROUND_Y: i32 = 0;
const DEFAULT_GRAVITY: i32 = -80;

/// Fighter visual size in pixels for rendering.
const FIGHTER_W: f32 = 100.0;
const FIGHTER_H: f32 = 109.0;

/// MUGEN character data loaded from .def file at startup.
struct CharacterData {
    cns: Cns,
    /// Parsed command list for input recognition
    cmd: Option<Cmd>,
    // Physics derived from CNS
    move_speed: i32,
    #[allow(dead_code)]
    jump_vel_x: i32,
    jump_vel_y: i32,
    gravity: i32,
    friction: i32,
}

impl CharacterData {
    fn from_cns(
        mut cns: Cns,
        common_statedefs: std::collections::HashMap<i32, StateDef>,
        common_global_controllers: Vec<StateController>,
        cmd: Option<Cmd>,
    ) -> Self {
        let move_speed = (cns.velocity.walk_fwd.x * 100.0) as i32;
        let jump_vel_x = 0;
        let jump_vel_y = (cns.velocity.jump_neu.y.abs() * 100.0) as i32;
        let gravity = -(cns.movement.yaccel * 100.0) as i32;
        let friction = (cns.movement.stand_friction * 100.0) as i32;
        // Merge common statedefs into cns.statedefs so mugen_tick_with_p2 can find all states.
        let char_states = std::mem::take(&mut cns.statedefs);
        cns.statedefs = merge_statedefs(common_statedefs, char_states);
        // Inject fallback global controllers if the character has none of its own.
        if cns.global_state_controllers.is_empty() {
            cns.global_state_controllers = common_global_controllers;
        }
        Self {
            cns,
            cmd,
            move_speed,
            jump_vel_x,
            jump_vel_y,
            gravity,
            friction,
        }
    }

    #[allow(dead_code)]
    fn anim_for_state(&self, state_num: i32) -> Option<i32> {
        self.cns.statedefs.get(&state_num).and_then(|sd| sd.anim)
    }
}

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
// Owned fighter data snapshot for cross-fighter logic
// ---------------------------------------------------------------------------

struct FighterData {
    fs: FighterState,
    mugen: MugenFighterState,
    pos: Position,
    vel: Velocity,
    hp: Health,
    power: PowerGauge,
    facing: Facing,
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

struct App {
    render_ctx: Option<RenderContext>,
    quad_renderer: Option<QuadRenderer>,
    text_renderer: Option<TextRenderer>,
    window: Option<Arc<Window>>,
    world: World,
    game_loop: GameLoop,
    p1_input: RawInput,
    p2_input: RawInput,
    ui_renderer: ui::UIRenderer,
    audio: Option<AudioSystem>,
    stage: Stage,
    menu: MenuSystem,
    pending_menu_input: MenuInput,
    music_started: bool,
    // Entity IDs for direct write-back after MUGEN tick
    p1_entity: hecs::Entity,
    p2_entity: hecs::Entity,
    // Fighter sprites — MUGEN character
    fighter_texture: Option<Texture>,
    use_sprites: bool,
    kfm_atlas: Option<SpriteAtlas>,
    kfm_air: Option<Air>,
    char_data: Option<CharacterData>,
}

impl App {
    fn new() -> Self {
        let mut world = World::new();
        let (p1_entity, p2_entity) = spawn_fighters(&mut world);

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

        // Load MUGEN character from .def file
        let char_data = Self::load_character_data("./assets/mugen/kyo");
        if let Some(ref cd) = char_data {
            log::info!(
                "Character CNS loaded: life={}, walk_speed={}, gravity={}",
                cd.cns.data.life, cd.move_speed, cd.gravity
            );
        }

        Self {
            render_ctx: None,
            quad_renderer: None,
            text_renderer: None,
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
            p1_entity,
            p2_entity,
            fighter_texture: None,
            use_sprites: false,
            kfm_atlas: None,
            kfm_air: None,
            char_data,
        }
    }
}

impl App {
    fn load_character_data(char_dir: &str) -> Option<CharacterData> {
        // Try to find a .def file in the character directory
        let def_path = std::fs::read_dir(char_dir)
            .ok()?
            .filter_map(|e| e.ok())
            .find(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext.eq_ignore_ascii_case("def"))
                    .unwrap_or(false)
            })?
            .path();

        log::info!("Loading character from: {}", def_path.display());
        let char_def = match CharacterDef::parse(&def_path) {
            Ok(d) => d,
            Err(e) => {
                log::warn!("Failed to parse .def file: {}", e);
                return None;
            }
        };

        log::info!(
            "Character: {} (by {})",
            char_def.info.displayname,
            char_def.info.author.as_deref().unwrap_or("unknown")
        );

        let cns_path = format!("{}/{}", char_dir, char_def.files.cns);
        let cns = match CnsParser::parse(&cns_path) {
            Ok(c) => c,
            Err(e) => {
                log::warn!("Failed to parse CNS file: {}", e);
                return None;
            }
        };

        // Load CMD file for command/motion input recognition.
        let cmd_path = format!("{}/{}", char_dir, char_def.files.cmd);
        let cmd = match CmdParser::parse(&cmd_path) {
            Ok(c) => {
                log::info!("CMD loaded: {} commands", c.commands.len());
                Some(c)
            }
            Err(e) => {
                log::warn!("Failed to parse CMD file ({}): {}", cmd_path, e);
                None
            }
        };

        // Load common1.cns, falling back to built-in minimal states using character velocities.
        let (common_statedefs, common_global_ctrls) =
            CnsParser::parse("assets/mugen/common1.cns")
                .map(|c| (c.statedefs, c.global_state_controllers))
                .unwrap_or_else(|_| {
                    log::warn!("common1.cns not found; generating fallback common states from character data");
                    Self::make_fallback_common_statedefs(&cns)
                });

        Some(CharacterData::from_cns(cns, common_statedefs, common_global_ctrls, cmd))
    }

    /// Generate minimal common statedefs and global controllers using the character's own velocity values.
    /// Used as a fallback when common1.cns is not available.
    /// Returns `(statedefs, global_controllers)`.
    fn make_fallback_common_statedefs(
        cns: &Cns,
    ) -> (std::collections::HashMap<i32, StateDef>, Vec<StateController>) {
        let walk_fwd  = cns.velocity.walk_fwd.x;
        let walk_back = cns.velocity.walk_back.x;
        let jump_y    = cns.velocity.jump_neu.y;
        let jump_fwd_x  = cns.velocity.jump_fwd.x;
        let jump_back_x = cns.velocity.jump_back.x;

        // In MUGEN coords: -y=up, jump_y is negative (upward).
        // Statedef -1: global transitions (walk, jump).
        // State 50: air stand (used by Kyo's state 40 → 50 transition).
        // State 52: landing.
        let content = format!(
r#"[Statedef -1]

[State -1, Walk Fwd]
type = ChangeState
value = 20
triggerall = ctrl
triggerall = statetype = S
trigger1 = command = "holdfwd"

[State -1, Walk Back]
type = ChangeState
value = 21
triggerall = ctrl
triggerall = statetype = S
trigger1 = command = "holdback"

[State -1, Jump Neutral]
type = ChangeState
value = 40
triggerall = ctrl
triggerall = statetype = S
triggerall = command != "holdfwd"
triggerall = command != "holdback"
trigger1 = command = "holdup"

[State -1, Jump Forward]
type = ChangeState
value = 40
triggerall = ctrl
triggerall = statetype = S
trigger1 = command = "holdfwd"
trigger1 = command = "holdup"

[State -1, Jump Back]
type = ChangeState
value = 40
triggerall = ctrl
triggerall = statetype = S
trigger1 = command = "holdback"
trigger1 = command = "holdup"

[Statedef 0]
type = S
physics = S
anim = 0
ctrl = 1

[State 0, Vel Reset]
type = VelSet
trigger1 = 1
x = 0
y = 0

[Statedef 20]
type = S
physics = S
anim = 20
ctrl = 1
velset = {walk_fwd}, 0

[State 20, Keep Walking Fwd]
type = VelSet
trigger1 = command = "holdfwd"
x = {walk_fwd}
y = 0

[State 20, Stop]
type = ChangeState
value = 0
trigger1 = command != "holdfwd"

[Statedef 21]
type = S
physics = S
anim = 21
ctrl = 1
velset = {walk_back}, 0

[State 21, Keep Walking Back]
type = VelSet
trigger1 = command = "holdback"
x = {walk_back}
y = 0

[State 21, Stop]
type = ChangeState
value = 0
trigger1 = command != "holdback"

[Statedef 40]
type = A
physics = A
anim = 40
ctrl = 0
velset = 0, {jump_y}

[State 40, Land]
type = ChangeState
value = 0
triggerall = pos y >= 0
trigger1 = vel y >= 0

[Statedef 41]
type = A
physics = A
anim = 41
ctrl = 0
velset = {jump_fwd_x}, {jump_y}

[State 41, Land]
type = ChangeState
value = 0
triggerall = pos y >= 0
trigger1 = vel y >= 0

[Statedef 42]
type = A
physics = A
anim = 42
ctrl = 0
velset = {jump_back_x}, {jump_y}

[State 42, Land]
type = ChangeState
value = 0
triggerall = pos y >= 0
trigger1 = vel y >= 0

[Statedef 50]
type = A
physics = A
anim = 50
ctrl = 1

[State 50, Land]
type = ChangeState
value = 52
triggerall = pos y >= 0
trigger1 = vel y >= 0

[Statedef 52]
type = S
physics = S
anim = 52
ctrl = 0

[State 52, Return to Stand]
type = ChangeState
value = 0
trigger1 = animtime = 0

[Statedef 5000]
type = L
physics = N
anim = 5000
ctrl = 0
"#,
            walk_fwd = walk_fwd,
            walk_back = walk_back,
            jump_y = jump_y,
            jump_fwd_x = jump_fwd_x,
            jump_back_x = jump_back_x,
        );

        CnsParser::parse_statedefs_only(&content)
    }
}

/// Spawn both fighters and return their entity IDs.
fn spawn_fighters(world: &mut World) -> (hecs::Entity, hecs::Entity) {
    // Player 1 -- left side (blue)
    let p1 = world.spawn((
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
        FighterState::new(),
        MugenFighterState::default(),
        InputBuffer::new(),
        Health::new(10_000),
        PowerGauge::new(),
        FighterColor([0.2, 0.4, 0.9, 1.0]),
    ));
    // Player 2 -- right side (red)
    let p2 = world.spawn((
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
        FighterState::new(),
        MugenFighterState::default(),
        InputBuffer::new(),
        Health::new(10_000),
        PowerGauge::new(),
        FighterColor([0.9, 0.2, 0.2, 1.0]),
    ));
    (p1, p2)
}

// ---------------------------------------------------------------------------
// ApplicationHandler
// ---------------------------------------------------------------------------

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        log::info!("resumed() called");
        if self.window.is_some() {
            log::info!("Window already exists, skipping initialization");
            return;
        }
        log::info!("Creating window...");
        let attrs = Window::default_attributes()
            .with_title("Tickle Fighting Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        log::info!("Window created successfully");
        self.window = Some(window.clone());

        log::info!("Initializing render context...");
        let ctx = pollster::block_on(RenderContext::new(window)).unwrap();
        log::info!("Render context initialized");

        log::info!("Creating quad renderer...");
        let qr = QuadRenderer::new(&ctx.device, ctx.surface_format());
        self.quad_renderer = Some(qr);
        log::info!("Quad renderer created");

        log::info!("Creating text renderer...");
        let tr = TextRenderer::new(&ctx.device, &ctx.queue, ctx.surface_format());
        self.text_renderer = Some(tr);
        log::info!("Text renderer created");

        // Load MUGEN character sprites and animations
        log::info!("Loading MUGEN character sprites...");
        let kyo_base = "./assets/mugen/kyo";
        let sff_file = format!("{}/kyo.sff", kyo_base);
        match SffV1::load(&sff_file) {
            Ok(sff) => {
                log::info!("Kyo SFF loaded: {} sprites", sff.sprite_count());
                let atlas = SpriteAtlas::build(&sff);
                log::info!("Kyo atlas built: {}x{}", atlas.width, atlas.height);

                // Upload atlas as GPU texture (raw RGBA bytes)
                match Texture::from_rgba(
                    &ctx.device, &ctx.queue,
                    &atlas.rgba, atlas.width, atlas.height, "kyo_atlas",
                ) {
                    Ok(texture) => {
                        log::info!("Kyo atlas uploaded to GPU");

                        // Export atlas for debugging
                        let debug_path = "./debug_kyo_atlas.png";
                        if let Err(e) = image::save_buffer(
                            debug_path,
                            &atlas.rgba,
                            atlas.width,
                            atlas.height,
                            image::ColorType::Rgba8,
                        ) {
                            log::error!("Failed to save debug atlas: {}", e);
                        } else {
                            log::info!("Debug atlas saved to: {}", debug_path);
                        }

                        self.fighter_texture = Some(texture);
                        self.kfm_atlas = Some(atlas);
                        self.use_sprites = true;
                    }
                    Err(e) => log::warn!("Failed to upload Kyo atlas: {}", e),
                }
            }
            Err(e) => log::warn!("Failed to load kyo.sff: {}", e),
        }
        match Air::load(format!("{}/kyo.air", kyo_base)) {
            Ok(air) => {
                log::info!("Kyo AIR loaded: {} actions", air.action_count());
                self.kfm_air = Some(air);
            }
            Err(e) => log::warn!("Failed to load kyo.air: {}", e),
        }
        log::info!("use_sprites = {}", self.use_sprites);

        self.render_ctx = Some(ctx);
        log::info!("All renderers initialized successfully");
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
        GameState::RoundIntro => MenuInput::None,
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

/// Advance MugenFighterState::anim_elem to the correct AIR frame based on elapsed anim_time.
/// Handles looping: if all frames have finite duration, wraps anim_time via modulo.
fn advance_anim_elem(mugen: &mut MugenFighterState, air: &Air) {
    let action = match air
        .get_action(mugen.anim_num as u32)
        .or_else(|| air.get_action(0))
    {
        Some(a) => a,
        None => return,
    };
    if action.frames.is_empty() {
        return;
    }

    // Compute total finite duration (stop at first frame with duration < 0 = infinite hold).
    let mut total_duration = 0i32;
    let mut loops = true;
    for frame in &action.frames {
        if frame.duration < 0 {
            loops = false;
            break;
        }
        total_duration = total_duration.saturating_add(frame.duration);
    }

    // Effective time: loop if all frames are finite; otherwise advance monotonically.
    let effective_time = if loops && total_duration > 0 {
        mugen.anim_time % total_duration
    } else {
        mugen.anim_time
    };

    let mut cumulative = 0i32;
    for (i, frame) in action.frames.iter().enumerate() {
        let dur = if frame.duration < 0 { i32::MAX / 2 } else { frame.duration };
        if effective_time < cumulative + dur {
            mugen.anim_elem = (i + 1) as i32; // anim_elem is 1-based
            return;
        }
        cumulative = cumulative.saturating_add(dur);
    }
    // Past all frames: hold on last frame (only reachable for non-looping anims)
    mugen.anim_elem = action.frames.len() as i32;
}

fn logic_update(
    world: &mut World,
    p1_input: &InputState,
    p2_input: &InputState,
    stage: &stage::Stage,
    char_data: Option<&CharacterData>,
    kfm_air: Option<&Air>,
) -> Vec<HitEvent> {
    // Save previous positions for render interpolation.
    for (_, (pos, prev)) in world.query_mut::<(&Position, &mut PreviousPosition)>() {
        prev.pos = pos.pos;
    }

    // Push raw input into InputBuffers and run command recognition.
    for (_, (_, ib, mugen, facing)) in
        world.query_mut::<(&Player1, &mut InputBuffer, &mut MugenFighterState, &Facing)>()
    {
        ib.push(*p1_input);
        if let Some(cd) = char_data {
            if let Some(cmd) = &cd.cmd {
                mugen.active_commands =
                    MugenCommandRecognizer::recognize(&cmd.commands, ib, facing.dir == Facing::RIGHT);
            }
        }
    }
    for (_, (_, ib, mugen, facing)) in
        world.query_mut::<(&Player2, &mut InputBuffer, &mut MugenFighterState, &Facing)>()
    {
        ib.push(*p2_input);
        if let Some(cd) = char_data {
            if let Some(cmd) = &cd.cmd {
                mugen.active_commands =
                    MugenCommandRecognizer::recognize(&cmd.commands, ib, facing.dir == Facing::RIGHT);
            }
        }
    }

    // Extract owned copies of both fighters (allows cross-fighter reads during tick).
    let mut p1_opt: Option<FighterData> = None;
    for (_, (_, fs, mugen, pos, vel, hp, power, facing)) in world
        .query::<(
            &Player1,
            &FighterState,
            &MugenFighterState,
            &Position,
            &Velocity,
            &Health,
            &PowerGauge,
            &Facing,
        )>()
        .iter()
    {
        p1_opt = Some(FighterData {
            fs: *fs,
            mugen: mugen.clone(),
            pos: *pos,
            vel: *vel,
            hp: *hp,
            power: *power,
            facing: *facing,
        });
    }
    let mut p2_opt: Option<FighterData> = None;
    for (_, (_, fs, mugen, pos, vel, hp, power, facing)) in world
        .query::<(
            &Player2,
            &FighterState,
            &MugenFighterState,
            &Position,
            &Velocity,
            &Health,
            &PowerGauge,
            &Facing,
        )>()
        .iter()
    {
        p2_opt = Some(FighterData {
            fs: *fs,
            mugen: mugen.clone(),
            pos: *pos,
            vel: *vel,
            hp: *hp,
            power: *power,
            facing: *facing,
        });
    }

    let (mut p1, mut p2) = match (p1_opt, p2_opt) {
        (Some(a), Some(b)) => (a, b),
        _ => return vec![],
    };

    let mut hit_events = Vec::new();

    if let Some(cd) = char_data {
        // Snapshot pre-tick opponent state for simultaneous-tick semantics.
        let p1_snap_fs = p1.fs;
        let p1_snap_mugen = p1.mugen.clone();
        let p1_snap_pos = p1.pos;
        let p1_snap_vel = p1.vel;
        let p1_snap_hp = p1.hp;

        let p2_snap_fs = p2.fs;
        let p2_snap_mugen = p2.mugen.clone();
        let p2_snap_pos = p2.pos;
        let p2_snap_vel = p2.vel;
        let p2_snap_hp = p2.hp;

        // Run MUGEN state machine tick for both fighters.
        mugen_tick_with_p2(
            &cd.cns,
            &mut p1.fs,
            &mut p1.mugen,
            &mut p1.pos,
            &mut p1.vel,
            &mut p1.hp,
            &mut p1.power,
            &p2_snap_fs,
            &p2_snap_mugen,
            &p2_snap_pos,
            &p2_snap_vel,
            &p2_snap_hp,
        );
        mugen_tick_with_p2(
            &cd.cns,
            &mut p2.fs,
            &mut p2.mugen,
            &mut p2.pos,
            &mut p2.vel,
            &mut p2.hp,
            &mut p2.power,
            &p1_snap_fs,
            &p1_snap_mugen,
            &p1_snap_pos,
            &p1_snap_vel,
            &p1_snap_hp,
        );

        // Integrate velocity into position (MUGEN coords: -y = up, +y = down/gravity).
        p1.pos.pos.x += p1.vel.vel.x;
        p1.pos.pos.y += p1.vel.vel.y;
        p2.pos.pos.x += p2.vel.vel.x;
        p2.pos.pos.y += p2.vel.vel.y;

        // Advance animation element based on elapsed anim_time and AIR frame durations.
        if let Some(air) = kfm_air {
            advance_anim_elem(&mut p1.mugen, air);
            advance_anim_elem(&mut p2.mugen, air);
        }

        // Auto-facing: fighters always face each other.
        if p1.pos.pos.x < p2.pos.pos.x {
            p1.facing = Facing { dir: Facing::RIGHT };
            p2.facing = Facing { dir: Facing::LEFT };
        } else if p1.pos.pos.x > p2.pos.pos.x {
            p1.facing = Facing { dir: Facing::LEFT };
            p2.facing = Facing { dir: Facing::RIGHT };
        }

        // MUGEN combat: HitDef collision, damage, hitstun, knockback.
        let combat_result = {
            let mut p1_cf = MugenCollisionFighter {
                position: &p1.pos,
                velocity: &mut p1.vel,
                facing: &p1.facing,
                fighter_state: &mut p1.fs,
                mugen: &mut p1.mugen,
                health: &mut p1.hp,
                power: &mut p1.power,
            };
            let mut p2_cf = MugenCollisionFighter {
                position: &p2.pos,
                velocity: &mut p2.vel,
                facing: &p2.facing,
                fighter_state: &mut p2.fs,
                mugen: &mut p2.mugen,
                health: &mut p2.hp,
                power: &mut p2.power,
            };
            mugen_combat_frame(&mut p1_cf, &mut p2_cf, kfm_air, 8000)
        };

        // Reset combo counters when defenders recover.
        reset_combo_if_recovered(&mut p1.mugen, &p2.fs);
        reset_combo_if_recovered(&mut p2.mugen, &p1.fs);

        // Drain pending sounds (MUGEN PlaySnd controllers).
        // Logged for now; full MUGEN SND audio integration is future work.
        for snd in p1.mugen.pending_sounds.drain(..) {
            log::trace!("P1 PlaySnd: group={} sound={}", snd.group, snd.sound);
        }
        for snd in p2.mugen.pending_sounds.drain(..) {
            log::trace!("P2 PlaySnd: group={} sound={}", snd.group, snd.sound);
        }

        // Ground clamping and stage bounds.
        for data in [&mut p1, &mut p2] {
            // MUGEN coords: -y=up, so y > 0 means below ground → clamp.
            if data.pos.pos.y > GROUND_Y {
                data.pos.pos.y = GROUND_Y;
                data.vel.vel.y = 0;
            }
            data.pos.pos.x = stage.clamp_x(data.pos.pos.x);
        }

        // Build HitEvents for the audio/UI pipeline.
        if let Some(ref hit) = combat_result.p1_hit {
            hit_events.push(HitEvent {
                attacker: 0,
                defender: 1,
                hitbox: Hitbox {
                    rect: LogicRect::new(0, 0, 100, 100),
                    damage: hit.damage,
                    hitstun: hit.hitstun,
                    blockstun: 0,
                    knockback: LogicVec2::ZERO,
                    hit_type: HitType::Mid,
                },
            });
        }
        if let Some(ref hit) = combat_result.p2_hit {
            hit_events.push(HitEvent {
                attacker: 1,
                defender: 0,
                hitbox: Hitbox {
                    rect: LogicRect::new(0, 0, 100, 100),
                    damage: hit.damage,
                    hitstun: hit.hitstun,
                    blockstun: 0,
                    knockback: LogicVec2::ZERO,
                    hit_type: HitType::Mid,
                },
            });
        }
    } else {
        // Fallback: basic physics when no CNS is loaded.
        p1.vel.vel.y += DEFAULT_GRAVITY;
        p1.pos.pos.x += p1.vel.vel.x;
        p1.pos.pos.y += p1.vel.vel.y;
        if p1.pos.pos.y < GROUND_Y {
            p1.pos.pos.y = GROUND_Y;
            p1.vel.vel.y = 0;
        }
        p1.pos.pos.x = stage.clamp_x(p1.pos.pos.x);

        p2.vel.vel.y += DEFAULT_GRAVITY;
        p2.pos.pos.x += p2.vel.vel.x;
        p2.pos.pos.y += p2.vel.vel.y;
        if p2.pos.pos.y < GROUND_Y {
            p2.pos.pos.y = GROUND_Y;
            p2.vel.vel.y = 0;
        }
        p2.pos.pos.x = stage.clamp_x(p2.pos.pos.x);
    }

    // Write processed state back to ECS.
    for (_, (_, fs, mugen, pos, vel, hp, power, facing)) in world.query_mut::<(
        &Player1,
        &mut FighterState,
        &mut MugenFighterState,
        &mut Position,
        &mut Velocity,
        &mut Health,
        &mut PowerGauge,
        &mut Facing,
    )>() {
        *fs = p1.fs;
        *mugen = p1.mugen.clone();
        *pos = p1.pos;
        *vel = p1.vel;
        *hp = p1.hp;
        *power = p1.power;
        *facing = p1.facing;
    }
    for (_, (_, fs, mugen, pos, vel, hp, power, facing)) in world.query_mut::<(
        &Player2,
        &mut FighterState,
        &mut MugenFighterState,
        &mut Position,
        &mut Velocity,
        &mut Health,
        &mut PowerGauge,
        &mut Facing,
    )>() {
        *fs = p2.fs;
        *mugen = p2.mugen.clone();
        *pos = p2.pos;
        *vel = p2.vel;
        *hp = p2.hp;
        *power = p2.power;
        *facing = p2.facing;
    }

    hit_events
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
        let tr = match self.text_renderer.as_mut() {
            Some(t) => t,
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
        let audio = &mut self.audio;
        let char_data = self.char_data.as_ref();
        let kfm_air = self.kfm_air.as_ref();

        let result = self.game_loop.tick(|| {
            if menu.should_run_logic() {
                // Capture pre-update state numbers for change detection.
                let mut p1_pre = 0i32;
                let mut p2_pre = 0i32;
                for (_, (_, fs)) in world.query::<(&Player1, &FighterState)>().iter() {
                    p1_pre = fs.state_num;
                }
                for (_, (_, fs)) in world.query::<(&Player2, &FighterState)>().iter() {
                    p2_pre = fs.state_num;
                }

                let hit_events = logic_update(world, &p1_input, &p2_input, stage, char_data, kfm_air);

                // Register hits for combo counter.
                for hit in &hit_events {
                    ui.register_hit(hit.attacker == 0);
                }

                ui.update(world);
                ui.set_wins(menu.p1_wins(), menu.p2_wins());

                // Capture post-update state numbers.
                let mut p1_post = 0i32;
                let mut p2_post = 0i32;
                for (_, (_, fs)) in world.query::<(&Player1, &FighterState)>().iter() {
                    p1_post = fs.state_num;
                }
                for (_, (_, fs)) in world.query::<(&Player2, &FighterState)>().iter() {
                    p2_post = fs.state_num;
                }

                let state_changes = (
                    if p1_pre != p1_post { Some(p1_post) } else { None },
                    if p2_pre != p2_post { Some(p2_post) } else { None },
                );

                let (new_round, round_audio) = menu.update_round(world, ui);
                if let Some(audio_evt) = round_audio {
                    if let Some(ref mut audio_sys) = audio {
                        process_audio_events(audio_sys, &[audio_evt]);
                    }
                }
                if new_round {
                    MenuSystem::reset_fighters(world);
                    ui.reset();
                }
                (hit_events, state_changes, new_round)
            } else {
                // Still need to tick round-end timer even when logic is paused.
                let (new_round, _) = menu.update_round(world, ui);
                if new_round {
                    MenuSystem::reset_fighters(world);
                    ui.reset();
                }
                (Vec::new(), (None, None), new_round)
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

                // Play sounds for state transitions into hit/guard states.
                let mut state_audio = Vec::new();
                if let Some(new_state) = state_changes.0 {
                    if is_hit_state(new_state) || is_guard_state(new_state) {
                        state_audio.push(AudioEvent::PlayActionSound {
                            id: "hit_light".to_string(),
                        });
                    }
                }
                if let Some(new_state) = state_changes.1 {
                    if is_hit_state(new_state) || is_guard_state(new_state) {
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

        // Background quads (no texture)
        let mut bg_instances: Vec<QuadInstance> = Vec::with_capacity(64);

        // Stage background layers (rendered behind everything).
        bg_instances.extend(
            self.stage
                .render_layers(screen_w, screen_h, ground_screen_y),
        );

        // Ground line (scrolls with camera).
        bg_instances.push(QuadInstance {
            rect: [-camera_x, ground_screen_y, screen_w + camera_x * 2.0, 4.0],
            color: [0.3, 0.3, 0.3, 1.0],
            ..Default::default()
        });

        // Fighter quads (with texture)
        let mut fighter_instances: Vec<QuadInstance> = Vec::with_capacity(2);

        // Render fighters using MUGEN anim_num / anim_elem from MugenFighterState.
        for (_, (pos, prev, facing, mugen, _hp, color)) in self
            .world
            .query::<(
                &Position,
                &PreviousPosition,
                &Facing,
                &MugenFighterState,
                &Health,
                &FighterColor,
            )>()
            .iter()
        {
            let prev_render = prev.pos.to_render();
            let cur_render = pos.pos.to_render();
            let x = prev_render[0] + (cur_render[0] - prev_render[0]) * alpha;
            let y = prev_render[1] + (cur_render[1] - prev_render[1]) * alpha;

            // Use MUGEN anim_num directly as the AIR action number.
            let action_num = mugen.anim_num as u32;

            // Resolve current anim_elem → SFF sprite key.
            let sprite_key: Option<(u16, u16)> = self.kfm_air.as_ref().and_then(|air| {
                let action = air.get_action(action_num)
                    .or_else(|| air.get_action(0))?;
                if action.frames.is_empty() {
                    return None;
                }
                // anim_elem is 1-based; clamp to valid range.
                let frame_idx = ((mugen.anim_elem - 1).max(0) as usize)
                    .min(action.frames.len() - 1);
                let f = &action.frames[frame_idx];
                Some((f.group, f.image))
            });

            const SPRITE_SCALE: f32 = 3.0;
            let (render_w, render_h, screen_x, screen_y, uv) =
                if let (Some(atlas), Some((g, i))) = (self.kfm_atlas.as_ref(), sprite_key) {
                    if let (Some(info), Some(uv)) = (atlas.get_info(g, i), atlas.get_uv(g, i)) {
                        let w = info.width as f32;
                        let h = info.height as f32;
                        let sx = x - info.axis_x as f32 * SPRITE_SCALE - camera_x;
                        // MUGEN -y=up: ground_screen_y + y places sprite above ground when y<0.
                        let sy = ground_screen_y + y - info.axis_y as f32 * SPRITE_SCALE;
                        let scaled_w = w * SPRITE_SCALE;
                        let scaled_h = h * SPRITE_SCALE;
                        (scaled_w, scaled_h, sx, sy, uv)
                    } else {
                        (FIGHTER_W, FIGHTER_H, x - FIGHTER_W / 2.0 - camera_x, ground_screen_y + y - FIGHTER_H, [0.0, 0.0, 1.0, 1.0])
                    }
                } else {
                    (FIGHTER_W, FIGHTER_H, x - FIGHTER_W / 2.0 - camera_x, ground_screen_y + y - FIGHTER_H, [0.0, 0.0, 1.0, 1.0])
                };

            // Horizontal flip based on facing direction
            let uv = if facing.dir == Facing::RIGHT {
                uv
            } else {
                [uv[0] + uv[2], uv[1], -uv[2], uv[3]]
            };

            fighter_instances.push(QuadInstance {
                rect: [screen_x, screen_y, render_w, render_h],
                color: if self.use_sprites {
                    [1.0, 1.0, 1.0, 1.0]
                } else {
                    color.0
                },
                uv,
            });
        }

        // UI overlay (health bars, gauges, timer, combo).
        bg_instances.extend(self.ui_renderer.render(&self.world, screen_w));

        // Menu overlay (main menu, pause, round/match end).
        bg_instances.extend(self.menu.render(screen_w, screen_h));

        // Collect text areas.
        let mut text_areas: Vec<TextArea> = Vec::new();
        text_areas.extend(self.ui_renderer.render_text(&self.world, screen_w, screen_h));
        text_areas.extend(self.menu.render_text(screen_w, screen_h));

        // Prepare text renderer.
        tr.prepare(&ctx.device, &ctx.queue, screen_w, screen_h, &text_areas);

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

        // Pass 1: Background + UI (no texture, clears screen).
        qr.draw(
            &ctx.device,
            &mut encoder,
            &view,
            &ctx.queue,
            ctx.size.width as f32,
            ctx.size.height as f32,
            &bg_instances,
            None,
        );

        // Pass 2: Fighters (with texture, no screen clear).
        qr.draw_overlay(
            &ctx.device,
            &mut encoder,
            &view,
            &ctx.queue,
            ctx.size.width as f32,
            ctx.size.height as f32,
            &fighter_instances,
            self.fighter_texture.as_ref(),
        );

        // Pass 3: text (LoadOp::Load - draws on top without clearing).
        {
            let mut text_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("text_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            tr.render(&mut text_pass);
        }

        ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        tr.trim_atlas();
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
    log::info!("=== Tickle Fighting Engine Starting ===");
    let network_mode = NetworkMode::from_args();
    log::info!("Starting Tickle Fighting Engine in {:?} mode", network_mode);
    log::info!("Creating event loop...");
    let event_loop = EventLoop::new().unwrap();
    log::info!("Event loop created successfully");
    let mut app = App::new();
    log::info!("App initialized, starting event loop...");
    event_loop.run_app(&mut app).unwrap();
    log::info!("Event loop exited");
}
