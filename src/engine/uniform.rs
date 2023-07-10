use crate::texture::Texture;
use wgpu::{BindGroup, Buffer};

pub enum UniformData {
    Buffer(Buffer),
    Texture(Texture),
}

pub struct Uniform {
    pub location: u32,
    pub bind_group: BindGroup,
    pub data: UniformData,
}
