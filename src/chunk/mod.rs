//! Module for all things involving chunks,
//! is just a prototype.

#![allow(dead_code)]

use std::collections::HashMap;

pub mod block;
pub mod chunk_renderer;
pub mod collision;
pub mod cube_model;
pub mod culling;
pub mod generation;
pub mod loading;
pub mod meshing;
use block::BlockDictionary;

/// We load chunks by an area of
/// depth + 2 * depth + 2 * depth + 2
/// but then only mesh the inside of the chunk
/// ie. depth * depth * depth
/// so that we can generate the correct chunk borders for our mesh
/// this also permits us to use a fairly parallelizable method of
/// chunk generation

pub type Position = (i32, i32, i32);
pub type ChunkData = HashMap<Position, u32>;
pub type ChunkStorage = HashMap<String, ChunkData>;

use libnoise::prelude::*;

use self::{block::Block, cube_model::cube_model};

pub struct ChunkConfig {
    // initialized noise function
    // height bias
    // squish bias
    pub noise: libnoise::Simplex<3>, // fn([f64; 3]) -> f64, //Arc<dyn NoiseFn<f64, 3> + Send + Sync>, // fn([f64; 3]) -> f64, // chunk size?
    pub noise_amplitude: (f64, f64, f64),
    pub depth: i32,

    pub uv_size: f32,
    pub load_radius: u32,

    pub dict: BlockDictionary,
}

impl ChunkConfig {
    #[allow(dead_code)]
    fn new(seed: u32, depth: i32, load_radius: u32) -> Self {
        Self {
            noise: Simplex::new(seed as u64),
            depth,
            load_radius,
            uv_size: 0.0625,
            noise_amplitude: (0.001, 0.01, 0.001),
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
        }
    }
}

/// convert player float tuple to i32 tuple
/// ie. a player with position (xf32, yf32, zf32) -> (xi32, yi32, zi32)
pub fn player_to_position(position: &(f32, f32, f32)) -> Position {
    (
        position.0.floor() as i32,
        position.1.floor() as i32,
        position.2.floor() as i32,
    )
}

#[allow(dead_code)]
/// convert a world space block pos tuple into the chunk local block pos tuple
pub fn local_position(chunk_config: &ChunkConfig, pos: &Position) -> Position {
    (
        local_block_pos(chunk_config, pos.0),
        local_block_pos(chunk_config, pos.1),
        local_block_pos(chunk_config, pos.2),
    )
}

/// convert a world space block pos into the chunk local block pos
pub fn local_block_pos(chunk_config: &ChunkConfig, pos: i32) -> i32 {
    let size = chunk_config.depth;
    ((pos % size) + size) % size
}

/// convert a world space position tuple to get a chunk pos tuple
pub fn chunk_position(chunk_config: &ChunkConfig, pos: &Position) -> Position {
    (
        global_chunk_pos(chunk_config, pos.0),
        global_chunk_pos(chunk_config, pos.1),
        global_chunk_pos(chunk_config, pos.2),
    )
}

/// convert a world space position to get a chunk pos
pub fn global_chunk_pos(chunk_config: &ChunkConfig, pos: i32) -> i32 {
    (pos as f32 / chunk_config.depth as f32).floor() as i32
}

/// take a position and return a chunk_id
pub fn chunk_id(pos: &Position) -> String {
    format!("chunk,{},{},{}", pos.0, pos.1, pos.2)
}

pub fn chunk_pos_from_id(chunk_id: &String) -> Position {
    let mut pos = (0, 0, 0);

    let mut c = 0;
    for part in chunk_id.split(",") {
        if c == 0 {
            c += 1;
            continue;
        }

        if c == 1 {
            pos.0 = part.parse().unwrap();
        }
        if c == 2 {
            pos.1 = part.parse().unwrap();
        }
        if c == 3 {
            pos.2 = part.parse().unwrap();
        }
        c += 1;
    }

    pos
}

#[allow(dead_code)]
pub fn get_block(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    raw_position: &Position,
) -> u32 {
    let chunk_pos = chunk_id(&chunk_position(chunk_config, raw_position));
    let chunk_query = loaded_chunks.get(&chunk_pos);
    if let Some(chunk_data) = chunk_query {
        let block_pos = local_position(chunk_config, raw_position);
        let block = chunk_data.get(&block_pos);

        if let Some(block_id) = block {
            return *block_id;
        }
    }

    0
}

// Level of detail
// how many blocks comprise the chunk's mesh
// max level of detail is used as the chunk size
pub struct LOD(u32);
impl LOD {
    const MIN: LOD = LOD(4);
    const MED: LOD = LOD(8);
    const MAX: LOD = LOD(16);
}

// Level of detail

pub fn calc_lod() -> LOD {
    LOD::MAX
}
