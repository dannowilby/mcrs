use std::{
    collections::{HashMap, VecDeque},
    ops::ControlFlow,
};

use glam::Vec4Swizzles;

use crate::{
    engine::{
        matrix::Matrix,
        render::render_pass::{RenderPass, RenderPassViews},
        render_group::RenderGroup,
        render_object::RenderObject,
        uniform::Uniform,
    },
    player::Player,
    window_state,
    world::GameData,
};

use super::{
    chunk_id, chunk_position,
    culling::{get_neighbors, Side},
    player_to_position, Position,
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

fn calculate_frustum_bounds(player: &Player) -> Vec<glam::Vec3> {
    let (yaw_sin, yaw_cos) = player.yaw.sin_cos();
    let (pitch_sin, pitch_cos) = player.pitch.sin_cos();
    let facing = glam::vec3(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();

    // vectors to rotate around
    let side_tilt_vector = glam::vec3(0.0, 1.0, 0.0);
    let forward_tilt = side_tilt_vector.cross(facing);

    let aspect_ratio = window_state().config.width as f32 / window_state().config.height as f32;
    let fov_x = player.fov;
    let fov_y = 2.0 * f32::atan(f32::tan(fov_x * 0.5) * aspect_ratio);

    // get rotation matrices for finding frustum edges
    // apparently multiplying the matrix by negative one
    // doesn't rotate in the opposite direction
    // so we will either have to create a matrix for each rotation direction
    // or check if the inverse will work
    // also order of rotation matrix multiplication matters
    let pos_side_rot =
        glam::Mat4::from_quat(glam::Quat::from_axis_angle(side_tilt_vector, fov_x / 2.0));
    let neg_side_rot =
        glam::Mat4::from_quat(glam::Quat::from_axis_angle(side_tilt_vector, -fov_x / 2.0));
    let pos_forward_rot =
        glam::Mat4::from_quat(glam::Quat::from_axis_angle(forward_tilt, fov_y / 2.0));
    let neg_forward_rot =
        glam::Mat4::from_quat(glam::Quat::from_axis_angle(forward_tilt, -fov_y / 2.0));

    vec![
        pos_side_rot.transform_vector3(facing),
        neg_side_rot.transform_vector3(facing),
        pos_forward_rot.transform_vector3(facing),
        neg_forward_rot.transform_vector3(facing),
    ]
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

            let facing = calculate_frustum_bounds(&data.player);

            let chunk_pos = chunk_position(
                &data.chunk_config,
                &player_to_position(&(pos.x, pos.y, pos.z)),
            );

            data.drawn_chunks = 0;
            data.chunks_removed_by_visibility = 0;

            // set up a search queue, start with the chunk the player is in.
            let mut previously_drawn = Vec::<Position>::new();
            let mut visited = Vec::<(Side, Position)>::new();
            let mut search_queue = VecDeque::<(Option<Side>, Position)>::from([(None, chunk_pos)]);

            while !search_queue.is_empty() {
                let (from_side, chunk_pos) = search_queue
                    .pop_front()
                    .expect("Queue was made unexpectedly empty");

                // if we've already encountered this chunk, don't consider it again
                if visited.contains(&(from_side.unwrap_or(Side::TOP), chunk_pos)) {
                    continue;
                }

                // render this chunk
                if let Some(chunk) = data.loaded_chunks.get(&chunk_id(&chunk_pos)) {
                    if !chunk.is_empty() && !previously_drawn.contains(&chunk_pos) {
                        self.render_chunk(&chunk_pos, &mut render_pass);
                        data.drawn_chunks += 1;
                        previously_drawn.push(chunk_pos.clone());
                    }
                }
                visited.push((from_side.unwrap_or(Side::TOP), chunk_pos).clone());

                get_neighbors(&data.loaded_chunks, &chunk_pos)
                    .into_iter()
                    .for_each(|(to_side, chunk_pos)| {
                        // correct direction filter:
                        // check if neighbor is in forward direction we are looking
                        // if the dot product is negative, then it should render
                        // maybe use the corners of the frustum viewport as the facing vector(s)

                        let mut in_view = false;
                        facing.iter().for_each(|c| {
                            if c.dot(to_side.normal()) < 0.0 {
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
                            .get(&chunk_id(&chunk_pos))
                            .expect("Chunk is loaded, so visibility graph should be loaded too.");
                        if let Some(side) = from_side {
                            if !visibility_graph.can_reach_from(side, to_side) {
                                data.chunks_removed_by_visibility += 1;
                                return;
                            }
                        }

                        // might want to add an actual frustum cull step here.

                        // if chunk has passed the filters, then add it
                        search_queue.push_back((Some(to_side.opposite()), chunk_pos.clone()));
                    });
            }
        }
        window_state()
            .queue
            .submit(std::iter::once(encoder.finish()));
        Ok(())
    }
}
