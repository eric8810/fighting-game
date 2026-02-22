use std::sync::Arc;
use tickle_render::texture::Texture;
use tickle_render::{RenderContext, SpriteBatchRenderer, SpriteInstance};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

struct App {
    render_ctx: Option<RenderContext>,
    window: Option<Arc<Window>>,
    renderer: Option<SpriteBatchRenderer>,
    texture_bind_group: Option<wgpu::BindGroup>,
    frame: u64,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attrs = Window::default_attributes()
            .with_title("Tickle Render - Sprite Batch (100+ sprites)")
            .with_inner_size(winit::dpi::LogicalSize::new(800, 600));
        let window = Arc::new(event_loop.create_window(attrs).unwrap());
        self.window = Some(window.clone());

        let ctx = pollster::block_on(RenderContext::new(window)).unwrap();

        let renderer = SpriteBatchRenderer::new(&ctx.device, ctx.surface_format());
        renderer.update_camera(&ctx.queue, ctx.size.width as f32, ctx.size.height as f32);

        // Use a 1x1 white texture so we can draw solid-colored quads
        let white = Texture::white_1x1(&ctx.device, &ctx.queue);
        let bind_group = renderer.create_texture_bind_group(&ctx.device, &white);

        self.texture_bind_group = Some(bind_group);
        self.renderer = Some(renderer);
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
                    if let Some(renderer) = self.renderer.as_ref() {
                        renderer.update_camera(
                            &ctx.queue,
                            new_size.width as f32,
                            new_size.height as f32,
                        );
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                self.frame += 1;
                let ctx = self.render_ctx.as_ref().unwrap();
                let renderer = self.renderer.as_mut().unwrap();
                let bind_group = self.texture_bind_group.as_ref().unwrap();

                renderer.begin();

                // Draw 144 sprites in a 12x12 grid with animated colors
                let cols = 12u32;
                let rows = 12u32;
                let sprite_w = 48.0f32;
                let sprite_h = 48.0f32;
                let padding = 8.0f32;
                let offset_x = 40.0f32;
                let offset_y = 40.0f32;

                for row in 0..rows {
                    for col in 0..cols {
                        let idx = row * cols + col;
                        let t = (self.frame as f32 + idx as f32 * 5.0) * 0.02;
                        let r = (t.sin() * 0.5 + 0.5).clamp(0.0, 1.0);
                        let g = ((t + 2.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
                        let b = ((t + 4.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);

                        renderer.draw_sprite(SpriteInstance {
                            position: [
                                offset_x + col as f32 * (sprite_w + padding),
                                offset_y + row as f32 * (sprite_h + padding),
                            ],
                            size: [sprite_w, sprite_h],
                            uv_offset: [0.0, 0.0],
                            uv_size: [1.0, 1.0],
                            color: [r, g, b, 1.0],
                        });
                    }
                }

                // Render
                let output = ctx.surface.get_current_texture().unwrap();
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    ctx.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("sprite_batch_encoder"),
                        });

                {
                    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("sprite_batch_pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.05,
                                    g: 0.05,
                                    b: 0.08,
                                    a: 1.0,
                                }),
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                        multiview_mask: None,
                    });

                    renderer.flush(&ctx.queue, &mut pass, bind_group);
                }

                ctx.queue.submit(std::iter::once(encoder.finish()));
                output.present();

                // Request continuous redraw for animation
                ctx.window().request_redraw();
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
        renderer: None,
        texture_bind_group: None,
        frame: 0,
    };
    event_loop.run_app(&mut app).unwrap();
}
