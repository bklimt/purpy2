use std::path::Path;
use std::{fs, path::PathBuf};

use anyhow::{Context, Result};

use crate::imagemanager::ImageManager;
use crate::inputmanager::BinaryInput;
use crate::inputmanager::InputManager;
use crate::rendercontext::RenderContext;
use crate::rendercontext::RenderLayer;
use crate::scene::Scene;
use crate::scene::SceneResult;
use crate::soundmanager::SoundManager;

pub struct LevelSelect {
    directory: PathBuf,
    files: Vec<(PathBuf, String)>,
    current: i32,
    start: i32,
}

impl LevelSelect {
    pub fn new(directory: &Path) -> Result<LevelSelect> {
        let mut files = Vec::new();
        let file_list =
            fs::read_dir(&directory).context(format!("unable to read {:?}", directory))?;
        for file in file_list {
            let file = file.context(format!(
                "error iterating through contents of {:?}",
                directory
            ))?;
            let path = file.path();
            let name = file
                .file_name()
                .to_str()
                .context(format!("unable to encode name {:?}", path))?
                .to_owned();
            files.push((path, name));
        }
        let directory = directory.to_owned();
        Ok(LevelSelect {
            directory,
            files,
            current: 0,
            start: 0,
        })
    }
}

impl<'a> Scene<'a> for LevelSelect {
    fn update(&mut self, inputs: &InputManager, sounds: &SoundManager) -> SceneResult {
        if inputs.is_on(BinaryInput::Cancel) {
            return SceneResult::Pop;
        }
        if inputs.is_on(BinaryInput::MenuUp) {
            self.current = (self.current - 1) % self.files.len() as i32;
        }
        if inputs.is_on(BinaryInput::MenuDown) {
            self.current = (self.current + 1) % self.files.len() as i32;
        }
        if inputs.is_on(BinaryInput::Ok) {
            let new_path = self.directory.join(&self.files[self.current as usize].0);
            if new_path.is_dir() {
                SceneResult::PushLevelSelect { path: new_path }
            } else {
                SceneResult::PushLevel { path: new_path }
            }
        } else {
            SceneResult::Continue
        }
    }

    fn draw<'b, 'c>(&mut self, context: &'b mut RenderContext<'a>, images: &'c ImageManager<'a>)
    where
        'a: 'b,
        'a: 'c,
    {
        let layer = RenderLayer::Hud;
        let font_height = images.font().char_height;
        let line_spacing = font_height / 2;

        let x = line_spacing;
        let mut y = line_spacing;
        let dir_str = self.directory.to_string_lossy();
        images
            .font()
            .draw_string(context, layer, (x, y).into(), &dir_str);
        y += font_height + line_spacing;

        if self.current < self.start {
            // You scrolled up past what was visible.
            self.start = self.current;
        }
        if self.current >= self.start + 10 {
            // You scrolled off the bottom.
            self.start = self.current - 10;
        }
        if self.start != 0 {
            images
                .font()
                .draw_string(context, layer, (x, y).into(), " ...")
        }
        y += (font_height + line_spacing);

        for i in self.start..self.start + 11 {
            if i < 0 || i >= self.files.len() as i32 {
                continue;
            }
            let cursor = if i == self.current { '>' } else { ' ' };
            images.font().draw_string(
                context,
                layer,
                (x, y).into(),
                &format!("{}{}", cursor, &self.files[i as usize].1),
            );
            y += font_height + line_spacing;
        }

        if self.start + 12 <= self.files.len() as i32 {
            images
                .font()
                .draw_string(context, layer, (x, y).into(), " ...");
        }
    }
}
