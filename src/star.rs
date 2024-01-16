use std::rc::Rc;

use anyhow::{Context, Result};
use rand::random;

use crate::constants::SUBPIXELS;
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::tilemap::MapObject;
use crate::tileset::{TileIndex, TileSet};
use crate::utils::{intersect, Point, Rect};

pub struct Star<'a> {
    area: Rect,
    tileset: Rc<TileSet<'a>>,
    source: Rect,
}

fn star_rand() -> i32 {
    ((((random::<i32>() % 41) - 20) as f32) / 20.0).trunc() as i32
}

impl<'a> Star<'a> {
    pub fn new<'b>(obj: &MapObject, tileset: Rc<TileSet<'b>>) -> Result<Star<'b>> {
        let gid = obj.gid.context("star must have gid")?;
        let source = tileset.get_source_rect(gid as TileIndex - 1);
        let area = Rect {
            x: obj.position.x * SUBPIXELS,
            y: obj.position.y * SUBPIXELS,
            w: obj.position.w * SUBPIXELS,
            h: obj.position.h * SUBPIXELS,
        };
        Ok(Star {
            area,
            tileset,
            source,
        })
    }

    pub fn intersects(&self, player_rect: Rect) -> bool {
        return intersect(self.area, player_rect);
    }

    pub fn draw<'b>(&self, context: &'b mut RenderContext<'a>, layer: RenderLayer, offset: Point) {
        let mut x = self.area.x + offset.x();
        let mut y = self.area.y + offset.y();
        x += star_rand() * SUBPIXELS;
        y += star_rand() * SUBPIXELS;
        let rect = Rect {
            x,
            y,
            w: self.area.w,
            h: self.area.h,
        };
        let sprite = &self.tileset.sprite;
        context.draw(sprite, layer, rect, self.source);
        // TODO: Add lights.
    }
}
