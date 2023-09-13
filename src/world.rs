use noise::{Simplex, Worley};
use std::collections::HashMap;
use winit::event::VirtualKeyCode;

use crate::chunk::block::{Block, BlockDictionary};
use crate::chunk::cube_model::cube_model;
use crate::chunk::meshing;
use crate::chunk::meshing::mesh_chunk;
use crate::chunk::{calc_lod, ChunkConfig, ChunkData};
use crate::engine::game_state::{self, GameState};
use crate::engine::input::Input;
use crate::engine::matrix::Matrix;
use crate::engine::render_group::RenderGroupBuilder;
use crate::engine::renderer::Renderer;
use crate::engine::resources::load_string;
use crate::engine::texture;

use crate::chunk::generation::load_chunk;
use crate::player::{focus_window, player_input, update_camera, update_perspective, Player};
use crate::window_state;

#[derive(PartialEq, Eq, Hash)]
pub enum Event {
    Init,
    Tick,

    Resized,

    PlayerMoved,
}

pub struct GameData {
    // component data
    loaded_chunks: HashMap<String, ChunkData>,

    // singleton data
    chunk_config: ChunkConfig,
    block_dictionary: BlockDictionary,
    pub player: Player,
    time: f64,
    frames: f64,
    pub focused: bool,
}

// pseudocode atm
fn load_world(
    renderer: &mut Renderer,
    input: &mut Input,
    data: &mut GameData,
    queue: &mut Vec<Event>,
    delta: f64,
) {
    /*
    // this shouldn't be bad to parallelize
    for (x, y, z) in player_chunk_radius {
        let chunk_id = "";
        let lod = calc_lod(player, (x, y, z));

        data.chunk_data[chunk_id] = generate(x, y, z);

        let (mesh_vertices, mesh_indices) = generate_mesh(data.chunk_data, lod);
        renderer.add_object(chunk_id).set_uniform();
        data.meshes[chunk_id] = chunk_id;
    }
    */
    //

    for x in -2..3 {
        for y in -2..3 {
            for z in -2..3 {
                let chunk_pos = (x, y, z);
                let chunk_id = format!("chunk-{}-{}-{}", x, y, z);
                let lod = calc_lod();

                let chunk = load_chunk(
                    &data.loaded_chunks,
                    &data.block_dictionary,
                    &data.chunk_config,
                    &chunk_pos,
                );
                data.loaded_chunks.insert(chunk_id.clone(), chunk.clone());
            }
        }
    }
    for x in -2..3 {
        for y in -2..3 {
            for z in -2..3 {
                let chunk_pos = (x, y, z);
                let chunk_id = format!("chunk-{}-{}-{}", x, y, z);
                let lod = calc_lod();
                let chunk = data.loaded_chunks.get(&chunk_id).unwrap();
                let mesh = mesh_chunk(
                    &data.loaded_chunks,
                    &data.block_dictionary,
                    &data.chunk_config,
                    &chunk_pos,
                    &chunk,
                    lod,
                );

                let mat = Matrix::new(glam::Mat4::from_translation(glam::f32::vec3(
                    0.0, //x as f32 * data.chunk_config.depth as f32,
                    0.0, //y as f32 * data.chunk_config.depth as f32,
                    0.0, //z as f32 * data.chunk_config.depth as f32,
                )))
                .uniform(&Matrix::create_layout(2));
                renderer
                    .add_object(&chunk_id, mesh)
                    .set_uniform("model", mat);
            }
        }
    }
}

pub async fn init() -> GameState<GameData, Event> {
    let seed = 123456789;
    let mut game_state = GameState::new(
        Renderer::new(),
        GameData {
            // chunk_data: HashMap::new(),
            loaded_chunks: Default::default(),

            chunk_config: ChunkConfig {
                noise: Box::new(Simplex::new(seed)),
                noise_amplitude: (0.025, 0.025, 0.025),
                depth: 16,

                uv_size: 0.0625,
            },

            block_dictionary: BlockDictionary::from([
                (0, Block::default()),
                (
                    1,
                    Block {
                        model: cube_model,
                        transparent: false,
                        ident: "grass".to_owned(),
                        uv: [0.0, 0.0],
                    },
                ),
                (
                    2,
                    Block {
                        model: cube_model,
                        transparent: false,
                        ident: "stone".to_owned(),
                        uv: [0.0625, 0.0],
                    },
                ),
            ]),
            player: Player::new(),
            time: 0.0,
            frames: 0.0,
            focused: false,
        },
    );

    let shader_source = load_string("chunk.wgsl")
        .await
        .expect("error loading shader... :(");
    game_state.renderer.create_group(
        "chunk_render_group",
        RenderGroupBuilder::new()
            .with("projection", Matrix::create_layout(0))
            .with("view", Matrix::create_layout(1))
            .with("model", Matrix::create_layout(2))
            .with("texture_atlas", texture::Texture::create_layout(3))
            .vertex_format(meshing::Vertex::description())
            .shader(&shader_source)
            .build(),
    );

    let texture_uniform = texture::Texture::load("texture_atlas.png").await;
    game_state.renderer.set_global_uniform(
        "texture_atlas",
        texture_uniform.uniform(&texture::Texture::create_layout(3)),
    );

    let config = &window_state().config;
    let projection = glam::Mat4::perspective_rh_gl(
        game_state.data.player.fov,
        config.width as f32 / config.height as f32,
        0.1,
        100.0,
    );
    let proj = Matrix::new(projection).uniform(&Matrix::create_layout(0));
    game_state.renderer.set_global_uniform("projection", proj);

    let camera = Matrix::new(glam::Mat4::IDENTITY).uniform(&Matrix::create_layout(1));
    game_state.renderer.set_global_uniform("view", camera);

    game_state.add_system(Event::Init, load_world);
    game_state.add_system(Event::Tick, player_input);
    game_state.add_system(Event::Tick, track_time);
    game_state.add_system(Event::Tick, focus_window);
    // game_state.add_system(Event::Tick, cursor_lock);

    game_state.add_system(Event::PlayerMoved, update_camera);
    game_state.add_system(Event::Resized, update_perspective);

    game_state.queue_event(Event::Init);
    game_state
}

fn cursor_lock(
    renderer: &mut Renderer,
    input: &mut Input,
    data: &mut GameData,
    queue: &mut Vec<Event>,
    delta: f64,
) {
    println!("{}, {}", input.movement.0, input.movement.1);
    /*
    let window = &window_state().window;
    window
        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
        .unwrap_or_default();
    window.set_cursor_visible(false);
    */
}

fn track_time(
    renderer: &mut Renderer,
    input: &mut Input,
    data: &mut GameData,
    queue: &mut Vec<Event>,
    delta: f64,
) {
    data.time = data.time + delta;
    data.frames = data.frames + 1.0;

    if data.time > 1000.0 {
        println!("{}", 1000.0 * data.frames / data.time);
        data.frames = 0.0;
        data.time = 0.0;
        let (x, y, z) = data.player.position;
        println!("player: {}, {}, {}", x, y, z);
    }
}
