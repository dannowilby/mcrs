use noise::{Simplex, Worley};
use rayon::Scope;
use rayon::ThreadPoolBuilder;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use winit::event::VirtualKeyCode;

use crate::chunk::block::{Block, BlockDictionary};
use crate::chunk::chunk_id;
use crate::chunk::cube_model::cube_model;
use crate::chunk::get_chunk_pos;
use crate::chunk::meshing;
use crate::chunk::meshing::mesh_chunk;
use crate::chunk::{calc_lod, ChunkConfig, ChunkData};
use crate::engine::game_state::{self, GameState};
use crate::engine::input::Input;
use crate::engine::matrix::Matrix;
use crate::engine::render_group::RenderGroupBuilder;
use crate::engine::render_object::RenderObject;
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
    loaded_chunks: Arc<RwLock<HashMap<String, ChunkData>>>,

    // singleton data
    chunk_config: Arc<ChunkConfig>,
    pub player: Player,
    time: f64,
    frames: f64,
    pub focused: bool,
}

// TODO:
// - offload chunk loading onto a separate thread
// - improve terrain generation
//   - be able to query for a block position, just for the terrain
// - maybe create new event for when a player moves chunk to load
// TODO (stetch):
// - Frustrum culling
// - Occulsion culling
fn load_world(
    renderer: Arc<RwLock<Renderer>>,
    input: &mut Input,
    data: &mut GameData,
    queue: &mut Vec<Event>,
    delta: f64,
) {
    // chunk loading dimensions
    let (player_i, player_j, player_k) = data.player.position;
    let i = get_chunk_pos(&data.chunk_config, player_i.floor() as i32);
    let j = get_chunk_pos(&data.chunk_config, player_j.floor() as i32);
    let k = get_chunk_pos(&data.chunk_config, player_k.floor() as i32);
    let radius = data.player.load_radius as i32;

    let mut chunks_to_load = Vec::new();
    let mut chunks_to_remove: Vec<String> = data
        .loaded_chunks
        .read()
        .unwrap()
        .iter()
        .map(|(k, v)| k.clone())
        .collect();

    // calculate chunks to modify
    for x in (i - radius)..(i + radius) {
        for y in (j - radius)..(j + radius) {
            for z in (k - radius)..(k + radius) {
                let chunk_id = chunk_id(x, y, z);

                let index = chunks_to_remove.iter().position(|r| r == &chunk_id);
                if let Some(x) = index {
                    chunks_to_remove.swap_remove(x);
                }

                // if loaded chunks doesn't contain it, but it should
                if !data.loaded_chunks.read().unwrap().contains_key(&chunk_id) {
                    chunks_to_load.push((chunk_id, (x, y, z)));
                }
            }
        }
    }

    let thread_pool = ThreadPoolBuilder::new().num_threads(16).build().unwrap();

    // generate all the chunks
    thread_pool.scope(|scope| {
        for (chunk_id, chunk_pos) in &chunks_to_load {
            let config = data.chunk_config.clone();
            let loaded_chunks = data.loaded_chunks.clone();
            scope.spawn(move |_| {
                // load chunk
                let chunk = load_chunk(&config, &chunk_pos);
                loaded_chunks
                    .write()
                    .unwrap()
                    .insert(chunk_id.clone(), chunk);
            });
        }
    });

    // mesh all the chunks
    thread_pool.scope(|scope| {
        for (chunk_id, chunk_pos) in &chunks_to_load {
            let config = data.chunk_config.clone();
            let loaded_chunks = data.loaded_chunks.clone();
            let renderer = renderer.clone();
            scope.spawn(move |_| {
                // meshing takes to long
                // need to make faster
                let mesh = mesh_chunk(
                    &loaded_chunks.read().unwrap(),
                    &config,
                    &chunk_pos,
                    calc_lod(),
                );

                // add mesh to renderer
                let mat = Matrix::new(glam::Mat4::from_translation(glam::f32::vec3(
                    0.0, //x as f32 * data.chunk_config.depth as f32,
                    0.0, //y as f32 * data.chunk_config.depth as f32,
                    0.0, //z as f32 * data.chunk_config.depth as f32,
                )))
                .uniform(&Matrix::create_layout(2));
                renderer.write().unwrap().add_object(&chunk_id, mesh);
                renderer
                    .write()
                    .unwrap()
                    .set_object_uniform(&chunk_id, "model", mat);
            })
        }
    });

    // remove unneeded chunks
    for c in chunks_to_remove {
        data.loaded_chunks.write().unwrap().remove(&c);
        renderer.write().unwrap().remove_object(&c);
    }
}

pub async fn init() -> GameState<GameData, Event> {
    let seed = 123456789;
    let mut game_state = GameState::new(
        Renderer::new(),
        GameData {
            // chunk_data: HashMap::new(),
            loaded_chunks: Default::default(),

            chunk_config: Arc::new(ChunkConfig {
                noise: |[x, y, z]| f64::sin(x) + f64::sin(y) + f64::sin(z),
                noise_amplitude: (0.25, 0.25, 0.25),
                depth: 16,

                uv_size: 0.0625,
                dict: BlockDictionary::from([
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
                    (
                        3,
                        Block {
                            model: cube_model,
                            transparent: false,
                            ident: "dirt".to_owned(),
                            uv: [0.125, 0.0],
                        },
                    ),
                ]),
            }),

            player: Player::new(),
            time: 0.0,
            frames: 0.0,
            focused: false,
        },
    );

    let shader_source = load_string("chunk.wgsl")
        .await
        .expect("error loading shader... :(");
    game_state.renderer.write().unwrap().create_group(
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
    game_state.renderer.write().unwrap().set_global_uniform(
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
    game_state
        .renderer
        .write()
        .unwrap()
        .set_global_uniform("projection", proj);

    let camera = Matrix::new(glam::Mat4::IDENTITY).uniform(&Matrix::create_layout(1));
    game_state
        .renderer
        .write()
        .unwrap()
        .set_global_uniform("view", camera);

    game_state.add_system(Event::Init, load_world);
    game_state.add_system(Event::Tick, player_input);
    game_state.add_system(Event::Tick, track_time);
    game_state.add_system(Event::Tick, focus_window);
    // game_state.add_system(Event::Tick, cursor_lock);

    game_state.add_system(Event::PlayerMoved, update_camera);
    game_state.add_system(Event::PlayerMoved, load_world);
    game_state.add_system(Event::Resized, update_perspective);

    game_state.queue_event(Event::Init);
    game_state
}

fn track_time(
    renderer: Arc<RwLock<Renderer>>,
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
