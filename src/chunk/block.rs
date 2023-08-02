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
        &(u32, u32, u32),
    ) -> (Vec<Vertex>, Vec<u16>),
    pub transparent: bool,
    ident: String,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            model: |_, _, _, _| (Vec::new(), Vec::new()),
            transparent: true,
            ident: "air".to_owned(),
        }
    }
}

pub type BlockDictionary = HashMap<u32, Block>;
