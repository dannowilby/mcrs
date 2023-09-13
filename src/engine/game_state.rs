use std::collections::HashMap;

use crate::engine::input::Input;
use crate::engine::renderer::Renderer;
use crate::window_state_mut;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};

// D = game data
// E = event enum: hashable
pub type System<D, E> = fn(&mut Renderer, &mut Input, &mut D, &mut Vec<E>, f64);

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

    // must set window_state to new size since
    // renderer reads from it
    // might want to change and just pass directly in the future
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        window_state_mut().resize(new_size);
        self.renderer.resize();
    }

    pub fn add_system(&mut self, event: E, system: System<D, E>) {
        self.systems
            .entry(event)
            .and_modify(|storage| storage.push(system))
            .or_insert(vec![system]);
    }

    // update
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
