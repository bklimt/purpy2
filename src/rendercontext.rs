use anyhow::Result;

use crate::constants::SUBPIXELS;
use crate::sprite::Sprite;
use crate::utils::{Color, Rect};

pub enum SpriteBatchEntry {
    Sprite {
        sprite: Sprite,
        source: Rect,
        destination: Rect,
        reversed: bool,
    },
    FillRect {
        destination: Rect,
        color: Color,
    },
}

pub struct SpriteBatch {
    pub clear_color: Color,
    pub entries: Vec<SpriteBatchEntry>,
}

impl SpriteBatch {
    pub fn new() -> SpriteBatch {
        SpriteBatch {
            clear_color: Color {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            },
            entries: Vec::new(),
        }
    }

    pub fn draw(&mut self, sprite: Sprite, dst: Rect, src: Rect, reversed: bool) {
        let dst = Rect {
            x: dst.x / SUBPIXELS,
            y: dst.y / SUBPIXELS,
            w: dst.w / SUBPIXELS,
            h: dst.h / SUBPIXELS,
        };
        self.entries.push(SpriteBatchEntry::Sprite {
            sprite: sprite.clone(),
            source: src,
            destination: dst,
            reversed,
        });
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        let rect = Rect {
            x: rect.x / SUBPIXELS,
            y: rect.y / SUBPIXELS,
            w: rect.w / SUBPIXELS,
            h: rect.h / SUBPIXELS,
        };
        self.entries.push(SpriteBatchEntry::FillRect {
            destination: rect,
            color,
        });
    }

    /*pub fn clear(&mut self, color: Color) {
        self.entries.push(SpriteBatchEntry::Clear { color })
    }*/
}

#[derive(Debug, Clone, Copy)]
pub enum RenderLayer {
    Player,
    Hud,
}

pub struct RenderContext {
    pub player_batch: SpriteBatch,
    pub width: u32,
    pub height: u32,
    pub frame: u32,
}

impl RenderContext {
    pub fn new(width: u32, height: u32, frame: u32) -> Result<RenderContext> {
        let player_batch = SpriteBatch::new();
        Ok(RenderContext {
            player_batch,
            width,
            height,
            frame,
        })
    }

    pub fn logical_area_in_subpixels(&self) -> Rect {
        Rect {
            x: 0,
            y: 0,
            w: self.width as i32 * SUBPIXELS,
            h: self.height as i32 * SUBPIXELS,
        }
    }

    // TODO: Get rid of these.

    pub fn draw(&mut self, sprite: Sprite, _layer: RenderLayer, dst: Rect, src: Rect) {
        self.player_batch.draw(sprite, dst, src, false)
    }

    pub fn draw_reversed(&mut self, sprite: Sprite, _layer: RenderLayer, dst: Rect, src: Rect) {
        self.player_batch.draw(sprite, dst, src, true)
    }

    pub fn fill_rect(&mut self, rect: Rect, _layer: RenderLayer, color: Color) {
        self.player_batch.fill_rect(rect, color);
    }

    pub fn clear(&mut self) {
        self.player_batch.entries.clear();
        self.player_batch.clear_color = Color {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };
    }
}
