use std::collections::HashMap;

use crate::engine::renderer::Renderer;
use crate::engine::{render_group::RenderGroup, render_object::RenderObject};
use crate::window::WindowState;

type EntityId = String;

// singleton components
pub struct World {}

// component storages
#[derive(Default)]
pub struct Components {}

// system needed parameters:
// components, queue, renderer, world
type System = fn(&mut Renderer, &mut Components, &mut World, &mut Vec<Event>);

#[derive(Default)]
pub struct Systems(HashMap<Event, Vec<System>>);

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Event {
    Tick,
    Render,

    Init,
}

// S = state
// E = event enum: hashable
pub struct GameState<D, E>
where
    E: PartialEq + Eq + std::hash::Hash,
{
    data: D,
    renderer: Renderer,
    systems: HashMap<E, Vec<fn(&mut Renderer, &mut D, &mut Vec<E>)>>,
    queue: [Vec<E>; 2],
    plex: usize,
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
        }
    }

    pub fn add_system(&mut self, event: E, system: fn(&mut Renderer, &mut D, &mut Vec<E>)) {
        self.systems
            .entry(event)
            .and_modify(|storage| storage.push(system))
            .or_insert(vec![system]);
    }

    pub fn process_events(&mut self) {
        while let Some(event) = self.queue[self.plex].pop() {
            for system in self.systems.get(&event).unwrap().iter() {
                system(
                    &mut self.renderer,
                    &mut self.data,
                    &mut self.queue[1 - self.plex],
                );
            }
        }
    }
}
