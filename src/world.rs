use priomutex::Mutex;
use std::collections::HashSet;
use std::sync::Arc;

use crate::chunk::block::{Block, BlockDictionary};
use crate::chunk::cube_model::cube_model;
use crate::chunk::culling::VisibilityGraph;
use crate::chunk::culling::VisibilityGraphStorage;
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
use crate::engine::render::render_group::RenderGroupBuilder;
use crate::engine::render::render_object::RenderObject;
use crate::engine::resources::load_string;
use crate::engine::texture;

use crate::physics::PhysicsEngine;
use crate::player::create_player;
use crate::player::player_changed_chunk;
use crate::player::simulate_player;
use crate::player::{focus_window, player_input, update_camera, update_perspective, Player};
use crate::window_state;
use crate::world_renderer::toggle_debug_menu;
use crate::world_renderer::WorldRenderer;

use rapier3d::prelude::*;

#[derive(PartialEq, Eq, Hash)]
pub enum Event {
    Init,
    Tick,

    Resized,

    PlayerMoved,
    PlayerChunkChanged,
}

pub struct GameData {
    // component data
    pub show_debug_menu: bool,

    // chunks
    pub loaded_chunks: ChunkStorage,
    pub visibility_graphs: VisibilityGraphStorage,

    pub loading: HashSet<String>,
    pub done_loading: Arc<
        Mutex<
            indexmap::IndexMap<
                String,
                (Position, ChunkData, VisibilityGraph, RenderObject, Collider),
            >,
        >,
    >,

    // physics
    pub physics_engine: PhysicsEngine,

    pub thread_pool: rayon::ThreadPool,

    // singleton data
    pub chunk_config: Arc<ChunkConfig>,
    pub player: Player,
    time: f64,
    frames: f64,
    average_fps: f64,
    total_frames: f64,
    pub drawn_chunks: u64,
    pub chunks_removed_by_visibility: u64,
    pub amount_of_culled_blocks: u64,
    pub focused: bool,
}

use libnoise::prelude::*;

pub async fn init() -> GameState<GameData, WorldRenderer, Event> {
    let seed = 123456789;
    let frame_shader_source = load_string("frame.wgsl", true).await.unwrap();
    let mut game_state = GameState::new(
        WorldRenderer::new(&frame_shader_source),
        GameData {
            show_debug_menu: false,

            // chunk_data: HashMap::new(),
            loaded_chunks: ChunkStorage::new(),
            visibility_graphs: VisibilityGraphStorage::new(),

            loading: HashSet::new(),
            done_loading: Arc::new(Mutex::new(indexmap::IndexMap::new())),

            physics_engine: PhysicsEngine::new(),

            thread_pool: rayon::ThreadPoolBuilder::new()
                // .num_threads(2)
                .build()
                .unwrap(),

            chunk_config: Arc::new(ChunkConfig {
                noise: Source::simplex(seed), // apply a closure to the noise Source::worley(123), //Arc.fbm(3, 0.013, 2.0, 0.5); // ::new(Worley::new(0)), // |[x, y, z]| f64::sin(x) + f64::sin(y) + f64::sin(z),
                noise_amplitude: (0.005, 0.005, 0.005),
                depth: 32,
                load_radius: 4,

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
            drawn_chunks: 0,
            chunks_removed_by_visibility: 0,
            amount_of_culled_blocks: 0,
            time: 0.0,
            frames: 0.0,
            average_fps: 0.0,
            total_frames: 0.0,
            focused: false,
        },
    );

    let shader_source = load_string("chunk.wgsl", true)
        .await
        .expect("error loading shader... :(");
    game_state.renderer.chunk_render_pass.render_groups.insert(
        "chunk_render_group".to_string(),
        RenderGroupBuilder::new()
            .with("projection", Matrix::create_layout(0))
            .with("view", Matrix::create_layout(1))
            .with("model", Matrix::create_layout(2))
            .with("texture_atlas", texture::Texture::create_layout(3))
            .vertex_format(meshing::Vertex::description())
            .shader(&shader_source)
            .build(true),
    );

    let texture_uniform = texture::Texture::load("texture_atlas.png").await;
    game_state.renderer.chunk_render_pass.uniforms.insert(
        "texture_atlas".to_string(),
        texture_uniform.uniform(&texture::Texture::create_layout(3)),
    );

    let config = &window_state().config;
    let projection = glam::Mat4::perspective_rh_gl(
        game_state.data.player.fov,
        config.width as f32 / config.height as f32,
        0.1,
        1000.0,
    );
    let proj = Matrix::new(projection).uniform(&Matrix::create_layout(0));
    game_state
        .renderer
        .chunk_render_pass
        .uniforms
        .insert("projection".to_string(), proj);

    let camera = Matrix::new(glam::Mat4::IDENTITY).uniform(&Matrix::create_layout(1));
    game_state
        .renderer
        .chunk_render_pass
        .uniforms
        .insert("view".to_string(), camera);

    // load player
    // create_player(&mut game_state.data, &(-15, 10, 0));
    create_player(&mut game_state.data, &(0, 10, 0));
    update_camera(
        &mut game_state.renderer,
        &mut game_state.input,
        &mut game_state.data,
        &mut vec![],
        0.0,
    );

    game_state.add_system(Event::Init, load_world);
    game_state.add_system(Event::Tick, player_input);
    game_state.add_system(Event::Tick, debug);
    // game_state.add_system(Event::Tick, visibility_cull);
    game_state.add_system(Event::Tick, focus_window);
    game_state.add_system(Event::Tick, toggle_debug_menu);
    // game_state.add_system(Event::Tick, cursor_lock);
    game_state.add_system(Event::Tick, simulate_player);
    game_state.add_system(Event::PlayerMoved, update_camera);
    game_state.add_system(Event::PlayerMoved, player_changed_chunk);

    game_state.add_system(Event::PlayerChunkChanged, load_world);
    game_state.add_system(Event::Tick, check_done_load_world);
    game_state.add_system(Event::Resized, update_perspective);
    // game_state.add_system(Event::Tick, mesh_chunks);

    game_state.queue_event(Event::Init);
    game_state
}

fn debug(
    renderer: &mut WorldRenderer,
    _input: &mut Input,
    data: &mut GameData,
    _queue: &mut Vec<Event>,
    delta: f64,
) {
    data.total_frames += 1.0;
    data.time = data.time + delta;
    data.frames = data.frames + 1.0;

    let fps = 1000.0 * data.frames / data.time;
    data.average_fps *= (data.total_frames - 1.0) / data.total_frames;
    data.average_fps += fps / data.total_frames;

    if data.show_debug_menu {
        let d = delta.clone();
        renderer.imgui_render_pass.windows.push(Box::new(
            move |ui: &mut imgui::Ui, game_data: &mut GameData| {
                ui.window("Player State")
                    .size([400.0, 200.0], imgui::Condition::FirstUseEver)
                    .build(|| {
                        let pos = game_data
                            .physics_engine
                            .get_rigid_body("player".to_string())
                            .unwrap()
                            .translation();
                        ui.text(format!("Player position: {}, {}, {}", pos.x, pos.y, pos.z));
                        ui.text(format!(
                            "Is player colliding: {}",
                            game_data.physics_engine.is_colliding("player")
                        ));
                        ui.checkbox("Flying", &mut game_data.player.is_flying);
                        ui.slider(
                            "Player jump power: ",
                            0.0,
                            50.0,
                            &mut game_data.player.max_jump,
                        );
                        ui.slider(
                            "Player gravity: ",
                            -15.0,
                            0.0,
                            &mut game_data.physics_engine.gravity.y,
                        );
                        ui.slider("Player speed:", 0.0, 15.0, &mut game_data.player.move_speed);
                    });
                ui.window("Statistics")
                    .size([400.0, 200.0], imgui::Condition::FirstUseEver)
                    .position([0.0, 500.0], imgui::Condition::FirstUseEver)
                    .build(|| {
                        let fps = 1000.0 * game_data.frames / game_data.time;
                        ui.text(format!("FPS: {}", fps));
                        ui.text(format!("Average FPS: {}", game_data.average_fps));
                        ui.text(format!("Frame delta: {}", d));
                        ui.text(format!(
                            "Chunks drawn this frame: {}",
                            game_data.drawn_chunks
                        ));
                        ui.text(format!(
                            "Num chunks removed because of visibility: {}",
                            game_data.chunks_removed_by_visibility
                        ));
                        ui.text(format!(
                            "Num loaded chunks: {}",
                            game_data.loaded_chunks.len()
                        ));
                        ui.text(format!(
                            "Number of chunks currently loading: {}",
                            game_data.loading.len()
                        ));
                    });
            },
        ));
    }

    // update fps counter variables
    if data.time > 1000.0 {
        data.frames = 0.0;
        data.time = 0.0;
    }
}
