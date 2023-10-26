use std::collections::HashMap;

use crate::engine::input::Input;
use crate::engine::renderer::Renderer;
use crate::window_state_mut;
use winit::dpi::PhysicalSize;

/// Used by the game state struct to more ergonomically refer to its systems.
pub type System<D, E> = fn(&mut Renderer, &mut Input, &mut D, &mut Vec<E>, f64);

/// Stores all the systems, event queues, delta, input, renderer, and systems for the game state. \
/// ```D``` is the game state data, there are no restrictions on what this can be. \
/// ```E``` is the enum of Events, has to be hashable.
pub struct GameState<D, E>
where
    E: PartialEq + Eq + std::hash::Hash,
{
    pub data: D,
    pub renderer: Renderer,
    systems: HashMap<E, Vec<System<D, E>>>,
    queue: [Vec<E>; 2],
    plex: usize,
    pub delta: f64,
    start_delta: f64,
    pub input: Input,
}

impl<D, E> GameState<D, E>
where
    E: PartialEq + Eq + std::hash::Hash,
{
    /// Make a new game state from a renderer and game data.
    pub fn new(renderer: Renderer, data: D) -> Self {
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
    pub fn add_system(&mut self, event: E, system: System<D, E>) {
        self.systems
            .entry(event)
            .and_modify(|storage| storage.push(system))
            .or_insert(vec![system]);
    }

    /// Drain the event queue and process the events.
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
