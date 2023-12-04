use std::collections::{HashMap, VecDeque};

use crate::{
    engine::{
        render::render_group::RenderGroup,
        render::render_object::RenderObject,
        render::render_pass::{RenderPass, RenderPassViews},
        render::uniform::{Uniform, UniformData},
    },
    window_state,
    world::GameData,
};

use super::{
    chunk_id, chunk_position,
    culling::{get_neighbors, Side},
    player_to_position, ChunkConfig, Position,
};

pub struct ChunkRenderPass {
    pub render_groups: HashMap<String, RenderGroup>,
    pub render_objects: HashMap<String, RenderObject>,
    pub uniforms: HashMap<String, Uniform>,
    pub clear_color: wgpu::Color,
}

impl ChunkRenderPass {
    pub fn new() -> Self {
        Self {
            render_groups: HashMap::new(),
            render_objects: HashMap::new(),
            uniforms: HashMap::new(),
            clear_color: wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
            },
        }
    }
    fn render_chunk<'a>(&'a self, chunk: &Position, render_pass: &mut wgpu::RenderPass<'a>) {
        let wrapped_object = self.render_objects.get(&chunk_id(chunk));
        if wrapped_object.is_none() {
            return;
        }
        let object = wrapped_object.unwrap();
        let group = self
            .render_groups
            .get(&object.render_group)
            .expect("Referenced a render group that does not exist! You are using this wrong!");

        // for the uniforms in the group
        for uniform_name in group.uniforms.iter() {
            // check if a global uniform first
            let global_uniform = self.uniforms.get(uniform_name);
            let uniform = match global_uniform {
                Some(x) => x,
                // if not a global uniform, then it's a object uniform
                None => object
                    .uniforms
                    .get(uniform_name)
                    .expect(&format!("Uniform {} not specified", uniform_name)),
            };

            // set the uniform
            render_pass.set_bind_group(uniform.location, &uniform.bind_group, &[]);
        }

        // set the vertex buffer
        render_pass.set_vertex_buffer(0, object.vertex_buffer.slice(..));
        // set the index buffer
        render_pass.set_index_buffer(object.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        let num_indices = object.index_buffer.size() as u32 / std::mem::size_of::<u16>() as u32;
        // draw
        render_pass.draw_indexed(0..num_indices, 0, 0..1);
    }
}

fn calculate_frustum_planes(renderer: &ChunkRenderPass) -> [glam::Vec4; 6] {
    let mut view_projection_matrix = glam::Mat4::IDENTITY;

    if let UniformData::Matrix(proj) = &renderer
        .uniforms
        .get("projection")
        .expect("No projection matrix set!")
        .data
    {
        if let UniformData::Matrix(view) = &renderer
            .uniforms
            .get("view")
            .expect("No view matrix set!")
            .data
        {
            view_projection_matrix = proj.matrix().mul_mat4(view.matrix());
        }
    }

    let row0 = view_projection_matrix.row(0);
    let row1 = view_projection_matrix.row(1);
    let row2 = view_projection_matrix.row(2);
    let row3 = view_projection_matrix.row(3);

    [
        row3 + row0,
        row3 - row0,
        row3 + row1,
        row3 - row1,
        row3 + row2,
        row3 - row2,
    ]
}

/// Frustum cull if chunk is completely outside of frustum.
/// Code is a mix of ChatGPT code and the article found [here](https://iquilezles.org/articles/frustumcorrect/).
fn is_chunk_inside_frustum(
    config: &ChunkConfig,
    chunk: &Position,
    frustum_planes: &[glam::Vec4; 6],
) -> bool {
    let min = (
        chunk.0 * config.depth,
        chunk.1 * config.depth,
        chunk.2 * config.depth,
    );
    let max = (
        chunk.0 * config.depth + config.depth,
        chunk.1 * config.depth + config.depth,
        chunk.2 * config.depth + config.depth,
    );

    for plane in frustum_planes {
        let mut output = 0;

        if plane.dot(glam::vec4(min.0 as f32, min.1 as f32, min.2 as f32, 1.0)) < 0.0 {
            output += 1;
        }
        if plane.dot(glam::vec4(max.0 as f32, min.1 as f32, min.2 as f32, 1.0)) < 0.0 {
            output += 1;
        }
        if plane.dot(glam::vec4(min.0 as f32, max.1 as f32, min.2 as f32, 1.0)) < 0.0 {
            output += 1;
        }
        if plane.dot(glam::vec4(min.0 as f32, min.1 as f32, max.2 as f32, 1.0)) < 0.0 {
            output += 1;
        }
        if plane.dot(glam::vec4(max.0 as f32, max.1 as f32, min.2 as f32, 1.0)) < 0.0 {
            output += 1;
        }
        if plane.dot(glam::vec4(max.0 as f32, min.1 as f32, max.2 as f32, 1.0)) < 0.0 {
            output += 1;
        }
        if plane.dot(glam::vec4(min.0 as f32, max.1 as f32, max.2 as f32, 1.0)) < 0.0 {
            output += 1;
        }
        if plane.dot(glam::vec4(max.0 as f32, max.1 as f32, max.2 as f32, 1.0)) < 0.0 {
            output += 1;
        }

        if output == 8 {
            return false;
        }

        /*
        let positive = if normal.x >= 0.0 { max.0 } else { min.0 };
        let negative = if normal.x < 0.0 { max.0 } else { min.0 };
        let dist1 = plane.dot(glam::Vec4::new(
            positive as f32,
            max.1 as f32,
            max.2 as f32,
            1.0,
        ));

        let dist2 = plane.dot(glam::Vec4::new(
            negative as f32,
            min.1 as f32,
            min.2 as f32,
            1.0,
        ));

        if dist1 < 0.0 && dist2 < 0.0 {
            return false;
        }
        */
    }
    true
}

impl RenderPass<GameData> for ChunkRenderPass {
    fn render(
        &mut self,
        data: &mut GameData,
        views: RenderPassViews,
        _delta: f64,
    ) -> Result<(), wgpu::SurfaceError> {
        let view = views
            .color
            .expect("No color attachment specified on Object Render Pass...");
        let depth_view = views
            .depth
            .expect("No depth attachment specified on Object Render Pass...");

        let mut encoder: wgpu::CommandEncoder =
            window_state()
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Object Render Pass"),
                });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Object Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            render_pass.set_pipeline(
                &self
                    .render_groups
                    .get("chunk_render_group")
                    .unwrap()
                    .pipeline,
            );

            let player = data.physics_engine.get_rigid_body("player".to_string());
            if player.is_none() {
                return Ok(());
            }
            let pos = player.unwrap().translation();

            // let facing = calculate_frustum_bounds(&data.player);
            let frustum_planes = calculate_frustum_planes(&self);

            let start_chunk_pos = chunk_position(
                &data.chunk_config,
                &player_to_position(&(pos.x, pos.y, pos.z)),
            );

            data.drawn_chunks = 0;
            data.chunks_removed_by_visibility = 0;

            /*
            // Naive approach to rendering, just frustum culling
            for chunk_id in data.loaded_chunks.keys() {
                let chunk_pos = chunk_pos_from_id(chunk_id);
                if is_chunk_inside_frustum(&data.chunk_config, &chunk_pos, &frustum_planes) {
                    self.render_chunk(&chunk_pos, &mut render_pass);
                    data.drawn_chunks += 1;
                } else {
                    data.chunks_removed_by_visibility += 1;
                }
            }
            */

            let mut search_queue: VecDeque<(Position, Option<Side>, Vec<Side>)> = VecDeque::new();
            search_queue.push_back((start_chunk_pos, None, vec![]));
            let mut visited: Vec<(Side, Position)> = Vec::new();
            let mut drawn_chunks: Vec<Position> = Vec::new();

            while !search_queue.is_empty() {
                // the current chunk
                // this chunk should be rendered
                let (chunk_pos, side, constraints) = search_queue
                    .pop_front()
                    .expect("Search queue made suddenly empty!");

                // do what we need to do to the current chunk, ie. render it
                if !drawn_chunks.contains(&chunk_pos) {
                    self.render_chunk(&chunk_pos, &mut render_pass);
                    drawn_chunks.push(chunk_pos.clone());
                    data.drawn_chunks = data.drawn_chunks + 1;
                }

                // check the valid neighbors
                // all these neighbors should be loaded
                get_neighbors(&data.loaded_chunks, &chunk_pos)
                    .into_iter()
                    .for_each(|(next_side, next_chunk_pos)| {
                        // if we go in a direction that we shouldn't/go back on ourselves
                        if constraints.contains(&next_side.opposite()) {
                            return;
                        }

                        // add to constraints if not contained yet
                        let mut next_constraints = constraints.clone();
                        if !next_constraints.contains(&next_side) {
                            next_constraints.push(next_side);
                        }

                        // if we've already visited the chunk from this side
                        if visited.contains(&(next_side, next_chunk_pos)) {
                            return;
                        }
                        visited.push((next_side, next_chunk_pos));

                        // check visibility of chunk
                        if let Some(current_side) = side {
                            // check that it's in view/in forward direction
                            /*
                            if facing[0].dot(next_side.normal())
                                > -1.0 * f32::cos(data.player.fov * 2.0)
                            {
                                return;
                            }
                            */

                            // get the graph from the parent chunk
                            if let Some(graph) = data.visibility_graphs.get(&chunk_id(&chunk_pos)) {
                                // if we can't see through the chunk to the neighbors side
                                // then don't queue it up

                                if !graph.can_reach_from(current_side, next_side) {
                                    data.chunks_removed_by_visibility += 1;
                                    return;
                                }
                            }
                        }

                        // frustum cull
                        if !is_chunk_inside_frustum(
                            &data.chunk_config,
                            &next_chunk_pos,
                            &frustum_planes,
                        ) {
                            return;
                        }

                        // push back this neighbor
                        search_queue.push_back((
                            next_chunk_pos.clone(),
                            Some(next_side.opposite()),
                            next_constraints,
                        ));
                    });
            }

            /*

            // set up a search queue, start with the chunk the player is in.
            let mut visited = Vec::<(Side, Position)>::new();
            let mut chunks_to_draw = Vec::new();
            let mut search_queue = VecDeque::<(Option<Side>, Position, Vec<Side>)>::from([(None, start_chunk_pos, vec![])]);
            let mut nodes_traversed = 0;

            while !search_queue.is_empty() {
                let (from_side, chunk_pos, constraints) = search_queue
                    .pop_front()
                    .expect("Queue was made unexpectedly empty");

                if !data.loaded_chunks.contains_key(&chunk_id(&chunk_pos)) {
                    continue;
                }

                // push chunk to the render list if it hasn't already been drawn
                if !chunks_to_draw.contains(&chunk_pos) {
                    self.render_chunk(&chunk_pos, &mut render_pass);
                    chunks_to_draw.push(chunk_pos.clone());
                }
                nodes_traversed = nodes_traversed + 1;

                // check to see what neighbor chunks we need to render
                get_neighbors(&data.loaded_chunks, &chunk_pos)
                    .into_iter()
                    .for_each(|(to_side, next_chunk_pos)| {

                        let mut passable = false;
                        for constraint in constraints.iter() {
                            if constraint == &to_side.opposite() {
                                passable = true;
                            }
                        }
                        if passable {
                            return;
                        }

                        // if we've already encountered this chunk, don't consider it again
                        if visited.contains(&(to_side, next_chunk_pos)) {
                            return;
                        }
                        visited.push((to_side, next_chunk_pos).clone());


                        // if it is the chunk we are just coming from we don't want to recheck it
                        if let Some(side) = from_side {
                            if side.opposite() == to_side {
                                return;
                            }
                        }


                        // if the dot product of the view and the side normal is negative,
                        // then it should render as its face normal is opposite to our look vector
                        // we use 0.2 to soften the look angle because this can over-cull
                        // some chunks
                        /*

                        if facing[0].dot(to_side.normal()) > 0.2 {
                            return;
                        }
                        */
                        let mut in_view = false;
                        facing.iter().for_each(|c| {
                            if c.dot(to_side.normal()) < 0.2 {
                                in_view = true;
                            }
                        });
                        if !in_view {
                            return;
                        }


                        // visibility filter:
                        // check the chunk's visibility graph to see if we can reach it.
                        let visibility_graph = data
                            .visibility_graphs
                            .get(&chunk_id(&chunk_pos)) // might need to use chunk vis graph, not next chunk
                            .expect("Chunk is loaded, so visibility graph should be loaded too.");
                        if next_chunk_pos != start_chunk_pos {
                            if let Some(side) = from_side {
                            if !visibility_graph.can_reach_from(side, to_side) {
                                return;
                            }
                        }
                    }

                        let mut next_constraints = constraints.clone();
                        if !next_constraints.contains(&to_side) {
                            next_constraints.push(to_side);
                        }

                        // if chunk has passed the filters, then add it
                        search_queue.push_back((Some(to_side.opposite()), next_chunk_pos.clone(), next_constraints));
                    });

            }
            // println!("{}", nodes_traversed);
            /*
            for chunk_pos in chunks_to_draw.iter() {
                self.render_chunk(chunk_pos, &mut render_pass);
            }
            */
            data.drawn_chunks = chunks_to_draw.len() as u64;
            */
        }

        window_state()
            .queue
            .submit(std::iter::once(encoder.finish()));
        // println!("{}", instant::now() - now);
        Ok(())
    }
}
