use glam::Mat4;
use wgpu::Buffer;

use crate::window_state;

use super::uniform::{Uniform, UniformLayout};

pub struct Matrix {
    data: Mat4,
    buffer: Buffer,
}

impl Matrix {
    pub fn new(matrix: Mat4) -> Self {
        use wgpu::util::DeviceExt;
        let device = &window_state().device;
        Matrix {
            data: matrix,
            buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Matrix Buffer"),
                contents: bytemuck::cast_slice(&[matrix.to_cols_array_2d()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }),
        }
    }

    pub fn matrix(&mut self) -> &mut Mat4 {
        &mut self.data
    }

    pub fn update_buffer(&mut self) {
        let queue = &window_state().queue;
        queue.write_buffer(
            &self.buffer,
            0,
            bytemuck::cast_slice(&[self.data.to_cols_array_2d()]),
        );
    }

    pub fn create_layout(location: u32) -> UniformLayout {
        let device = &window_state().device;

        UniformLayout {
            layout: device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("matrix"),
            }),
            location,
        }
    }

    pub fn uniform(self, layout: &UniformLayout) -> Uniform {
        let device = &window_state().device;
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.buffer.as_entire_binding(),
            }],
            label: Some("matrix_bind_group"),
        });

        Uniform {
            location: layout.location,
            bind_group,
            data: super::uniform::UniformData::Matrix(self),
        }
    }
}
