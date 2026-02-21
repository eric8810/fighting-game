use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

/// A single colored quad instance for rendering fighters as rectangles.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct QuadInstance {
    /// Position (x, y) in pixels, size (w, h) in pixels.
    pub rect: [f32; 4],
    /// RGBA color.
    pub color: [f32; 4],
}

/// Minimal colored-quad renderer for the integration demo.
/// Draws axis-aligned rectangles with solid colors.
pub struct QuadRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    max_instances: usize,
}

// Screen-space projection uniform: [width, height, 0, 0]
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    screen_size: [f32; 4],
}

// Unit quad vertices (0,0)-(1,1), expanded per-instance in the shader.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pos: [f32; 2],
}

const QUAD_VERTICES: &[Vertex] = &[
    Vertex { pos: [0.0, 0.0] },
    Vertex { pos: [1.0, 0.0] },
    Vertex { pos: [1.0, 1.0] },
    Vertex { pos: [0.0, 1.0] },
];

const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

const SHADER_SRC: &str = r#"
struct Uniforms {
    screen_size: vec4<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) pos: vec2<f32>,
    // Per-instance
    @location(1) rect: vec4<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    // Expand unit quad by instance rect (x, y, w, h) in pixel coords.
    let pixel_pos = vec2<f32>(
        in.rect.x + in.pos.x * in.rect.z,
        in.rect.y + in.pos.y * in.rect.w,
    );
    // Convert pixel coords to clip space: (0,0) top-left, (w,h) bottom-right.
    let ndc = vec2<f32>(
        pixel_pos.x / uniforms.screen_size.x * 2.0 - 1.0,
        1.0 - pixel_pos.y / uniforms.screen_size.y * 2.0,
    );
    var out: VertexOutput;
    out.clip_pos = vec4<f32>(ndc, 0.0, 1.0);
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

impl QuadRenderer {
    const MAX_INSTANCES: usize = 64;

    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("quad_shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC.into()),
        });

        // -- Buffers --
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad_vb"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad_ib"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad_instances"),
            size: (Self::MAX_INSTANCES * std::mem::size_of::<QuadInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad_uniforms"),
            size: std::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // -- Bind group --
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("quad_bgl"),
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
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("quad_bg"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // -- Pipeline --
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("quad_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[
                    // Vertex buffer (per-vertex)
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &wgpu::vertex_attr_array![0 => Float32x2],
                    },
                    // Instance buffer (per-instance)
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<QuadInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &wgpu::vertex_attr_array![
                            1 => Float32x4,  // rect
                            2 => Float32x4,  // color
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            uniform_buffer,
            bind_group,
            max_instances: Self::MAX_INSTANCES,
        }
    }

    /// Draw a batch of colored quads in a single draw call.
    pub fn draw(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        queue: &wgpu::Queue,
        screen_width: f32,
        screen_height: f32,
        instances: &[QuadInstance],
    ) {
        let count = instances.len().min(self.max_instances);
        if count == 0 {
            return;
        }

        // Upload uniforms.
        let uniforms = Uniforms {
            screen_size: [screen_width, screen_height, 0.0, 0.0],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        // Upload instances.
        queue.write_buffer(
            &self.instance_buffer,
            0,
            bytemuck::cast_slice(&instances[..count]),
        );

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("quad_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
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
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            pass.draw_indexed(0..6, 0, 0..count as u32);
        }
    }
}
