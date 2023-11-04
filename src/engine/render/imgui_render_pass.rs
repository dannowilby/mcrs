//! Implementation for ImGui.
use imgui::{FontSource, Ui};
use imgui_wgpu::{Renderer, RendererConfig};
use wgpu::TextureFormat;

use crate::window_state;

use super::render_pass::{RenderPass, RenderPassViews};

/// Used to store the windows the imgui windwows.
pub struct ImguiRenderPass<T> {
    pub windows: Vec<Box<dyn Fn(&mut Ui, &mut T)>>,
    pub context: imgui::Context,
    pub platform: imgui_winit_support::WinitPlatform,
    renderer: imgui_wgpu::Renderer,
}

impl<T> ImguiRenderPass<T> {
    pub fn new() -> Self {
        let window = &window_state().window;
        let device = &window_state().device;
        let queue = &window_state().queue;
        let hidpi_factor = &window.scale_factor();

        // Set up dear imgui
        let mut imgui = imgui::Context::create();
        let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);
        platform.attach_window(
            imgui.io_mut(),
            &window,
            imgui_winit_support::HiDpiMode::Default,
        );
        imgui.set_ini_filename(None);

        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;

        imgui.fonts().add_font(&[FontSource::DefaultFontData {
            config: Some(imgui::FontConfig {
                oversample_h: 1,
                pixel_snap_h: true,
                size_pixels: font_size,
                ..Default::default()
            }),
        }]);

        let renderer_config = RendererConfig {
            texture_format: TextureFormat::Bgra8UnormSrgb,
            ..Default::default()
        };

        let renderer = Renderer::new(&mut imgui, &device, &queue, renderer_config);

        Self {
            context: imgui,
            platform,
            renderer,
            windows: vec![],
        }
    }
}

impl<T> RenderPass<T> for ImguiRenderPass<T> {
    fn render(
        &mut self,
        game_data: &mut T,
        views: RenderPassViews,
        delta: f64,
    ) -> Result<(), wgpu::SurfaceError> {
        let window = &window_state().window;
        let device = &window_state().device;
        let queue = &window_state().queue;
        let view = views
            .color
            .expect("No color attachment specified on Imgui Render Pass...");

        let mut encoder: wgpu::CommandEncoder =
            window_state()
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("ImGui Render Pass"),
                });

        let imgui = &mut self.context;
        let platform = &mut self.platform;
        let renderer = &mut self.renderer;

        imgui
            .io_mut()
            .update_delta_time(std::time::Duration::from_millis(delta as u64));

        platform
            .prepare_frame(imgui.io_mut(), &window)
            .expect("Failed to prepare frame");
        let ui = imgui.frame();

        self.windows.drain(..).for_each(|f| {
            f(ui, game_data);
        });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            renderer
                .render(imgui.render(), &queue, &device, &mut rpass)
                .expect("Rendering failed");
        }
        window_state()
            .queue
            .submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}
