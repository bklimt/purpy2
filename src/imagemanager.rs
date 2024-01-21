use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::font::Font;
use crate::renderer::Renderer;
use crate::sprite::{Animation, Sprite, SpriteSheet};

pub struct ImageManager<'a> {
    path_to_sprite: HashMap<PathBuf, Sprite>,
    renderer: &'a mut dyn Renderer,
    font: Option<Font>,
}

impl<'a> ImageManager<'a> {
    pub fn new(renderer: &'a mut dyn Renderer) -> Result<Self> {
        let path_to_sprite = HashMap::new();

        let mut im = ImageManager {
            path_to_sprite,
            renderer,
            font: None,
        };

        let font = Font::new(Path::new("assets/8bitfont.tsx"), &im)?;
        im.font = Some(font);

        Ok(im)
    }

    pub fn load_sprite(&mut self, path: &Path) -> Result<Sprite> {
        if let Some(existing) = self.path_to_sprite.get(path) {
            return Ok(*existing);
        }
        let sprite = self.renderer.load_sprite(path)?;
        self.path_to_sprite.insert(path.to_owned(), sprite);
        Ok(sprite)
    }

    pub fn load_spritesheet(
        &mut self,
        path: &Path,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Result<SpriteSheet> {
        let sprite = self.load_sprite(path)?;
        SpriteSheet::new(sprite, sprite_width, sprite_height)
    }

    pub fn load_animation(
        &mut self,
        path: &Path,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Result<Animation> {
        let sprite = self.load_sprite(path)?;
        Animation::new(sprite, sprite_width, sprite_height)
    }

    pub fn font(&self) -> &Font {
        self.font
            .as_ref()
            .expect("should have been initialized in new()")
    }
}
