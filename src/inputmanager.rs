use std::collections::HashMap;

use sdl2::{keyboard::Keycode, mouse::MouseButton};

use crate::utils::Point;

struct InputState {
    keys_down: HashMap<Keycode, bool>,
    joystick_buttons_down: HashMap<u8, bool>,
    mouse_buttons_down: HashMap<MouseButton, bool>,
    mouse_position: Point,
}

impl InputState {
    fn new() -> InputState {
        InputState {
            keys_down: HashMap::new(),
            joystick_buttons_down: HashMap::new(),
            mouse_buttons_down: HashMap::new(),
            mouse_position: Point::new(0, 0),
        }
    }

    fn set_key_down(&mut self, key: Keycode) {
        self.keys_down.insert(key, true);
    }

    fn set_key_up(&mut self, key: Keycode) {
        self.keys_down.insert(key, false);
    }

    fn is_key_down(&self, key: Keycode) -> bool {
        *self.keys_down.get(&key).unwrap_or(&false)
    }

    fn set_joystick_button_down(&mut self, button: u8) {
        self.joystick_buttons_down.insert(button, true);
    }

    fn set_joystick_button_up(&mut self, button: u8) {
        self.joystick_buttons_down.insert(button, false);
    }

    fn is_joystick_button_down(&self, button: u8) -> bool {
        *self.joystick_buttons_down.get(&button).unwrap_or(&false)
    }

    fn set_mouse_button_down(&mut self, button: MouseButton) {
        self.mouse_buttons_down.insert(button, true);
    }

    fn set_mouse_button_up(&mut self, button: MouseButton) {
        self.mouse_buttons_down.insert(button, false);
    }

    fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        *self.mouse_buttons_down.get(&button).unwrap_or(&false)
    }
}
