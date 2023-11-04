use rapier3d::prelude::*;

use crate::{
    chunk::{chunk_id, chunk_position, get_block, ChunkConfig, ChunkStorage, Position},
    engine::{
        input::Input,
        uniform::{Uniform, UniformData},
    },
    window_state,
    world::{Event, GameData},
    world_renderer::WorldRenderer,
};

pub struct Player {
    yaw: f32,
    pitch: f32,
    pub fov: f32,
    move_speed: f32,
    pub max_jump: f32,
    sensitivity: f32,
    pub is_flying: bool,
    is_airborne: bool,
}

impl Player {
    pub fn new() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            fov: 1.0472,
            move_speed: 1.5,
            max_jump: 15.0,
            sensitivity: 0.2,
            is_flying: true,
            is_airborne: false,
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
    let collider = ColliderBuilder::capsule_y(0.75, 0.25).build();
    data.physics_engine
        .insert_entity("player", rigidbody, collider);
}

/// Check if the player is on the ground.
fn on_ground<'a>(
    config: &'a ChunkConfig,
    loaded_chunks: &'a ChunkStorage,
    pos: &'a Vector<Real>,
) -> bool {
    if f32::fract(pos.y) > 0.5 || f32::fract(pos.y) < 0.48 {
        return false;
    }
    let block_id = get_block(
        config,
        loaded_chunks,
        &(pos.x as i32, f32::round(pos.y) as i32 - 1, pos.z as i32),
    );
    !config
        .dict
        .get(&block_id)
        .unwrap_or(&Default::default())
        .transparent
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
        pos -= forward * move_speed * delta as f32;
    }
    if w > 0.0 {
        pos += forward * move_speed * delta as f32;
    }
    if a > 0.0 {
        pos -= right * move_speed * delta as f32;
    }
    if d > 0.0 {
        pos += right * move_speed * delta as f32;
    }
    if space > 0.0 && space <= delta {
        pos.y += move_speed * player.max_jump;
    }
    if shift > 0.0 {
        pos.y -= move_speed * delta as f32;
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

/// Merge player input and physics velocity to get the next total velocity.
fn calculate_player_velocity(
    input_vel: glam::Vec3,
    physics_vel: glam::Vec3,
    player: &Player,
) -> Vector<Real> {
    if player.is_flying {
        return vector![input_vel.x, input_vel.y, input_vel.z];
    }

    let mut up_vel = physics_vel.y;
    if !player.is_airborne && input_vel.y > 0.0 {
        up_vel += input_vel.y;
        // up_vel = f32::min(up_vel, player.max_jump);
    }

    vector![input_vel.x, up_vel, input_vel.z]
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

    // get input
    let input_vel = calculate_player_input_velocity(input, &data.player, delta);
    // get physics data
    let is_colliding = data.physics_engine.is_colliding("player");
    let player = data
        .physics_engine
        .get_mut_rigid_body("player".to_string())
        .unwrap();
    // update is_airborne
    data.player.is_airborne = !is_colliding
        && !on_ground(
            &data.chunk_config,
            &data.loaded_chunks,
            player.translation(),
        );

    let physics_vel = glam::Vec3::new(player.linvel().x, player.linvel().y, player.linvel().z);
    player.set_linvel(
        calculate_player_velocity(input_vel, physics_vel, &data.player),
        true,
    );

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
    }) = renderer.object_render_pass.uniforms.get_mut("view")
    {
        let mat = m.matrix();

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
    }) = renderer.object_render_pass.uniforms.get_mut("projection")
    {
        let config = &window_state().config;
        let mat = m.matrix();
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
    renderer: &mut WorldRenderer,
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
