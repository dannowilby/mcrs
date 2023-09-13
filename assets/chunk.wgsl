// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) ao: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) ao: f32,
}

@group(0) @binding(0)
var<uniform> projection: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> view: mat4x4<f32>;

@group(2) @binding(0)
var<uniform> model: mat4x4<f32>;

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = input.tex_coords;
    out.ao = input.ao;
    out.clip_position = projection * view * model * vec4<f32>(input.position, 1.0);
    return out;
}

// Fragment shader

@group(3) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(3)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.ao * textureSample(t_diffuse, s_diffuse, in.tex_coords);
}