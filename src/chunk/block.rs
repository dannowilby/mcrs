use std::collections::HashMap;
use std::sync::RwLock;

use crate::chunk::meshing::Vertex;
use crate::chunk::ChunkData;

use super::ChunkConfig;

pub struct Block {
    // will take in a raw position and chunk data
    pub model: fn(
        &HashMap<String, ChunkData>,
        &ChunkConfig,
        &(i32, i32, i32),
        &mut Vec<Vertex>,
        &mut Vec<u16>,
    ),
    pub transparent: bool,
    pub ident: String,
    pub uv: [f32; 2],
}

impl Default for Block {
    fn default() -> Self {
        Block {
            model: |_, _, _, _, _| {},
            transparent: true,
            ident: "air".to_owned(),
            uv: [0.0, 0.0],
        }
    }
}

pub type BlockDictionary = HashMap<u32, Block>;
