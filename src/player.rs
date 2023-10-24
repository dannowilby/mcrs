use rapier3d::prelude::*;

use crate::{
    engine::{
        input::Input,
        renderer::Renderer,
        uniform::{Uniform, UniformData},
    },
    window_state,
    world::{Event, GameData}, chunk::{chunk_position, chunk_id},
};

pub struct Player {
    pub position: (f32, f32, f32),
    yaw: f32,
    pitch: f32,
    pub fov: f32,
    move_speed: f32,
    sensitivity: f32,
    pub load_radius: u32,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: (30.0, 100.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: 1.0472,
            move_speed: 0.25,
            sensitivity: 0.2,
            load_radius: 4,
        }
    }
}

// should separate out the logic to get the players position and yaw/pitch from input
// also need to check that if the chunk the player is in is a actually loaded so that
// there is no infinite falling
// should also probably move load_radius member to the ChunkConfig struct

fn calculate_player_input_velocity(input: &Input, player: &Player, delta: f64) -> glam::Vec3 {
    
    let (yaw_sin, yaw_cos) = player.yaw.sin_cos();
    let forward = glam::vec3(yaw_cos, 0.0, yaw_sin).normalize();
    let right = glam::vec3(-yaw_sin, 0.0, yaw_cos).normalize();
    
    let mut pos = glam::vec3(0.0, 0.0, 0.0);
    let move_speed = player.move_speed;
    if input.get_key(winit::event::VirtualKeyCode::S) > 0.0 {
        pos -= forward * move_speed * delta as f32;
    }
    if input.get_key(winit::event::VirtualKeyCode::W) > 0.0 {
        pos += forward * move_speed * delta as f32;
    }
    if input.get_key(winit::event::VirtualKeyCode::A) > 0.0 {
        pos -= right * move_speed * delta as f32;
    }
    if input.get_key(winit::event::VirtualKeyCode::D) > 0.0 {
        pos += right * move_speed * delta as f32;
    }
    if input.get_key(winit::event::VirtualKeyCode::Space) > 0.0 {
        pos.y += move_speed * delta as f32;
    }
    if input.get_key(winit::event::VirtualKeyCode::LShift) > 0.0 {
        pos.y -= move_speed * delta as f32;
    }
    
    pos
}

pub fn simulate_player(
    renderer: &mut Renderer,
    input: &mut Input,
    data: &mut GameData,
    queue: &mut Vec<Event>,
    delta: f64,
) {
    
    let pos = data.physics_engine.get_rigid_body("player".to_string()).unwrap().translation();
    let current_chunk = chunk_id(&chunk_position(&data.chunk_config, &(pos.x as i32, pos.y as i32, pos.z as i32)));   
    if data.focused && !data.loaded_chunks.get(&current_chunk).is_none() {
       data.physics_engine.step(); 
    }
   
}

pub fn init_player(
    _renderer: &mut Renderer,
    _input: &mut Input,
    data: &mut GameData,
    _queue: &mut Vec<Event>,
    _delta: f64,
) {
    let translation = Isometry::translation(data.player.position.0, data.player.position.1, data.player.position.2);
    let mut rigidbody = RigidBodyBuilder::dynamic().lock_rotations().enabled_rotations(false, false, false).build();
    rigidbody.set_position(translation, true);
    let collider = ColliderBuilder::capsule_y(0.75, 0.25).build();
    data.physics_engine.insert_entity("player", rigidbody, collider);
}
// camera controller input
pub fn player_input(
    _renderer: &mut Renderer,
    input: &mut Input,
    data: &mut GameData,
    queue: &mut Vec<Event>,
    delta: f64,
) { 
    if !data.focused {
        return;
    }

    let pos = calculate_player_input_velocity(input, &data.player, delta);
    let player = data.physics_engine.get_mut_rigid_body("player".to_string()).unwrap();
    player.set_linvel(vector![ pos.x, pos.y, pos.z ], true);
    
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

    // if should_update {
        queue.push(Event::PlayerMoved);
    // }

}

/*
pub fn update_player() {
    let t = data.physics_engine.get_rigid_body("player".to_string()).unwrap().translation();
    data.player.position = (t.x, t.y, t.z); //pos.into();
}
*/
// this does the actual updating of the camera buffer
// the input method just updates the values
pub fn update_camera(
    renderer: &mut Renderer,
    _input: &mut Input,
    data: &mut GameData,
    _queue: &mut Vec<Event>,
    _delta: f64,
) {
    if let Some(Uniform {
        data: UniformData::Matrix(m),
        ..
    }) = renderer.get_global_uniform("view")
    {
        let mat = m.matrix();

        //let (x, y, z) = ;
        let p_t = data.physics_engine.get_mut_rigid_body("player".to_string()).unwrap().center_of_mass(); //.translation();
        let (x, y, z) = (p_t.x + 0.5, p_t.y + 1.25, p_t.z + 0.5);
        let position = glam::vec3(x, y, z);
        let up = glam::vec3(0.0, 1.0, 0.0);

        let (yaw_sin, yaw_cos) = data.player.yaw.sin_cos();
        let (pitch_sin, pitch_cos) = data.player.pitch.sin_cos();
        let facing = glam::vec3(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();

        let look = glam::Mat4::look_to_rh(position, facing, up);
        *mat = look; // glam::f32::Mat4::from_translation(glam::vec3(x, y, z));
        m.update_buffer();
    }
}

// perspective matrix configuration
// called on resize
pub fn update_perspective(
    renderer: &mut Renderer,
    _input: &mut Input,
    data: &mut GameData,
    _queue: &mut Vec<Event>,
    _delta: f64,
) {
    if let Some(Uniform {
        data: UniformData::Matrix(m),
        ..
    }) = renderer.get_global_uniform("projection")
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

// check when the player is actively in the window
// currently ESC backs them out
pub fn focus_window(
    _renderer: &mut Renderer,
    input: &mut Input,
    data: &mut GameData,
    _queue: &mut Vec<Event>,
    _delta: f64,
) {
    if !data.focused {
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
