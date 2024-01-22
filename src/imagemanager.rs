use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::font::Font;
use crate::renderer::Renderer;
use crate::sprite::{Animation, Sprite, SpriteSheet};

pub trait ImageLoader {
    fn load_sprite(&mut self, path: &Path) -> Result<Sprite>;

    fn load_spritesheet(
        &mut self,
        path: &Path,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Result<SpriteSheet>;

    fn load_animation(
        &mut self,
        path: &Path,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Result<Animation>;
}

pub struct ImageManager<T: Renderer> {
    path_to_sprite: HashMap<PathBuf, Sprite>,
    renderer: T,
}

impl<T> ImageManager<T>
where
    T: Renderer,
{
    pub fn new(renderer: T) -> Result<Self> {
        let path_to_sprite = HashMap::new();
        Ok(ImageManager {
            path_to_sprite,
            renderer,
        })
    }

    pub fn load_font(&mut self) -> Result<Font> {
        Font::new(Path::new("assets/8bitfont.tsx"), self)
    }

    pub fn renderer(&self) -> &T {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut T {
        &mut self.renderer
    }
}

impl<T> ImageLoader for ImageManager<T>
where
    T: Renderer,
{
    fn load_sprite(&mut self, path: &Path) -> Result<Sprite> {
        if let Some(existing) = self.path_to_sprite.get(path) {
            return Ok(*existing);
        }
        let sprite = self.renderer.load_sprite(path)?;
        self.path_to_sprite.insert(path.to_owned(), sprite);
        Ok(sprite)
    }

    fn load_spritesheet(
        &mut self,
        path: &Path,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Result<SpriteSheet> {
        let sprite = self.load_sprite(path)?;
        SpriteSheet::new(sprite, sprite_width, sprite_height)
    }

    fn load_animation(
        &mut self,
        path: &Path,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Result<Animation> {
        let sprite = self.load_sprite(path)?;
        Animation::new(sprite, sprite_width, sprite_height)
    }
}
