use anyhow::Result;
use log::warn;
use num_traits::Zero;

use crate::constants::MAX_LIGHTS;
use crate::geometry::{Pixels, Point, Rect, Subpixels};
use crate::sprite::Sprite;
use crate::utils::Color;

pub enum SpriteBatchEntry {
    Sprite {
        sprite: Sprite,
        source: Rect<Pixels>,
        destination: Rect<Pixels>,
        reversed: bool,
    },
    FillRect {
        destination: Rect<Pixels>,
        color: Color,
    },
}

pub struct SpriteBatch {
    pub clear_color: Color,
    pub entries: Vec<SpriteBatchEntry>,
}

impl SpriteBatch {
    #[allow(clippy::new_without_default)]
    pub fn new() -> SpriteBatch {
        SpriteBatch {
            clear_color: Color {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            },
            entries: Vec::new(),
        }
    }

    pub fn draw(
        &mut self,
        sprite: Sprite,
        dst: Rect<Subpixels>,
        src: Rect<Pixels>,
        reversed: bool,
    ) {
        let dst = dst.as_pixels();
        self.entries.push(SpriteBatchEntry::Sprite {
            sprite,
            source: src,
            destination: dst,
            reversed,
        });
    }

    pub fn fill_rect(&mut self, rect: Rect<Subpixels>, color: Color) {
        let rect = rect.as_pixels();
        self.entries.push(SpriteBatchEntry::FillRect {
            destination: rect,
            color,
        });
    }
}

pub struct Light {
    pub position: Point<Subpixels>,
    pub radius: Subpixels,
}

#[derive(Debug, Clone, Copy)]
pub enum RenderLayer {
    Player,
    Hud,
}

pub struct RenderContext {
    pub player_batch: SpriteBatch,
    pub hud_batch: SpriteBatch,
    pub width: u32,
    pub height: u32,
    pub frame: u64,
    pub lights: Vec<Light>,
    pub is_dark: bool,
}

impl RenderContext {
    pub fn new(width: u32, height: u32, frame: u64) -> Result<RenderContext> {
        let player_batch = SpriteBatch::new();
        let hud_batch = SpriteBatch::new();
        let lights = Vec::new();
        let is_dark = false;
        Ok(RenderContext {
            player_batch,
            hud_batch,
            width,
            height,
            frame,
            lights,
            is_dark,
        })
    }

    pub fn logical_area_in_subpixels(&self) -> Rect<Subpixels> {
        // TODO: This should be cacheable.
        Rect {
            x: Pixels::zero(),
            y: Pixels::zero(),
            w: Pixels::new(self.width as i32),
            h: Pixels::new(self.height as i32),
        }
        .into()
    }

    pub fn draw(
        &mut self,
        sprite: Sprite,
        layer: RenderLayer,
        dst: Rect<Subpixels>,
        src: Rect<Pixels>,
    ) {
        match layer {
            RenderLayer::Player => self.player_batch.draw(sprite, dst, src, false),
            RenderLayer::Hud => self.hud_batch.draw(sprite, dst, src, false),
        }
    }

    pub fn draw_reversed(
        &mut self,
        sprite: Sprite,
        layer: RenderLayer,
        dst: Rect<Subpixels>,
        src: Rect<Pixels>,
    ) {
        match layer {
            RenderLayer::Player => self.player_batch.draw(sprite, dst, src, true),
            RenderLayer::Hud => self.hud_batch.draw(sprite, dst, src, true),
        }
    }

    pub fn fill_rect(&mut self, rect: Rect<Subpixels>, layer: RenderLayer, color: Color) {
        match layer {
            RenderLayer::Player => self.player_batch.fill_rect(rect, color),
            RenderLayer::Hud => self.hud_batch.fill_rect(rect, color),
        }
    }

    pub fn clear(&mut self) {
        self.player_batch.entries.clear();
        self.hud_batch.entries.clear();
        self.player_batch.clear_color = Color {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        };
        self.hud_batch.clear_color = Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }

    pub fn add_light(&mut self, position: Point<Subpixels>, radius: Subpixels) {
        if self.lights.len() >= MAX_LIGHTS {
            warn!("too many lights set");
            return;
        }
        self.lights.push(Light { position, radius });
    }
}
