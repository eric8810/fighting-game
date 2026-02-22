//! Minimal test to verify text rendering without full game loop
//! Run with: cargo run --example text_minimal

use pollster::block_on;
use std::sync::Arc;
use tickle_render::RenderContext;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

// Import from game crate
use game::text_renderer::{TextArea, TextRenderer};

struct MinimalApp {
    window: Option<Arc<Window>>,
    render_ctx: Option<RenderContext>,
    text_renderer: Option<TextRenderer>,
    frame_count: u32,
}

impl MinimalApp {
    fn new() -> Self {
        Self {
            window: None,
            render_ctx: None,
            text_renderer: None,
            frame_count: 0,
        }
    }
}

impl ApplicationHandler for MinimalApp {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window_attrs = Window::default_attributes()
            .with_title("Text Rendering Test")
            .with_inner_size(winit::dpi::PhysicalSize::new(800, 600));

        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        let ctx = block_on(RenderContext::new(window.clone())).unwrap();
        let tr = TextRenderer::new(&ctx.device, &ctx.queue, ctx.surface_format());

        self.window = Some(window);
        self.render_ctx = Some(ctx);
        self.text_renderer = Some(tr);

        println!("✅ Window and text renderer initialized successfully");
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                println!("✅ Window close requested, exiting");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.frame_count += 1;

                let ctx = match self.render_ctx.as_ref() {
                    Some(c) => c,
                    None => return,
                };
                let tr = match self.text_renderer.as_mut() {
                    Some(t) => t,
                    None => return,
                };

                let screen_w = ctx.size.width as f32;
                let screen_h = ctx.size.height as f32;

                // Create test text areas
                let text_areas = vec![
                    TextArea {
                        text: "Text Rendering Test".to_string(),
                        x: 200.0,
                        y: 50.0,
                        size: 32.0,
                        color: [1.0, 1.0, 1.0, 1.0],
                        bounds: None,
                    },
                    TextArea {
                        text: format!("Frame: {}", self.frame_count),
                        x: 200.0,
                        y: 100.0,
                        size: 24.0,
                        color: [0.9, 0.9, 0.3, 1.0],
                        bounds: None,
                    },
                    TextArea {
                        text: "P1".to_string(),
                        x: 50.0,
                        y: 150.0,
                        size: 18.0,
                        color: [0.1, 0.9, 0.3, 1.0],
                        bounds: None,
                    },
                    TextArea {
                        text: "P2".to_string(),
                        x: 700.0,
                        y: 150.0,
                        size: 18.0,
                        color: [0.9, 0.3, 0.2, 1.0],
                        bounds: None,
                    },
                    TextArea {
                        text: "99".to_string(),
                        x: 380.0,
                        y: 200.0,
                        size: 28.0,
                        color: [0.9, 0.9, 0.3, 1.0],
                        bounds: None,
                    },
                    TextArea {
                        text: "5 HITS".to_string(),
                        x: 300.0,
                        y: 300.0,
                        size: 20.0,
                        color: [1.0, 0.8, 0.0, 1.0],
                        bounds: None,
                    },
                ];

                // Prepare text
                tr.prepare(&ctx.device, &ctx.queue, screen_w, screen_h, &text_areas);

                // Render
                let output = match ctx.surface.get_current_texture() {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("❌ Failed to get surface texture: {}", e);
                        return;
                    }
                };

                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = ctx
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("text_test_encoder"),
                    });

                // Clear screen to dark blue
                {
                    let _clear_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("clear_pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            depth_slice: None,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color {
                                    r: 0.05,
                                    g: 0.05,
                                    b: 0.15,
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
                }

                // Render text
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

                if self.frame_count == 1 {
                    println!("✅ First frame rendered successfully");
                }
                if self.frame_count % 60 == 0 {
                    println!("✅ Frame {} rendered", self.frame_count);
                }

                // Request next frame
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }
}

fn main() {
    env_logger::init();
    println!("🚀 Starting minimal text rendering test...");

    let event_loop = EventLoop::new().unwrap();
    let mut app = MinimalApp::new();

    match event_loop.run_app(&mut app) {
        Ok(_) => println!("✅ Test completed successfully"),
        Err(e) => eprintln!("❌ Test failed: {}", e),
    }
}
