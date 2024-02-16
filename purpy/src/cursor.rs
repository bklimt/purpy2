use std::path::Path;

use anyhow::Result;
use num_traits::Zero;

use crate::geometry::{Pixels, Point, Rect, Subpixels};
use crate::imagemanager::ImageLoader;
use crate::inputmanager::InputSnapshot;
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::sprite::Sprite;

pub struct Cursor {
    position: Point<Subpixels>,
    sprite: Sprite,
}

impl Cursor {
    pub fn new(images: &mut dyn ImageLoader) -> Result<Self> {
        let position = Point::zero();
        let sprite = images.load_sprite(Path::new("assets/cursor.png"))?;
        Ok(Cursor { position, sprite })
    }

    pub fn draw(&self, context: &mut RenderContext, layer: RenderLayer) {
        let src = Rect {
            x: Pixels::zero(),
            y: Pixels::zero(),
            w: Pixels::new(8),
            h: Pixels::new(8),
        };

        let dest = src.as_subpixels() + self.position;

        context.draw(self.sprite, layer, dest, src);
    }

    pub fn update(&mut self, input: &InputSnapshot) {
        self.position = input.mouse_position.into();
    }
}
