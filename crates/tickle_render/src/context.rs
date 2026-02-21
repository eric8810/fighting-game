use anyhow::Result;
use std::sync::Arc;
use wgpu::{
    Device, DeviceDescriptor, Instance, InstanceDescriptor, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration, TextureUsages,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

/// Core rendering context holding all wgpu state needed for rendering.
pub struct RenderContext {
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub config: SurfaceConfiguration,
    pub size: PhysicalSize<u32>,
    window: Arc<Window>,
}

impl RenderContext {
    /// Initialize wgpu and create a render context bound to the given window.
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();

        let instance = Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find a suitable GPU adapter"))?;

        log::info!("Using adapter: {:?}", adapter.get_info().name);

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: Some("tickle_render_device"),
                    ..Default::default()
                },
                None,
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        Ok(Self {
            device,
            queue,
            surface,
            config,
            size,
            window,
        })
    }

    /// Handle window resize by reconfiguring the surface.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    /// Returns the surface texture format.
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Returns a reference to the window.
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Render a single frame, clearing the screen to the given color.
    pub fn render_clear(&self, color: wgpu::Color) -> Result<()> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("clear_encoder"),
            });

        // Begin and immediately end a render pass to clear the screen.
        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
