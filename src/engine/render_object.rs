use std::collections::HashMap;
use wgpu::Buffer;

use crate::engine::uniform::Uniform;
use crate::window_state;

pub struct RenderObject {
    pub render_group: String,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub uniforms: HashMap<String, Uniform>,
}

impl RenderObject {
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
        }
    }

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

    #[allow(dead_code)]
    pub fn set_uniform(&mut self, uniform_name: &str, uniform: Uniform) {
        self.uniforms.insert(uniform_name.to_owned(), uniform);
    }
}
