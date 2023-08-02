use std::collections::HashMap;
use wgpu::VertexBufferLayout;

use super::super::engine::render_object::RenderObject;
use super::block::{Block, BlockDictionary};
use super::LOD;
use super::{get_block, ChunkData};
use super::{ChunkConfig, ChunkPos};

#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct Vertex {
    position: [f32; 3],
    uv: [f32; 2],
    ao: f32,
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

pub fn mesh_chunk(
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    config: &ChunkConfig,
    chunk_pos: &ChunkPos,
    chunk_data: &ChunkData,
    lod: LOD,
) -> RenderObject {
    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::<u16>::new();

    // loop over all blocks in chunk
    for (position, block_id) in chunk_data.iter() {
        // ideally, we want to change the mesh/vertex attributes
        // based on the surrounding blocks
        let block_world_position = (
            chunk_pos.0 * config.depth + position.0,
            chunk_pos.1 * config.depth + position.1,
            chunk_pos.2 * config.depth + position.2,
        );

        let block = dict.get(block_id);
        let model = block.map(|b| b.model).unwrap_or(Block::default().model);
        let (mut verts, mut inds) = model(config, loaded_chunks, &block_world_position);
        vertices.append(&mut verts);
        indices.append(&mut inds);

        // calc faces to add to mesh

        // let visible_block_faces = get_visible_block_faces(position, block_id, chunk_data);
    }

    RenderObject::new(
        "chunk_bind_group",
        bytemuck::cast_slice(vertices.as_slice()),
        bytemuck::cast_slice(indices.as_slice()),
    )
}

// return the faces (vertices, indices) for the block
pub fn cube_model(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    position: &(u32, u32, u32),
) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let (p0, p1, p2) = position;

    let tb = dict
        .get(&get_block(chunk_config, loaded_chunks, &(*p0, p1 + 1, *p2)))
        .map_or(true, |b| b.transparent);
    if tb {
        // vertices.append();
        // indices.append()
    }

    (vertices, indices)
}
