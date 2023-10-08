use super::{block::BlockDictionary, chunk_id, ChunkConfig, ChunkData, ChunkPos};
use std::collections::HashMap;

use noise::NoiseFn;

pub fn ao_test() -> ChunkData {
    let mut output = ChunkData::new();

    output.insert((0, 0, 0), 1);
    output.insert((0, 0, 1), 1);
    output.insert((1, 0, 0), 1);
    output.insert((1, 1, 1), 1);
    output.insert((2, 0, 0), 1);
    output.insert((0, 0, 2), 1);
    output.insert((1, 0, 2), 1);
    output.insert((2, 0, 1), 1);
    output.insert((2, 0, 2), 1);
    output.insert((0, 2, 0), 1);
    output.insert((0, 2, 1), 1);
    output.insert((1, 2, 0), 1);
    output.insert((2, 2, 0), 1);
    output.insert((0, 2, 2), 1);
    output.insert((1, 2, 2), 1);
    output.insert((2, 2, 1), 1);
    output.insert((2, 2, 2), 1);

    output.insert((4, 0, 0), 1);
    output.insert((4, 1, 0), 1);
    output.insert((4, 2, 0), 1);
    output.insert((5, 0, 0), 1);
    output.insert((5, 2, 0), 1);
    output.insert((6, 0, 0), 1);
    output.insert((6, 1, 0), 1);
    output.insert((6, 2, 0), 1);
    output.insert((5, 1, 1), 1);
    output.insert((4, 0, 2), 1);
    output.insert((4, 1, 2), 1);
    output.insert((4, 2, 2), 1);
    output.insert((5, 0, 2), 1);
    output.insert((5, 2, 2), 1);
    output.insert((6, 0, 2), 1);
    output.insert((6, 1, 2), 1);
    output.insert((6, 2, 2), 1);

    output.insert((8, 0, 0), 1);
    output.insert((8, 0, 1), 1);
    output.insert((8, 0, 2), 1);
    output.insert((8, 1, 0), 1);
    output.insert((8, 1, 2), 1);
    output.insert((8, 2, 0), 1);
    output.insert((8, 2, 1), 1);
    output.insert((8, 2, 2), 1);
    output.insert((9, 1, 1), 1);
    output.insert((10, 0, 0), 1);
    output.insert((10, 0, 1), 1);
    output.insert((10, 0, 2), 1);
    output.insert((10, 1, 0), 1);
    output.insert((10, 1, 2), 1);
    output.insert((10, 2, 0), 1);
    output.insert((10, 2, 1), 1);
    output.insert((10, 2, 2), 1);

    output
}

// need to rework this function
pub fn ground_threshold(config: &ChunkConfig, pos: i32) -> f64 {
    let ground_level = 0;
    let change = 64;
    let max_threshold = 0.5;
    let min_threshold = -0.05;

    let bias = (pos - ground_level) as f64 / change as f64;

    if pos > ground_level {
        return bias;
    }
    bias.max(min_threshold)
}

// could abstract out the generating functions
// would just overcomplicate things at the moment
pub fn generate(config: &ChunkConfig, pos: &ChunkPos) -> ChunkData {
    let mut output = ChunkData::new();
    generate_terrain(config, pos, &mut output);
    // generate_foliage(config, pos, &mut output);
    // generate ores
    // generate structures

    output
}

/*
fn get_local_height(
    config: &ChunkConfig,
    data: &ChunkData,
    pos: &ChunkPos,
    x: i32,
    z: i32,
) -> Option<i32> {
    for i in (0..(config.depth - 1)).rev() {
        if let Some(b) = data.get(&(x, i, z)) {
            // if there is a block at the top of the chunk
            if i == (config.depth - 1) {
                // then get the bottom block of the chunk above
                if let Some(c) = loaded_chunks.get(&chunk_id(pos.0, pos.1 + 1, pos.2)) {
                    if let Some(d) = c.get(&(x, 0, z)) {
                        // and if it is air, return the top
                        if dict.get(d).map_or(true, |b| b.transparent) {
                            return Some(i);
                        }
                    }
                }
            }
            return Some(i);
        }
    }

    None
}


pub fn generate_foliage(config: &ChunkConfig, pos: &ChunkPos, data: &mut ChunkData) {
    // get height

    // if chunk is less than ground level
    // then don't add grass/dirt/etc...
    // new feature idea: might look cooler if we don't do this
    // if pos.1 < 0 {
    //    return;
    // }

    for x in 0..config.depth {
        for z in 0..config.depth {
            if let Some(y) = get_local_height(config, dict, loaded_chunks, data, pos, x, z) {
                // add dirt
                for i in 1..4 {
                    data.insert((x, (y - i).max(0), z), 3);
                }
                // add grass at the top
                data.insert((x, y, z), 1);
            }
        }
    }

    // need a function to get top block of chunk,
    // well, uhh..., not that
    // function to get the y pos of the top block at x, y

    // if chunkpos is at or above ground height
    // cover with grass and trees and foliage
    // else
    // underground foliage
}
    */

pub fn generate_terrain(config: &ChunkConfig, pos: &ChunkPos, output: &mut ChunkData) {
    for x in 0..config.depth {
        for y in 0..config.depth {
            for z in 0..config.depth {
                let position = (x, y, z);

                let global_position = [
                    x + pos.0 * config.depth,
                    y + pos.1 * config.depth,
                    z + pos.2 * config.depth,
                ];
                let noise_position = [
                    config.noise_amplitude.0 * global_position[0] as f64,
                    config.noise_amplitude.1 * global_position[1] as f64,
                    config.noise_amplitude.2 * global_position[2] as f64,
                ];
                let noise =
                    (config.noise)(noise_position) + ground_threshold(config, global_position[1]);

                if noise < 0.0 {
                    output.insert(position, 2);
                }
            }
        }
    }
}

pub fn load_chunk(config: &ChunkConfig, pos: &ChunkPos) -> ChunkData {
    // load chuhnk from fs

    // if no chunk data found,
    // generate chunk
    generate(config, pos)
}
