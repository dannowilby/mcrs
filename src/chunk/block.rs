use std::collections::HashMap;

use crate::chunk::meshing::Vertex;
use crate::chunk::ChunkData;

use super::{ChunkConfig, Position};

pub type BlockModel = fn(&ChunkData, &ChunkConfig, &Position, &mut Vec<Vertex>, &mut Vec<u16>);

pub struct Block {
    pub model: BlockModel,
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
