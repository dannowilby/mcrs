use priomutex::Mutex;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;

use crate::chunk::block::{Block, BlockDictionary};
use crate::chunk::cube_model::cube_model;
use crate::chunk::loading::check_done_load_world;
use crate::chunk::loading::load_world;
use crate::chunk::meshing;
use crate::chunk::ChunkConfig;
use crate::chunk::ChunkData;
use crate::chunk::ChunkStorage;
use crate::chunk::Position;
use crate::engine::game_state::GameState;
use crate::engine::input::Input;
use crate::engine::matrix::Matrix;
use crate::engine::render_group::RenderGroupBuilder;
use crate::engine::render_object::RenderObject;
use crate::engine::renderer::Renderer;
use crate::engine::resources::load_string;
use crate::engine::texture;

use crate::physics::simulate_physics;
use crate::physics::PhysicsEngine;
use crate::player::init_player;
use crate::player::simulate_player;
use crate::player::{focus_window, player_input, update_camera, update_perspective, Player};
use crate::window_state;

use rapier3d::prelude::*;

#[derive(PartialEq, Eq, Hash)]
pub enum Event {
    Init,
    Tick,

    Resized,

    PlayerMoved,
}

pub struct GameData {
    // component data

    // chunks
    pub loaded_chunks: ChunkStorage,

    pub loading: HashSet<String>,
    pub done_loading: Arc<Mutex<HashMap<String, (Position, ChunkData, RenderObject, Collider)>>>,

    // physics
    pub physics_engine: PhysicsEngine,

    pub thread_pool: rayon::ThreadPool,

    // singleton data
    pub chunk_config: Arc<ChunkConfig>,
    pub player: Player,
    time: f64,
    frames: f64,
    pub focused: bool,
}

use libnoise::prelude::*;

pub async fn init() -> GameState<GameData, Event> {
    let seed = 123456789;
    let mut game_state = GameState::new(
        Renderer::new(),
        GameData {
            // chunk_data: HashMap::new(),
            loaded_chunks: ChunkStorage::new(),
            loading: HashSet::new(),
            done_loading: Arc::new(Mutex::new(HashMap::new())),

            physics_engine: PhysicsEngine::new(),

            thread_pool: rayon::ThreadPoolBuilder::new()
                .num_threads(16)
                .build()
                .unwrap(),

            chunk_config: Arc::new(ChunkConfig {
                noise: Source::simplex(seed) // start with simplex noise
                    .fbm(5, 0.013, 2.0, 0.5) // apply fractal brownian motion
                    .blend(
                        // apply blending...
                        Source::worley(43).scale([0.05, 0.05, 0.05]), // ...with scaled worley noise
                        Source::worley(44).scale([0.02, 0.02, 0.02]),
                    ) // ...controlled by other worley noise
                    .lambda(|f| (f * 2.0).sin() * 0.3 + f * 0.7), // apply a closure to the noise Source::worley(123), //Arc.fbm(3, 0.013, 2.0, 0.5); // ::new(Worley::new(0)), // |[x, y, z]| f64::sin(x) + f64::sin(y) + f64::sin(z),
                noise_amplitude: (0.5, 0.5, 0.5),
                depth: 32,

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
    game_state.add_system(Event::Init, init_player);
    game_state.add_system(Event::Tick, player_input);
    game_state.add_system(Event::Tick, track_time);
    game_state.add_system(Event::Tick, focus_window);
    // game_state.add_system(Event::Tick, cursor_lock);
    game_state.add_system(Event::Tick, simulate_player);

    game_state.add_system(Event::PlayerMoved, update_camera);
    game_state.add_system(Event::PlayerMoved, load_world);
    game_state.add_system(Event::Tick, check_done_load_world);
    game_state.add_system(Event::Resized, update_perspective);
    // game_state.add_system(Event::Tick, mesh_chunks);

    game_state.queue_event(Event::Init);
    game_state
}

fn track_time(
    _renderer: &mut Renderer,
    _input: &mut Input,
    data: &mut GameData,
    _queue: &mut Vec<Event>,
    delta: f64,
) {
    data.time = data.time + delta;
    data.frames = data.frames + 1.0;

    if data.time > 1000.0 {
        println!("{}", 1000.0 * data.frames / data.time);
        data.frames = 0.0;
        data.time = 0.0;
        let pos = data.physics_engine.get_rigid_body("player".to_string()).unwrap().translation();
        // let (x, y, z) = data.player.position;
        println!("player: {}, {}, {}", pos.x, pos.y, pos.z);
        println!("Num chunks loading: {}", data.loading.len());
    }
}
