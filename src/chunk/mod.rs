use noise::NoiseFn;
use std::collections::HashMap;

pub mod block;
pub mod cube_model;
pub mod generation;
pub mod meshing;
use block::BlockDictionary;

pub type ChunkPos = (i32, i32, i32);
pub type ChunkData = HashMap<ChunkPos, u32>;

pub struct ChunkConfig {
    // initialized noise function
    // height bias
    // squish bias
    pub noise: fn([f64; 3]) -> f64, // chunk size?
    pub noise_amplitude: (f64, f64, f64),
    pub depth: i32,

    pub uv_size: f32,

    pub dict: BlockDictionary,
}

fn get_local_block_pos(chunk_config: &ChunkConfig, pos: i32) -> i32 {
    let size = chunk_config.depth;
    ((pos % size) + size) % size
}

pub fn get_chunk_pos(chunk_config: &ChunkConfig, pos: i32) -> i32 {
    (pos as f32 / chunk_config.depth as f32).floor() as i32
}

pub fn chunk_id(x: i32, y: i32, z: i32) -> String {
    format!("chunk-{}-{}-{}", x, y, z)
}

pub fn is_transparent(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    position: &(i32, i32, i32),
) -> bool {
    let (p0, p1, p2) = position;
    dict.get(&get_block(chunk_config, loaded_chunks, &(*p0, p1 + 1, *p2)))
        .map_or(true, |b| b.transparent)
}

pub fn get_block(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    raw_position: &(i32, i32, i32),
) -> u32 {
    let chunk_pos = chunk_id(
        get_chunk_pos(chunk_config, raw_position.0),
        get_chunk_pos(chunk_config, raw_position.1),
        get_chunk_pos(chunk_config, raw_position.2),
    );
    let chunk_query = loaded_chunks.get(&chunk_pos);
    if let Some(chunk_data) = chunk_query {
        let block_pos = (
            get_local_block_pos(chunk_config, raw_position.0),
            get_local_block_pos(chunk_config, raw_position.1),
            get_local_block_pos(chunk_config, raw_position.2),
        );
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
