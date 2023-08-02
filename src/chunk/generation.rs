use super::{ChunkConfig, ChunkData, ChunkPos};

use noise::NoiseFn;

pub fn generate(config: &ChunkConfig, pos: ChunkPos) -> ChunkData {
    let mut output = ChunkData::new();

    for x in 0..config.depth {
        for y in 0..config.depth {
            for z in 0..config.depth {
                let position = (x, y, z);
                let noise_position = [
                    config.noise_amplitude.0 * (x + pos.0 * config.depth) as f64,
                    config.noise_amplitude.0 * (y + pos.1 * config.depth) as f64,
                    config.noise_amplitude.0 * (z + pos.2 * config.depth) as f64,
                ];
                let noise = config.noise.get(noise_position);

                if noise > 0.0 {
                    output.insert(position, 1);
                }
            }
        }
    }

    output
}

pub fn load_chunk(config: &ChunkConfig, pos: ChunkPos) -> ChunkData {
    // load chuhnk from fs

    // if no chunk data found,
    // generate chunk
    generate(config, pos)
}
