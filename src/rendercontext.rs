use std::mem;
use std::rc::Rc;

use anyhow::{anyhow, Result};
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::constants::SUBPIXELS;
use crate::sprite::Sprite;
use crate::utils::{Color, Rect};

enum SpriteBatchEntry<'a> {
    Sprite {
        sprite: Rc<Sprite<'a>>,
        source: Rect,
        destination: Rect,
        reversed: bool,
    },
    FillRect {
        destination: Rect,
        color: Color,
    },
    Clear {
        color: Color,
    },
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

    pub fn draw(&mut self, sprite: &Rc<Sprite<'a>>, dst: Rect, src: Rect, reversed: bool) {
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

    pub fn clear(&mut self, color: Color) {
        self.entries.push(SpriteBatchEntry::Clear { color })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RenderLayer {
    Player,
    Hud,
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

    fn logical_area(&self) -> Rect {
        Rect {
            x: 0,
            y: 0,
            w: self.width as i32,
            h: self.height as i32,
        }
    }

    pub fn logical_area_in_subpixels(&self) -> Rect {
        Rect {
            x: 0,
            y: 0,
            w: self.width as i32 * SUBPIXELS,
            h: self.height as i32 * SUBPIXELS,
        }
    }

    pub fn draw(&mut self, sprite: &Rc<Sprite<'a>>, layer: RenderLayer, dst: Rect, src: Rect) {
        self.player_batch.draw(sprite, dst, src, false)
    }

    pub fn draw_reversed(
        &mut self,
        sprite: &Rc<Sprite<'a>>,
        layer: RenderLayer,
        dst: Rect,
        src: Rect,
    ) {
        self.player_batch.draw(sprite, dst, src, true)
    }

    pub fn fill_rect(&mut self, rect: Rect, layer: RenderLayer, color: Color) {
        self.player_batch.fill_rect(rect, color);
    }

    pub fn clear(&mut self) {
        self.player_batch.clear(Color {
            r: 80,
            g: 80,
            b: 80,
            a: 255,
        });
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) -> Result<()> {
        let texture_creator = canvas.texture_creator();
        let mut player_texture =
            texture_creator.create_texture_target(self.pixel_format, self.width, self.height)?;

        canvas.with_texture_canvas(&mut player_texture, |canvas| {
            for entry in self.player_batch.entries.iter() {
                match entry {
                    SpriteBatchEntry::Sprite {
                        sprite,
                        source,
                        destination,
                        reversed,
                    } => canvas
                        .copy_ex(
                            &sprite.texture,
                            *source,
                            *destination,
                            0.0,
                            None,
                            *reversed,
                            false,
                        )
                        .map_err(|s| anyhow!("unable to copy sprite: {}", s))
                        .expect("must succeed"),
                    SpriteBatchEntry::FillRect { destination, color } => {
                        canvas.set_draw_color(*color);
                        canvas
                            .fill_rect(*destination)
                            .map_err(|s| anyhow!("unable to fill rect: {}", s))
                            .expect("must succeed");
                    }
                    SpriteBatchEntry::Clear { color } => {
                        canvas.set_draw_color(*color);
                        canvas.clear();
                    }
                }
            }
        })?;

        let src = Rect {
            x: 0,
            y: 0,
            w: self.width as i32,
            h: self.height as i32,
        };
        let dst = Rect {
            x: 0,
            y: 0,
            w: self.width as i32 * SUBPIXELS,
            h: self.height as i32 * SUBPIXELS,
        };

        canvas
            .copy(&player_texture, None, None)
            .map_err(|s| anyhow!("unable to copy player texture to window canvas: {}", s))?;

        Ok(())
    }
}
