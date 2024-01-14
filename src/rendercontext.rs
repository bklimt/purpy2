use std::mem;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};

use crate::sprite::Sprite;
use crate::utils::{Color, Rect};

struct SpriteBatchEntry<'a> {
    sprite: Rc<Sprite<'a>>,
    source: Rect,
    destination: Rect,
}

pub struct SpriteBatch<'a> {
    entries: Vec<SpriteBatchEntry<'a>>,
}

impl<'a> SpriteBatch<'a> {
    pub fn new<'b>() -> SpriteBatch<'b> {
        SpriteBatch {
            entries: Vec::new(),
        }
    }

    pub fn draw(&mut self, sprite: &Rc<Sprite<'a>>, dst: Rect, src: Rect) {
        self.entries.push(SpriteBatchEntry {
            sprite: sprite.clone(),
            source: src,
            destination: dst,
        });
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        /*
        canvas.with_texture_canvas(&mut self.texture, |canvas| {
            canvas.set_draw_color(color);
            canvas
                .draw_rect(rect.into())
                .map_err(|s| anyhow!("unable to copy sprite: {}", s))
                .expect("must succeed");
        });
        */
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RenderLayer {
    Player,
}

pub struct RenderContext<'a> {
    player_batch: SpriteBatch<'a>,
    width: u32,
    height: u32,
    pixel_format: PixelFormatEnum,
    pub frame: u32,
}

impl<'a> RenderContext<'a> {
    pub fn new<'b>(
        width: u32,
        height: u32,
        pixel_format: PixelFormatEnum,
        frame: u32,
    ) -> Result<RenderContext<'b>> {
        let player_batch = SpriteBatch::new();
        Ok(RenderContext {
            player_batch,
            width,
            height,
            pixel_format,
            frame,
        })
    }

    pub fn draw(&mut self, sprite: &Rc<Sprite<'a>>, layer: RenderLayer, dst: Rect, src: Rect) {
        self.player_batch.draw(sprite, dst, src)
    }

    pub fn fill_rect(&mut self, rect: Rect, layer: RenderLayer, color: Color) {
        self.player_batch.fill_rect(rect, color);
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<()> {
        let texture_creator = canvas.texture_creator();
        let mut player_texture =
            texture_creator.create_texture_target(self.pixel_format, self.width, self.height)?;

        canvas.with_texture_canvas(&mut player_texture, |canvas| {
            for entry in self.player_batch.entries.iter() {
                canvas
                    .copy(&entry.sprite.texture, entry.source, entry.destination)
                    .map_err(|s| anyhow!("unable to copy sprite: {}", s))
                    .expect("must succeed");
            }
        })?;

        canvas
            .copy(&player_texture, None, None)
            .map_err(|s| anyhow!("unable to copy player texture to window canvas: {}", s))?;

        Ok(())
    }
}
