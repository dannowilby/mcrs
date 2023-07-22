use std::collections::HashMap;

use crate::engine::game_state::GameState;
use crate::engine::renderer::Renderer;

#[derive(PartialEq, Eq, Hash)]
pub enum Event {
    Init,
}

pub struct GameData {
    // component data
    chunk_data: HashMap<String, HashMap<(u32, u32, u32), u32>>,

    // singleton data
    player_position: (u32, u32, u32),
}

fn load_world(renderer: &mut Renderer, data: &mut GameData, queue: &mut Vec<Event>) {

    // generate chunks
}

pub fn init() -> GameState<GameData, Event> {
    let mut game_state = GameState::new(
        Renderer::new(),
        GameData {
            chunk_data: HashMap::new(),

            player_position: (0, 0, 0),
        },
    );

    game_state.add_system(Event::Init, load_world);

    game_state
}
