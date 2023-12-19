//! Used to capture mouse and keyboard events.

use std::collections::HashMap;

use winit::event::{ElementState, KeyboardInput, MouseButton, VirtualKeyCode, WindowEvent};

use crate::window_state;

/// Struct used to track how long a key or mouse button has been pressed. Also
/// stores mouse movement delta and whether or not the window is focused.
pub struct Input {
    pub is_focused: bool,
    pub keys: HashMap<VirtualKeyCode, f64>,
    pub mouse: HashMap<MouseButton, f64>,
    pub movement: (f64, f64),
}

impl Input {
    /// Create a new Input struct.
    pub fn new() -> Self {
        Self {
            is_focused: false,
            keys: HashMap::new(),
            mouse: HashMap::new(),
            movement: (0.0, 0.0),
        }
    }

    /// Get the duration a key has been pressed.
    pub fn get_key(&self, vk: VirtualKeyCode) -> f64 {
        if let Some(k) = self.keys.get(&vk) {
            return instant::now() - k;
        }

        return 0.0;
    }

    /// Get the duration a mouse button has been pressed.
    pub fn get_click(&self, button: MouseButton) -> f64 {
        if let Some(k) = self.mouse.get(&button) {
            return instant::now() - k;
        }

        return 0.0;
    }

    /// Process window event to update what keys or mouse buttons are pressed.
    pub fn handle(&mut self, event: &WindowEvent) {
        self.mouse_event(event);
        self.keyboard_event(event);
    }

    /// Update the mouse movement delta.
    pub fn mouse_delta(&mut self, delta: (f64, f64)) {
        self.movement = delta;
    }

    /// Get mouse button press state from event.
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

    /// Get key press state from event.
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

#[cfg(test)]
mod tests {
    use winit::event::{DeviceId, ModifiersState};

    use super::*;

    #[test]
    fn keyboard_press_event_test() {
        let mut input = Input::new();

        let key_press_event = WindowEvent::KeyboardInput {
            device_id: unsafe { DeviceId::dummy() },
            input: KeyboardInput {
                state: ElementState::Pressed,
                scancode: 0x1e,
                virtual_keycode: Some(VirtualKeyCode::A),
                modifiers: ModifiersState::default(),
            },
            is_synthetic: false,
        };
        let key_release_event = WindowEvent::KeyboardInput {
            device_id: unsafe { DeviceId::dummy() },
            input: KeyboardInput {
                state: ElementState::Released,
                scancode: 0x1e,
                virtual_keycode: Some(VirtualKeyCode::A),
                modifiers: ModifiersState::default(),
            },
            is_synthetic: false,
        };

        input.handle(&key_press_event);
        assert!(input.get_key(VirtualKeyCode::A) > 0.0);

        input.handle(&key_release_event);
        assert!(input.get_key(VirtualKeyCode::A) == 0.0);
    }

    #[test]
    fn mouse_press_event_test() {
        let mut input = Input::new();

        let mouse_press_event = WindowEvent::MouseInput {
            device_id: unsafe { DeviceId::dummy() },
            state: ElementState::Pressed,
            button: MouseButton::Left,
            modifiers: ModifiersState::default(),
        };
        let mouse_release_event = WindowEvent::MouseInput {
            device_id: unsafe { DeviceId::dummy() },
            state: ElementState::Released,
            button: MouseButton::Left,
            modifiers: ModifiersState::default(),
        };

        input.handle(&mouse_press_event);
        assert!(input.get_click(MouseButton::Left) > 0.0);

        input.handle(&mouse_release_event);
        assert!(input.get_click(MouseButton::Left) == 0.0);
    }
}
