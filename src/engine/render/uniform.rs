//! Better encapsulation of uniform data.
use crate::engine::matrix::Matrix;
use crate::engine::texture::Texture;
use wgpu::{BindGroup, BindGroupLayout, Buffer};

/// Enum used to store the uniform data.
#[allow(dead_code)]
#[derive(Debug)]
pub enum UniformData {
    Buffer(Buffer),
    Texture(Texture),
    Matrix(Matrix),
}

/// A general purpose struct used to store everything needed to bind and use a uniform. \
/// `location` is the bind group index to be used in the shader.
#[derive(Debug)]
pub struct Uniform {
    pub location: u32,
    pub bind_group: BindGroup,
    pub data: UniformData,
}

/// The layout data for a uniform.
pub struct UniformLayout {
    pub layout: BindGroupLayout,
    pub location: u32,
}
