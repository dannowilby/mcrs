use std::collections::HashMap;

use crate::window_state;

use super::{
    render_group::RenderGroup, render_object::RenderObject, texture::Texture, uniform::Uniform,
};

pub struct Renderer {
    render_groups: HashMap<String, RenderGroup>,
    render_objects: HashMap<String, RenderObject>,
    uniforms: HashMap<String, Uniform>,
    depth_texture: Texture,
}

impl Renderer {
    pub fn new() -> Self {
        let device = &window_state().device;
        let config = &window_state().config;
        Renderer {
            render_groups: HashMap::new(),
            render_objects: HashMap::new(),
            uniforms: HashMap::new(),
            depth_texture: Texture::create_depth_texture(&device, &config, "depth_texture"),
        }
    }

    pub fn create_group(&mut self, label: &str, render_group: RenderGroup) {
        self.render_groups.insert(label.to_owned(), render_group);
    }

    // should always return a valid reference, panic if None is found
    pub fn add_object(&mut self, id: &str, object: RenderObject) {
        self.render_objects.insert(id.to_owned(), object);
    }

    pub fn remove_object(&mut self, id: &str) -> Option<RenderObject> {
        self.render_objects.remove(id)
    }

    pub fn set_object_uniform(&mut self, id: &str, uniform_name: &str, uniform: Uniform) {
        self.render_objects
            .get_mut(id)
            .unwrap()
            .set_uniform(uniform_name, uniform);
    }

    pub fn contains_object(&self, id: &str) -> bool {
        self.render_objects.contains_key(id)
    }

    pub fn get_mut_object<'a>(&'a mut self, id: &str) -> &'a mut RenderObject {
        self.render_objects.get_mut(id).unwrap()
    }

    pub fn set_global_uniform(&mut self, uniform_name: &str, uniform: Uniform) {
        self.uniforms.insert(uniform_name.to_owned(), uniform);
    }

    pub fn get_global_uniform(&mut self, uniform_name: &str) -> Option<&mut Uniform> {
        self.uniforms.get_mut(uniform_name)
    }

    pub fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let device = &window_state().device;
        let surface = &window_state().surface;
        let queue = &window_state().queue;

        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            // encode the render commands
            for (_id, object) in self.render_objects.iter() {
                let group = self.render_groups.get(&object.render_group).expect(
                    "Referenced a render group that does not exist! You are using this wrong!",
                );

                render_pass.set_pipeline(&group.pipeline);

                for uniform_name in group.uniforms.iter() {
                    let global_uniform = self.uniforms.get(uniform_name);
                    let uniform = match global_uniform {
                        Some(x) => x,
                        None => match object.uniforms.get(uniform_name) {
                            Some(x) => x,
                            None => {
                                println!("What: {}", uniform_name);
                                continue;
                            }
                        },
                    };

                    render_pass.set_bind_group(uniform.location, &uniform.bind_group, &[]);
                }

                render_pass.set_vertex_buffer(0, object.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(object.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                let num_indices =
                    object.index_buffer.size() as u32 / std::mem::size_of::<u16>() as u32;
                render_pass.draw_indexed(0..num_indices, 0, 0..1);
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn resize(&mut self) {
        let device = &window_state().device;
        let config = &window_state().config;
        self.depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");
    }
}
