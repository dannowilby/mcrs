use std::collections::HashMap;
use std::sync::RwLock;
use wgpu::VertexBufferLayout;

use super::super::engine::render_object::RenderObject;
use super::block::{Block, BlockDictionary};
use super::{chunk_id, LOD};
use super::{get_block, ChunkData};
use super::{ChunkConfig, ChunkPos};

#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub uv: [f32; 2],
    pub ao: f32,
}

impl Vertex {
    pub fn description<'a>() -> VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 5]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32,
                },
            ],
        }
    }
}

// see what's taking so long in this function/and why
pub fn mesh_chunk(
    loaded_chunks: &HashMap<String, ChunkData>,
    config: &ChunkConfig,
    chunk_pos: &ChunkPos,
    lod: LOD,
) -> RenderObject {
    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::<u16>::new();

    // loop over all blocks in chunk
    let chunk_id = chunk_id(chunk_pos.0, chunk_pos.1, chunk_pos.2);
    for (position, block_id) in loaded_chunks.get(&chunk_id).unwrap().iter() {
        // ideally, we want to change the mesh/vertex attributes
        // based on the surrounding blocks
        let block_world_position = (
            chunk_pos.0 * config.depth + position.0,
            chunk_pos.1 * config.depth + position.1,
            chunk_pos.2 * config.depth + position.2,
        );

        let block = config.dict.get(block_id);
        let model = block.unwrap_or(config.dict.get(&0).unwrap()).model;
        model(
            loaded_chunks,
            config,
            &block_world_position,
            &mut vertices,
            &mut indices,
        );
    }

    RenderObject::new(
        "chunk_render_group",
        bytemuck::cast_slice(vertices.as_slice()),
        bytemuck::cast_slice(indices.as_slice()),
    )
}
