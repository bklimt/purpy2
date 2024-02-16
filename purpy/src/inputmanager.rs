use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use gilrs::Gilrs;
use log::{debug, error, info};
use num_traits::Zero;

use crate::filemanager::FileManager;
use crate::geometry::{Pixels, Point};
use crate::smallintmap::SmallIntMap;
use crate::{RENDER_HEIGHT, RENDER_WIDTH};

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
    #[cfg(feature = "sdl2")]
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

    #[cfg(feature = "winit")]
    fn from_keycode(key: winit::keyboard::KeyCode) -> Option<Self> {
        use winit::keyboard::KeyCode;
        Some(match key {
            KeyCode::Escape => KeyboardKey::Escape,
            KeyCode::Space => KeyboardKey::Space,
            KeyCode::Enter => KeyboardKey::Enter,
            KeyCode::KeyW => KeyboardKey::W,
            KeyCode::KeyA => KeyboardKey::A,
            KeyCode::KeyS => KeyboardKey::S,
            KeyCode::KeyD => KeyboardKey::D,
            KeyCode::ArrowUp => KeyboardKey::Up,
            KeyCode::ArrowDown => KeyboardKey::Down,
            KeyCode::ArrowLeft => KeyboardKey::Left,
            KeyCode::ArrowRight => KeyboardKey::Right,
            _ => return None,
        })
    }
}

impl From<KeyboardKey> for usize {
    fn from(value: KeyboardKey) -> Self {
        value as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum JoystickButton {
    Up = 0,
    Down,
    Left,
    Right,
    North,
    South,
    East,
    West,
}

impl JoystickButton {
    fn from_button(value: gilrs::Button) -> Option<Self> {
        use gilrs::Button;

        Some(match value {
            Button::South => JoystickButton::South,
            Button::East => JoystickButton::East,
            Button::North => JoystickButton::North,
            Button::West => JoystickButton::West,
            Button::DPadUp => JoystickButton::Up,
            Button::DPadDown => JoystickButton::Down,
            Button::DPadLeft => JoystickButton::Left,
            Button::DPadRight => JoystickButton::Right,
            _ => return None,
        })
    }
}

impl From<JoystickButton> for usize {
    fn from(value: JoystickButton) -> Self {
        value as usize
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

// TODO: Is this needed?
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

#[derive(Debug, Copy, Clone)]
enum MouseButton {
    Left = 0,
}

impl From<MouseButton> for usize {
    fn from(value: MouseButton) -> Self {
        value as usize
    }
}

struct InputState {
    keys_down: SmallIntMap<KeyboardKey, bool>,
    joystick_buttons_down: SmallIntMap<JoystickButton, bool>,
    joy_axes: SmallIntMap<JoystickAxis, f32>,
    mouse_buttons_down: SmallIntMap<MouseButton, bool>,

    mouse_position: Point<Pixels>,
    adjust_mouse_position: bool,
    window_width: i32,
    window_height: i32,
}

impl InputState {
    fn new(window_width: i32, window_height: i32, adjust_mouse_position: bool) -> InputState {
        InputState {
            keys_down: SmallIntMap::new(),
            joystick_buttons_down: SmallIntMap::new(),
            joy_axes: SmallIntMap::new(),
            mouse_buttons_down: SmallIntMap::new(),
            mouse_position: Point::zero(),
            adjust_mouse_position,
            window_width,
            window_height,
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

    fn set_joystick_button_down(&mut self, button: JoystickButton) {
        self.joystick_buttons_down.insert(button, true);
    }

    fn set_joystick_button_up(&mut self, button: JoystickButton) {
        self.joystick_buttons_down.insert(button, false);
    }

    fn is_joystick_button_down(&self, button: JoystickButton) -> bool {
        *self.joystick_buttons_down.get(button).unwrap_or(&false)
    }

    fn set_joy_axis(&mut self, axis: JoystickAxis, value: f32) {
        self.joy_axes.insert(axis, value);
    }

    fn set_mouse_button_down(&mut self, button: MouseButton) {
        self.mouse_buttons_down.insert(button, true);
    }

    fn set_mouse_button_up(&mut self, button: MouseButton) {
        self.mouse_buttons_down.insert(button, false);
    }

    fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        *self.mouse_buttons_down.get(button).unwrap_or(&false)
    }

    fn set_window_size(&mut self, width: i32, height: i32) {
        self.window_width = width;
        self.window_height = height;
    }

    fn set_mouse_position(&mut self, x: i32, y: i32) {
        self.mouse_position = if self.adjust_mouse_position {
            self.get_adjusted_mouse_position(x, y)
        } else {
            Point::new(Pixels::new(x), Pixels::new(y))
        };
    }

    fn get_adjusted_mouse_position(&mut self, pos_x: i32, pos_y: i32) -> Point<Pixels> {
        let x = (pos_x as f32) / (self.window_width as f32);
        let y = (pos_y as f32) / (self.window_height as f32);
        let x = x * (RENDER_WIDTH as f32);
        let y = y * (RENDER_HEIGHT as f32);
        Point::new(Pixels::new(x as i32), Pixels::new(y as i32))
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
        CachedBinaryInput { inner, on: false }
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
            inner,
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
    button: JoystickButton,
}

impl JoystickButtonInput {
    fn new(button: JoystickButton) -> Self {
        JoystickButtonInput { button }
    }
}

impl TransientBinaryInput for JoystickButtonInput {
    fn is_on(&self, state: &InputState) -> bool {
        state.is_joystick_button_down(self.button)
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

    fn get_axis(&self, state: &InputState) -> Option<f32> {
        state.joy_axes.get(self.axis).copied()
    }
}

impl TransientBinaryInput for JoystickThresholdInput {
    fn is_on(&self, state: &InputState) -> bool {
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
    OkTrigger = 0,
    OkDown,
    Cancel,
    PlayerLeft,
    PlayerRight,
    PlayerCrouch,
    PlayerJumpTrigger,
    PlayerJumpDown,
    MenuDown,
    MenuUp,
    MenuLeft,
    MenuRight,
    MouseButtonLeft,
}

impl From<BinaryInput> for usize {
    fn from(value: BinaryInput) -> Self {
        value as usize
    }
}

fn all_binary_inputs() -> Vec<BinaryInput> {
    vec![
        BinaryInput::OkTrigger,
        BinaryInput::OkDown,
        BinaryInput::Cancel,
        BinaryInput::PlayerLeft,
        BinaryInput::PlayerRight,
        BinaryInput::PlayerCrouch,
        BinaryInput::PlayerJumpTrigger,
        BinaryInput::PlayerJumpDown,
        BinaryInput::MenuDown,
        BinaryInput::MenuUp,
        BinaryInput::MenuLeft,
        BinaryInput::MenuRight,
        BinaryInput::MouseButtonLeft,
    ]
}

fn key_input(key: KeyboardKey) -> Box<CachedBinaryInput<KeyInput>> {
    Box::new(CachedBinaryInput::from(KeyInput::new(key)))
}

fn key_trigger(key: KeyboardKey) -> Box<TriggerInput<KeyInput>> {
    Box::new(TriggerInput::from(KeyInput::new(key)))
}

fn joystick_button_input(button: JoystickButton) -> Box<CachedBinaryInput<JoystickButtonInput>> {
    Box::new(CachedBinaryInput::from(JoystickButtonInput::new(button)))
}

fn joystick_button_trigger(button: JoystickButton) -> Box<TriggerInput<JoystickButtonInput>> {
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

fn mouse_button_input(button: MouseButton) -> Box<CachedBinaryInput<MouseButtonInput>> {
    Box::new(CachedBinaryInput::from(MouseButtonInput::new(button)))
}

fn create_input(input: BinaryInput) -> AnyOfInput {
    AnyOfInput(match input {
        BinaryInput::OkTrigger => vec![
            key_trigger(KeyboardKey::Enter),
            joystick_button_trigger(JoystickButton::South),
        ],
        BinaryInput::OkDown => vec![
            key_input(KeyboardKey::Enter),
            joystick_button_input(JoystickButton::South),
        ],
        BinaryInput::Cancel => vec![
            key_trigger(KeyboardKey::Escape),
            joystick_button_trigger(JoystickButton::West),
        ],
        BinaryInput::PlayerLeft => vec![
            key_input(KeyboardKey::Left),
            key_input(KeyboardKey::A),
            joystick_button_input(JoystickButton::Left),
            joystick_threshold(JoystickAxis::Horizontal, Some(-0.5), None),
        ],
        BinaryInput::PlayerRight => vec![
            key_input(KeyboardKey::Right),
            key_input(KeyboardKey::D),
            joystick_button_input(JoystickButton::Right),
            joystick_threshold(JoystickAxis::Horizontal, None, Some(0.5)),
        ],
        BinaryInput::PlayerCrouch => vec![
            key_input(KeyboardKey::Down),
            key_input(KeyboardKey::S),
            joystick_button_input(JoystickButton::Down),
            joystick_threshold(JoystickAxis::Vertical, None, Some(0.5)),
        ],
        BinaryInput::PlayerJumpTrigger => vec![
            key_trigger(KeyboardKey::Space),
            key_trigger(KeyboardKey::W),
            key_trigger(KeyboardKey::Up),
            joystick_button_trigger(JoystickButton::South),
        ],
        BinaryInput::PlayerJumpDown => vec![
            key_input(KeyboardKey::Space),
            key_input(KeyboardKey::W),
            key_input(KeyboardKey::Up),
            joystick_button_input(JoystickButton::South),
        ],
        BinaryInput::MenuDown => vec![
            key_trigger(KeyboardKey::Down),
            key_trigger(KeyboardKey::S),
            joystick_button_trigger(JoystickButton::Down),
            joystick_trigger(JoystickAxis::Vertical, None, Some(0.5)),
        ],
        BinaryInput::MenuUp => vec![
            key_trigger(KeyboardKey::W),
            key_trigger(KeyboardKey::Up),
            joystick_button_trigger(JoystickButton::Up),
            joystick_trigger(JoystickAxis::Vertical, Some(-0.5), None),
        ],
        BinaryInput::MenuLeft => vec![
            key_trigger(KeyboardKey::Left),
            key_trigger(KeyboardKey::A),
            joystick_button_trigger(JoystickButton::Left),
            joystick_trigger(JoystickAxis::Horizontal, Some(-0.5), None),
        ],
        BinaryInput::MenuRight => vec![
            key_trigger(KeyboardKey::D),
            key_trigger(KeyboardKey::Right),
            joystick_button_trigger(JoystickButton::Right),
            joystick_trigger(JoystickAxis::Horizontal, None, Some(0.5)),
        ],
        BinaryInput::MouseButtonLeft => vec![mouse_button_input(MouseButton::Left)],
    })
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct InputSnapshot {
    pub ok_clicked: bool,
    pub ok_down: bool,
    pub cancel_clicked: bool,
    pub player_left_down: bool,
    pub player_right_down: bool,
    pub player_crouch_down: bool,
    pub player_jump_clicked: bool,
    pub player_jump_down: bool,
    pub menu_down_clicked: bool,
    pub menu_up_clicked: bool,
    pub menu_left_clicked: bool,
    pub menu_right_clicked: bool,

    pub mouse_button_left_down: bool,

    pub mouse_position: Point<Pixels>,
}

#[inline]
fn bool_to_bin(b: bool, n: u8) -> u64 {
    if b {
        1 << n
    } else {
        0
    }
}

#[inline]
fn bin_to_bool(encoded: u64, n: u8) -> bool {
    (encoded & (1 << n)) != 0
}

impl InputSnapshot {
    fn encode(&self) -> u64 {
        let mut result = 0;
        result |= bool_to_bin(self.ok_clicked, 0);
        result |= bool_to_bin(self.cancel_clicked, 1);
        result |= bool_to_bin(self.player_left_down, 2);
        result |= bool_to_bin(self.player_right_down, 3);
        result |= bool_to_bin(self.player_crouch_down, 4);
        result |= bool_to_bin(self.player_jump_clicked, 5);
        result |= bool_to_bin(self.player_jump_down, 6);
        result |= bool_to_bin(self.menu_down_clicked, 7);
        result |= bool_to_bin(self.menu_up_clicked, 8);
        result
    }

    // TODO: Update this.
    fn decode(n: u64) -> InputSnapshot {
        InputSnapshot {
            ok_clicked: bin_to_bool(n, 0),
            ok_down: false,
            cancel_clicked: bin_to_bool(n, 1),
            player_left_down: bin_to_bool(n, 2),
            player_right_down: bin_to_bool(n, 3),
            player_crouch_down: bin_to_bool(n, 4),
            player_jump_clicked: bin_to_bool(n, 5),
            player_jump_down: bin_to_bool(n, 6),
            menu_down_clicked: bin_to_bool(n, 7),
            menu_up_clicked: bin_to_bool(n, 8),
            menu_left_clicked: false,
            menu_right_clicked: false,

            mouse_button_left_down: false,
            mouse_position: Point::zero(),
        }
    }
}

struct RecorderEntry {
    frame: u64,
    snapshot: u64,
}

pub struct InputRecorder {
    previous: u64,
    queue: VecDeque<RecorderEntry>,
}

impl InputRecorder {
    fn new() -> InputRecorder {
        InputRecorder {
            previous: 0,
            queue: VecDeque::new(),
        }
    }

    fn record(&mut self, frame: u64, snapshot: &InputSnapshot) {
        let snapshot = snapshot.encode();
        if self.previous == snapshot {
            return;
        }
        self.previous = snapshot;
        self.queue.push_back(RecorderEntry { frame, snapshot });
    }

    fn playback(&mut self, frame: u64) -> InputSnapshot {
        if let Some(next) = self.queue.front() {
            if next.frame == frame {
                self.previous = next.snapshot;
                self.queue.pop_front();
            }
        }
        InputSnapshot::decode(self.previous)
    }

    fn save(&self, path: &Path) -> Result<()> {
        let mut lines = Vec::new();
        for entry in self.queue.iter() {
            lines.push(format!("{},{}", entry.frame, entry.snapshot));
        }
        let text = lines.join("\n");
        fs::write(path, text)?;
        Ok(())
    }

    fn load(&mut self, path: &Path, files: &FileManager) -> Result<()> {
        self.previous = 0;
        self.queue.clear();

        let text = files
            .read_to_string(path)
            .map_err(|e| anyhow!("unable to load input snapshot record at {:?}: {}", path, e))?;

        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let comma = line.find(',').context("missing comma")?;
            let (frame, snapshot) = line.split_at(comma);
            let snapshot = &snapshot[1..];

            let frame = frame.parse()?;
            let snapshot = snapshot.parse()?;

            self.queue.push_back(RecorderEntry { frame, snapshot });
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum RecordOption {
    None,
    Record(PathBuf),
    Playback(PathBuf),
}

pub struct InputManager {
    state: InputState,
    previous_snapshot: Option<InputSnapshot>,
    binary_hooks: SmallIntMap<BinaryInput, AnyOfInput>,
    all_binary_hooks: Vec<BinaryInput>,
    gilrs: Gilrs,
    current_gamepad: Option<gilrs::GamepadId>,
    record_option: RecordOption,
    recorder: InputRecorder,
}

impl InputManager {
    pub fn with_options(
        window_width: i32,
        window_height: i32,
        adjust_mouse_position: bool,
        record_option: RecordOption,
        files: &FileManager,
    ) -> Result<InputManager> {
        let mut recorder = InputRecorder::new();

        if let RecordOption::Playback(path) = &record_option {
            recorder.load(Path::new(path), files)?;
        }

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
            state: InputState::new(window_width, window_height, adjust_mouse_position),
            previous_snapshot: None,
            binary_hooks,
            all_binary_hooks,
            gilrs,
            current_gamepad,
            record_option,
            recorder,
        })
    }

    pub fn update(&mut self, frame: u64) -> InputSnapshot {
        if let RecordOption::Playback(_) = self.record_option {
            return self.recorder.playback(frame);
        }

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
            ok_clicked: self.is_on(BinaryInput::OkTrigger),
            ok_down: self.is_on(BinaryInput::OkDown),
            cancel_clicked: self.is_on(BinaryInput::Cancel),
            player_left_down: self.is_on(BinaryInput::PlayerLeft),
            player_right_down: self.is_on(BinaryInput::PlayerRight),
            player_crouch_down: self.is_on(BinaryInput::PlayerCrouch),
            player_jump_clicked: self.is_on(BinaryInput::PlayerJumpTrigger),
            player_jump_down: self.is_on(BinaryInput::PlayerJumpDown),
            menu_down_clicked: self.is_on(BinaryInput::MenuDown),
            menu_up_clicked: self.is_on(BinaryInput::MenuUp),
            menu_left_clicked: self.is_on(BinaryInput::MenuLeft),
            menu_right_clicked: self.is_on(BinaryInput::MenuRight),
            mouse_button_left_down: self.is_on(BinaryInput::MouseButtonLeft),
            mouse_position: self.state.mouse_position,
        };
        if Some(snapshot) != self.previous_snapshot {
            debug!("{:?}", snapshot);
            self.previous_snapshot = Some(snapshot);
        }

        if let RecordOption::Record(_) = &self.record_option {
            self.recorder.record(frame, &snapshot);
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
                if let Some(button) = JoystickButton::from_button(button) {
                    self.state.set_joystick_button_down(button);
                }
            }
            gilrs::EventType::ButtonReleased(button, _) => {
                if let Some(button) = JoystickButton::from_button(button) {
                    self.state.set_joystick_button_up(button);
                }
            }
            gilrs::EventType::AxisChanged(axis, amount, _) => {
                if let Some((axis, polarity)) = match axis {
                    gilrs::Axis::LeftStickY => Some((0, -1.0)),
                    gilrs::Axis::LeftStickX => Some((1, 1.0)),
                    _ => None,
                } {
                    let axis = axis.try_into().expect("should be valid");
                    self.state.set_joy_axis(axis, amount * polarity);
                }
            }
            _ => {}
        }
    }

    #[cfg(feature = "sdl2")]
    pub fn handle_sdl_event(&mut self, event: &sdl2::event::Event) {
        use sdl2::event::Event;
        use sdl2::event::WindowEvent;

        match event {
            Event::Window {
                win_event: WindowEvent::SizeChanged(new_width, new_height),
                ..
            } => {
                info!("new window size: {new_width}x{new_height}");
                self.state.set_window_size(*new_width, *new_height);
            }
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
            Event::MouseButtonDown {
                mouse_btn: sdl2::mouse::MouseButton::Left,
                x,
                y,
                ..
            } => {
                self.state.set_mouse_position(*x, *y);
                self.state.set_mouse_button_down(MouseButton::Left);
            }
            Event::MouseButtonUp {
                mouse_btn: sdl2::mouse::MouseButton::Left,
                x,
                y,
                ..
            } => {
                self.state.set_mouse_position(*x, *y);
                self.state.set_mouse_button_up(MouseButton::Left);
            }
            Event::MouseMotion { x, y, .. } => {
                // info!("mouse moved to {x}, {y}");
                self.state.set_mouse_position(*x, *y);
            }
            _ => {}
        }
    }

    #[cfg(feature = "winit")]
    pub fn handle_winit_event(&mut self, event: &winit::event::WindowEvent) {
        use winit::dpi::{PhysicalPosition, PhysicalSize};
        use winit::event::{ElementState, KeyEvent, WindowEvent};
        use winit::keyboard::PhysicalKey;

        match event {
            WindowEvent::Resized(new_size) => {
                let PhysicalSize { width, height } = new_size;
                info!("window resized to {width}, {height}");
                self.state.set_window_size(*width as i32, *height as i32);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(key_code),
                        ..
                    },
                ..
            } => {
                if let Some(key) = KeyboardKey::from_keycode(*key_code) {
                    self.state.set_key_down(key);
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Released,
                        physical_key: PhysicalKey::Code(key_code),
                        ..
                    },
                ..
            } => {
                if let Some(key) = KeyboardKey::from_keycode(*key_code) {
                    self.state.set_key_up(key);
                }
            }
            WindowEvent::CursorMoved {
                position: PhysicalPosition { x, y },
                ..
            } => {
                let x = *x as i32;
                let y = *y as i32;
                // info!("mouse moved to {x}, {y}");
                self.state.set_mouse_position(x, y);
            }
            WindowEvent::MouseInput {
                state,
                button: winit::event::MouseButton::Left,
                ..
            } => match state {
                ElementState::Pressed => self.state.set_mouse_button_down(MouseButton::Left),
                ElementState::Released => self.state.set_mouse_button_up(MouseButton::Left),
            },
            _ => {}
        }
    }
}

impl Drop for InputManager {
    fn drop(&mut self) {
        if let RecordOption::Record(record) = &self.record_option {
            match self.recorder.save(record) {
                Ok(_) => info!("wrote input snapshot to {:?}", record),
                Err(e) => error!("unable to write input snapshot to {:?}: {}", record, e),
            }
        }
    }
}
