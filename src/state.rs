use std::collections::HashMap;

use crate::engine::{render_group::RenderGroup, render_object::RenderObject};
use crate::window::WindowState;

type EntityId = String;

pub struct World {
    camera: crate::camera::Camera,
}

#[derive(Default)]
pub struct Components {
    render_groups: HashMap<EntityId, RenderGroup>,
}

#[derive(Default)]
pub struct Systems {
    render: Vec<fn(&mut Components, &mut Vec<Event>)>,
    tick: Vec<fn(&mut Components, &mut Vec<Event>, &f32)>,
}

pub enum Event {
    Tick { delta: f32 },
    Render,
}

#[derive(Default)]
pub struct State {
    components: Components,
    systems: Systems,
    queue: [Vec<Event>; 2],
    plex: usize,
    world: World,
}

pub fn render_groups(
    window_state: &WindowState,
    components: &mut Components,
    queue: &mut Vec<Event>,
) {
    components
        .render_groups
        .iter()
        .for_each(|(entity_id, render_group)| {
            render_group.render(window_state).ok();
        })
}

impl State {
    fn new() -> Self {
        Self {
            components: Default::default(),
            systems: Default::default(),
            queue: Default::default(),
            plex: 0,
            world: World {
                camera: crate::camera::Camera {},
            },
        }
    }

    fn process_events(&mut self) {
        while let Some(event) = self.queue[self.plex].pop() {
            match event {
                Event::Tick { delta } => {
                    self.systems.tick.iter().for_each(|f| {
                        f(&mut self.components, &mut self.queue[1 - self.plex], &delta)
                    })
                }

                Event::Render => self
                    .systems
                    .render
                    .iter()
                    .for_each(|f| f(&mut self.components, &mut self.queue[1 - self.plex])),

                _ => {}
            }
        }
    }
}
