// Sprite batch shader for Tickle Fighting Engine
// Renders textured quads with per-instance color tinting and alpha blending.

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(1) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(1)
var s_diffuse: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
};

struct InstanceInput {
    @location(2) inst_pos: vec2<f32>,
    @location(3) inst_size: vec2<f32>,
    @location(4) inst_uv_offset: vec2<f32>,
    @location(5) inst_uv_size: vec2<f32>,
    @location(6) inst_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
};

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Scale the unit quad by instance size and translate by instance position
    let world_pos = vertex.position * instance.inst_size + instance.inst_pos;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 0.0, 1.0);

    // Map unit UV [0,1] to the atlas sub-region
    out.uv = instance.inst_uv_offset + vertex.uv * instance.inst_uv_size;
    out.color = instance.inst_color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex_color = textureSample(t_diffuse, s_diffuse, in.uv);
    return tex_color * in.color;
}
