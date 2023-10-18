use crate::{
    engine::{
        input::Input,
        renderer::Renderer,
        uniform::{Uniform, UniformData},
    },
    window_state,
    world::{Event, GameData},
};
use std::sync::{Arc, RwLock};

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
            position: (0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            fov: 1.0472,
            move_speed: 0.01,
            sensitivity: 0.2,
            load_radius: 4,
        }
    }
}

// camera controller input
pub fn player_input(
    renderer: &mut Renderer,
    input: &mut Input,
    data: &mut GameData,
    queue: &mut Vec<Event>,
    delta: f64,
) {
    if !data.focused {
        return;
    }

    let (yaw_sin, yaw_cos) = data.player.yaw.sin_cos();
    let forward = glam::vec3(yaw_cos, 0.0, yaw_sin).normalize();
    let right = glam::vec3(-yaw_sin, 0.0, yaw_cos).normalize();
    let (x, y, z) = data.player.position;
    let mut pos = glam::vec3(x, y, z);

    let mut should_update = false;
    let move_speed = data.player.move_speed;
    if input.get_key(winit::event::VirtualKeyCode::S) > 0.0 {
        pos -= forward * move_speed * delta as f32;
        should_update = true;
    }
    if input.get_key(winit::event::VirtualKeyCode::W) > 0.0 {
        pos += forward * move_speed * delta as f32;
        should_update = true;
    }
    if input.get_key(winit::event::VirtualKeyCode::A) > 0.0 {
        pos -= right * move_speed * delta as f32;
        should_update = true;
    }
    if input.get_key(winit::event::VirtualKeyCode::D) > 0.0 {
        pos += right * move_speed * delta as f32;
        should_update = true;
    }
    if input.get_key(winit::event::VirtualKeyCode::Space) > 0.0 {
        pos.y += move_speed * delta as f32;
        should_update = true;
    }
    if input.get_key(winit::event::VirtualKeyCode::LShift) > 0.0 {
        pos.y -= move_speed * delta as f32;
        should_update = true;
    }

    data.player.position = pos.into();

    if input.movement.0 != 0.0 || input.movement.1 != 0.0 {
        data.player.yaw +=
            (input.movement.0 / 360.0) as f32 * delta as f32 * data.player.sensitivity;
        data.player.pitch -=
            (input.movement.1 / 360.0) as f32 * delta as f32 * data.player.sensitivity;
        input.movement = (0.0, 0.0);
        should_update = true;
    }

    // 1.55 is just below 2pi
    if data.player.pitch > 1.55 {
        data.player.pitch = 1.55;
    } else if data.player.pitch < -1.55 {
        data.player.pitch = -1.55;
    }

    if should_update {
        queue.push(Event::PlayerMoved);
    }
}

// this does the actual updating of the camera buffer
// the input method just updates the values
pub fn update_camera(
    renderer: &mut Renderer,
    input: &mut Input,
    data: &mut GameData,
    queue: &mut Vec<Event>,
    delta: f64,
) {
    if let Some(Uniform {
        data: UniformData::Matrix(m),
        ..
    }) = renderer.get_global_uniform("view")
    {
        let mat = m.matrix();

        let (x, y, z) = data.player.position;
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
    input: &mut Input,
    data: &mut GameData,
    queue: &mut Vec<Event>,
    delta: f64,
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
    renderer: &mut Renderer,
    input: &mut Input,
    data: &mut GameData,
    queue: &mut Vec<Event>,
    delta: f64,
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
