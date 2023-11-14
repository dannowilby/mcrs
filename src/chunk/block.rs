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

/// We provide two different implementations of the default block so we can
/// have a default for the Block and &Block types

const CONST_DEFAULT_BLOCK: Block = Block {
    model: |_, _, _, _, _| {},
    transparent: true,
    ident: String::new(),
    uv: [0.0, 0.0],
};

static STATIC_DEFAULT_BLOCK: Block = Block {
    model: |_, _, _, _, _| {},
    transparent: true,
    ident: String::new(),
    uv: [0.0, 0.0],
};

impl Default for Block {
    fn default() -> Self {
        CONST_DEFAULT_BLOCK
    }
}

impl Default for &Block {
    fn default() -> Self {
        &STATIC_DEFAULT_BLOCK
    }
}

pub type BlockDictionary = HashMap<u32, Block>;
