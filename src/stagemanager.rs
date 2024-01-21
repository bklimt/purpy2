use std::{mem, path::Path};

use anyhow::Result;

use crate::{
    imagemanager::ImageManager,
    inputmanager::InputSnapshot,
    killscreen::KillScreen,
    level::Level,
    levelselect::LevelSelect,
    rendercontext::RenderContext,
    scene::{Scene, SceneResult},
    soundmanager::SoundManager,
};

// A placeholder to use when swapping out scenes.
struct SceneTombstone(());

impl Scene for SceneTombstone {
    fn draw(&mut self, _context: &mut RenderContext, _images: &ImageManager) {
        unimplemented!()
    }

    fn update(&mut self, _inputs: &InputSnapshot, _sounds: &mut SoundManager) -> SceneResult {
        unimplemented!()
    }
}

pub struct StageManager {
    current: Box<dyn Scene>,
    stack: Vec<Box<dyn Scene>>,
}

impl StageManager {
    pub fn new(_images: &ImageManager) -> Result<StageManager> {
        let path = Path::new("../purpy/assets/levels");
        let level_select = LevelSelect::new(&path)?;
        Ok(StageManager {
            current: Box::new(level_select),
            stack: Vec::new(),
        })
    }

    pub fn update(
        &mut self,
        inputs: &InputSnapshot,
        images: &ImageManager,
        sounds: &mut SoundManager,
    ) -> Result<bool> {
        let result = self.current.update(inputs, sounds);
        Ok(match result {
            SceneResult::Continue => true,
            SceneResult::Pop => {
                if let Some(next) = self.stack.pop() {
                    self.current = next;
                    true
                } else {
                    false
                }
            }
            SceneResult::PushLevel { path } => {
                let level = Level::new(&path, &images)?;
                let level = Box::new(level);
                let previous = mem::replace(&mut self.current, level);
                self.stack.push(previous);
                true
            }
            SceneResult::SwitchToLevel { path } => {
                self.current = Box::new(Level::new(&path, &images)?);
                true
            }
            SceneResult::PushLevelSelect { path } => {
                let level_select = LevelSelect::new(&path)?;
                let level_select = Box::new(level_select);
                let previous = mem::replace(&mut self.current, level_select);
                self.stack.push(previous);
                true
            }
            SceneResult::SwitchToKillScreen { path } => {
                let mut previous: Box<dyn Scene> = Box::new(SceneTombstone(()));
                mem::swap(&mut self.current, &mut previous);
                let kill_screen = KillScreen::new(previous, path);
                let kill_screen = Box::new(kill_screen);
                self.current = kill_screen;
                true
            }
        })
    }

    pub fn draw(&mut self, context: &mut RenderContext, images: &ImageManager) {
        self.current.draw(context, images)
    }
}
