use wgpu::util::DeviceExt;

use crate::sprite_batch::CameraUniform;
use tickle_core::{LogicRect, LogicVec2};

/// Maximum number of line vertices per frame (2 vertices per line segment).
const MAX_VERTICES: usize = 16384;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct LineVertex {
    position: [f32; 2],
    color: [f32; 4],
}

impl LineVertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x4,
    ];

    fn layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LineVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

/// Debug overlay renderer for visualizing hitboxes, hurtboxes, pushboxes,
/// and position markers. Uses line-list topology for efficient batch rendering.
///
/// Toggle visibility at runtime with F1 key.
pub struct DebugRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    vertices: Vec<LineVertex>,
    pub enabled: bool,
}

impl DebugRenderer {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("debug_line_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("debug_lines.wgsl").into()),
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("debug_camera_bgl"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("debug_camera_buffer"),
            contents: bytemuck::cast_slice(&[CameraUniform::orthographic(800.0, 600.0)]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("debug_camera_bg"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("debug_pipeline_layout"),
            bind_group_layouts: &[&camera_bind_group_layout],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("debug_line_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[LineVertex::layout()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("debug_vertex_buffer"),
            size: (MAX_VERTICES * std::mem::size_of::<LineVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            camera_buffer,
            camera_bind_group,
            vertices: Vec::with_capacity(MAX_VERTICES),
            enabled: true,
        }
    }

    /// Clear all queued debug geometry for a new frame.
    pub fn begin(&mut self) {
        self.vertices.clear();
    }

    /// Push a single line segment (two vertices).
    fn push_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, color: [f32; 4]) {
        if self.vertices.len() + 2 > MAX_VERTICES {
            return;
        }
        self.vertices.push(LineVertex {
            position: [x0, y0],
            color,
        });
        self.vertices.push(LineVertex {
            position: [x1, y1],
            color,
        });
    }

    /// Draw a rectangle outline from a LogicRect.
    /// Coordinates are converted from logic units (1/100 pixel) to render pixels.
    pub fn draw_rect(&mut self, rect: LogicRect, color: [f32; 4]) {
        let x0 = rect.x as f32 / 100.0;
        let y0 = rect.y as f32 / 100.0;
        let x1 = (rect.x + rect.w) as f32 / 100.0;
        let y1 = (rect.y + rect.h) as f32 / 100.0;

        self.push_line(x0, y0, x1, y0, color); // top
        self.push_line(x1, y0, x1, y1, color); // right
        self.push_line(x1, y1, x0, y1, color); // bottom
        self.push_line(x0, y1, x0, y0, color); // left
    }

    /// Draw a cross marker at a LogicVec2 position.
    /// `size` is in logic units (1/100 pixel).
    pub fn draw_cross(&mut self, pos: LogicVec2, size: i32, color: [f32; 4]) {
        let cx = pos.x as f32 / 100.0;
        let cy = pos.y as f32 / 100.0;
        let half = size as f32 / 100.0;

        self.push_line(cx - half, cy, cx + half, cy, color); // horizontal
        self.push_line(cx, cy - half, cx, cy + half, color); // vertical
    }

    /// Update the camera projection (call on resize or when camera changes).
    pub fn update_camera(&self, queue: &wgpu::Queue, camera: &CameraUniform) {
        queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[*camera]));
    }

    /// Upload debug geometry and record draw commands into the render pass.
    /// Does nothing if the renderer is disabled or there are no vertices.
    pub fn flush<'a>(&'a self, queue: &wgpu::Queue, render_pass: &mut wgpu::RenderPass<'a>) {
        if !self.enabled || self.vertices.is_empty() {
            return;
        }

        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices));

        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.draw(0..self.vertices.len() as u32, 0..1);
    }

    /// Returns the number of line vertices currently queued.
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }
}
