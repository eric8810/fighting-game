pub mod game_loop;
mod quad_renderer;

use std::sync::Arc;

use hecs::World;
use tickle_core::{
    Direction, Facing, FighterState, Health, InputState, LogicVec2, Position, PreviousPosition,
    StateType, Velocity, BUTTON_A,
};
use tickle_render::RenderContext;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use game_loop::GameLoop;
use quad_renderer::{QuadInstance, QuadRenderer};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Stage dimensions in logic units (1 unit = 1/100 pixel).
const STAGE_WIDTH: i32 = 80_000; // 800 px
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

struct Player1;
struct Player2;

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
}

impl App {
    fn new() -> Self {
        let mut world = World::new();
        spawn_fighters(&mut world);
        Self {
            render_ctx: None,
            quad_renderer: None,
            window: None,
            world,
            game_loop: GameLoop::new(),
            p1_input: RawInput::default(),
            p2_input: RawInput::default(),
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
        FighterState::new(),
        Health::new(10_000),
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
        FighterState::new(),
        Health::new(10_000),
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
                    handle_key(&mut self.p1_input, &mut self.p2_input, key, pressed);
                }
            }

            WindowEvent::RedrawRequested => {
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

// ---------------------------------------------------------------------------
// Game logic (runs at fixed 60 FPS)
// ---------------------------------------------------------------------------

fn logic_update(world: &mut World, p1_input: &InputState, p2_input: &InputState) {
    // Save previous positions for interpolation.
    for (_, (pos, prev)) in world.query_mut::<(&Position, &mut PreviousPosition)>() {
        prev.pos = pos.pos;
    }

    // Apply input to Player 1.
    for (_, (_, vel, state)) in world.query_mut::<(&Player1, &mut Velocity, &mut FighterState)>() {
        apply_input(vel, state, p1_input);
    }
    // Apply input to Player 2.
    for (_, (_, vel, state)) in world.query_mut::<(&Player2, &mut Velocity, &mut FighterState)>() {
        apply_input(vel, state, p2_input);
    }

    // Physics: gravity + velocity integration + ground detection.
    for (_, (pos, vel, state)) in
        world.query_mut::<(&mut Position, &mut Velocity, &mut FighterState)>()
    {
        // Gravity (only when airborne).
        if pos.pos.y > GROUND_Y || vel.vel.y > 0 {
            vel.vel.y += GRAVITY;
        }
        // Ground friction.
        if pos.pos.y <= GROUND_Y && state.current_state != StateType::Jump {
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
            if state.current_state == StateType::Jump {
                state.change_state(StateType::Idle);
            }
        }
        // Stage bounds.
        pos.pos.x = pos.pos.x.clamp(0, STAGE_WIDTH);
        // Advance state frame.
        state.state_frame += 1;
    }
}

fn apply_input(vel: &mut Velocity, state: &mut FighterState, input: &InputState) {
    let on_ground = state.current_state != StateType::Jump;
    if on_ground {
        if input.direction.is_left() {
            vel.vel.x = -MOVE_SPEED;
        } else if input.direction.is_right() {
            vel.vel.x = MOVE_SPEED;
        }
        if input.direction.is_up() {
            vel.vel.y = JUMP_VEL;
            state.change_state(StateType::Jump);
        }
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

        let p1_input = self.p1_input.to_input_state();
        let p2_input = self.p2_input.to_input_state();
        let world = &mut self.world;

        let result = self.game_loop.tick(|| {
            logic_update(world, &p1_input, &p2_input);
        });

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
        let screen_h = ctx.size.height as f32;
        let mut instances = Vec::new();

        // Ground line.
        let ground_screen_y = screen_h - 100.0; // ground at 100px from bottom
        instances.push(QuadInstance {
            rect: [0.0, ground_screen_y, ctx.size.width as f32, 4.0],
            color: [0.3, 0.3, 0.3, 1.0],
        });

        // Fighters.
        for (_, (pos, prev, _facing, _hp, color)) in world
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

            // Convert logic coords to screen coords.
            // Logic Y=0 is ground; screen ground is at ground_screen_y.
            let screen_x = x - FIGHTER_W / 2.0;
            let screen_y = ground_screen_y - y - FIGHTER_H;

            instances.push(QuadInstance {
                rect: [screen_x, screen_y, FIGHTER_W, FIGHTER_H],
                color: color.0,
            });
        }

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
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = App::new();
    event_loop.run_app(&mut app).unwrap();
}
