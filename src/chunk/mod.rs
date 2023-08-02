use noise::NoiseFn;
use std::collections::HashMap;

pub mod block;
pub mod generation;
pub mod meshing;

pub type ChunkPos = (u32, u32, u32);
pub type ChunkData = HashMap<ChunkPos, u32>;

pub struct ChunkConfig {
    // initialized noise function
    // height bias
    // squish bias
    pub noise: Box<dyn NoiseFn<f64, 3>>, // chunk size?
    pub noise_amplitude: (f64, f64, f64),
    pub depth: u32,
}

pub fn get_block(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    raw_position: &(u32, u32, u32),
) -> u32 {
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
