use crate::{engine::render_window::RenderWindow, texture, window::WindowState};
use std::collections::HashMap;

use wgpu::{
    BindGroup, BindGroupLayout, BindingResource, Buffer, CommandEncoderDescriptor, Device, LoadOp,
    Operations, PipelineLayoutDescriptor, RenderPassColorAttachment, RenderPassDescriptor,
    RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource,
    SurfaceConfiguration, SurfaceError, TextureFormat, TextureViewDescriptor, VertexBufferLayout,
};

pub struct RenderGroupBuilder<'a> {
    vertex_format: Option<VertexBufferLayout<'a>>,
    uniforms: Vec<BindGroupLayout>,
    shader: Option<ShaderModule>,
    label: String,
}

///
///  let rgb = RenderGroupBuilder::new()
///  rgb.shader();
///  rgb.vertex_format();
///  rgb.with();
///  let render_group = rgb.build();
///
///  Used to build a RenderGroup
///  Uses
impl<'a> RenderGroupBuilder<'a> {
    pub fn new(label: &str) -> Self {
        RenderGroupBuilder {
            vertex_format: None,
            uniforms: Vec::new(),
            shader: None,
            label: label.to_string(),
        }
    }

    pub fn shader(&mut self, window_state: &WindowState, source: &str) {
        let shader = window_state
            .device
            .create_shader_module(ShaderModuleDescriptor {
                label: Some("Shader"),
                source: ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
            });

        self.shader = Some(shader);
    }

    pub fn vertex_format(&mut self, format: VertexBufferLayout<'a>) {
        self.vertex_format = Some(format);
    }

    pub fn with(&mut self, layout: BindGroupLayout) {
        self.uniforms.push(layout);
    }

    pub fn build(self, window_state: &WindowState) -> RenderGroup {
        let layouts: Vec<&BindGroupLayout> = self.uniforms.iter().collect();

        let shader = self.shader.expect("No shader set for RenderGroup.");

        let vertex_format = self
            .vertex_format
            .expect("No vertex format specified for RenderGroup.");

        let render_pipeline_layout =
            window_state
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &layouts,
                    push_constant_ranges: &[],
                });

        let pipeline = window_state
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",    // 1.
                    buffers: &[vertex_format], // 2.
                },
                fragment: Some(wgpu::FragmentState {
                    // 3.
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        // 4.
                        format: window_state.config.format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw, // 2.
                    cull_mode: Some(wgpu::Face::Back),
                    // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less, // 1.
                    stencil: wgpu::StencilState::default(),     // 2.
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview: None, // 5.
            });

        RenderGroup {
            depth_texture: texture::Texture::create_depth_texture(
                &window_state.device,
                &window_state.config,
                "depth_texture",
            ),
            pipeline,
            objects: HashMap::new(),
            uniforms: HashMap::new(),
        }
    }
}

/// render_group.add_object("chunk-0-0-0", render_object);
/// render_group.render();
struct RenderGroup {
    depth_texture: texture::Texture,
    pipeline: RenderPipeline,
    objects: HashMap<String, RenderObject>,
    uniforms: HashMap<String, Uniform>,
}

impl RenderGroup {
    pub fn add_render_object(&mut self) {}

    pub fn get_mut_render_object(&mut self, id: &str) -> Option<&mut RenderObject> {
        self.objects.get_mut(id)
    }

    pub fn render(&self, window_state: &WindowState) -> Result<(), SurfaceError> {
        let output = window_state.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());
        let mut encoder = window_state
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(wgpu::Color {
                        r: 0.0,
                        g: 0.0,
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

        render_pass.set_pipeline(&self.pipeline);

        for uniform in self.uniforms.values().into_iter() {
            let Uniform {
                location,
                bind_group,
                data: _,
            } = uniform;

            render_pass.set_bind_group(*location, &bind_group, &[]);
        }

        for object in self.objects.values().into_iter() {
            for uniform in object.uniforms.values().into_iter() {
                let Uniform {
                    location,
                    bind_group,
                    data: _,
                } = uniform;

                render_pass.set_bind_group(*location, &bind_group, &[]);
            }

            render_pass.set_vertex_buffer(0, object.vertex_buffer.slice(..));
            render_pass.set_index_buffer(object.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            let index_length =
                object.index_buffer.size() as u32 / std::mem::size_of::<f32>() as u32;
            render_pass.draw_indexed(0..(index_length), 0, 0..1);
        }

        window_state.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

struct RenderObject {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    uniforms: HashMap<String, Uniform>,
}

struct Uniform {
    location: u32,
    bind_group: BindGroup,
    data: Buffer,
}
