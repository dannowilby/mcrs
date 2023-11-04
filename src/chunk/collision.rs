use rapier3d::prelude::*;

use super::{ChunkConfig, ChunkData, Position};

/// creates a collider based on the chunk
pub fn calculate_collider(
    chunk: &ChunkData,
    chunk_pos: &Position,
    config: &ChunkConfig,
) -> Collider {
    let mut collider_data = Vec::new();

    for x in 0..config.depth {
        for y in 0..config.depth {
            for z in 0..config.depth {
                if let Some(block_id) = chunk.get(&(x, y, z)) {
                    if let Some(block) = config.dict.get(block_id) {
                        if !block.transparent {
                            collider_data.push((
                                Isometry::translation(x as f32, y as f32, z as f32),
                                SharedShape::cuboid(0.5, 0.5, 0.5),
                            ));
                        }
                    }
                }
            }
        }
    }

    if collider_data.len() < 1 {
        return ColliderBuilder::ball(0.5).build();
    }

    let cx = chunk_pos.0 as f32 * config.depth as f32;
    let cy = chunk_pos.1 as f32 * config.depth as f32;
    let cz = chunk_pos.2 as f32 * config.depth as f32;
    let translation = vector![cx, cy, cz]; // Isometry3::translation(cx, cy, cz).to_matrix();
    ColliderBuilder::compound(collider_data)
        .translation(translation).friction(0.0)
        .build()
}
