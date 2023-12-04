// Vertex shader

struct VertexInput {
    @location(0) position: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    input: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    
    out.clip_position = vec4<f32>(
        f32((input.position & 0xffff0000u) >> u32(16)) - 1.0,
        f32((input.position & 0x0000ffffu) >> u32(0)) - 1.0,
        0.0,
        1.0
    );
    out.tex_coords = vec2<f32>(out.clip_position.x / 2.0 + 0.5, 1.0 - (out.clip_position.y / 2.0 + 0.5));
    /*
    let x = f32(1 - i32(in_vertex_index)) * 0.5;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 0.5;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    */
    return out;
}

// Fragment shader

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
