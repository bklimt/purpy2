use std::{mem, path::Path};

use anyhow::Result;

use crate::{
    filemanager::FileManager,
    font::Font,
    imagemanager::ImageLoader,
    inputmanager::InputSnapshot,
    level::Level,
    levelselect::LevelSelect,
    menu::Menu,
    rendercontext::RenderContext,
    scene::{Scene, SceneResult},
    soundmanager::SoundManager,
};

// A placeholder to use when swapping out scenes.
struct SceneTombstone(());

impl Scene for SceneTombstone {
    fn draw(&self, _context: &mut RenderContext, _font: &Font) {
        unimplemented!()
    }

    fn update(
        &mut self,
        _context: &RenderContext,
        _inputs: &InputSnapshot,
        _sounds: &mut SoundManager,
    ) -> SceneResult {
        unimplemented!()
    }
}

pub struct StageManager {
    current: Box<dyn Scene>,
    stack: Vec<Box<dyn Scene>>,
}

impl StageManager {
    pub fn new(file_manager: &FileManager, images: &mut dyn ImageLoader) -> Result<StageManager> {
        let path = Path::new("assets/menus/start.tmx");
        let menu = Menu::new_menu(path, file_manager, images)?;
        Ok(StageManager {
            current: Box::new(menu),
            stack: Vec::new(),
        })
    }

    pub fn update(
        &mut self,
        context: &RenderContext,
        inputs: &InputSnapshot,
        files: &FileManager,
        images: &mut dyn ImageLoader,
        sounds: &mut SoundManager,
    ) -> Result<bool> {
        let result = self.current.update(context, inputs, sounds);
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
                let level = Level::new(&path, files, images)?;
                let level = Box::new(level);
                let previous = mem::replace(&mut self.current, level);
                self.stack.push(previous);
                true
            }
            SceneResult::SwitchToLevel { path } => {
                self.current = Box::new(Level::new(&path, files, images)?);
                true
            }
            SceneResult::PushMenu { path } => {
                let menu = Menu::new_menu(&path, files, images)?;
                let menu = Box::new(menu);
                let previous = mem::replace(&mut self.current, menu);
                self.stack.push(previous);
                true
            }
            SceneResult::PushLevelSelect { path } => {
                let level_select = LevelSelect::new(&path, files)?;
                let level_select = Box::new(level_select);
                let previous = mem::replace(&mut self.current, level_select);
                self.stack.push(previous);
                true
            }
            SceneResult::SwitchToKillScreen { path } => {
                let mut previous: Box<dyn Scene> = Box::new(SceneTombstone(()));
                mem::swap(&mut self.current, &mut previous);
                //let kill_screen = KillScreen::new(previous, path);
                let kill_screen = Menu::new_death_screen(previous, path, files, images)?;
                let kill_screen = Box::new(kill_screen);
                self.current = kill_screen;
                true
            }
        })
    }

    pub fn draw(&mut self, context: &mut RenderContext, font: &Font) {
        self.current.draw(context, font)
    }
}
