use anyhow::Result;
use std::path::Path;

use crate::constants::SUBPIXELS;
use crate::imagemanager::ImageManager;
use crate::sprite::SpriteBatch;
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

    pub fn draw_string(&self, batch: &mut SpriteBatch, pos: Point, s: &str) {
        let mut pos = pos;
        for c in s.chars() {
            let c = (c as u32).min(127) as TileIndex;
            let area = self
                .tileset
                .get_source_rect(c)
                .expect("tileset should be large enough");
            let dest = Rect {
                x: pos.x(),
                y: pos.y(),
                w: self.char_width,
                h: self.char_height,
            };
            if dest.bottom() <= 0 || dest.right() <= 0 {
                continue;
            }
            batch.draw(&self.tileset.sprite, Some(dest), Some(area));
            pos = Point::new(pos.x() + self.char_width, pos.y());
        }
    }
}
