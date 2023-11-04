use winit::event::{Event, VirtualKeyCode};

use crate::engine::input::Input;
use crate::engine::render::imgui_render_pass::ImguiRenderPass;
use crate::engine::render::object_render_pass::ObjectRenderPass;
use crate::engine::render::render_pass::{RenderPass, RenderPassViews};
use crate::engine::render::renderer::Renderer;
use crate::engine::texture::Texture;
use crate::window_state;
use crate::world::GameData;

pub struct WorldRenderer {
    pub object_render_pass: ObjectRenderPass<GameData>,
    pub imgui_render_pass: ImguiRenderPass<GameData>,

    depth_texture: Texture,
}

impl WorldRenderer {
    pub fn new() -> Self {
        let device = &window_state().device;
        let config = &window_state().config;
        Self {
            object_render_pass: ObjectRenderPass::new(),
            imgui_render_pass: ImguiRenderPass::new(),
            depth_texture: Texture::create_depth_texture(device, config, "depth_texture"),
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

        let _ = self.object_render_pass.render(
            game_data,
            RenderPassViews {
                color: Some(&view),
                depth: Some(&self.depth_texture.view),
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
        self.depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");
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
