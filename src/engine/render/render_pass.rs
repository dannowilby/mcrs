//! Abstract trait for a rendering pass.

/// We pass in the textures we want to render to and also a mutable reference to the game state.
/// We do this so that the rendering can change variables, ie. for gui.
pub trait RenderPass<T> {
    /// Execute the render pass.
    fn render(
        &mut self,
        game_data: &mut T,
        views: RenderPassViews,
        delta: f64,
    ) -> Result<(), wgpu::SurfaceError>;
}

pub struct RenderPassViews<'a> {
    pub color: Option<&'a wgpu::TextureView>,
    pub depth: Option<&'a wgpu::TextureView>,
}
