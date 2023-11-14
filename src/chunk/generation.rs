use super::{ChunkConfig, ChunkData, Position};

// need to rework this function
pub fn ground_threshold(_config: &ChunkConfig, pos: i32) -> f64 {
    let ground_level = 0;
    let change = 32;
    let _max_threshold = 0.5;
    let min_threshold = -0.05;

    let bias = (pos - ground_level) as f64 / change as f64;

    if pos > ground_level {
        return bias;
    }
    bias.max(min_threshold)
}

pub fn island_threshold(config: &ChunkConfig, pos: [i32; 3]) -> f64 {
    let [x, _y, z] = pos;
    let t = config.depth as f64 * f64::exp(-(x as f64).abs()) - 1.0;
    let r = config.depth as f64 * f64::exp(-(z as f64).abs()) - 1.0;
    t.min(r)
}

// could abstract out the generating functions
// would just overcomplicate things at the moment
pub fn generate(config: &ChunkConfig, pos: &Position) -> ChunkData {
    let mut output = ChunkData::new();
    generate_terrain(config, pos, &mut output);
    generate_foliage(config, pos, &mut output);
    // generate ores
    // generate structures

    output
}

pub fn generate_foliage(config: &ChunkConfig, pos: &Position, output: &mut ChunkData) {
    for (k, v) in output.iter_mut() {
        let (x, y, z) = k;
        let global_position = [
            x + pos.0 * config.depth,
            y + pos.1 * config.depth,
            z + pos.2 * config.depth,
        ];
        if has_air_within_dist(config, global_position, 4) {
            *v = 3;
        }
        if has_air_within_dist(config, global_position, 2) {
            *v = 1;
        }
    }
}

use libnoise::prelude::*;

pub fn has_air_within_dist(config: &ChunkConfig, pos: [i32; 3], dist: i32) -> bool {
    for i in 0..dist.abs() {
        if get_terrain_at(config, [pos[0], pos[1] + dist.signum() * i, pos[2]]) >= 0.0 {
            return true;
        }
    }

    false
}

pub fn get_terrain_at(config: &ChunkConfig, global_position: [i32; 3]) -> f64 {
    let mut noise_position = [
        config.noise_amplitude.0 * global_position[0] as f64,
        config.noise_amplitude.1 * global_position[1] as f64,
        config.noise_amplitude.2 * global_position[2] as f64,
    ];

    let mut noise = 0.0;
    let mut amplitude = 0.5;
    for _ in 0..4 {
        noise += amplitude * (&config.noise).sample(noise_position);
        noise_position[0] *= 2.0;
        noise_position[1] *= 2.0;
        noise_position[2] *= 2.0;
        amplitude *= 0.5;
    }

    noise + ground_threshold(config, global_position[1])
    // + island_threshold(config, global_position)
}

pub fn generate_terrain(config: &ChunkConfig, pos: &Position, output: &mut ChunkData) {
    for x in (-1)..(config.depth + 1) {
        for y in (-1)..(config.depth + 1) {
            for z in (-1)..(config.depth + 1) {
                let position = (x, y, z);

                let global_position = [
                    x + pos.0 * config.depth,
                    y + pos.1 * config.depth,
                    z + pos.2 * config.depth,
                ];

                if get_terrain_at(config, global_position) < 0.0 {
                    output.insert(position, 2);
                }
            }
        }
    }
}

pub fn load_chunk(config: &ChunkConfig, pos: &Position) -> ChunkData {
    // load chuhnk from fs

    // if no chunk data found,
    // generate chunk
    generate(config, pos)
}

#[allow(dead_code)]
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
