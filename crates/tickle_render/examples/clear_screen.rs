use std::sync::Arc;
use tickle_render::RenderContext;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

struct App {
    render_ctx: Option<RenderContext>,
    window: Option<Arc<Window>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("Tickle Render - Clear Screen Test")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());
        self.render_ctx = Some(pollster::block_on(RenderContext::new(window)).unwrap());
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
            WindowEvent::RedrawRequested => {
                if let Some(ctx) = self.render_ctx.as_ref() {
                    let color = wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    };
                    if let Err(e) = ctx.render_clear(color) {
                        eprintln!("Render error: {e}");
                    }
                    ctx.window().request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut app = App {
        render_ctx: None,
        window: None,
    };
    event_loop.run_app(&mut app).unwrap();
}
