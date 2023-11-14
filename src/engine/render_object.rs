//! Used to store a game object's rendering information.

use std::collections::HashMap;
use wgpu::Buffer;

use crate::engine::uniform::Uniform;
use crate::window_state;

/// Stores the id of its associated [render group](super::render_group::RenderGroup), a vertex and index buffer,
/// and this render object's associated uniforms.
#[derive(Debug)]
pub struct RenderObject {
    pub render_group: String,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub uniforms: HashMap<String, Uniform>,
    pub visible: bool,
}

impl RenderObject {
    /// Create a new render object with bind group id, vertex and index buffer.
    pub fn new(bind_group: &str, vertices: &[u8], indices: &[u8]) -> Self {
        use wgpu::util::DeviceExt;
        let device = &window_state().device;
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: vertices, // bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: indices, //  bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });
        RenderObject {
            render_group: bind_group.to_owned(),
            vertex_buffer,
            index_buffer,
            uniforms: HashMap::new(),
            visible: true,
        }
    }

    /// Update the render object's buffers. Writes the buffers using the WGPU device.
    #[allow(dead_code)]
    pub fn set_buffers(&mut self, vertices: &[u8], indices: &[u8]) {
        use wgpu::util::DeviceExt;
        let device = &window_state().device;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: vertices, // bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: indices, //  bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        self.vertex_buffer = vertex_buffer;
        self.index_buffer = index_buffer;
    }
}
