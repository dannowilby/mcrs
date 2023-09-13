use std::collections::HashMap;

use crate::chunk::meshing::Vertex;
use crate::chunk::ChunkData;

use super::ChunkConfig;

pub struct Block {
    // will take in a raw position and chunk data
    pub model: fn(
        &ChunkConfig,
        &HashMap<String, ChunkData>,
        &BlockDictionary,
        &(i32, i32, i32),
        u16,
    ) -> (Vec<Vertex>, Vec<u16>),
    pub transparent: bool,
    pub ident: String,
    pub uv: [f32; 2],
}

impl Default for Block {
    fn default() -> Self {
        Block {
            model: |_, _, _, _, _| (Vec::new(), Vec::new()),
            transparent: true,
            ident: "air".to_owned(),
            uv: [0.0, 0.0],
        }
    }
}

pub type BlockDictionary = HashMap<u32, Block>;
