use std::rc::Rc;

use anyhow::{Context, Result};
use rand::random;

use crate::constants::SUBPIXELS;
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::tilemap::TileIndex;
use crate::tilemap::{MapObject, TileMap};
use crate::utils::{intersect, Point, Rect};

pub struct Star {
    area: Rect,
    tilemap: Rc<TileMap>,
    tile_gid: TileIndex,
}

fn star_rand() -> i32 {
    ((((random::<i32>() % 41) - 20) as f32) / 20.0).trunc() as i32
}

impl Star {
    pub fn new<'b>(obj: &MapObject, tilemap: Rc<TileMap>) -> Result<Star> {
        let gid = obj.gid.context("star must have gid")?;
        let tile_gid = gid as TileIndex;
        let area = Rect {
            x: obj.position.x * SUBPIXELS,
            y: obj.position.y * SUBPIXELS,
            w: obj.position.w * SUBPIXELS,
            h: obj.position.h * SUBPIXELS,
        };
        Ok(Star {
            area,
            tile_gid,
            tilemap,
        })
    }

    pub fn intersects(&self, player_rect: Rect) -> bool {
        return intersect(self.area, player_rect);
    }

    pub fn draw(&self, context: &mut RenderContext, layer: RenderLayer, offset: Point) {
        let mut x = self.area.x + offset.x();
        let mut y = self.area.y + offset.y();
        x += star_rand() * SUBPIXELS;
        y += star_rand() * SUBPIXELS;
        let dest = Rect {
            x,
            y,
            w: self.area.w,
            h: self.area.h,
        };
        self.tilemap.draw_tile(context, self.tile_gid, layer, dest);
        // TODO: Add lights.
    }
}
