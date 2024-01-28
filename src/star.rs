use std::rc::Rc;

use anyhow::{Context, Result};
use rand::random;

use crate::geometry::{Pixels, Point, Rect, Subpixels};
use crate::rendercontext::{RenderContext, RenderLayer};
use crate::tilemap::TileIndex;
use crate::tilemap::{MapObject, TileMap};
use crate::utils::intersect;

pub struct Star {
    area: Rect<Subpixels>,
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
        let area = obj.position.into();
        Ok(Star {
            area,
            tile_gid,
            tilemap,
        })
    }

    pub fn intersects(&self, player_rect: Rect<Subpixels>) -> bool {
        return intersect(self.area, player_rect);
    }

    pub fn draw(&self, context: &mut RenderContext, layer: RenderLayer, offset: Point<Subpixels>) {
        let rand = Point::new(Pixels::new(star_rand()), Pixels::new(star_rand()));
        let pos = self.area.top_left() + offset + rand.into();
        let dest = Rect {
            x: pos.x,
            y: pos.y,
            w: self.area.w,
            h: self.area.h,
        };
        self.tilemap.draw_tile(context, self.tile_gid, layer, dest);
        // TODO: Add lights.
    }
}
