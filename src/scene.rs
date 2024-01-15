use std::path::PathBuf;

use anyhow::Result;

use crate::imagemanager::ImageManager;
use crate::inputmanager::InputManager;
use crate::soundmanager::SoundManager;

pub enum SceneResult {
    Continue,
    Quit,
    Pop,
    PushLevelSelect { path: PathBuf },
    PushLevel { path: PathBuf },
    SwitchToKillScreen { path: PathBuf },
    SwitchToLevel { path: PathBuf },
}

pub trait Scene {
    fn update(&mut self, inputs: &InputManager, sounds: SoundManager) -> Result<SceneResult>;
    fn draw(&self, images: &ImageManager);
}
