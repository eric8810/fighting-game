use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;
use tickle_render::Texture;

/// A single quad instance for rendering fighters with textures.
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct QuadInstance {
    /// Position (x, y) in pixels, size (w, h) in pixels.
    pub rect: [f32; 4],
    /// RGBA color (tint/alpha).
    pub color: [f32; 4],
    /// UV offset (u, v) and size (w, h) in texture space [0-1].
    pub uv: [f32; 4],
}

impl Default for QuadInstance {
    fn default() -> Self {
        Self {
            rect: [0.0, 0.0, 1.0, 1.0],
            color: [1.0, 1.0, 1.0, 1.0],
            uv: [0.0, 0.0, 1.0, 1.0],
        }
    }
}

/// Textured quad renderer for fighters.
/// Draws axis-aligned rectangles with texture sampling.
pub struct QuadRenderer {
    pipeline_textured: wgpu::RenderPipeline,
    pipeline_solid: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group_layout_textured: wgpu::BindGroupLayout,
    bind_group_solid: wgpu::BindGroup,
    sampler: wgpu::Sampler,
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

const SHADER_SRC_TEXTURED: &str = r#"
struct Uniforms {
    screen_size: vec4<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var texture: texture_2d<f32>;
@group(0) @binding(2) var tex_sampler: sampler;

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) rect: vec4<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let pixel_pos = vec2<f32>(
        in.rect.x + in.pos.x * in.rect.z,
        in.rect.y + in.pos.y * in.rect.w,
    );
    let ndc = vec2<f32>(
        pixel_pos.x / uniforms.screen_size.x * 2.0 - 1.0,
        1.0 - pixel_pos.y / uniforms.screen_size.y * 2.0,
    );
    let uv = vec2<f32>(
        in.uv.x + in.pos.x * in.uv.z,
        in.uv.y + in.pos.y * in.uv.w,
    );
    var out: VertexOutput;
    out.clip_pos = vec4<f32>(ndc, 0.0, 1.0);
    out.color = in.color;
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(texture, tex_sampler, in.uv);
    return tex_color * in.color;
}
"#;

const SHADER_SRC_SOLID: &str = r#"
struct Uniforms {
    screen_size: vec4<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) pos: vec2<f32>,
    @location(1) rect: vec4<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let pixel_pos = vec2<f32>(
        in.rect.x + in.pos.x * in.rect.z,
        in.rect.y + in.pos.y * in.rect.w,
    );
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
        let shader_textured = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("quad_shader_textured"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC_TEXTURED.into()),
        });
        let shader_solid = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("quad_shader_solid"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SRC_SOLID.into()),
        });

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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("quad_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        // Solid pipeline bind group layout
        let bind_group_layout_solid = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("quad_bgl_solid"),
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

        // Textured pipeline bind group layout
        let bind_group_layout_textured = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("quad_bgl_textured"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group_solid = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("quad_bg_solid"),
            layout: &bind_group_layout_solid,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let vertex_buffers = &[
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as u64,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &wgpu::vertex_attr_array![0 => Float32x2],
            },
            wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<QuadInstance>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &wgpu::vertex_attr_array![
                    1 => Float32x4,
                    2 => Float32x4,
                    3 => Float32x4,
                ],
            },
        ];

        // Solid pipeline
        let pipeline_layout_solid = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("quad_pipeline_layout_solid"),
            bind_group_layouts: &[&bind_group_layout_solid],
            immediate_size: 0,
        });
        let pipeline_solid = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad_pipeline_solid"),
            layout: Some(&pipeline_layout_solid),
            vertex: wgpu::VertexState {
                module: &shader_solid,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_solid,
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
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        // Textured pipeline
        let pipeline_layout_textured = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("quad_pipeline_layout_textured"),
            bind_group_layouts: &[&bind_group_layout_textured],
            immediate_size: 0,
        });
        let pipeline_textured = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad_pipeline_textured"),
            layout: Some(&pipeline_layout_textured),
            vertex: wgpu::VertexState {
                module: &shader_textured,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_textured,
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
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview_mask: None,
            cache: None,
        });

        Self {
            pipeline_textured,
            pipeline_solid,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            uniform_buffer,
            bind_group_layout_textured,
            bind_group_solid,
            sampler,
            max_instances: Self::MAX_INSTANCES,
        }
    }

    /// Draw a batch of quads with texture support
    pub fn draw(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        queue: &wgpu::Queue,
        screen_width: f32,
        screen_height: f32,
        instances: &[QuadInstance],
        texture: Option<&Texture>,
    ) {
        self.draw_internal(
            device,
            encoder,
            target,
            queue,
            screen_width,
            screen_height,
            instances,
            texture,
            true, // clear screen
        )
    }

    /// Draw a batch of quads without clearing the screen (for layered rendering)
    pub fn draw_overlay(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        queue: &wgpu::Queue,
        screen_width: f32,
        screen_height: f32,
        instances: &[QuadInstance],
        texture: Option<&Texture>,
    ) {
        self.draw_internal(
            device,
            encoder,
            target,
            queue,
            screen_width,
            screen_height,
            instances,
            texture,
            false, // don't clear screen
        )
    }

    fn draw_internal(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        queue: &wgpu::Queue,
        screen_width: f32,
        screen_height: f32,
        instances: &[QuadInstance],
        texture: Option<&Texture>,
        clear_screen: bool,
    ) {
        let count = instances.len().min(self.max_instances);
        if count == 0 {
            return;
        }

        let uniforms = Uniforms {
            screen_size: [screen_width, screen_height, 0.0, 0.0],
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        // Create a per-draw instance buffer to avoid overwriting shared buffer
        // before the GPU has consumed the previous draw's data.
        let instance_data = bytemuck::cast_slice(&instances[..count]);
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad_instances_tmp"),
            contents: instance_data,
            usage: wgpu::BufferUsages::VERTEX,
        });

        let load_op = if clear_screen {
            wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.05,
                g: 0.05,
                b: 0.08,
                a: 1.0,
            })
        } else {
            wgpu::LoadOp::Load
        };

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("quad_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: load_op,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        if let Some(tex) = texture {
            // Create textured bind group
            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("quad_bg_textured"),
                layout: &self.bind_group_layout_textured,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.uniform_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&tex.view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                ],
            });

            pass.set_pipeline(&self.pipeline_textured);
            pass.set_bind_group(0, &bind_group, &[]);
        } else {
            pass.set_pipeline(&self.pipeline_solid);
            pass.set_bind_group(0, &self.bind_group_solid, &[]);
        }

        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, instance_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..6, 0, 0..count as u32);
    }
}
