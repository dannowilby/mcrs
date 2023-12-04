use winit::event::{Event, VirtualKeyCode};

use crate::chunk::chunk_renderer::ChunkRenderPass;
use crate::engine::input::Input;
use crate::engine::render::frame_render_pass::FrameRenderPass;
use crate::engine::render::imgui_render_pass::ImguiRenderPass;
use crate::engine::render::render_pass::{RenderPass, RenderPassViews};
use crate::engine::render::renderer::Renderer;
use crate::engine::texture::Texture;
use crate::window_state;
use crate::world::GameData;

pub struct WorldRenderer {
    pub chunk_render_pass: ChunkRenderPass,
    pub imgui_render_pass: ImguiRenderPass<GameData>,
    pub frame_render_pass: FrameRenderPass,

    downscale_factor: u32,

    depth_texture: Texture,
}

impl WorldRenderer {
    pub fn new(frame_source: &str) -> Self {
        let device = &window_state().device;
        let config = &window_state().config;
        let downscale_factor = 4;
        Self {
            chunk_render_pass: ChunkRenderPass::new(),
            imgui_render_pass: ImguiRenderPass::new(),
            frame_render_pass: FrameRenderPass::new(downscale_factor, frame_source),
            downscale_factor,
            depth_texture: Texture::create_depth_texture(
                device,
                config,
                "depth_texture",
                config.width / downscale_factor,
                config.height / downscale_factor,
            ),
        }
    }
}

impl Renderer<GameData> for WorldRenderer {
    fn render(&mut self, game_data: &mut GameData, delta: f64) -> Result<(), wgpu::SurfaceError> {
        let surface = &window_state().surface;
        let output = surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let downscaled_view = self.frame_render_pass.get_render_texture_view().unwrap();

        let _ = self.chunk_render_pass.render(
            game_data,
            RenderPassViews {
                color: Some(&downscaled_view),
                depth: Some(&self.depth_texture.view),
            },
            delta,
        )?;

        let _ = self.frame_render_pass.render(
            game_data,
            RenderPassViews {
                color: Some(&view),
                depth: None,
            },
            delta,
        )?;

        if game_data.show_debug_menu {
            let _ = self.imgui_render_pass.render(
                game_data,
                RenderPassViews {
                    color: Some(&view),
                    depth: None,
                },
                delta,
            )?;
        }

        // submit the command buffer
        // window_state()
        //     .queue
        //    .submit(std::iter::once(encoder.finish()));

        output.present();
        Ok(())
    }

    fn resize(&mut self) {
        let device = &window_state().device;
        let config = &window_state().config;
        self.frame_render_pass.resize(self.downscale_factor);
        self.depth_texture = Texture::create_depth_texture(
            &device,
            &config,
            "depth_texture",
            config.width / self.downscale_factor,
            config.height / self.downscale_factor,
        );
    }

    fn handle_event(&mut self, event: &Event<()>) {
        self.imgui_render_pass.platform.handle_event(
            self.imgui_render_pass.context.io_mut(),
            &window_state().window,
            &event,
        );
    }
}

pub fn toggle_debug_menu(
    _renderer: &mut WorldRenderer,
    input: &mut Input,
    data: &mut GameData,
    _queue: &mut Vec<super::world::Event>,
    delta: f64,
) {
    let key = input.get_key(VirtualKeyCode::K);

    if key <= delta && key > 0.0 {
        data.show_debug_menu = !data.show_debug_menu;
    }
}
