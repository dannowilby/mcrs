use std::collections::HashMap;

use crate::engine::renderer::RenderObjectHandle;

type EntityId = String;

#[derive(Default)]
struct Components {
    render_objects: HashMap<EntityId, RenderObjectHandle>,
}

#[derive(Default)]
struct Systems {
    render: Vec<fn(&mut Components, &mut Vec<Event>)>,
    tick: Vec<fn(&mut Components, &mut Vec<Event>, &f32)>,
}

enum Event {
    Tick { delta: f32 },
    Render,
}

#[derive(Default)]
pub struct State {
    components: Components,
    systems: Systems,
    queue: [Vec<Event>; 2],
    plex: usize,
}

fn test_fn(components: &Components, queue: &mut Vec<Event>, delta: &f32) {}

impl State {
    fn enqueue_event<T>(&mut self, event: Event) {
        let mut queue = self.queue[self.plex];

        queue.push(event);
    }

    fn process_events(&mut self) {
        while let Some(event) = self.queue[self.plex].pop() {
            match event {
                Event::Tick { delta } => {
                    self.systems.tick.into_iter().for_each(|f| {
                        f(&mut self.components, &mut self.queue[1 - self.plex], &delta)
                    })
                }

                Event::Render => self
                    .systems
                    .render
                    .into_iter()
                    .for_each(|f| f(&mut self.components, &mut self.queue[1 - self.plex])),

                _ => {}
            }
        }
    }
}
