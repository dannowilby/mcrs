use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    engine::input::Input,
    world::{Event, GameData},
    world_renderer::WorldRenderer,
};

use super::{
    chunk_id, chunk_position, player_to_position, vec_set::VecSet, ChunkConfig, ChunkData,
    ChunkStorage, Position,
};

#[derive(Clone, Copy)]
pub enum Side {
    FRONT = 0,
    BACK = 1,
    TOP = 2,
    BOTTOM = 3,
    LEFT = 4,
    RIGHT = 5,
}

impl Side {
    pub fn normal(&self) -> glam::Vec3 {
        match self {
            Side::FRONT => glam::vec3(0.0, 0.0, 1.0),
            Side::BACK => glam::vec3(0.0, 0.0, -1.0),
            Side::TOP => glam::vec3(0.0, -1.0, 0.0),
            Side::BOTTOM => glam::vec3(0.0, 1.0, 0.0),
            Side::LEFT => glam::vec3(1.0, 0.0, 0.0),
            Side::RIGHT => glam::vec3(-1.0, 0.0, 0.0),
        }
    }
    pub fn opposite(self) -> Self {
        match self {
            Side::FRONT => Side::BACK,
            Side::BACK => Side::FRONT,
            Side::TOP => Side::BOTTOM,
            Side::BOTTOM => Side::TOP,
            Side::LEFT => Side::RIGHT,
            Side::RIGHT => Side::LEFT,
        }
    }
}

/// A 6x6 matrix to keep track of which sides we can enter and exit from.
pub struct VisibilityGraph([[bool; 6]; 6]);
pub type VisibilityGraphStorage = HashMap<String, VisibilityGraph>;

impl VisibilityGraph {
    pub const EMPTY_GRAPH: VisibilityGraph = VisibilityGraph([
        [false, false, false, false, false, false],
        [false, false, false, false, false, false],
        [false, false, false, false, false, false],
        [false, false, false, false, false, false],
        [false, false, false, false, false, false],
        [false, false, false, false, false, false],
    ]);

    // flood fill in this function
    pub fn from_chunk(config: &ChunkConfig, chunk_data: &ChunkData) -> Self {
        let mut connections = Vec::<(Side, Side)>::new();
        let depth = config.depth;

        // get a vector of blocks we can flood fill on
        let mut fill_seeds = VecSet::new();
        for x in 0..depth {
            for y in 0..depth {
                for z in 0..depth {
                    // only do flood fill if we are on the edge of the chunk
                    if x != 0
                        && x - 1 != depth
                        && y != 0
                        && y - 1 != depth
                        && z != 0
                        && z - 1 != depth
                    {
                        continue;
                    }

                    let block = chunk_data.get(&(x, y, z));

                    // start at an empty block
                    if block.is_none() || config.dict.get(block.unwrap()).unwrap().transparent {
                        fill_seeds.insert((x, y, z));
                    }
                }
            }
        }

        while !fill_seeds.is_empty() {
            let pos = fill_seeds.remove_front().unwrap();
            // flood fill to get sides we can exit out of
            let sides = flood_fill(&config, &chunk_data, &pos, &mut fill_seeds);

            // create tuples of sides that can reach each other from the result of flood fill
            // add the tuples to the connections vec
            for i in 0..(sides.len() - 1) {
                for j in i..sides.len() {
                    connections.push((sides[i], sides[j]));
                    connections.push((sides[j], sides[i]));
                }
            }
        }

        // start the floodfill
        // everytime we hit a block in our vector, we remove it

        // once we have our connections vec,
        // modify the graph to reflect the connections we found
        let mut output = VisibilityGraph::EMPTY_GRAPH;
        for (side1, side2) in connections.into_iter() {
            let x = side1 as usize;
            let y = side2 as usize;
            output.0[x][y] = true;
        }

        output
    }

    pub fn can_reach_from(&self, side1: Side, side2: Side) -> bool {
        let matrix = self.0;

        let x = side1 as usize;
        let y = side2 as usize;

        matrix[x][y]
    }
}

fn flood_fill(
    config: &ChunkConfig,
    chunk_data: &ChunkData,
    start_pos: &Position,
    fill_seeds: &mut VecSet<Position>,
) -> Vec<Side> {
    // might be more semantic to return a set as we want the values in the vec to be unique
    let mut output = Vec::new();
    let mut visited = Vec::new();
    let mut stack = VecDeque::<Position>::from([start_pos.clone()]);

    while !stack.is_empty() {
        let pos = stack.pop_front().unwrap();

        // remove it from the possible seeds in the future
        // this will help cut down on duplicate fills
        fill_seeds.remove(&pos);

        // if we've already visited, then continue
        if visited.contains(&pos) {
            continue;
        }
        visited.push(pos.clone());

        // skip if pos isn't in the dimensions of the chunk
        if pos.0 < 0
            || pos.0 - 1 > config.depth
            || pos.1 < 0
            || pos.1 - 1 > config.depth
            || pos.2 < 0
            || pos.2 - 1 > config.depth
        {
            continue;
        }
        
        // check if air
        // if not continue
        let block = chunk_data.get(&pos);
        if block.is_some()
            && !config
                .dict
                .get(block.unwrap())
                .unwrap_or_default()
                .transparent
        {
            continue;
        }

        // if it is air/transparent
        // check if it is on the side
        // if on the side, add the side to the output if not already added
        // and add its neighbors to the queue
        if pos.0 == 0 {
            output.push(Side::RIGHT);
        }
        if pos.0 == config.depth - 1 {
            output.push(Side::LEFT);
        }

        if pos.1 == 0 {
            output.push(Side::BOTTOM);
        }
        if pos.1 == config.depth - 1 {
            output.push(Side::TOP);
        }

        if pos.2 == 0 {
            output.push(Side::FRONT);
        }
        if pos.2 == config.depth - 1 {
            output.push(Side::BACK);
        }

        stack.push_back((pos.0 + 1, pos.1, pos.2));
        stack.push_back((pos.0 - 1, pos.1, pos.2));
        stack.push_back((pos.0, pos.1 + 1, pos.2));
        stack.push_back((pos.0, pos.1 - 1, pos.2));
        stack.push_back((pos.0, pos.1, pos.2 + 1));
        stack.push_back((pos.0, pos.1, pos.2 - 1));
    }

    output
}

/// Returns a vector of loaded chunks neighboring the passed in chunk_pos.
pub fn get_neighbors(loaded_chunks: &ChunkStorage, pos: &Position) -> Vec<(Side, Position)> {
    let mut output = Vec::new();

    let top = (pos.0, pos.1 + 1, pos.2);
    if loaded_chunks.contains_key(&chunk_id(&top)) {
        output.push((Side::TOP, top));
    }

    let bottom = (pos.0, pos.1 - 1, pos.2);
    if loaded_chunks.contains_key(&chunk_id(&bottom)) {
        output.push((Side::BOTTOM, bottom));
    }

    let left = (pos.0 - 1, pos.1, pos.2);
    if loaded_chunks.contains_key(&chunk_id(&left)) {
        output.push((Side::LEFT, left));
    }

    let right = (pos.0 + 1, pos.1, pos.2);
    if loaded_chunks.contains_key(&chunk_id(&right)) {
        output.push((Side::RIGHT, right));
    }

    let front = (pos.0, pos.1, pos.2 - 1);
    if loaded_chunks.contains_key(&chunk_id(&front)) {
        output.push((Side::FRONT, front));
    }

    let back = (pos.0, pos.1, pos.2 + 1);
    if loaded_chunks.contains_key(&chunk_id(&back)) {
        output.push((Side::BACK, back));
    }

    output
}

/*
/// implement the visibility graph here
pub fn visibility_cull(
    renderer: &mut WorldRenderer,
    _input: &mut Input,
    data: &mut GameData,
    _queue: &mut Vec<Event>,
    _delta: f64,
) {
    let player = data.physics_engine.get_rigid_body("player".to_string());
    if player.is_none() {
        return;
    }
    let pos = player.unwrap().translation();

    let (yaw_sin, yaw_cos) = data.player.yaw.sin_cos();
    let (pitch_sin, pitch_cos) = data.player.pitch.sin_cos();
    let facing = glam::vec3(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();

    let chunk_pos = chunk_position(
        &data.chunk_config,
        &player_to_position(&(pos.x, pos.y, pos.z)),
    );

    // reset chunks to be rendered, set them all to invisible
    for (_, v) in renderer.object_render_pass.render_objects.iter_mut() {
        v.visible = false;
    }

    // set up a search queue, start with the chunk the player is in.
    let mut search_queue = VecDeque::<(Option<Side>, Position)>::from([(None, chunk_pos)]);

    while !search_queue.is_empty() {
        let (from_side, chunk_pos) = search_queue
            .pop_front()
            .expect("Queue was made unexpectedly empty");

        // make this chunk visible
        if let Some(chunk) = renderer
            .object_render_pass
            .render_objects
            .get_mut(&chunk_id(&chunk_pos))
        {
            chunk.visible = true;
        }

        get_neighbors(&data.loaded_chunks, &chunk_pos)
            .into_iter()
            .for_each(|(to_side, chunk_pos)| {
                // correct direction filter:
                // check if neighbor is in forward direction we are looking
                // if the dot product is negative, then it should render
                if to_side.normal().dot(facing) >= 0.0 {
                    return;
                }

                // visibility filter:
                // check the chunk's visibility graph to see if we can reach it.
                /*
                let visibility_graph = VisibilityGraph::TEST_GRAPH;
                if let Some(side) = from_side {
                    if !visibility_graph.can_reach_from(side, to_side) {
                        return;
                    }
                }
                */

                // might want to add an actual frustum cull step here.

                // if chunk has passed the filters, then add it
                search_queue.push_back((Some(to_side.opposite()), chunk_pos.clone()));
            });
    }
}
*/
