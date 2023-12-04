// Vertex shader

struct VertexInput {
    @location(0) position: u32,
    // @location(1) tex_coords: u32,
    // @location(2) ao: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) ao: f32,
    @location(2) distance: f32,
}

@group(0) @binding(0)
var<uniform> projection: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> view: mat4x4<f32>;

@group(2) @binding(0)
var<uniform> model: mat4x4<f32>;

fn unpack_vertex(in_vertex: u32) -> VertexOutput {
    
    var output: VertexOutput;
    
    output.clip_position = vec4<f32>(
        f32((in_vertex & 0xfe000000u) >> u32(25)),
        f32((in_vertex & 0x01FC0000u) >> u32(18)),
        f32((in_vertex & 0x0003F800u) >> u32(11)),
        1.0
    );
    
    output.tex_coords = vec2<f32>(
        f32((in_vertex & 0x00000780u) >> u32(7)) / 16.0,
        f32((in_vertex & 0x00000078u) >> u32(3)) / 16.0
    );
    
    output.ao = f32((in_vertex & 0x00000007u) >> u32(0)) / 2.0;
    
    output.distance = 1.0;
    
    return output;
}

@vertex
fn vs_main(
    input: VertexInput,
) -> VertexOutput {
    var out: VertexOutput = unpack_vertex(input.position);
    
    out.clip_position = projection * view * model * out.clip_position;
    out.distance = length(out.clip_position);
    
    return out;
}

// Fragment shader

@group(3) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(3)@binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // we manually set the fog color and fog start/end at this point, might be better to pass in as 
    var distance = in.distance;
    if distance < 96.0 {
        distance = 0.0;
    } else {
        distance = distance - 96.0;
    }
//     return mix(in.ao * textureSample(t_diffuse, s_diffuse, in.tex_coords), vec4<f32>(0.1, 0.2, 0.3, 1.0), min(distance / 32.0, 1.0));
    return in.ao * textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
