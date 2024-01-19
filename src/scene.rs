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

pub trait Scene<'a> {
    fn update<'b, 'c>(
        &mut self,
        inputs: &'b InputSnapshot,
        sounds: &'c mut SoundManager,
    ) -> SceneResult;

    // TODO: It's unfortunate that draw has to be mutable for now.
    fn draw<'b, 'c>(&mut self, context: &'b mut RenderContext<'a>, images: &'c ImageManager<'a>)
    where
        'a: 'b,
        'a: 'c;
}
