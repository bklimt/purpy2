use std::path::Path;

use anyhow::Result;

use crate::{rendercontext::RenderContext, sprite::Sprite};

pub trait Renderer {
    fn load_sprite(&mut self, path: &Path) -> Result<Sprite>;

    fn render(&self, context: &RenderContext) -> Result<()>;
}
