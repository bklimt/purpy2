use anyhow::Result;

use crate::imagemanager::ImageManager;
use crate::inputmanager::InputManager;
use crate::soundmanager::SoundManager;

pub enum SceneResult {
    Continue,
    Quit,
    Pop,
    PushLevelSelect { path: String },
    PushLevel { path: String },
    SwitchToKillScreen { path: String },
    SwitchToLevel { path: String },
}

pub trait Scene {
    fn update(&mut self, inputs: &InputManager, sounds: SoundManager) -> Result<SceneResult>;
    fn draw(&self, images: &ImageManager);
}
