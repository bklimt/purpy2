use std::{collections::HashMap, fmt::Binary};

use sdl2::{event::Event, joystick::HatState, keyboard::Keycode, mouse::MouseButton};

use crate::utils::Point;

// TODO: Consider changing most of these to not be hashmaps.
struct InputState {
    keys_down: HashMap<Keycode, bool>,
    joystick_buttons_down: HashMap<u8, bool>,
    joy_axes: HashMap<u8, i16>,
    joy_hats: HashMap<u8, HatState>,
    mouse_buttons_down: HashMap<MouseButton, bool>,
    mouse_position: Point,
}

impl InputState {
    fn new() -> InputState {
        InputState {
            keys_down: HashMap::new(),
            joystick_buttons_down: HashMap::new(),
            joy_axes: HashMap::new(),
            joy_hats: HashMap::new(),
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

    fn set_joy_axis(&mut self, axis: u8, value: i16) {
        self.joy_axes.insert(axis, value);
    }

    fn set_joy_hat(&mut self, hat: u8, state: HatState) {
        self.joy_hats.insert(hat, state);
    }
}

trait TransientBinaryInput {
    fn is_on(&self, state: &InputState) -> bool;
}

trait StatefulBinaryInput {
    fn update(&mut self, state: &InputState);
    fn is_on(&self) -> bool;
}

struct CachedBinaryInput<T: TransientBinaryInput> {
    on: bool,
    inner: T,
}

impl<T> CachedBinaryInput<T>
where
    T: TransientBinaryInput,
{
    fn from(inner: T) -> CachedBinaryInput<T> {
        CachedBinaryInput {
            inner: inner,
            on: false,
        }
    }
}

impl<T> StatefulBinaryInput for CachedBinaryInput<T>
where
    T: TransientBinaryInput,
{
    fn update(&mut self, state: &InputState) {
        self.on = self.inner.is_on(state);
    }

    fn is_on(&self) -> bool {
        self.on
    }
}

struct TriggerInput<T: TransientBinaryInput> {
    inner: T,
    already_pressed: bool,
    on: bool,
}

impl<T> TriggerInput<T>
where
    T: TransientBinaryInput,
{
    fn from(inner: T) -> TriggerInput<T> {
        TriggerInput {
            inner: inner,
            already_pressed: false,
            on: false,
        }
    }
}

impl<T> StatefulBinaryInput for TriggerInput<T>
where
    T: TransientBinaryInput,
{
    fn update(&mut self, state: &InputState) {
        self.on = if self.inner.is_on(state) {
            if !self.already_pressed {
                self.already_pressed = true;
                true
            } else {
                false
            }
        } else {
            self.already_pressed = false;
            false
        };
    }

    fn is_on(&self) -> bool {
        self.on
    }
}

struct KeyInput {
    key: Keycode,
}

impl KeyInput {
    fn new(key: Keycode) -> KeyInput {
        KeyInput { key }
    }
}

impl TransientBinaryInput for KeyInput {
    fn is_on(&self, state: &InputState) -> bool {
        state.is_key_down(self.key)
    }
}

struct JoystickButtonInput {
    button: u8,
}

impl JoystickButtonInput {
    fn new(button: u8) -> Self {
        JoystickButtonInput { button }
    }
}

impl TransientBinaryInput for JoystickButtonInput {
    fn is_on(&self, state: &InputState) -> bool {
        state.is_joystick_button_down(self.button)
    }
}

struct MouseButtonInput {
    button: MouseButton,
}

impl MouseButtonInput {
    fn new(button: MouseButton) -> Self {
        MouseButtonInput { button }
    }
}

impl TransientBinaryInput for MouseButtonInput {
    fn is_on(&self, state: &InputState) -> bool {
        state.is_mouse_button_down(self.button)
    }
}

enum JoystickAxis {
    Vertical,
    Horizontal,
}

struct JoystickThresholdInput {
    axis: JoystickAxis,
    low_threshold: Option<f32>,
    high_threshold: Option<f32>,
}

impl JoystickThresholdInput {
    fn new(axis: JoystickAxis, low: Option<f32>, high: Option<f32>) -> JoystickThresholdInput {
        JoystickThresholdInput {
            axis,
            low_threshold: low,
            high_threshold: high,
        }
    }

    fn get_hat(&self, state: &InputState) -> Option<f32> {
        let diag = 0.7;
        state.joy_hats.get(&0).map(|hat| match self.axis {
            JoystickAxis::Horizontal => match hat {
                HatState::Centered => 0.0,
                HatState::Up => 0.0,
                HatState::RightUp => diag,
                HatState::Right => 1.0,
                HatState::RightDown => diag,
                HatState::Down => 0.0,
                HatState::LeftDown => -diag,
                HatState::Left => -1.0,
                HatState::LeftUp => -diag,
            },
            JoystickAxis::Vertical => match hat {
                HatState::Centered => 0.0,
                HatState::Up => 1.0,
                HatState::RightUp => diag,
                HatState::Right => 0.0,
                HatState::RightDown => -diag,
                HatState::Down => -1.0,
                HatState::LeftDown => -diag,
                HatState::Left => 0.0,
                HatState::LeftUp => diag,
            },
        })
    }

    fn get_axis(&self, state: &InputState) -> Option<f32> {
        state.joy_axes.get(&0).map(|n| *n as f32 / i16::MAX as f32)
    }
}

impl TransientBinaryInput for JoystickThresholdInput {
    fn is_on(&self, state: &InputState) -> bool {
        if let Some(hat) = self.get_hat(state) {
            if let Some(low) = self.low_threshold {
                if hat < low {
                    return true;
                }
            }
            if let Some(high) = self.high_threshold {
                if hat > high {
                    return true;
                }
            }
        }
        if let Some(axis) = self.get_axis(state) {
            if let Some(low) = self.low_threshold {
                if axis < low {
                    return true;
                }
            }
            if let Some(high) = self.high_threshold {
                if axis > high {
                    return true;
                }
            }
        }
        false
    }
}

/*
          }
            if self.low_threshold is not None and hat < self.low_threshold:
                return True
            if self.high_threshold is not None and hat > self.high_threshold:
                return True
        axis = self.get_axis(state)
        if axis is not None:
            if self.low_threshold is not None and axis < self.low_threshold:
                return True
            if self.high_threshold is not None and axis > self.high_threshold:
                return True
        return False
    }
}*/

/*
class JoystickThresholdInput(CachedBinaryInput):
    def __init__(self, axis: int, low_threshold: float | None, high_threshold: float | None):
        self.axis = axis
        self.low_threshold = low_threshold
        self.high_threshold = high_threshold

    def get_hat(self, state: InputState) -> float | None:
        if state.joystick is None:
            return None
        if state.joystick.get_numhats() < 1:
            return None
        hat = state.joystick.get_hat(0)
        value = hat[self.axis]
        if self.axis == 1:
            value *= -1
        return value

    def get_axis(self, state: InputState) -> float | None:
        if state.joystick is None:
            return None
        if state.joystick.get_numaxes() < 2:
            return None
        return state.joystick.get_axis(self.axis)

    def update_on(self, state: InputState) -> bool:
        hat = self.get_hat(state)
        if hat is not None:
            if self.low_threshold is not None and hat < self.low_threshold:
                return True
            if self.high_threshold is not None and hat > self.high_threshold:
                return True
        axis = self.get_axis(state)
        if axis is not None:
            if self.low_threshold is not None and axis < self.low_threshold:
                return True
            if self.high_threshold is not None and axis > self.high_threshold:
                return True
        return False
*/

struct AnyOfInput(Vec<Box<dyn StatefulBinaryInput>>);

impl StatefulBinaryInput for AnyOfInput {
    fn update(&mut self, state: &InputState) {
        for input in self.0.iter_mut() {
            input.update(state);
        }
    }

    fn is_on(&self) -> bool {
        for input in self.0.iter() {
            if input.is_on() {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum BinaryInput {
    Ok,
    Cancel,
    PlayerLeft,
    PlayerRight,
    PlayerCrouch,
    PlayerJumpTrigger,
    PlayerJumpDown,
    MenuDown,
    MenuUp,
}

fn all_binary_inputs() -> Vec<BinaryInput> {
    vec![
        BinaryInput::Ok,
        BinaryInput::Cancel,
        BinaryInput::PlayerLeft,
        BinaryInput::PlayerRight,
        BinaryInput::PlayerCrouch,
        BinaryInput::PlayerJumpTrigger,
        BinaryInput::PlayerJumpDown,
        BinaryInput::MenuDown,
        BinaryInput::MenuUp,
    ]
}

fn key_input(key: Keycode) -> Box<CachedBinaryInput<KeyInput>> {
    Box::new(CachedBinaryInput::from(KeyInput::new(key)))
}

fn key_trigger(key: Keycode) -> Box<TriggerInput<KeyInput>> {
    Box::new(TriggerInput::from(KeyInput::new(key)))
}

fn joystick_button_input(button: u8) -> Box<CachedBinaryInput<JoystickButtonInput>> {
    Box::new(CachedBinaryInput::from(JoystickButtonInput::new(button)))
}

fn joystick_button_trigger(button: u8) -> Box<TriggerInput<JoystickButtonInput>> {
    Box::new(TriggerInput::from(JoystickButtonInput::new(button)))
}

fn joystick_threshold(
    axis: JoystickAxis,
    low: Option<f32>,
    high: Option<f32>,
) -> Box<CachedBinaryInput<JoystickThresholdInput>> {
    Box::new(CachedBinaryInput::from(JoystickThresholdInput::new(
        axis, low, high,
    )))
}

fn create_input(input: BinaryInput) -> AnyOfInput {
    AnyOfInput(match input {
        BinaryInput::Ok => vec![key_trigger(Keycode::Return), joystick_button_trigger(0)],
        BinaryInput::Cancel => vec![key_trigger(Keycode::Escape), joystick_button_trigger(2)],
        BinaryInput::PlayerLeft => vec![
            key_input(Keycode::Left),
            key_input(Keycode::A),
            joystick_threshold(JoystickAxis::Horizontal, Some(-0.5), None),
        ],
        BinaryInput::PlayerRight => vec![
            key_input(Keycode::Right),
            key_input(Keycode::D),
            joystick_threshold(JoystickAxis::Horizontal, None, Some(0.5)),
        ],
        BinaryInput::PlayerCrouch => vec![
            key_input(Keycode::Down),
            key_input(Keycode::S),
            joystick_threshold(JoystickAxis::Vertical, None, Some(0.5)),
        ],
        BinaryInput::PlayerJumpTrigger => vec![
            key_trigger(Keycode::Space),
            key_trigger(Keycode::W),
            key_trigger(Keycode::Up),
            joystick_button_trigger(0),
        ],
        BinaryInput::PlayerJumpDown => vec![
            key_input(Keycode::Space),
            key_input(Keycode::W),
            key_input(Keycode::Up),
            joystick_button_input(0),
        ],
        BinaryInput::MenuDown => vec![
            key_input(Keycode::Down),
            key_input(Keycode::S),
            joystick_threshold(JoystickAxis::Vertical, None, Some(0.5)),
        ],
        BinaryInput::MenuUp => vec![
            key_trigger(Keycode::W),
            key_trigger(Keycode::Up),
            joystick_threshold(JoystickAxis::Vertical, Some(-0.5), None),
        ],
    })
}

pub struct InputManager {
    state: InputState,
    binary_hooks: HashMap<BinaryInput, AnyOfInput>,
}

impl InputManager {
    pub fn new() -> InputManager {
        let mut binary_hooks = HashMap::new();
        for hook in all_binary_inputs() {
            binary_hooks.insert(hook.clone(), create_input(hook));
        }
        InputManager {
            state: InputState::new(),
            binary_hooks,
        }
    }

    pub fn update(&mut self) {
        for (_, input) in self.binary_hooks.iter_mut() {
            input.update(&self.state);
        }
    }

    pub fn is_on(&self, hook: BinaryInput) -> bool {
        self.binary_hooks
            .get(&hook)
            .expect("all inputs should be configured")
            .is_on()
    }

    pub fn handle_event(&mut self, event: &Event) {
        match event {
            Event::KeyDown {
                keycode: Some(key), ..
            } => self.state.set_key_down(*key),
            Event::KeyUp {
                keycode: Some(key), ..
            } => self.state.set_key_up(*key),
            Event::JoyButtonDown {
                button_idx: button, ..
            } => self.state.set_joystick_button_down(*button),
            Event::JoyButtonUp {
                button_idx: button, ..
            } => self.state.set_joystick_button_up(*button),
            Event::JoyAxisMotion {
                axis_idx: axis,
                value,
                ..
            } => self.state.set_joy_axis(*axis, *value),
            Event::JoyHatMotion {
                hat_idx: hat,
                state: state,
                ..
            } => self.state.set_joy_hat(*hat, *state),
            Event::MouseButtonDown {
                mouse_btn: button, ..
            } => self.state.set_mouse_button_down(*button),
            Event::MouseButtonUp {
                mouse_btn: button, ..
            } => self.state.set_mouse_button_up(*button),
            _ => {}
        }
    }
}
