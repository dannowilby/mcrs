#![allow(dead_code)]

use crate::{
    engine::{
        render::{render_group::RenderGroup, render_object::RenderObject, uniform::Uniform},
        texture::Texture,
    },
    window_state,
};

use super::{
    render_group::RenderGroupBuilder,
    render_pass::{RenderPass, RenderPassViews},
    uniform::UniformData,
};

pub struct FrameRenderPass {
    group: RenderGroup,
    frame: RenderObject,
    pub render_texture: Uniform,
    pub clear_color: wgpu::Color,
}

struct FrameVertex {
    data: u32,
}

impl FrameVertex {
    fn description<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<FrameVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Uint32,
                },
            ],
        }
    }
}

impl FrameRenderPass {
    pub fn new(downscale_factor: u32, shader_source: &str) -> Self {
        let config = &window_state().config;
        let layout = Texture::create_layout(0);
        Self {
            render_texture: Texture::create_render_texture(
                config.width / downscale_factor,
                config.height / downscale_factor,
            )
            .uniform(&layout),
            group: RenderGroupBuilder::new()
                .shader(shader_source)
                .vertex_format(FrameVertex::description())
                .with("frame-buffer", layout)
                .build(false),
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            frame: RenderObject::new(
                "",
                bytemuck::cast_slice(
                    vec![
                        0x0000_0000,
                        0x0002_0000,
                        0x0002_0002,
                        0x0000_0000,
                        0x0002_0002,
                        0x0000_0002,
                    ]
                    .as_slice(),
                ),
                bytemuck::cast_slice(vec![0u16, 2u16, 3u16, 0u16, 3u16, 1u16].as_slice()),
            ),
        }
    }

    pub fn get_render_texture_view(&self) -> Option<&wgpu::TextureView> {
        if let UniformData::Texture(texture) = &self.render_texture.data {
            return Some(&texture.view);
        }
        None
    }

    pub fn resize(&mut self, downscale_factor: u32) {
        let config = &window_state().config;
        self.render_texture = Texture::create_render_texture(
            config.width / downscale_factor,
            config.height / downscale_factor,
        )
        .uniform(&Texture::create_layout(0));
    }
}

impl<T> RenderPass<T> for FrameRenderPass {
    fn render(
        &mut self,
        _game_state: &mut T,
        views: RenderPassViews,
        _delta: f64,
    ) -> Result<(), wgpu::SurfaceError> {
        let view = views
            .color
            .expect("No color attachment specified on Object Render Pass...");

        let mut encoder: wgpu::CommandEncoder =
            window_state()
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Frame Render Pass"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Frame Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            // encode the render commands
            // loop over all render objects

            render_pass.set_pipeline(&self.group.pipeline);
            render_pass.set_bind_group(
                self.render_texture.location,
                &self.render_texture.bind_group,
                &[],
            );

            // set the vertex buffer
            render_pass.set_vertex_buffer(0, self.frame.vertex_buffer.slice(..));
            // set the index buffer
            render_pass
                .set_index_buffer(self.frame.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            // let num_indices =
            //    self.frame.index_buffer.size() as u32 / std::mem::size_of::<u16>() as u32;
            // draw
            render_pass.draw(0..6, 0..1);
        }
        window_state()
            .queue
            .submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
