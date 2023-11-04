//! Used to encasuplate WGPU rendering functions. \
//! Used to store global uniforms, a common depth texture, and the following: \
//! [Render groups](super::render_group::RenderGroup) contain the layout data for render objects to be rendererd. \
//! [Render objects](super::render_object::RenderObject) contain mesh data and uniform data for that object. \
//! There is no programmatic way to specify separate render passes yet, and as a result the ImGui instance is thrown in here.

use std::collections::HashMap;

use crate::window_state;

use super::{
    imgui::ImGui, render_group::RenderGroup, render_object::RenderObject, texture::Texture,
    uniform::Uniform,
};

/// The idea behind this struct is that we register the things we want to
/// render and how we want to render it with this struct using string handles, and
/// then keep all other game object data in the [GameData](super::game_state::GameData).
pub struct Renderer {
    render_groups: HashMap<String, RenderGroup>,
    render_objects: HashMap<String, RenderObject>,
    uniforms: HashMap<String, Uniform>,
    depth_texture: Texture,
    pub imgui: ImGui,
}

impl Renderer {
    /// Create a new Renderer.
    pub fn new() -> Self {
        let device = &window_state().device;
        let config = &window_state().config;
        Renderer {
            render_groups: HashMap::new(),
            render_objects: HashMap::new(),
            uniforms: HashMap::new(),
            depth_texture: Texture::create_depth_texture(&device, &config, "depth_texture"),
            imgui: ImGui::new(),
        }
    }

    /// Add a render group to the renderer. \
    /// We store each render group with a string id so that our render objects can refer to it. \
    /// If we remove the render group and not the object that refer to it then the render method will panic.
    pub fn add_group(&mut self, label: &str, render_group: RenderGroup) {
        self.render_groups.insert(label.to_owned(), render_group);
    }

    /// Add a render object into the renderer.
    /// The render group associated with this must also be added to the renderer
    /// otherwise the render method will panic.
    pub fn add_object(&mut self, id: &str, object: RenderObject) {
        self.render_objects.insert(id.to_owned(), object);
    }

    /// Removes the render object and returns it if it exists.
    pub fn remove_object(&mut self, id: &str) -> Option<RenderObject> {
        self.render_objects.remove(id)
    }

    /// Set the uniform on the specified render object.
    pub fn set_object_uniform(&mut self, id: &str, uniform_name: &str, uniform: Uniform) {
        self.render_objects
            .get_mut(id)
            .unwrap()
            .set_uniform(uniform_name, uniform);
    }

    #[allow(dead_code)]
    /// Check if a the renderer contains the specified render object by id.
    pub fn contains_object(&self, id: &str) -> bool {
        self.render_objects.contains_key(id)
    }

    #[allow(dead_code)]
    /// Get a mutable reference to the added render object.
    pub fn get_mut_object(&mut self, id: &str) -> Option<&mut RenderObject> {
        self.render_objects.get_mut(id)
    }

    /// Set a uniform that will be used in all render groups.
    pub fn set_global_uniform(&mut self, uniform_name: &str, uniform: Uniform) {
        self.uniforms.insert(uniform_name.to_owned(), uniform);
    }

    /// Get a mutable reference to the queried global uniform.
    pub fn get_global_uniform(&mut self, uniform_name: &str) -> Option<&mut Uniform> {
        self.uniforms.get_mut(uniform_name)
    }

    /// Render all added render objects.
    pub fn render(&mut self, delta: f64) -> Result<(), wgpu::SurfaceError> {
        let mut encoder: wgpu::CommandEncoder = window_state()
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let surface = &window_state().surface;
        let output = surface.get_current_texture().unwrap();
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // main chunks render pass
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

        let _ = self.imgui.render(&view, &mut encoder, delta);
        window_state()
            .queue
            .submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Reconfigure the depth texture to the updated window state. \
    /// Must update the window state first.
    pub fn resize(&mut self) {
        let device = &window_state().device;
        let config = &window_state().config;
        self.depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");
    }
}
