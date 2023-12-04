use std::collections::{HashMap, VecDeque};
use std::slice::Iter;

use super::{
    super::util::vec_set::VecSet, chunk_id, ChunkConfig, ChunkData, ChunkStorage, Position,
};

#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum Side {
    FRONT = 0,
    BACK = 1,
    TOP = 2,
    BOTTOM = 3,
    LEFT = 4,
    RIGHT = 5,
}

impl Side {
    #[allow(dead_code)]
    pub fn normal(&self) -> glam::Vec3 {
        match self {
            Side::FRONT => glam::vec3(0.0, 0.0, 1.0),
            Side::BACK => glam::vec3(0.0, 0.0, -1.0),
            Side::TOP => glam::vec3(0.0, 1.0, 0.0),
            Side::BOTTOM => glam::vec3(0.0, -1.0, 0.0),
            Side::LEFT => glam::vec3(-1.0, 0.0, 0.0),
            Side::RIGHT => glam::vec3(1.0, 0.0, 0.0),
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
    pub fn iter() -> Iter<'static, Side> {
        static SIDES: [Side; 6] = [
            Side::FRONT,
            Side::BACK,
            Side::TOP,
            Side::BOTTOM,
            Side::LEFT,
            Side::RIGHT,
        ];

        SIDES.iter()
    }
}

/// A 6x6 matrix to keep track of which sides we can enter and exit from.
#[derive(Debug)]
pub struct VisibilityGraph([[bool; 6]; 6]);
pub type VisibilityGraphStorage = HashMap<String, VisibilityGraph>;

impl VisibilityGraph {
    pub const EMPTY_GRAPH: VisibilityGraph = VisibilityGraph([
        [true, false, false, false, false, false],
        [false, true, false, false, false, false],
        [false, false, true, false, false, false],
        [false, false, false, true, false, false],
        [false, false, false, false, true, false],
        [false, false, false, false, false, true],
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
                    if block.is_none()
                        || config.dict.get(block.unwrap()).unwrap().transparent
                            && !fill_seeds.contains(&(x, y, z))
                    {
                        fill_seeds.insert((x, y, z));
                    }
                }
            }
        }

        let mut visited = VecSet::new();
        while !fill_seeds.is_empty() {
            let pos = fill_seeds.remove_front().unwrap();
            // flood fill to get sides we can exit out of
            let sides = flood_fill(&config, &chunk_data, &pos, &mut fill_seeds, &mut visited);

            // create tuples of sides that can reach each other from the result of flood fill
            // add the tuples to the connections vec
            for i in 0..(sides.len() - 1) {
                for j in i..sides.len() {
                    let connection1 = (sides[i], sides[j]);
                    let connection2 = (sides[j], sides[i]);
                    if !connections.contains(&connection1) {
                        connections.push(connection1);
                    }
                    if !connections.contains(&connection2) {
                        connections.push(connection2);
                    }
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
    visited: &mut VecSet<Position>,
) -> Vec<Side> {
    // might be more semantic to return a set as we want the values in the vec to be unique
    let mut output = Vec::new();
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
        visited.insert(pos.clone());

        // skip if pos isn't in the dimensions of the chunk
        if pos.0 < 0
            || pos.0 + 1 > config.depth
            || pos.1 < 0
            || pos.1 + 1 > config.depth
            || pos.2 < 0
            || pos.2 + 1 > config.depth
        {
            continue;
        }

        // check if air
        // if not continue
        let block = chunk_data.get(&pos);
        if block.is_some() && !config.dict.get(block.unwrap()).unwrap().transparent {
            continue;
        }

        // if it is air/transparent
        // check if it is on the side
        // if on the side, add the side to the output if not already added
        // and add its neighbors to the queue
        if pos.0 == 0 && !output.contains(&Side::RIGHT) {
            output.push(Side::RIGHT);
        }
        if pos.0 == config.depth - 1 && !output.contains(&Side::LEFT) {
            output.push(Side::LEFT);
        }

        if pos.1 == 0 && !output.contains(&Side::BOTTOM) {
            output.push(Side::BOTTOM);
        }
        if pos.1 == config.depth - 1 && !output.contains(&Side::TOP) {
            output.push(Side::TOP);
        }

        if pos.2 == 0 && !output.contains(&Side::FRONT) {
            output.push(Side::FRONT);
        }
        if pos.2 == config.depth - 1 && !output.contains(&Side::BACK) {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_config() -> ChunkConfig {
        ChunkConfig::new(10, 8, 3)
    }

    #[test]
    fn visibility_graph_full_chunk() {
        let config = create_mock_config();
        let mut full_chunk_data = ChunkData::new();

        // fill the chunk with blocks
        for x in 0..config.depth {
            for y in 0..config.depth {
                for z in 0..config.depth {
                    // 2 is just some random transparent that we want to check
                    full_chunk_data.insert((x, y, z), 2);
                }
            }
        }

        let vis_graph = VisibilityGraph::from_chunk(&config, &full_chunk_data);

        // iter over each entry to check if any are false
        for side1 in Side::iter() {
            for side2 in Side::iter() {
                if side1 == side2 {
                    assert!(vis_graph.can_reach_from(*side1, *side2));
                    continue;
                }
                assert!(!vis_graph.can_reach_from(*side1, *side2));
            }
        }
    }

    #[test]
    fn visibility_graph_empty_chunk() {
        let config = create_mock_config();
        let empty_chunk_data = ChunkData::new();
        let vis_graph = VisibilityGraph::from_chunk(&config, &empty_chunk_data);

        // iter over each entry to check if any are false
        for side1 in Side::iter() {
            for side2 in Side::iter() {
                assert!(vis_graph.can_reach_from(*side1, *side2));
            }
        }
    }

    #[test]
    fn visibility_graph_split_chunk() {
        let config = create_mock_config();
        let mut split_chunk_data = ChunkData::new();

        // fill the chunk with blocks
        for x in 0..config.depth {
            for y in 0..config.depth {
                for z in 0..config.depth {
                    // 2 is just some random transparent that we want to check
                    if x == 3 {
                        split_chunk_data.insert((x, y, z), 2);
                    }
                }
            }
        }

        let vis_graph = VisibilityGraph::from_chunk(&config, &split_chunk_data);

        for side1 in Side::iter() {
            for side2 in Side::iter() {
                if (*side1 == Side::RIGHT && *side2 == Side::LEFT)
                    || (*side2 == Side::RIGHT && *side1 == Side::LEFT)
                {
                    assert!(!vis_graph.can_reach_from(*side1, *side2));
                    continue;
                }

                assert!(
                    vis_graph.can_reach_from(*side1, *side2),
                    "{:?} - {:?}",
                    side1,
                    side2
                );
            }
        }
    }
}
