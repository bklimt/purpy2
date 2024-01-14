use anyhow::Result;
use std::path::Path;

use crate::constants::SUBPIXELS;
use crate::imagemanager::ImageManager;
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::tileset::{TileIndex, TileSet};
use crate::utils::{Point, Rect};

pub struct Font<'a> {
    tileset: TileSet<'a>,
    char_width: i32,
    char_height: i32,
}

impl<'a> Font<'a> {
    pub fn new<'b>(path: &Path, images: &ImageManager<'b>) -> Result<Font<'b>> {
        Ok(Font {
            tileset: TileSet::from_file(path, images)?,
            char_width: 8 * SUBPIXELS,
            char_height: 8 * SUBPIXELS,
        })
    }

    pub fn draw_string<'b>(
        &self,
        context: &'b mut RenderContext<'a>,
        layer: RenderLayer,
        pos: Point,
        s: &str,
    ) where
        'a: 'b,
    {
        let mut pos = pos;
        for c in s.chars() {
            let c = (c as u32).min(127) as TileIndex;
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
            context.draw(&self.tileset.sprite, layer, dest, area);
            pos = Point::new(pos.x() + self.char_width, pos.y());
        }
    }
}
