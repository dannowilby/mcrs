//! RenderGroups are used to hold specific render pipelines and their associated uniforms.

use crate::engine::render::uniform::UniformLayout;

use crate::window_state;

use wgpu::{
    BindGroupLayout, PipelineLayoutDescriptor, RenderPipeline, RenderPipelineDescriptor,
    ShaderModule, ShaderModuleDescriptor, ShaderSource, TextureFormat, VertexBufferLayout,
};

/// Used to build a RenderGroup.
/// Add uniforms with the [.with()](RenderGroupBuilder::with) method. \
/// Can only be constructed with one shader and one vertex format.
pub struct RenderGroupBuilder<'a> {
    vertex_format: Option<VertexBufferLayout<'a>>,
    uniforms: Vec<UniformLayout>,
    uniform_names: Vec<String>,
    shader: Option<ShaderModule>,
}

impl<'a> RenderGroupBuilder<'a> {
    /// Create a new RenderGroupBuilder with empty fields.
    pub fn new() -> Self {
        RenderGroupBuilder {
            vertex_format: None,
            uniforms: Vec::new(),
            uniform_names: Vec::new(),
            shader: None,
        }
    }

    /// Set the shader for the render group. Takes in the shader source as a `&str`.
    pub fn shader(mut self, source: &str) -> Self {
        let shader = window_state()
            .device
            .create_shader_module(ShaderModuleDescriptor {
                label: Some("Shader"),
                source: ShaderSource::Wgsl(source.into()),
            });

        self.shader = Some(shader);
        self
    }

    /// Set the vertex format with a [VertexBufferLayout](wgpu::VertexBufferLayout).
    pub fn vertex_format(mut self, format: VertexBufferLayout<'a>) -> Self {
        self.vertex_format = Some(format);
        self
    }

    /// Add a uniform to the group. Must pass in a string id and a [UniformLayout](super::uniform::UniformLayout).
    pub fn with(mut self, uniform_name: &str, layout: UniformLayout) -> Self {
        self.uniforms.push(layout);
        self.uniform_names.push(uniform_name.to_owned());
        self
    }

    /// Consume the builder and return a built render group. Will panic if no vertex format or shader set.
    pub fn build(self, depth: bool) -> RenderGroup {
        let layouts: Vec<&BindGroupLayout> = self.uniforms.iter().map(|x| &x.layout).collect();

        let shader = self.shader.expect("No shader set for RenderGroup.");

        let vertex_format = self
            .vertex_format
            .expect("No vertex format specified for RenderGroup.");

        let mut depth_stencil = None;
        if depth {
            depth_stencil = Some(wgpu::DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less, // 1.
                stencil: wgpu::StencilState::default(),     // 2.
                bias: wgpu::DepthBiasState::default(),
            });
        }

        let render_pipeline_layout =
            window_state()
                .device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &layouts,
                    push_constant_ranges: &[],
                });

        let pipeline = window_state()
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
                        format: window_state().config.format,
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
                depth_stencil,
                multisample: wgpu::MultisampleState {
                    count: 1,                         // 2.
                    mask: !0,                         // 3.
                    alpha_to_coverage_enabled: false, // 4.
                },
                multiview: None, // 5.
            });

        RenderGroup {
            pipeline,
            uniforms: self.uniform_names,
        }
    }
}

/// Holds a list of associated uniforms and a render pipeline.
pub struct RenderGroup {
    pub pipeline: RenderPipeline,
    pub uniforms: Vec<String>,
}
