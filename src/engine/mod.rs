pub mod game_state;
pub mod input;
pub mod matrix;
pub mod render_group;
pub mod render_object;
pub mod renderer;
pub mod resources;
pub mod texture;
pub mod uniform;

use glam::Mat4;
use matrix::Matrix;
use render_group::RenderGroupBuilder;
use render_object::RenderObject;
use renderer::Renderer;
use texture::Texture;

//
// example usage of the rendering engine
//

/*
pub async fn test_render_initialization(window_state: &WindowState) -> RenderGroup {
    let device = &window_state.device;
    let queue = &window_state.queue;

    let diffuse_bytes = include_bytes!("../happy-tree.png");
    let diffuse_texture =
        texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();
    let (texture_uniform, texture_bind_group_layout) =
        diffuse_texture.build_uniform(&window_state, 0);

    let mut render_object = RenderObject::new(
        &window_state,
        bytemuck::cast_slice(VERTICES),
        bytemuck::cast_slice(INDICES),
    );

    let mut render_group = RenderGroupBuilder::new(&window_state, "test_render_group")
        .with(texture_bind_group_layout)
        .vertex_format(vertex_description())
        .shader("../test_shader.wgsl")
        .build();

    render_group
        .uniforms
        .insert("texture".to_string(), texture_uniform);
    render_group
        .objects
        .insert("test-object".to_string(), render_object);

    render_group
}
*/
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

fn vertex_description<'a>() -> wgpu::VertexBufferLayout<'a> {
    use std::mem;
    wgpu::VertexBufferLayout {
        array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            wgpu::VertexAttribute {
                offset: 0,
                shader_location: 0,
                format: wgpu::VertexFormat::Float32x3,
            },
            wgpu::VertexAttribute {
                offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                shader_location: 1,
                format: wgpu::VertexFormat::Float32x2,
            },
        ],
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.00759614],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.43041354],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.949397],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.84732914],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.2652641],
    }, // E
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4, /* padding */ 0];

use self::resources::load_string;
pub async fn new_renderer_full() -> Renderer {
    let chunk_id = "chunk-0-0-0";

    let mut renderer = Renderer::new();

    // we should make the layout before we create the resource
    let texture_layout = Texture::create_layout(0);
    let texture_uniform = Texture::load("happy-tree.png")
        .await
        .uniform(&texture_layout);

    let matrix_layout = Matrix::create_layout(1);
    let matrix_uniform = Matrix::new(Mat4::IDENTITY).uniform(&matrix_layout);

    let shader_source = load_string("test_shader.wgsl").await.unwrap();

    renderer.create_group(
        "chunk_render_group",
        RenderGroupBuilder::new()
            .shader(&shader_source)
            .with("texture_atlas", texture_layout)
            .with("model", matrix_layout)
            .vertex_format(vertex_description())
            .build(),
    );

    renderer.set_global_uniform("texture_atlas", texture_uniform);

    // for now, let each render object have a handle on their associated render group
    // we will naively draw all objects
    // add/set object will return a mutable reference to the object
    // so we can set the uniforms and do anything else to it that
    // we want

    // let model_uniform = Matrix::identity();
    let vertices = bytemuck::cast_slice(VERTICES);
    let indices = bytemuck::cast_slice(INDICES);
    renderer
        .add_object(
            chunk_id,
            RenderObject::new("chunk_render_group", vertices, indices),
        )
        .set_uniform("model", matrix_uniform);
    // .set_uniform("model", model_uniform);
    /*
        let mut render_object = renderer.get_mut_object(chunkId).unwrap();
        render_object.set_buffers(vertices, indices);
        render_object.set_uniform("model", model_uniform);
    */
    renderer
}
