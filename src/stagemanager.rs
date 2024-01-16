use std::{mem, path::Path};

use anyhow::Result;

use crate::{
    imagemanager::ImageManager,
    inputmanager::InputManager,
    level::Level,
    levelselect::LevelSelect,
    rendercontext::RenderContext,
    scene::{Scene, SceneResult},
    soundmanager::SoundManager,
};

pub struct StageManager<'a> {
    current: Box<dyn Scene<'a> + 'a>,
    stack: Vec<Box<dyn Scene<'a> + 'a>>,
}

impl<'a> StageManager<'a> {
    pub fn new<'b>(_images: &'b ImageManager<'a>) -> Result<StageManager<'a>>
    where
        'a: 'b,
    {
        let path = Path::new("../purpy/assets/levels");
        let level_select = LevelSelect::new(&path)?;
        Ok(StageManager {
            current: Box::new(level_select),
            stack: Vec::new(),
        })
    }

    pub fn update<'b, 'c, 'd>(
        &mut self,
        inputs: &'b InputManager,
        images: &'c ImageManager<'a>,
        sounds: &'d SoundManager,
    ) -> Result<bool>
    where
        'a: 'c,
    {
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
            SceneResult::SwitchToKillScreen { path } => unimplemented!(),
        })
    }

    pub fn draw(&mut self, context: &mut RenderContext<'a>, images: &ImageManager<'a>) {
        self.current.draw(context, images)
    }
}
