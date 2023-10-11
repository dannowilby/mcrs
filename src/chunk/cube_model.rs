use crate::chunk::{block::BlockDictionary, get_block, meshing::Vertex, ChunkConfig, ChunkData};
use std::collections::HashMap;
use std::sync::RwLock;

use super::{get_local_block_pos, is_transparent};

fn ao(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    p1: &(i32, i32, i32),
    p2: &(i32, i32, i32),
    p3: &(i32, i32, i32),
) -> f32 {
    let mut output = 1.0;

    if !is_transparent(chunk_config, loaded_chunks, &(p1.0, p1.1 + 1, p1.2))
        || !is_transparent(chunk_config, loaded_chunks, &(p2.0, p2.1 + 1, p2.2))
        || !is_transparent(chunk_config, loaded_chunks, &(p3.0, p3.1 + 1, p3.2))
    {
        output = 0.5;
    }

    output
}

pub fn cube_model(
    loaded_chunks: &HashMap<String, ChunkData>,
    chunk_config: &ChunkConfig,
    position: &(i32, i32, i32),
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
) {
    let dict = &chunk_config.dict;

    let (p0, p1, p2) = position;
    /*
    let relative_position = (
        get_local_block_pos(chunk_config, p0),
        get_local_block_pos(chunk_config, p1),
        get_local_block_pos(chunk_config, p2),
    );
    */

    // get target block info, ~~if air then return~~ won't be air b/c air's model isn't this method
    let block = get_block(chunk_config, loaded_chunks, &position);
    let target_block = dict.get(&block);
    let uv = target_block.unwrap().uv;

    // get surrounding block transparency info
    let top_block = is_transparent(chunk_config, loaded_chunks, &(*p0, p1 + 1, *p2));
    let bottom_block = is_transparent(chunk_config, loaded_chunks, &(*p0, p1 - 1, *p2));

    let front_block = is_transparent(chunk_config, loaded_chunks, &(*p0, *p1, p2 + 1));
    let back_block = is_transparent(chunk_config, loaded_chunks, &(*p0, *p1, p2 - 1));

    let right_block = is_transparent(chunk_config, loaded_chunks, &(p0 + 1, *p1, *p2));
    let left_block = is_transparent(chunk_config, loaded_chunks, &(p0 - 1, *p1, *p2));

    // top
    if top_block {
        top_face(
            chunk_config,
            loaded_chunks,
            dict,
            position,
            &uv,
            vertices,
            indices,
        );
    }

    // bottom
    if bottom_block {
        bottom_face(
            chunk_config,
            loaded_chunks,
            dict,
            position,
            &uv,
            vertices,
            indices,
        );
    }

    // front
    if front_block {
        front_face(
            chunk_config,
            loaded_chunks,
            dict,
            position,
            &uv,
            vertices,
            indices,
        );
    }

    // back
    if back_block {
        back_face(
            chunk_config,
            loaded_chunks,
            dict,
            position,
            &uv,
            vertices,
            indices,
        );
    }

    if right_block {
        right_face(
            chunk_config,
            loaded_chunks,
            dict,
            position,
            &uv,
            vertices,
            indices,
        );
    }

    if left_block {
        left_face(
            chunk_config,
            loaded_chunks,
            dict,
            position,
            &uv,
            vertices,
            indices,
        );
    }
}

fn top_face(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    position: &(i32, i32, i32),
    uv: &[f32; 2],
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
) {
    let (i, j, k) = position;
    let x = *i as f32;
    let y = *j as f32;
    let z = *k as f32;
    let u = uv[0];
    let v = uv[1];
    let duv = chunk_config.uv_size;

    let last_index = vertices.len() as u16;
    indices.extend_from_slice(&[
        last_index + 0,
        last_index + 3,
        last_index + 2,
        last_index + 0,
        last_index + 1,
        last_index + 3,
    ]);
    vertices.extend_from_slice(&[
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
    ]);
}

fn bottom_face(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    position: &(i32, i32, i32),
    uv: &[f32; 2],
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
) {
    let (i, j, k) = position;
    let x = *i as f32;
    let y = *j as f32;
    let z = *k as f32;
    let u = uv[0];
    let v = uv[1];
    let duv = chunk_config.uv_size;

    let last_index = vertices.len() as u16;
    indices.extend_from_slice(&[
        last_index + 0,
        last_index + 2,
        last_index + 3,
        last_index + 0,
        last_index + 3,
        last_index + 1,
    ]);
    vertices.extend_from_slice(&[
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
    ]);
}

fn front_face(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    position: &(i32, i32, i32),
    uv: &[f32; 2],
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
) {
    let (i, j, k) = position;
    let x = *i as f32;
    let y = *j as f32;
    let z = *k as f32;
    let u = uv[0];
    let v = uv[1];
    let duv = chunk_config.uv_size;

    let last_index = vertices.len() as u16;
    indices.extend_from_slice(&[
        last_index + 0,
        last_index + 3,
        last_index + 2,
        last_index + 0,
        last_index + 1,
        last_index + 3,
    ]);
    vertices.extend_from_slice(&[
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
    ]);
}

fn back_face(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    position: &(i32, i32, i32),
    uv: &[f32; 2],
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
) {
    let (i, j, k) = position;
    let x = *i as f32;
    let y = *j as f32;
    let z = *k as f32;
    let u = uv[0];
    let v = uv[1];
    let duv = chunk_config.uv_size;

    let last_index = vertices.len() as u16;
    indices.extend_from_slice(&[
        last_index + 0,
        last_index + 2,
        last_index + 3,
        last_index + 0,
        last_index + 3,
        last_index + 1,
    ]);
    vertices.extend_from_slice(&[
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
    ]);
}

fn right_face(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    position: &(i32, i32, i32),
    uv: &[f32; 2],
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
) {
    let (i, j, k) = position;
    let x = *i as f32;
    let y = *j as f32;
    let z = *k as f32;
    let u = uv[0];
    let v = uv[1];
    let duv = chunk_config.uv_size;

    let last_index = vertices.len() as u16;
    indices.extend_from_slice(&[
        last_index + 0,
        last_index + 3,
        last_index + 1,
        last_index + 0,
        last_index + 2,
        last_index + 3,
    ]);
    vertices.extend_from_slice(&[
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
    ]);
}
fn left_face(
    chunk_config: &ChunkConfig,
    loaded_chunks: &HashMap<String, ChunkData>,
    dict: &BlockDictionary,
    position: &(i32, i32, i32),
    uv: &[f32; 2],
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
) {
    let (i, j, k) = position;
    let x = *i as f32;
    let y = *j as f32;
    let z = *k as f32;
    let u = uv[0];
    let v = uv[1];
    let duv = chunk_config.uv_size;

    let last_index = vertices.len() as u16;
    indices.extend_from_slice(&[
        last_index + 0,
        last_index + 1,
        last_index + 3,
        last_index + 0,
        last_index + 3,
        last_index + 2,
    ]);
    vertices.extend_from_slice(&[
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
    ]);
}
