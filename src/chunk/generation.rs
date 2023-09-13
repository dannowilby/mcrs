use super::{block::BlockDictionary, cube_model::is_transparent, ChunkConfig, ChunkData, ChunkPos};
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

pub fn ground_threshold(config: &ChunkConfig, pos: i32) -> f64 {
    let ground_level = 8;
    let change = 64;
    let max_threshold = 0.5;
    let min_threshold = -1.0;
    (pos - ground_level) as f64 / change as f64
}

pub fn generate(
    loaded_chunks: &HashMap<String, HashMap<(i32, i32, i32), u32>>,
    dict: &BlockDictionary,
    config: &ChunkConfig,
    pos: &ChunkPos,
) -> ChunkData {
    let mut output = generate_terrain(loaded_chunks, dict, config, pos);
    // output = generate_foliage(loaded_chunks, dict, config, pos, output);
    // generate foliage
    // generate ores
    // generate structures

    output
}

pub fn generate_foliage(
    loaded_chunks: &HashMap<String, HashMap<(i32, i32, i32), u32>>,
    dict: &BlockDictionary,
    config: &ChunkConfig,
    pos: &ChunkPos,
    data: ChunkData,
) -> ChunkData {
    let mut output = ChunkData::new();

    for (pos, id) in data.iter() {}

    output
}
pub fn generate_terrain(
    loaded_chunks: &HashMap<String, HashMap<(i32, i32, i32), u32>>,
    dict: &BlockDictionary,
    config: &ChunkConfig,
    pos: &ChunkPos,
) -> ChunkData {
    let mut output = ChunkData::new();

    for x in 0..config.depth {
        for y in 0..config.depth {
            for z in 0..config.depth {
                let position = (x, y, z);

                let noise_position = [
                    config.noise_amplitude.0 * (x + pos.0 * config.depth) as f64,
                    config.noise_amplitude.1 * (y + pos.1 * config.depth) as f64,
                    config.noise_amplitude.2 * (z + pos.2 * config.depth) as f64,
                ];
                let noise = config.noise.get(noise_position)
                    + ground_threshold(config, pos.1 * config.depth + y);

                if noise < 0.0 {
                    output.insert(position, 2);
                }

                /*
                if (x * x + z * z) + y < 16 || (x * x + z * z) - y > 64 {
                    output.insert(position, 1);
                }
                */
            }
        }
    }

    output
}

pub fn load_chunk(
    loaded_chunks: &HashMap<String, HashMap<(i32, i32, i32), u32>>,
    dict: &BlockDictionary,
    config: &ChunkConfig,
    pos: &ChunkPos,
) -> ChunkData {
    // load chuhnk from fs

    // if no chunk data found,
    // generate chunk
    generate(loaded_chunks, dict, config, pos)
}
