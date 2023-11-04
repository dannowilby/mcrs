use winit::event::Event;

pub trait Renderer<D> {
    fn render(&mut self, game_data: &mut D, delta: f64) -> Result<(), wgpu::SurfaceError>;

    fn resize(&mut self);

    fn handle_event(&mut self, event: &Event<()>);
}
