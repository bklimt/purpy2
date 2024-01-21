use std::path::Path;

use anyhow::Result;

use crate::sprite::Sprite;

pub trait Renderer {
    fn load_sprite(&mut self, path: &Path) -> Result<Sprite>;
}
