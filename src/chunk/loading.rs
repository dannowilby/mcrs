use crate::engine::input::Input;
use crate::engine::matrix::Matrix;
use crate::engine::renderer::Renderer;
use crate::world::{Event, GameData};

use super::collision::calculate_collider;
use super::generation::load_chunk;
use super::meshing::mesh_chunk;
use super::{calc_lod, chunk_id, chunk_position, player_to_position};

// TODO (stetch):
// - Frustrum culling
// - Occulsion culling
pub fn load_world(
    renderer: &mut Renderer,
    _input: &mut Input,
    data: &mut GameData,
    _queue: &mut Vec<Event>,
    _delta: f64,
) {
    let thread_pool = &data.thread_pool;
    // chunk loading dimensions
    let (i, j, k) = chunk_position(
        &data.chunk_config,
        &player_to_position(&data.player.position),
    );
    let radius = data.player.load_radius as i32;

    let mut chunks_to_remove: Vec<String> =
        data.loaded_chunks.iter().map(|(k, _)| k.clone()).collect();

    let mut chunks_to_load = Vec::new();
    // calculate chunks to modify
    for x in (i - radius)..(i + radius) {
        for y in (j - radius)..(j + radius) {
            for z in (k - radius)..(k + radius) {
                let chunk_id = chunk_id(&(x, y, z));

                let index = chunks_to_remove.iter().position(|r| r == &chunk_id);
                if let Some(x) = index {
                    chunks_to_remove.swap_remove(x);
                }

                // if loaded chunks doesn't contain it, but it should
                if !data.loaded_chunks.contains_key(&chunk_id) && !data.loading.contains(&chunk_id)
                {
                    chunks_to_load.push((chunk_id, (x, y, z)));
                }
            }
        }
    }

    for (chunk_id, chunk_pos) in chunks_to_load.into_iter() {
        data.loading.insert(chunk_id.clone());

        let config = data.chunk_config.clone();
        let done_loading = data.done_loading.clone();
        thread_pool.spawn(move || {
            let chunk = load_chunk(&config, &chunk_pos);
            let mesh = mesh_chunk(&chunk, &config, calc_lod());
            let collider = calculate_collider(&chunk, &chunk_pos, &config);

            // collider.set_translation(translation);

            let mut done_loading = done_loading.lock(5).unwrap();
            done_loading.insert(chunk_id, (chunk_pos, chunk, mesh, collider));
        })
    }

    // remove unneeded chunks
    for c in chunks_to_remove {
        data.loaded_chunks.remove(&c);
        data.physics_engine.remove_collider(&c);
        renderer.remove_object(&c);
    }
}

pub fn check_done_load_world(
    renderer: &mut Renderer,
    _input: &mut Input,
    data: &mut GameData,
    _queue: &mut Vec<Event>,
    _delta: f64,
) {
    let mut done_loading = data.done_loading.lock(0).unwrap();
    for (chunk_id, (chunk_pos, chunk, mesh, collider)) in done_loading.drain() {
        data.loading.remove(&chunk_id);
        data.loaded_chunks.insert(chunk_id.clone(), chunk);
        data.physics_engine
            .insert_collider(chunk_id.clone(), collider);
        let (x, y, z) = chunk_pos;
        let mat = Matrix::new(glam::Mat4::from_translation(glam::f32::vec3(
            x as f32 * data.chunk_config.depth as f32,
            y as f32 * data.chunk_config.depth as f32,
            z as f32 * data.chunk_config.depth as f32,
        )))
        .uniform(&Matrix::create_layout(2));
        renderer.add_object(&chunk_id, mesh);
        renderer.set_object_uniform(&chunk_id, "model", mat);
    }
}
