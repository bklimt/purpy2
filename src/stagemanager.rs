use std::{mem, path::Path};

use anyhow::Result;

use crate::{
    imagemanager::ImageManager,
    inputmanager::InputManager,
    level::Level,
    scene::{Scene, SceneResult},
    soundmanager::SoundManager,
};

struct StageManager<'a> {
    current: Box<dyn Scene<'a> + 'a>,
    stack: Vec<Box<dyn Scene<'a> + 'a>>,
}

impl<'a> StageManager<'a> {
    fn new<'b>(images: &'b ImageManager<'a>) -> Result<StageManager<'a>>
    where
        'a: 'b,
    {
        let path = Path::new("/dev/null");
        let level = Level::new(&path, images)?;
        Ok(StageManager {
            current: Box::new(level),
            stack: Vec::new(),
        })
    }

    fn update<'b, 'c, 'd>(
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
            SceneResult::Quit => false,
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
            SceneResult::PushLevelSelect { path } => unimplemented!(),
            SceneResult::SwitchToKillScreen { path } => unimplemented!(),
        })
    }

    fn draw(&mut self) {}
}
