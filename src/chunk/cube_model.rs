use crate::chunk::{block::BlockDictionary, get_block, meshing::Vertex, ChunkConfig, ChunkData};
use std::collections::HashMap;

use super::is_transparent;

fn ao(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    p1: &(i32, i32, i32),
    p2: &(i32, i32, i32),
    p3: &(i32, i32, i32),
) -> f32 {
    let mut output = 1.0;

    if !is_transparent(chunk_config, loaded_chunks, dict, p1)
        || !is_transparent(chunk_config, loaded_chunks, dict, p2)
        || !is_transparent(chunk_config, loaded_chunks, dict, p3)
    {
        output = 0.5;
    }

    output
}

// return the faces (vertices, indices) for the block
// these types of functions might get pretty nasty
pub fn cube_model(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    position: &(i32, i32, i32),
    last_index: u16,
) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let mut local_last_index = last_index;

    let (p0, p1, p2) = position;
    let block = get_block(chunk_config, loaded_chunks, &position);
    let target_block = dict.get(&block);
    // only add faces if we need to
    if target_block.is_none() || target_block.unwrap().transparent {
        return (vertices, indices);
    }

    let uv = target_block.unwrap().uv;

    // top
    let top_face_visible = dict
        .get(&get_block(chunk_config, loaded_chunks, &(*p0, p1 + 1, *p2)))
        .map_or(true, |b| b.transparent);
    if top_face_visible {
        let (mut verts, mut inds) = top_face(
            chunk_config,
            loaded_chunks,
            dict,
            position,
            &uv,
            local_last_index,
        );
        local_last_index = local_last_index + verts.len() as u16;
        vertices.append(&mut verts);
        indices.append(&mut inds);
    }

    // bottom
    let bottom_face_visible = dict
        .get(&get_block(chunk_config, loaded_chunks, &(*p0, p1 - 1, *p2)))
        .map_or(true, |b| b.transparent);
    if bottom_face_visible {
        let (mut verts, mut inds) = bottom_face(
            chunk_config,
            loaded_chunks,
            dict,
            position,
            &uv,
            local_last_index,
        );
        local_last_index = local_last_index + verts.len() as u16;
        vertices.append(&mut verts);
        indices.append(&mut inds);
    }

    // front
    let front_face_visible = dict
        .get(&get_block(chunk_config, loaded_chunks, &(*p0, *p1, p2 + 1)))
        .map_or(true, |b| b.transparent);
    if front_face_visible {
        let (mut verts, mut inds) = front_face(
            chunk_config,
            loaded_chunks,
            dict,
            position,
            &uv,
            local_last_index,
        );
        local_last_index = local_last_index + verts.len() as u16;
        vertices.append(&mut verts);
        indices.append(&mut inds);
    }

    // back
    let back_face_visible = dict
        .get(&get_block(chunk_config, loaded_chunks, &(*p0, *p1, p2 - 1)))
        .map_or(true, |b| b.transparent);
    if back_face_visible {
        let (mut verts, mut inds) = back_face(
            chunk_config,
            loaded_chunks,
            dict,
            position,
            &uv,
            local_last_index,
        );
        local_last_index = local_last_index + verts.len() as u16;
        vertices.append(&mut verts);
        indices.append(&mut inds);
    }

    let right_face_visible = dict
        .get(&get_block(
            chunk_config,
            loaded_chunks,
            &(*p0 + 1, *p1, *p2),
        ))
        .map_or(true, |b| b.transparent);
    if right_face_visible {
        let (mut verts, mut inds) = right_face(
            chunk_config,
            loaded_chunks,
            dict,
            position,
            &uv,
            local_last_index,
        );
        local_last_index = local_last_index + verts.len() as u16;
        vertices.append(&mut verts);
        indices.append(&mut inds);
    }

    let left_face_visible = dict
        .get(&get_block(
            chunk_config,
            loaded_chunks,
            &(*p0 - 1, *p1, *p2),
        ))
        .map_or(true, |b| b.transparent);
    if left_face_visible {
        let (mut verts, mut inds) = left_face(
            chunk_config,
            loaded_chunks,
            dict,
            position,
            &uv,
            local_last_index,
        );
        // local_last_index = local_last_index + verts.len() as u16;
        vertices.append(&mut verts);
        indices.append(&mut inds);
    }

    (vertices, indices)
}

fn top_face(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    position: &(i32, i32, i32),
    uv: &[f32; 2],
    last_index: u16,
) -> (Vec<Vertex>, Vec<u16>) {
    let (i, j, k) = position;
    let x = *i as f32;
    let y = *j as f32;
    let z = *k as f32;
    let u = uv[0];
    let v = uv[1];
    let duv = chunk_config.uv_size;

    (
        vec![
            Vertex {
                position: [x, y + 1.0, z + 1.0],
                uv: [u, v],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i - 1, *j, *k),
                    &(*i, *j, *k + 1),
                    &(*i - 1, *j, *k + 1),
                ),
            },
            Vertex {
                position: [x + 1.0, y + 1.0, z + 1.0],
                uv: [u + duv, v],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i + 1, *j, *k),
                    &(*i, *j, *k + 1),
                    &(*i + 1, *j, *k + 1),
                ),
            },
            Vertex {
                position: [x, y + 1.0, z],
                uv: [u, v + duv],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i, *j, *k - 1),
                    &(*i - 1, *j, *k),
                    &(*i - 1, *j, *k - 1),
                ),
            },
            Vertex {
                position: [x + 1.0, y + 1.0, z],
                uv: [u + duv, v + duv],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i + 1, *j, *k),
                    &(*i, *j, *k - 1),
                    &(*i + 1, *j, *k - 1),
                ),
            },
        ],
        vec![
            last_index + 0,
            last_index + 3,
            last_index + 2,
            last_index + 0,
            last_index + 1,
            last_index + 3,
        ],
    )
}

fn bottom_face(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    position: &(i32, i32, i32),
    uv: &[f32; 2],
    last_index: u16,
) -> (Vec<Vertex>, Vec<u16>) {
    let (i, j, k) = position;
    let x = *i as f32;
    let y = *j as f32;
    let z = *k as f32;
    let u = uv[0];
    let v = uv[1];
    let duv = chunk_config.uv_size;

    (
        vec![
            Vertex {
                position: [x, y, z + 1.0],
                uv: [u, v],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i - 1, *j - 2, *k),
                    &(*i, *j - 2, *k + 1),
                    &(*i - 1, *j - 2, *k + 1),
                ),
            },
            Vertex {
                position: [x + 1.0, y, z + 1.0],
                uv: [u + duv, v],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i + 1, *j - 2, *k),
                    &(*i, *j - 2, *k + 1),
                    &(*i + 1, *j - 2, *k + 1),
                ),
            },
            Vertex {
                position: [x, y, z],
                uv: [u, v + duv],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i - 1, *j - 2, *k),
                    &(*i, *j - 2, *k - 1),
                    &(*i - 1, *j - 2, *k - 1),
                ),
            },
            Vertex {
                position: [x + 1.0, y, z],
                uv: [u + duv, v + duv],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i + 1, *j - 2, *k),
                    &(*i, *j - 2, *k - 1),
                    &(*i + 1, *j - 2, *k - 1),
                ),
            },
        ],
        vec![
            last_index + 0,
            last_index + 2,
            last_index + 3,
            last_index + 0,
            last_index + 3,
            last_index + 1,
        ],
    )
}

fn front_face(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    position: &(i32, i32, i32),
    uv: &[f32; 2],
    last_index: u16,
) -> (Vec<Vertex>, Vec<u16>) {
    let (i, j, k) = position;
    let x = *i as f32;
    let y = *j as f32;
    let z = *k as f32;
    let u = uv[0];
    let v = uv[1];
    let duv = chunk_config.uv_size;

    (
        vec![
            Vertex {
                position: [x, y, z + 1.0],
                uv: [u, v],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i - 1, *j - 1, *k + 1),
                    &(*i, *j - 2, *k + 1),
                    &(*i - 1, *j - 2, *k + 1),
                ),
            },
            Vertex {
                position: [x + 1.0, y, z + 1.0],
                uv: [u + duv, v],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i + 1, *j - 1, *k + 1),
                    &(*i, *j - 2, *k + 1),
                    &(*i + 1, *j - 2, *k + 1),
                ),
            },
            Vertex {
                position: [x, y + 1.0, z + 1.0],
                uv: [u, v + duv],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i, *j, *k + 1),
                    &(*i - 1, *j - 1, *k + 1),
                    &(*i - 1, *j, *k + 1),
                ),
            },
            Vertex {
                position: [x + 1.0, y + 1.0, z + 1.0],
                uv: [u + duv, v + duv],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i, *j, *k + 1),
                    &(*i + 1, *j - 1, *k + 1),
                    &(*i + 1, *j, *k + 1),
                ),
            },
        ],
        vec![
            last_index + 0,
            last_index + 3,
            last_index + 2,
            last_index + 0,
            last_index + 1,
            last_index + 3,
        ],
    )
}

fn back_face(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    position: &(i32, i32, i32),
    uv: &[f32; 2],
    last_index: u16,
) -> (Vec<Vertex>, Vec<u16>) {
    let (i, j, k) = position;
    let x = *i as f32;
    let y = *j as f32;
    let z = *k as f32;
    let u = uv[0];
    let v = uv[1];
    let duv = chunk_config.uv_size;

    (
        vec![
            Vertex {
                position: [x, y, z],
                uv: [u, v],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i - 1, *j - 1, *k - 1),
                    &(*i, *j - 2, *k - 1),
                    &(*i - 1, *j - 2, *k - 1),
                ),
            },
            Vertex {
                position: [x + 1.0, y, z],
                uv: [u + duv, v],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i + 1, *j - 1, *k - 1),
                    &(*i, *j - 2, *k - 1),
                    &(*i + 1, *j - 2, *k - 1),
                ),
            },
            Vertex {
                position: [x, y + 1.0, z],
                uv: [u, v + duv],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i, *j, *k - 1),
                    &(*i - 1, *j - 1, *k - 1),
                    &(*i - 1, *j, *k - 1),
                ),
            },
            Vertex {
                position: [x + 1.0, y + 1.0, z],
                uv: [u + duv, v + duv],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i, *j, *k - 1),
                    &(*i + 1, *j - 1, *k - 1),
                    &(*i + 1, *j, *k - 1),
                ),
            },
        ],
        vec![
            last_index + 0,
            last_index + 2,
            last_index + 3,
            last_index + 0,
            last_index + 3,
            last_index + 1,
        ],
    )
}

fn right_face(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    position: &(i32, i32, i32),
    uv: &[f32; 2],
    last_index: u16,
) -> (Vec<Vertex>, Vec<u16>) {
    let (i, j, k) = position;
    let x = *i as f32;
    let y = *j as f32;
    let z = *k as f32;
    let u = uv[0];
    let v = uv[1];
    let duv = chunk_config.uv_size;

    (
        vec![
            Vertex {
                position: [x + 1.0, y, z + 1.0],
                uv: [u, v],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i + 1, *j - 1, *k + 1),
                    &(*i + 1, *j - 2, *k),
                    &(*i + 1, *j - 2, *k + 1),
                ),
            },
            Vertex {
                position: [x + 1.0, y + 1.0, z + 1.0],
                uv: [u + duv, v],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i + 1, *j, *k + 1),
                    &(*i + 1, *j, *k),
                    &(*i + 1, *j - 1, *k + 1),
                ),
            },
            Vertex {
                position: [x + 1.0, y, z],
                uv: [u, v + duv],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i + 1, *j - 1, *k - 1),
                    &(*i + 1, *j - 2, *k),
                    &(*i + 1, *j - 2, *k - 1),
                ),
            },
            Vertex {
                position: [x + 1.0, y + 1.0, z],
                uv: [u + duv, v + duv],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i + 1, *j, *k - 1),
                    &(*i + 1, *j, *k),
                    &(*i + 1, *j - 1, *k - 1),
                ),
            },
        ],
        vec![
            last_index + 0,
            last_index + 3,
            last_index + 1,
            last_index + 0,
            last_index + 2,
            last_index + 3,
        ],
    )
}
fn left_face(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    position: &(i32, i32, i32),
    uv: &[f32; 2],
    last_index: u16,
) -> (Vec<Vertex>, Vec<u16>) {
    let (i, j, k) = position;
    let x = *i as f32;
    let y = *j as f32;
    let z = *k as f32;
    let u = uv[0];
    let v = uv[1];
    let duv = chunk_config.uv_size;

    (
        vec![
            Vertex {
                position: [x, y, z + 1.0],
                uv: [u, v],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i - 1, *j - 1, *k + 1),
                    &(*i - 1, *j - 2, *k),
                    &(*i - 1, *j - 2, *k + 1),
                ),
            },
            Vertex {
                position: [x, y + 1.0, z + 1.0],
                uv: [u + duv, v],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i - 1, *j, *k + 1),
                    &(*i - 1, *j, *k),
                    &(*i - 1, *j - 1, *k + 1),
                ),
            },
            Vertex {
                position: [x, y, z],
                uv: [u, v + duv],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i - 1, *j - 1, *k - 1),
                    &(*i - 1, *j - 2, *k),
                    &(*i - 1, *j - 2, *k - 1),
                ),
            },
            Vertex {
                position: [x, y + 1.0, z],
                uv: [u + duv, v + duv],
                ao: ao(
                    chunk_config,
                    loaded_chunks,
                    dict,
                    &(*i - 1, *j, *k - 1),
                    &(*i - 1, *j, *k),
                    &(*i - 1, *j - 1, *k - 1),
                ),
            },
        ],
        vec![
            last_index + 0,
            last_index + 1,
            last_index + 3,
            last_index + 0,
            last_index + 3,
            last_index + 2,
        ],
    )
}
