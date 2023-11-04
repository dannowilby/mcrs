use std::{collections::HashMap, marker::PhantomData};

use crate::{
    engine::{render_group::RenderGroup, render_object::RenderObject, uniform::Uniform},
    window_state,
};

use super::render_pass::{RenderPass, RenderPassViews};

pub struct ObjectRenderPass<T> {
    pub render_groups: HashMap<String, RenderGroup>,
    pub render_objects: HashMap<String, RenderObject>,
    pub uniforms: HashMap<String, Uniform>,
    pub clear_color: wgpu::Color,
    marker: PhantomData<T>,
}

impl<T> ObjectRenderPass<T> {
    pub fn new() -> Self {
        Self {
            render_groups: HashMap::new(),
            render_objects: HashMap::new(),
            uniforms: HashMap::new(),
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
            marker: PhantomData,
        }
    }
}

impl<T> RenderPass<T> for ObjectRenderPass<T> {
    fn render(
        &mut self,
        _game_state: &mut T,
        views: RenderPassViews,
        _delta: f64,
    ) -> Result<(), wgpu::SurfaceError> {
        let view = views
            .color
            .expect("No color attachment specified on Object Render Pass...");
        let depth_view = views
            .depth
            .expect("No depth attachment specified on Object Render Pass...");

        let mut encoder: wgpu::CommandEncoder =
            window_state()
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Object Render Pass"),
                });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Object Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            // encode the render commands
            // loop over all render objects
            for (_id, object) in self.render_objects.iter() {
                // get the render object's render group
                let group = self.render_groups.get(&object.render_group).expect(
                    "Referenced a render group that does not exist! You are using this wrong!",
                );

                render_pass.set_pipeline(&group.pipeline);

                // for the uniforms in the group
                for uniform_name in group.uniforms.iter() {
                    // check if a global uniform first
                    let global_uniform = self.uniforms.get(uniform_name);
                    let uniform = match global_uniform {
                        Some(x) => x,
                        // if not a global uniform, then it's a object uniform
                        None => object
                            .uniforms
                            .get(uniform_name)
                            .expect(&format!("Uniform {} not specified", uniform_name)),
                    };

                    // set the uniform
                    render_pass.set_bind_group(uniform.location, &uniform.bind_group, &[]);
                }

                // set the vertex buffer
                render_pass.set_vertex_buffer(0, object.vertex_buffer.slice(..));
                // set the index buffer
                render_pass
                    .set_index_buffer(object.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                let num_indices =
                    object.index_buffer.size() as u32 / std::mem::size_of::<u16>() as u32;
                // draw
                render_pass.draw_indexed(0..num_indices, 0, 0..1);
            }
        }
        window_state()
            .queue
            .submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
