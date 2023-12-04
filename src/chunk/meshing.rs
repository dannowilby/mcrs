use wgpu::VertexBufferLayout;

use super::super::engine::render::render_object::RenderObject;
use super::{ChunkConfig, ChunkData, Position, LOD};

/// 7 bits for each position, 4 bits for each texture, and 3 bits for lighting
#[derive(Copy, Clone, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct Vertex {
    pub data: u32,
}

impl Vertex {
    /// Packs floats into the vertex format.    
    pub fn from(position: [f32; 3], uv: [f32; 2], ao: f32) -> Self {
        let mut vertex: u32 = 0x0000_0000;
        vertex |= (position[0].floor() as u32).rotate_left(25) & 0xFE00_0000; // & 0b0111_1100_0000_0000;
        vertex |= (position[1].floor() as u32).rotate_left(18) & 0x01FC_0000; // & 0b0000_0011_1110_0000;
        vertex |= (position[2].floor() as u32).rotate_left(11) & 0x0003_F800; // & 0b0000_0000_0001_1111;

        vertex |= ((uv[0] * 16.0).floor() as u32).rotate_left(7) & 0x0000_0780;
        vertex |= ((uv[1] * 16.0).floor() as u32).rotate_left(3) & 0x0000_0078;

        vertex |= (ao * 2.0) as u32 & 0x0000_0007;

        Vertex { data: vertex }
    }

    pub fn description<'a>() -> VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Uint32,
                },
                /*
                // uv coords
                wgpu::VertexAttribute {
                    offset: mem::size_of::<u32>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Uint32,
                },
                // ao value
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[u32; 2]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Uint32,
                },
                */
            ],
        }
    }
}

pub fn should_mesh_block(config: &ChunkConfig, pos: &Position) -> bool {
    if pos.0 < 0
        || pos.1 < 0
        || pos.2 < 0
        || pos.0 >= config.depth
        || pos.1 >= config.depth
        || pos.2 >= config.depth
    {
        return false;
    }
    true
}

// see what's taking so long in this function/and why
pub fn mesh_chunk(chunk: &ChunkData, config: &ChunkConfig, _lod: LOD) -> RenderObject {
    let mut vertices = Vec::<Vertex>::new();
    let mut indices = Vec::<u16>::new();

    // loop over all blocks stored
    for (position, block_id) in chunk.iter() {
        // check for the boundary that we generated
        // and don't want to mesh
        if !should_mesh_block(config, position) {
            continue;
        }

        let block = config.dict.get(block_id);
        let model = block.unwrap_or(config.dict.get(&0).unwrap()).model;
        model(&chunk, config, position, &mut vertices, &mut indices);
    }

    RenderObject::new(
        "chunk_render_group",
        bytemuck::cast_slice(vertices.as_slice()),
        bytemuck::cast_slice(indices.as_slice()),
    )
}
