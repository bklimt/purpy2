use std::path::PathBuf;

use crate::imagemanager::ImageManager;
use crate::inputmanager::InputSnapshot;
use crate::rendercontext::RenderContext;
use crate::soundmanager::SoundManager;

pub enum SceneResult {
    Continue,
    Pop,
    PushLevelSelect { path: PathBuf },
    PushLevel { path: PathBuf },
    SwitchToKillScreen { path: PathBuf },
    SwitchToLevel { path: PathBuf },
}

pub trait Scene {
    fn update(&mut self, inputs: &InputSnapshot, sounds: &mut SoundManager) -> SceneResult;

    // TODO: It's unfortunate that draw has to be mutable for now.
    fn draw(&mut self, context: &mut RenderContext, images: &ImageManager);
}
