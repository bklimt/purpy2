use anyhow::{anyhow, bail, Result};
use gilrs::{Axis, Button, GamepadId, Gilrs};
use log::{debug, info};

use crate::smallintmap::SmallIntMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum KeyboardKey {
    Escape,
    Space,
    Enter,
    W,
    A,
    S,
    D,
    Up,
    Down,
    Left,
    Right,
}

impl KeyboardKey {
    fn from_sdl_key(key: sdl2::keyboard::Keycode) -> Option<Self> {
        use sdl2::keyboard::Keycode;
        Some(match key {
            Keycode::Escape => KeyboardKey::Escape,
            Keycode::Space => KeyboardKey::Space,
            Keycode::Return => KeyboardKey::Enter,
            Keycode::W => KeyboardKey::W,
            Keycode::A => KeyboardKey::A,
            Keycode::S => KeyboardKey::S,
            Keycode::D => KeyboardKey::D,
            Keycode::Up => KeyboardKey::Up,
            Keycode::Down => KeyboardKey::Down,
            Keycode::Left => KeyboardKey::Left,
            Keycode::Right => KeyboardKey::Right,
            _ => return None,
        })
    }

    fn from_virtual_keycode(key: winit::event::VirtualKeyCode) -> Option<Self> {
        use winit::event::VirtualKeyCode;
        Some(match key {
            VirtualKeyCode::Escape => KeyboardKey::Escape,
            VirtualKeyCode::Space => KeyboardKey::Space,
            VirtualKeyCode::Return => KeyboardKey::Enter,
            VirtualKeyCode::W => KeyboardKey::W,
            VirtualKeyCode::A => KeyboardKey::A,
            VirtualKeyCode::S => KeyboardKey::S,
            VirtualKeyCode::D => KeyboardKey::D,
            VirtualKeyCode::Up => KeyboardKey::Up,
            VirtualKeyCode::Down => KeyboardKey::Down,
            VirtualKeyCode::Left => KeyboardKey::Left,
            VirtualKeyCode::Right => KeyboardKey::Right,
            _ => return None,
        })
    }
}

impl Into<usize> for KeyboardKey {
    fn into(self) -> usize {
        self as usize
    }
}

struct MouseButtonIndex(sdl2::mouse::MouseButton);

impl Into<usize> for MouseButtonIndex {
    fn into(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy)]
enum JoystickAxis {
    Vertical = 0,
    Horizontal,
}

impl From<JoystickAxis> for usize {
    fn from(value: JoystickAxis) -> Self {
        value as usize
    }
}

impl TryInto<JoystickAxis> for gilrs::Axis {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<JoystickAxis, Self::Error> {
        Ok(match self {
            gilrs::Axis::LeftStickX => JoystickAxis::Horizontal,
            gilrs::Axis::LeftStickY => JoystickAxis::Vertical,
            _ => bail!("invalid axis: {:?}", self),
        })
    }
}

impl TryInto<JoystickAxis> for u8 {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<JoystickAxis, Self::Error> {
        Ok(match self {
            0 => JoystickAxis::Vertical,
            1 => JoystickAxis::Horizontal,
            _ => bail!("invalid axis: {:?}", self),
        })
    }
}

struct InputState {
    keys_down: SmallIntMap<KeyboardKey, bool>,
    joystick_buttons_down: SmallIntMap<u8, bool>,
    joy_axes: SmallIntMap<JoystickAxis, f32>,
    joy_hats: SmallIntMap<u8, sdl2::joystick::HatState>,
    mouse_buttons_down: SmallIntMap<MouseButtonIndex, bool>,
}

impl InputState {
    fn new() -> InputState {
        InputState {
            keys_down: SmallIntMap::new(),
            joystick_buttons_down: SmallIntMap::new(),
            joy_axes: SmallIntMap::new(),
            joy_hats: SmallIntMap::new(),
            mouse_buttons_down: SmallIntMap::new(),
        }
    }

    fn set_key_down(&mut self, key: KeyboardKey) {
        self.keys_down.insert(key, true);
    }

    fn set_key_up(&mut self, key: KeyboardKey) {
        self.keys_down.insert(key, false);
    }

    fn is_key_down(&self, key: KeyboardKey) -> bool {
        *self.keys_down.get(key).unwrap_or(&false)
    }

    fn set_joystick_button_down(&mut self, button: u8) {
        self.joystick_buttons_down.insert(button, true);
    }

    fn set_joystick_button_up(&mut self, button: u8) {
        self.joystick_buttons_down.insert(button, false);
    }

    fn is_joystick_button_down(&self, button: u8) -> bool {
        *self.joystick_buttons_down.get(button).unwrap_or(&false)
    }

    fn set_mouse_button_down(&mut self, button: sdl2::mouse::MouseButton) {
        self.mouse_buttons_down
            .insert(MouseButtonIndex(button), true);
    }

    fn set_mouse_button_up(&mut self, button: sdl2::mouse::MouseButton) {
        self.mouse_buttons_down
            .insert(MouseButtonIndex(button), false);
    }

    fn is_mouse_button_down(&self, button: sdl2::mouse::MouseButton) -> bool {
        *self
            .mouse_buttons_down
            .get(MouseButtonIndex(button))
            .unwrap_or(&false)
    }

    fn set_joy_axis(&mut self, axis: JoystickAxis, value: f32) {
        self.joy_axes.insert(axis, value);
    }

    fn set_joy_hat(&mut self, hat: u8, state: sdl2::joystick::HatState) {
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
    key: KeyboardKey,
}

impl KeyInput {
    fn new(key: KeyboardKey) -> KeyInput {
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
    button: sdl2::mouse::MouseButton,
}

impl MouseButtonInput {}

impl TransientBinaryInput for MouseButtonInput {
    fn is_on(&self, state: &InputState) -> bool {
        state.is_mouse_button_down(self.button)
    }
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
        use sdl2::joystick::HatState;

        let diag = 0.7;
        state.joy_hats.get(0).map(|hat| match self.axis {
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
        state.joy_axes.get(self.axis).copied()
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
enum BinaryInput {
    Ok = 0,
    Cancel,
    PlayerLeft,
    PlayerRight,
    PlayerCrouch,
    PlayerJumpTrigger,
    PlayerJumpDown,
    MenuDown,
    MenuUp,
}

impl Into<usize> for BinaryInput {
    fn into(self) -> usize {
        self as usize
    }
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

fn key_input(key: KeyboardKey) -> Box<CachedBinaryInput<KeyInput>> {
    Box::new(CachedBinaryInput::from(KeyInput::new(key)))
}

fn key_trigger(key: KeyboardKey) -> Box<TriggerInput<KeyInput>> {
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

fn joystick_trigger(
    axis: JoystickAxis,
    low: Option<f32>,
    high: Option<f32>,
) -> Box<TriggerInput<JoystickThresholdInput>> {
    Box::new(TriggerInput::from(JoystickThresholdInput::new(
        axis, low, high,
    )))
}

fn create_input(input: BinaryInput) -> AnyOfInput {
    AnyOfInput(match input {
        BinaryInput::Ok => vec![key_trigger(KeyboardKey::Enter), joystick_button_trigger(0)],
        BinaryInput::Cancel => vec![key_trigger(KeyboardKey::Escape), joystick_button_trigger(2)],
        BinaryInput::PlayerLeft => vec![
            key_input(KeyboardKey::Left),
            key_input(KeyboardKey::A),
            joystick_threshold(JoystickAxis::Horizontal, Some(-0.5), None),
        ],
        BinaryInput::PlayerRight => vec![
            key_input(KeyboardKey::Right),
            key_input(KeyboardKey::D),
            joystick_threshold(JoystickAxis::Horizontal, None, Some(0.5)),
        ],
        BinaryInput::PlayerCrouch => vec![
            key_input(KeyboardKey::Down),
            key_input(KeyboardKey::S),
            joystick_threshold(JoystickAxis::Vertical, None, Some(0.5)),
        ],
        BinaryInput::PlayerJumpTrigger => vec![
            key_trigger(KeyboardKey::Space),
            key_trigger(KeyboardKey::W),
            key_trigger(KeyboardKey::Up),
            joystick_button_trigger(0),
        ],
        BinaryInput::PlayerJumpDown => vec![
            key_input(KeyboardKey::Space),
            key_input(KeyboardKey::W),
            key_input(KeyboardKey::Up),
            joystick_button_input(0),
        ],
        BinaryInput::MenuDown => vec![
            key_trigger(KeyboardKey::Down),
            key_trigger(KeyboardKey::S),
            joystick_trigger(JoystickAxis::Vertical, None, Some(0.5)),
        ],
        BinaryInput::MenuUp => vec![
            key_trigger(KeyboardKey::W),
            key_trigger(KeyboardKey::Up),
            joystick_trigger(JoystickAxis::Vertical, Some(-0.5), None),
        ],
    })
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct InputSnapshot {
    pub ok: bool,
    pub cancel: bool,
    pub player_left: bool,
    pub player_right: bool,
    pub player_crouch: bool,
    pub player_jump_trigger: bool,
    pub player_jump_down: bool,
    pub menu_down: bool,
    pub menu_up: bool,
}

pub struct InputManager {
    state: InputState,
    previous_snapshot: Option<InputSnapshot>,
    binary_hooks: SmallIntMap<BinaryInput, AnyOfInput>,
    all_binary_hooks: Vec<BinaryInput>,
    gilrs: Gilrs,
    current_gamepad: Option<GamepadId>,
}

impl InputManager {
    pub fn new() -> Result<InputManager> {
        let mut binary_hooks = SmallIntMap::new();
        let all_binary_hooks = all_binary_inputs();
        for hook in all_binary_hooks.iter() {
            binary_hooks.insert(hook.clone(), create_input(hook.clone()));
        }

        debug!("Initializing gamepads");
        let gilrs = Gilrs::new().map_err(|e| anyhow!("unable to load game library: {}", e))?;
        let mut current_gamepad = None;
        for (id, gamepad) in gilrs.gamepads() {
            info!(
                "Gamepad found: {} {} {:?}",
                id,
                gamepad.name(),
                gamepad.power_info()
            );
            if current_gamepad.is_none() {
                current_gamepad = Some(id);
            }
        }

        Ok(InputManager {
            state: InputState::new(),
            previous_snapshot: None,
            binary_hooks,
            all_binary_hooks,
            gilrs,
            current_gamepad,
        })
    }

    pub fn update(&mut self) -> InputSnapshot {
        while let Some(event) = self.gilrs.next_event() {
            self.handle_gilrs_event(event);
        }
        self.gilrs.inc();

        for input in self.all_binary_hooks.iter() {
            self.binary_hooks
                .get_mut(input.clone())
                .expect("all inputs should be configured")
                .update(&self.state);
        }

        let snapshot = InputSnapshot {
            ok: self.is_on(BinaryInput::Ok),
            cancel: self.is_on(BinaryInput::Cancel),
            player_left: self.is_on(BinaryInput::PlayerLeft),
            player_right: self.is_on(BinaryInput::PlayerRight),
            player_crouch: self.is_on(BinaryInput::PlayerCrouch),
            player_jump_trigger: self.is_on(BinaryInput::PlayerJumpTrigger),
            player_jump_down: self.is_on(BinaryInput::PlayerJumpDown),
            menu_down: self.is_on(BinaryInput::MenuDown),
            menu_up: self.is_on(BinaryInput::MenuUp),
        };
        if Some(snapshot) != self.previous_snapshot {
            debug!("{:?}", snapshot);
            self.previous_snapshot = Some(snapshot);
        }
        snapshot
    }

    fn is_on(&self, hook: BinaryInput) -> bool {
        self.binary_hooks
            .get(hook)
            .expect("all inputs should be configured")
            .is_on()
    }

    fn handle_gilrs_event(&mut self, event: gilrs::Event) {
        let gilrs::Event { id, event, .. } = event;
        debug!("Gamepad event from {}: {:?}", id, event);
        match event {
            gilrs::EventType::Connected => {
                if self.current_gamepad.is_none() {
                    info!("Using new gamepad {}", id);
                    self.current_gamepad = Some(id);
                }
            }
            gilrs::EventType::Disconnected => {
                if self.current_gamepad == Some(id) {
                    info!("Lost gamepad {}", id);
                    self.current_gamepad = None;
                }
            }
            gilrs::EventType::ButtonPressed(button, _) => {
                if let Some(index) = match button {
                    Button::South => Some(0),
                    _ => None,
                } {
                    self.state.set_joystick_button_down(index);
                }
            }
            gilrs::EventType::ButtonReleased(button, _) => {
                if let Some(index) = match button {
                    Button::South => Some(0),
                    _ => None,
                } {
                    self.state.set_joystick_button_up(index);
                }
            }
            gilrs::EventType::AxisChanged(axis, amount, _) => {
                if let Some((axis, polarity)) = match axis {
                    Axis::LeftStickY => Some((0, -1.0)),
                    Axis::LeftStickX => Some((1, 1.0)),
                    _ => None,
                } {
                    let axis = axis.try_into().expect("should be valid");
                    self.state.set_joy_axis(axis, amount * polarity);
                }
            }
            _ => {}
        }
    }

    pub fn handle_sdl_event(&mut self, event: &sdl2::event::Event) {
        use sdl2::event::Event;

        match event {
            Event::KeyDown {
                keycode: Some(key), ..
            } => {
                if let Some(key) = KeyboardKey::from_sdl_key(*key) {
                    self.state.set_key_down(key);
                }
            }
            Event::KeyUp {
                keycode: Some(key), ..
            } => {
                if let Some(key) = KeyboardKey::from_sdl_key(*key) {
                    self.state.set_key_up(key);
                }
            }
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
            } => {
                let axis = *axis;
                if let Ok(axis) = axis.try_into() {
                    let value = *value as f32 / i16::MAX as f32;
                    self.state.set_joy_axis(axis, value);
                }
            }
            Event::JoyHatMotion {
                hat_idx: hat,
                state,
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

    pub fn handle_winit_event(&mut self, event: &winit::event::WindowEvent) {
        use winit::event::ElementState;
        use winit::event::KeyboardInput;
        use winit::event::WindowEvent;

        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(virtual_key_code),
                        ..
                    },
                ..
            } => {
                if let Some(key) = KeyboardKey::from_virtual_keycode(*virtual_key_code) {
                    self.state.set_key_down(key);
                }
            }
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        state: ElementState::Released,
                        virtual_keycode: Some(virtual_key_code),
                        ..
                    },
                ..
            } => {
                if let Some(key) = KeyboardKey::from_virtual_keycode(*virtual_key_code) {
                    self.state.set_key_up(key);
                }
            }
            _ => {}
        }
    }
}
