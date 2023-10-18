use wgpu::VertexBufferLayout;

use super::super::engine::render_object::RenderObject;
use super::{chunk_id, LOD, Position, ChunkData, ChunkConfig};

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

pub fn should_mesh_block(config: &ChunkConfig, pos: &Position) -> bool {
    if pos.0 < 0 || pos.1 < 0 || pos.2 < 0 || pos.0 >= config.depth || pos.1 >= config.depth || pos.2 >= config.depth {
        return false;
    }
    true
}

// see what's taking so long in this function/and why
pub fn mesh_chunk(
    chunk: &ChunkData,
    config: &ChunkConfig,
    chunk_pos: &Position,
    lod: LOD,
) -> RenderObject {
    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::<u16>::new();

    let chunk_id = chunk_id(&chunk_pos);

    // loop over all blocks stored
    for (position, block_id) in chunk.iter() {
        
        // check for the boundary that we generated
        // and don't want to mesh
        if !should_mesh_block(config, position) {
            continue;
        }
        
        
        let block = config.dict.get(block_id);
        let model = block.unwrap_or(config.dict.get(&0).unwrap()).model;
        model(
            &chunk,
            config,
            position,
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
