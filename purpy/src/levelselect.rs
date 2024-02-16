use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Result};
use log::debug;

use crate::filemanager::DirEntry;
use crate::filemanager::DirEntryType;
use crate::filemanager::FileManager;
use crate::font::Font;
use crate::inputmanager::InputSnapshot;
use crate::rendercontext::RenderContext;
use crate::rendercontext::RenderLayer;
use crate::scene::Scene;
use crate::scene::SceneResult;
use crate::soundmanager::SoundManager;

pub struct LevelSelect {
    directory: PathBuf,
    files: Vec<DirEntry>,
    current: i32,
    start: i32,
}

impl LevelSelect {
    pub fn new(directory: &Path, file_manager: &FileManager) -> Result<LevelSelect> {
        debug!("Scanning directory {:?}", directory);
        let mut files = Vec::new();
        let file_list = file_manager
            .read_dir(directory)
            .context(format!("unable to read {:?}", directory))?;
        for file in file_list {
            debug!("Found directory entry {:?}", &file.full_path);
            files.push(file);
        }

        files.sort_by_key(|entry| entry.name.clone());

        let directory = directory.to_owned();
        Ok(LevelSelect {
            directory,
            files,
            current: 0,
            start: 0,
        })
    }
}

impl Scene for LevelSelect {
    fn update(
        &mut self,
        _context: &RenderContext,
        inputs: &InputSnapshot,
        _sounds: &mut SoundManager,
    ) -> SceneResult {
        if inputs.cancel_clicked {
            return SceneResult::Pop;
        }
        if inputs.menu_up_clicked {
            self.current = ((self.current - 1) + self.files.len() as i32) % self.files.len() as i32;
        }
        if inputs.menu_down_clicked {
            self.current = (self.current + 1) % self.files.len() as i32;
        }
        if inputs.ok_clicked {
            let entry = &self.files[self.current as usize];
            let new_path = entry.full_path.clone();
            if matches!(entry.file_type, DirEntryType::Directory) {
                SceneResult::PushLevelSelect { path: new_path }
            } else {
                SceneResult::PushLevel { path: new_path }
            }
        } else {
            if self.current < self.start {
                // You scrolled up past what was visible.
                self.start = self.current;
            }
            if self.current >= self.start + 10 {
                // You scrolled off the bottom.
                self.start = self.current - 10;
            }

            SceneResult::Continue
        }
    }

    fn draw(&self, context: &mut RenderContext, font: &Font, _previous: Option<&dyn Scene>) {
        let layer = RenderLayer::Hud;
        let font_height = font.char_height;
        let line_spacing = font_height / 2;

        let x = line_spacing;
        let mut y = line_spacing;
        let dir_str = self.directory.to_string_lossy();
        font.draw_string(context, layer, (x, y).into(), &dir_str);
        y += font_height + line_spacing;

        if self.start != 0 {
            font.draw_string(context, layer, (x, y).into(), " ...")
        }
        y += font_height + line_spacing;

        for i in self.start..self.start + 11 {
            if i < 0 || i >= self.files.len() as i32 {
                continue;
            }
            let cursor = if i == self.current { '>' } else { ' ' };
            font.draw_string(
                context,
                layer,
                (x, y).into(),
                &format!("{}{}", cursor, &self.files[i as usize].name),
            );
            y += font_height + line_spacing;
        }

        if self.start + 12 <= self.files.len() as i32 {
            font.draw_string(context, layer, (x, y).into(), " ...");
        }
    }
}
