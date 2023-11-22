use rapier3d::prelude::*;

use crate::{
    chunk::{chunk_id, chunk_position, get_block, ChunkConfig, ChunkStorage, Position, player_to_position},
    engine::{
        input::Input,
        uniform::{Uniform, UniformData},
    },
    physics::PhysicsEngine,
    window_state,
    world::{Event, GameData},
    world_renderer::WorldRenderer,
};

pub struct Player {
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
    pub move_speed: f32,
    pub max_jump: f32,
    sensitivity: f32,
    pub is_flying: bool,
    last_chunk: Position,
}

impl Player {
    pub fn new() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            fov: 1.22173,
            move_speed: 4.0,
            max_jump: 1.25,
            sensitivity: 0.2,
            is_flying: true,
            last_chunk: (0,0,0)
        }
    }
}

/// Create the rigid body and collider for the player.
pub fn create_player(data: &mut GameData, pos: &Position) {
    let translation = Isometry::translation(pos.0 as f32, pos.1 as f32, pos.2 as f32);
    let mut rigidbody = RigidBodyBuilder::dynamic()
        .lock_rotations()
        .enabled_rotations(false, false, false)
        .build();
    rigidbody.set_position(translation, true);
    let collider = ColliderBuilder::cylinder(0.75, 0.25).friction(0.0).build();
    data.physics_engine
        .insert_entity("player", rigidbody, collider);
}

/// Check if the player is on the ground.
fn on_ground(physics_engine: &PhysicsEngine) -> bool {
    if let Some(handle) = physics_engine.get_collider_handle("player") {
        for i in physics_engine.narrow_phase.contacts_with(handle.clone()) {
            if i.has_any_active_contact {
                for manifold in &i.manifolds {
                    let p = manifold.local_n1;
                    // bottom of player model is touching something
                    if p.eq(&vector![0.0, -1.0, 0.0]) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Get the velocity from player input.
fn calculate_player_input_velocity(input: &Input, player: &Player, delta: f64) -> glam::Vec3 {
    let (yaw_sin, yaw_cos) = player.yaw.sin_cos();
    let forward = glam::vec3(yaw_cos, 0.0, yaw_sin).normalize();
    let right = glam::vec3(-yaw_sin, 0.0, yaw_cos).normalize();

    let mut pos = glam::vec3(0.0, 0.0, 0.0);
    let move_speed = player.move_speed;
    let s = input.get_key(winit::event::VirtualKeyCode::S);
    let w = input.get_key(winit::event::VirtualKeyCode::W);
    let a = input.get_key(winit::event::VirtualKeyCode::A);
    let d = input.get_key(winit::event::VirtualKeyCode::D);
    let space = input.get_key(winit::event::VirtualKeyCode::Space);
    let shift = input.get_key(winit::event::VirtualKeyCode::LShift);
    if s > 0.0 {
        pos -= forward * move_speed; //* delta as f32;
    }
    if w > 0.0 {
        pos += forward * move_speed; // * delta as f32;
    }
    if a > 0.0 {
        pos -= right * move_speed; // * delta as f32;
    }
    if d > 0.0 {
        pos += right * move_speed; // * delta as f32;
    }
    if space > 0.0 {
        // && space <= delta {
        pos.y += move_speed * player.max_jump;
    }
    if shift > 0.0 {
        pos.y -= move_speed;
    }

    pos
}

/// Update the physicsc engine only if the window is focused, and the chunk the player is in is loaded.
pub fn simulate_player(
    _renderer: &mut WorldRenderer,
    _input: &mut Input,
    data: &mut GameData,
    _queue: &mut Vec<Event>,
    delta: f64,
) {
    let pos = data
        .physics_engine
        .get_rigid_body("player".to_string())
        .unwrap()
        .translation();
    let current_chunk = chunk_id(&chunk_position(
        &data.chunk_config,
        &(pos.x as i32, pos.y as i32, pos.z as i32),
    ));
    if data.focused && !data.loaded_chunks.get(&current_chunk).is_none() {
        data.physics_engine.step(delta);
    }
}

/// System for updating player movement with input.
pub fn player_input(
    _renderer: &mut WorldRenderer,
    input: &mut Input,
    data: &mut GameData,
    queue: &mut Vec<Event>,
    delta: f64,
) {
    // don't update if not focused
    if !data.focused {
        return;
    }

    let input_vel = calculate_player_input_velocity(input, &data.player, delta);
    let physics_vel = data
        .physics_engine
        .get_mut_rigid_body("player".to_string())
        .unwrap()
        .linvel();
    let mut output_vel = vector!(input_vel.x, physics_vel.y, input_vel.z);

    if on_ground(&data.physics_engine) {
        output_vel.y += input_vel.y;
    }

    if data.player.is_flying {
        output_vel.x = input_vel.x;
        output_vel.y = input_vel.y;
        output_vel.z = input_vel.z;
    }

    let player = data
        .physics_engine
        .get_mut_rigid_body("player".to_string())
        .unwrap();
    player.set_linvel(output_vel, true);

    // change look direction
    if input.movement.0 != 0.0 || input.movement.1 != 0.0 {
        data.player.yaw +=
            (input.movement.0 / 360.0) as f32 * delta as f32 * data.player.sensitivity;
        data.player.pitch -=
            (input.movement.1 / 360.0) as f32 * delta as f32 * data.player.sensitivity;
        input.movement = (0.0, 0.0);
    }

    // 1.55 is just below 2pi
    if data.player.pitch > 1.55 {
        data.player.pitch = 1.55;
    } else if data.player.pitch < -1.55 {
        data.player.pitch = -1.55;
    }

    queue.push(Event::PlayerMoved);
}

pub fn player_changed_chunk(
    _renderer: &mut WorldRenderer,
    _input: &mut Input,
    data: &mut GameData,
    queue: &mut Vec<Event>,
    _delta: f64,
) {
    let mut position = (0, 0, 0);
    if let Some(player) = data.physics_engine.get_rigid_body("player".to_string()) {
        let player_pos = player.translation();
        position = player_to_position(&(player_pos.x, player_pos.y, player_pos.z));
    }

    // chunk loading dimensions
    let current_player_chunk = chunk_position(&data.chunk_config, &position);
    
    if data.player.last_chunk != current_player_chunk {
        queue.push(Event::PlayerChunkChanged);
        data.player.last_chunk = current_player_chunk;
    }
}

/// Update the player camera with look position and world position.
pub fn update_camera(
    renderer: &mut WorldRenderer,
    _input: &mut Input,
    data: &mut GameData,
    _queue: &mut Vec<Event>,
    _delta: f64,
) {
    if let Some(Uniform {
        data: UniformData::Matrix(m),
        ..
    }) = renderer.chunk_render_pass.uniforms.get_mut("view")
    {
        let mat = m.matrix_mut();

        // we use center of mass because then we clip less into walls
        let p_t = data
            .physics_engine
            .get_mut_rigid_body("player".to_string())
            .unwrap()
            .center_of_mass(); //.translation();
                               // then we translate the camera to where we want
        let (x, y, z) = (p_t.x + 0.5, p_t.y + 1.25, p_t.z + 0.5);
        let position = glam::vec3(x, y, z);
        let up = glam::vec3(0.0, 1.0, 0.0);

        let (yaw_sin, yaw_cos) = data.player.yaw.sin_cos();
        let (pitch_sin, pitch_cos) = data.player.pitch.sin_cos();
        let facing = glam::vec3(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();

        let look = glam::Mat4::look_to_rh(position, facing, up);
        *mat = look;
        m.update_buffer();
    }
}

/// Called when the window is resized or surface is changed.
pub fn update_perspective(
    renderer: &mut WorldRenderer,
    _input: &mut Input,
    data: &mut GameData,
    _queue: &mut Vec<Event>,
    _delta: f64,
) {
    if let Some(Uniform {
        data: UniformData::Matrix(m),
        ..
    }) = renderer.chunk_render_pass.uniforms.get_mut("projection")
    {
        let config = &window_state().config;
        let mat = m.matrix_mut();
        *mat = glam::Mat4::perspective_rh_gl(
            data.player.fov,
            config.width as f32 / config.height as f32,
            0.1,
            1000.0,
        );
        m.update_buffer();
    }
}

/// System for tracking if the user is actively using the window.
pub fn focus_window(
    _renderer: &mut WorldRenderer,
    input: &mut Input,
    data: &mut GameData,
    _queue: &mut Vec<Event>,
    _delta: f64,
) {
    if !data.focused && !data.show_debug_menu {
        let click = input.get_click(winit::event::MouseButton::Left);

        if click > 0.0 {
            data.focused = true;

            let window = &window_state().window;
            window
                .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                .unwrap();
            window.set_cursor_visible(false);
        }
    }

    if data.focused {
        let esc = input.get_key(winit::event::VirtualKeyCode::Escape);

        if esc > 0.0 {
            data.focused = false;

            let window = &window_state().window;
            window
                .set_cursor_grab(winit::window::CursorGrabMode::None)
                .unwrap();
            window.set_cursor_visible(true);
        }
    }
}
