use anyhow::{anyhow, Result};
use sdl2::image::LoadSurface;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

use crate::sprite::Sprite;

pub struct ImageManager {
    texture_creator: &'static TextureCreator<WindowContext>,
}

impl ImageManager {
    fn load_surface(&self, path: &str) -> Result<Surface<'static>> {
        Surface::from_file(path).map_err(|s: String| anyhow!("{}", s))
    }

    pub fn load_sprite(&self, path: &str) -> Result<Sprite> {
        let surface = self.load_surface(path)?;
        Sprite::new(surface, self.texture_creator)
    }
}
