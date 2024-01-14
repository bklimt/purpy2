use std::path::Path;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use sdl2::image::LoadSurface;
use sdl2::render::TextureCreator;
use sdl2::surface::Surface;
use sdl2::video::WindowContext;

use crate::font::Font;
use crate::sprite::{Animation, Sprite, SpriteSheet};

pub struct ImageManager<'a> {
    texture_creator: &'a TextureCreator<WindowContext>,
    font: Option<Font<'a>>,
}

impl<'a> ImageManager<'a> {
    pub fn new<'b>(canvas: &'b TextureCreator<WindowContext>) -> Result<ImageManager<'b>> {
        let mut im = ImageManager {
            texture_creator: canvas,
            font: None,
        };
        let font = Font::new(Path::new("assets/8bitfont.tsx"), &im)?;
        im.font = Some(font);
        Ok(im)
    }

    fn load_surface(&self, path: &Path) -> Result<Surface<'static>> {
        Surface::from_file(path).map_err(|s: String| anyhow!("{}", s))
    }

    pub fn load_sprite(&self, path: &Path) -> Result<Rc<Sprite<'a>>> {
        let surface = self.load_surface(path)?;
        Ok(Rc::new(Sprite::new(surface, self.texture_creator)?))
    }

    pub fn load_spritesheet(
        &self,
        path: &Path,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Result<SpriteSheet<'a>> {
        let surface = self.load_surface(path)?;
        SpriteSheet::new(surface, sprite_width, sprite_height, self.texture_creator)
    }

    pub fn load_animation(
        &self,
        path: &Path,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Result<Animation<'a>> {
        let surface = self.load_surface(path)?;
        Animation::new(surface, sprite_width, sprite_height, self.texture_creator)
    }

    pub fn font(&self) -> &Font<'a> {
        self.font
            .as_ref()
            .expect("should have been initialized in new()")
    }
}
