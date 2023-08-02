use noise::Simplex;
use std::collections::HashMap;

use crate::chunk::block::{Block, BlockDictionary};
use crate::chunk::meshing::mesh_chunk;
use crate::chunk::meshing::{self, cube_model};
use crate::chunk::{calc_lod, ChunkConfig, ChunkData};
use crate::engine::game_state::GameState;
use crate::engine::matrix::Matrix;
use crate::engine::render_group::RenderGroupBuilder;
use crate::engine::renderer::Renderer;
use crate::engine::texture;

use crate::chunk::generation::load_chunk;

#[derive(PartialEq, Eq, Hash)]
pub enum Event {
    Init,
    Tick,
}

pub struct GameData {
    // component data
    loaded_chunks: HashMap<String, ChunkData>,

    // singleton data
    chunk_config: ChunkConfig,
    block_dictionary: BlockDictionary,
    player_position: (u32, u32, u32),
    load_radius: u32,
    pub delta: f64,
}

// pseudocode atm
fn load_world(renderer: &mut Renderer, data: &mut GameData, queue: &mut Vec<Event>) {
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

    let chunk_pos = (0, 0, 0);
    let chunk_id = format!("chunk-{}-{}-{}", 0, 0, 0);
    let lod = calc_lod();

    let chunk = load_chunk(&data.chunk_config, chunk_pos);
    let mesh = mesh_chunk(
        &data.loaded_chunks,
        &data.block_dictionary,
        &data.chunk_config,
        &chunk_pos,
        &chunk,
        lod,
    );

    let mat = Matrix::new(glam::Mat4::IDENTITY).uniform(&Matrix::create_layout(0));
    renderer
        .add_object(&chunk_id, mesh)
        .set_uniform("model", mat);

    data.loaded_chunks.insert(chunk_id, chunk);
}

fn print_delta(renderer: &mut Renderer, data: &mut GameData, queue: &mut Vec<Event>) {
    println!("{}ms per frame", data.delta);
    // println!("{}fps", 1000 as f64 / data.delta);
}

pub fn init() -> GameState<GameData, Event> {
    let seed = 123456789;
    let mut game_state = GameState::new(
        Renderer::new(),
        GameData {
            // chunk_data: HashMap::new(),
            loaded_chunks: Default::default(),

            chunk_config: ChunkConfig {
                noise: Box::new(Simplex::new(seed)),
                noise_amplitude: (1.0, 1.0, 1.0),
                depth: 16,
            },

            block_dictionary: BlockDictionary::from([
                (0, Block::default()),
                (
                    1,
                    Block {
                        model: cube_model,
                        transparent: false,
                        ident: "stone".to_owned(),
                    },
                ),
            ]),

            player_position: (0, 0, 0),
            load_radius: 3,

            delta: 0f64,
        },
    );

    game_state.renderer.create_group(
        "chunk_render_group",
        RenderGroupBuilder::new()
            .with("camera", Matrix::create_layout(0))
            .with("model", Matrix::create_layout(1))
            .with("texture_atlas", texture::Texture::create_layout(2))
            .vertex_format(meshing::Vertex::description())
            .shader("chunk.wgsl")
            .build(),
    );

    game_state.add_system(Event::Init, load_world);
    // game_state.add_system(Event::Tick, print_delta);

    game_state
}
