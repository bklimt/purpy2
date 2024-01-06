use anyhow::{anyhow, Result};
use sdl2::image::LoadSurface;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use crate::sprite::{Animation, Sprite, SpriteSheet};

pub struct ImageManager<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
}

impl<'a> ImageManager<'a> {
    pub fn new<'b>(canvas: &'b TextureCreator<WindowContext>) -> ImageManager<'b> {
        ImageManager {
            texture_creator: canvas,
        }
    }

    fn load_surface(&self, path: &str) -> Result<Surface<'static>> {
        Surface::from_file(path).map_err(|s: String| anyhow!("{}", s))
    }

    pub fn load_sprite(&self, path: &str) -> Result<Sprite<'a>> {
        let surface = self.load_surface(path)?;
        Sprite::new(surface, self.texture_creator)
    }

    pub fn load_spritesheet(
        &self,
        path: &str,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Result<SpriteSheet<'a>> {
        let surface = self.load_surface(path)?;
        SpriteSheet::new(surface, sprite_width, sprite_height, self.texture_creator)
    }

    pub fn load_animation(
        &self,
        path: &str,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Result<Animation<'a>> {
        let surface = self.load_surface(path)?;
        Animation::new(surface, sprite_width, sprite_height, self.texture_creator)
    }
}
