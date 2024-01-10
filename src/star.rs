use anyhow::{Context, Result};
use rand::random;

use crate::{
    constants::SUBPIXELS,
    sprite::{Sprite, SpriteBatch},
    tilemap::MapObject,
    tileset::{TileIndex, TileSet},
    utils::{intersect, Point, Rect},
};

pub struct Star<'a> {
    area: Rect,
    sprite: &'a Sprite<'a>,
    source: Rect,
}

fn star_rand() -> i32 {
    ((((random::<i32>() % 41) - 20) as f32) / 20.0).trunc() as i32
}

impl<'a> Star<'a> {
    fn new<'b>(obj: MapObject, tileset: &'b TileSet<'b>) -> Result<Star<'b>> {
        let gid = obj.gid.context("star must have gid")?;
        let source = tileset.get_source_rect(gid as TileIndex - 1);
        let sprite = &tileset.sprite;
        let area = Rect {
            x: obj.position.x * SUBPIXELS,
            y: obj.position.y * SUBPIXELS,
            w: obj.position.w * SUBPIXELS,
            h: obj.position.h * SUBPIXELS,
        };
        Ok(Star {
            area,
            sprite,
            source,
        })
    }

    pub fn intersects(&self, player_rect: Rect) -> bool {
        return intersect(self.area, player_rect);
    }

    pub fn draw(&self, batch: &mut SpriteBatch, offset: Point) {
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
        batch.draw(self.sprite, Some(rect), Some(self.source));
        // TODO: Add lights.
    }
}
