use crate::engine::texture::Texture;
use wgpu::{BindGroup, BindGroupLayout, Buffer};

#[allow(dead_code)]
pub enum UniformData {
    Buffer(Buffer),
    Texture(Texture),
}

pub struct Uniform {
    pub location: u32,
    pub bind_group: BindGroup,
    pub data: UniformData,
}

pub struct UniformLayout {
    pub layout: BindGroupLayout,
    pub location: u32,
}
