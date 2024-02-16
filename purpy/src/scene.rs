use std::path::PathBuf;

use crate::font::Font;
use crate::inputmanager::InputSnapshot;
use crate::rendercontext::RenderContext;
use crate::soundmanager::SoundManager;

pub enum SceneResult {
    Continue,
    Pop,
    PopTwo,
    PushMenu { path: PathBuf },
    PushLevelSelect { path: PathBuf },
    PushLevel { path: PathBuf },
    ReloadLevel { path: PathBuf },
    PushKillScreen { path: PathBuf },
    PushPause { path: PathBuf },
    SwitchToLevel { path: PathBuf },
}

pub trait Scene {
    fn update(
        &mut self,
        context: &RenderContext,
        inputs: &InputSnapshot,
        sounds: &mut SoundManager,
    ) -> SceneResult;

    fn draw(&self, context: &mut RenderContext, font: &Font, previous: Option<&dyn Scene>);
}
