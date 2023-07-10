use std::collections::HashMap;
use wgpu::Buffer;

use crate::engine::uniform::Uniform;
use crate::window::WindowState;

pub struct RenderObject {
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub uniforms: HashMap<String, Uniform>,
}

impl RenderObject {
    pub fn new(window_state: &WindowState, vertices: &[u8], indices: &[u8]) -> Self {
        use wgpu::util::DeviceExt;
        let device = &window_state.device;
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
            vertex_buffer,
            index_buffer,
            uniforms: HashMap::new(),
        }
    }

    pub fn update_buffers(&mut self, window_state: &WindowState, vertices: &[u8], indices: &[u8]) {
        use wgpu::util::DeviceExt;
        let device = &window_state.device;

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

    pub fn render<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>) {
        for uniform in self.uniforms.values().into_iter() {
            let Uniform {
                location,
                bind_group,
                data: _,
            } = uniform;

            render_pass.set_bind_group(*location, &bind_group, &[]);
        }

        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        let index_length = 2 * self.index_buffer.size() as u32 / std::mem::size_of::<f32>() as u32;
        render_pass.draw_indexed(0..(index_length), 0, 0..1);
    }
}

pub type RenderObjectHandle = String;
