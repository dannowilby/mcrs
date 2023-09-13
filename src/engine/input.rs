use std::collections::HashMap;

use winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};

use crate::window_state;
/*
*/

pub struct Input {
    pub is_focused: bool,
    pub keys: HashMap<VirtualKeyCode, f64>,
    pub mouse: HashMap<MouseButton, f64>,
    pub movement: (f64, f64),
}

impl Input {
    pub fn new() -> Self {
        Self {
            is_focused: false,
            keys: HashMap::new(),
            mouse: HashMap::new(),
            movement: (0.0, 0.0),
        }
    }

    pub fn get_key(&self, vk: VirtualKeyCode) -> f64 {
        if let Some(k) = self.keys.get(&vk) {
            return instant::now() - k;
        }

        return 0.0;
    }

    pub fn get_click(&self, button: MouseButton) -> f64 {
        if let Some(k) = self.mouse.get(&button) {
            return instant::now() - k;
        }

        return 0.0;
    }

    pub fn handle(&mut self, event: &WindowEvent) {
        self.mouse_event(event);
        self.keyboard_event(event);
    }

    pub fn mouse_delta(&mut self, delta: (f64, f64)) {
        self.movement = delta;
    }

    fn mouse_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button,
                ..
            } => {
                self.mouse.entry(*button).or_insert(instant::now());
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button,
                ..
            } => {
                self.mouse.remove(button);
            }
            _ => {}
        }
    }

    fn keyboard_event(&mut self, event: &WindowEvent) {
        match event {
            WindowEvent::Focused(f) => {
                self.is_focused = *f;
                if *f {}
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(v),
                        ..
                    },
                ..
            } => {
                if v == &VirtualKeyCode::Escape {
                    let window = &window_state().window;
                    window
                        .set_cursor_grab(winit::window::CursorGrabMode::None)
                        .unwrap_or_default();
                    window.set_cursor_visible(true);
                }
                self.keys.entry(*v).or_insert(instant::now());
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(v),
                        ..
                    },
                ..
            } => {
                self.keys.remove(v);
            }
            _ => {}
        }
    }
}
