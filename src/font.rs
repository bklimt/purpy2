use anyhow::Result;
use std::path::Path;

use crate::constants::SUBPIXELS;
use crate::imagemanager::ImageLoader;
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::tileset::TileSet;
use crate::utils::{Point, Rect};

pub struct Font {
    tileset: TileSet,
    pub char_width: i32,
    pub char_height: i32,
}

impl Font {
    pub fn new(path: &Path, images: &mut dyn ImageLoader) -> Result<Font> {
        Ok(Font {
            tileset: TileSet::from_file(path, images)?,
            char_width: 8 * SUBPIXELS,
            char_height: 8 * SUBPIXELS,
        })
    }

    pub fn draw_string(
        &self,
        context: &mut RenderContext,
        layer: RenderLayer,
        pos: Point,
        s: &str,
    ) {
        let mut pos = pos;
        for c in s.chars() {
            let c = (c as usize).min(127).into();
            let area = self.tileset.get_source_rect(c);
            let dest = Rect {
                x: pos.x(),
                y: pos.y(),
                w: self.char_width,
                h: self.char_height,
            };
            if dest.bottom() <= 0 || dest.right() <= 0 {
                continue;
            }
            context.draw(self.tileset.sprite, layer, dest, area);
            pos = Point::new(pos.x() + self.char_width, pos.y());
        }
    }
}
