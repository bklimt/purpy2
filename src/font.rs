use anyhow::Result;
use num_traits::Zero;
use std::path::Path;

use crate::geometry::{Point, Rect, Subpixels};
use crate::imagemanager::ImageLoader;
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::tilemap::TileIndex;
use crate::tileset::TileSet;

pub struct Font {
    tileset: TileSet,
    pub char_width: Subpixels,
    pub char_height: Subpixels,
}

impl Font {
    pub fn new(path: &Path, images: &mut dyn ImageLoader) -> Result<Font> {
        // It doesn't actually matter what the global id is, since there is no map.
        let firstgid: TileIndex = 0.into();
        Ok(Font {
            tileset: TileSet::from_file(path, firstgid, images)?,
            char_width: Subpixels::from_pixels(8),
            char_height: Subpixels::from_pixels(8),
        })
    }

    pub fn draw_string(
        &self,
        context: &mut RenderContext,
        layer: RenderLayer,
        pos: Point<Subpixels>,
        s: &str,
    ) {
        let mut pos = pos;
        for c in s.chars() {
            let c = (c as usize).min(127).into();
            let area = self.tileset.get_source_rect(c);
            let dest = Rect {
                x: pos.x,
                y: pos.y,
                w: self.char_width,
                h: self.char_height,
            };
            if dest.bottom() <= Subpixels::zero() || dest.right() <= Subpixels::zero() {
                continue;
            }
            context.draw(self.tileset.sprite, layer, dest, area);
            pos = Point::new(pos.x + self.char_width, pos.y);
        }
    }
}
