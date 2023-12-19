use std::collections::HashMap;

use crate::engine::input::Input;
use crate::window_state_mut;
use winit::dpi::PhysicalSize;

use super::render::renderer::Renderer;

/// Used by the game state struct to more ergonomically refer to its systems.
pub type System<D, R, E> = fn(&mut R, &mut Input, &mut D, &mut Vec<E>, f64);

/// Stores all the systems, event queues, delta, input, renderer, and systems for the game state. \
/// ```D``` is the game state data, there are no restrictions on what this can be. \
/// ```E``` is the enum of Events, has to be hashable. \
/// ```R``` is the renderer. It must implement the ```Renderer``` trait with ```D``` as it's generic parameter. \
/// ```delta``` is stored in milliseconds.
pub struct GameState<D, R: Renderer<D>, E>
where
    E: PartialEq + Eq + std::hash::Hash,
{
    pub data: D,
    pub renderer: R,
    systems: HashMap<E, Vec<System<D, R, E>>>,
    queue: [Vec<E>; 2],
    plex: usize,
    pub delta: f64,
    start_delta: f64,
    pub input: Input,
}

impl<D, R, E> GameState<D, R, E>
where
    E: PartialEq + Eq + std::hash::Hash,
    R: Renderer<D>,
{
    /// Make a new game state from a renderer and game data.
    pub fn new(renderer: R, data: D) -> Self {
        Self {
            data,
            systems: Default::default(),
            renderer,
            queue: Default::default(),
            plex: 0,
            delta: 0.0,
            start_delta: 0.0,
            input: Input::new(),
        }
    }

    /// Updates the window state surface and calls the resize method on the renderer.
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        // must set window_state to new size since
        // renderer reads from it
        window_state_mut().resize(new_size);
        self.renderer.resize();
    }

    /// Add a new system for the corresponding event.
    pub fn add_system(&mut self, event: E, system: System<D, R, E>) {
        self.systems
            .entry(event)
            .and_modify(|storage| storage.push(system))
            .or_insert(vec![system]);
    }

    /// Drain the event queue and process the events.
    /// The last event in the queue will be popped first.
    pub fn process_events(&mut self) {
        while let Some(event) = self.queue[self.plex].pop() {
            if let Some(system) = self.systems.get(&event) {
                for system in system.iter() {
                    system(
                        &mut self.renderer,
                        &mut self.input,
                        &mut self.data,
                        &mut self.queue[1 - self.plex],
                        self.delta,
                    );
                }
            }
        }

        self.plex = 1 - self.plex;
    }

    /// Add an event to be processed next frame.
    pub fn queue_event(&mut self, event: E) {
        self.queue[self.plex].push(event);
    }

    pub fn delta_start(&mut self) {
        self.start_delta = instant::now();
    }

    pub fn delta_end(&mut self) {
        self.delta = instant::now() - self.start_delta;
    }
}

/// We test that the delta updates correctly, systems are ran when events are sent, and events are processed in the correct order.
#[cfg(test)]
mod tests {
    use instant::Duration;

    use super::*;

    #[derive(PartialEq, Eq, Hash)]
    enum MockEvents {
        Attempt,
        Trigger,
        Failure,
    }

    struct MockData {
        flag: bool,
    }
    struct MockRenderer;
    impl Renderer<MockData> for MockRenderer {
        fn render(
            &mut self,
            _game_data: &mut MockData,
            _delta: f64,
        ) -> Result<(), wgpu::SurfaceError> {
            Ok(())
        }
        fn handle_event(&mut self, _event: &winit::event::Event<()>) {}
        fn resize(&mut self) {}
    }

    fn mock_system_trigger_event(
        _: &mut MockRenderer,
        _: &mut Input,
        _: &mut MockData,
        queue: &mut Vec<MockEvents>,
        _: f64,
    ) {
        queue.push(MockEvents::Attempt);
    }

    fn mock_system_success(
        _: &mut MockRenderer,
        _: &mut Input,
        data: &mut MockData,
        _: &mut Vec<MockEvents>,
        _: f64,
    ) {
        data.flag = true;
    }

    fn mock_system_failure(
        _: &mut MockRenderer,
        _: &mut Input,
        data: &mut MockData,
        _: &mut Vec<MockEvents>,
        _: f64,
    ) {
        data.flag = false;
    }

    /// Test that only the event we send triggers the correct systems.
    #[test]
    fn single_system_event_test() {
        let mut gs = GameState::<MockData, MockRenderer, MockEvents>::new(
            MockRenderer,
            MockData { flag: false },
        );

        gs.add_system(MockEvents::Attempt, mock_system_success);
        gs.add_system(MockEvents::Failure, mock_system_failure);

        gs.queue_event(MockEvents::Attempt);

        gs.process_events();

        assert!(gs.data.flag);
    }

    /// Test that an event can trigger events for the future.
    #[test]
    fn multi_system_event_test() {
        let mut gs = GameState::<MockData, MockRenderer, MockEvents>::new(
            MockRenderer,
            MockData { flag: false },
        );

        gs.add_system(MockEvents::Trigger, mock_system_trigger_event);
        gs.add_system(MockEvents::Attempt, mock_system_success);
        gs.add_system(MockEvents::Failure, mock_system_failure);

        gs.queue_event(MockEvents::Trigger);

        gs.process_events();
        assert!(!gs.data.flag);

        gs.process_events();
        assert!(gs.data.flag);
    }

    /// Test event order is preserved.
    #[test]
    fn system_event_order_test() {
        let mut gs = GameState::<MockData, MockRenderer, MockEvents>::new(
            MockRenderer,
            MockData { flag: false },
        );

        gs.add_system(MockEvents::Attempt, mock_system_success);
        gs.add_system(MockEvents::Failure, mock_system_failure);

        gs.queue_event(MockEvents::Attempt);
        gs.queue_event(MockEvents::Failure);
        gs.queue_event(MockEvents::Failure);
        gs.queue_event(MockEvents::Attempt);
        gs.queue_event(MockEvents::Failure);

        gs.process_events();
        assert!(gs.data.flag);
    }

    /// Test that the delta updates correctly.
    #[test]
    fn delta_test() {
        let mut gs = GameState::<MockData, MockRenderer, MockEvents>::new(
            MockRenderer,
            MockData { flag: false },
        );
        gs.delta_start();

        std::thread::sleep(Duration::from_millis(10));

        gs.delta_end();

        assert!(gs.delta > 10.0);
        assert!(gs.delta < 15.0);
    }
}
