use crate::engine::matrix::Matrix;
use crate::engine::texture::Texture;
use wgpu::{BindGroup, BindGroupLayout, Buffer};

#[derive(Debug)]
pub enum UniformData {
    Buffer(Buffer),
    Texture(Texture),
    Matrix(Matrix),
}

#[derive(Debug)]
pub struct Uniform {
    pub location: u32,
    pub bind_group: BindGroup,
    pub data: UniformData,
}

pub struct UniformLayout {
    pub layout: BindGroupLayout,
    pub location: u32,
}
