use std::path::Path;
use std::{collections::HashMap, path::PathBuf};

use anyhow::{anyhow, Context, Result};

use sdl2::image::LoadSurface;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

use crate::rendercontext::{RenderContext, SpriteBatchEntry};
use crate::renderer::Renderer;
use crate::sprite::Sprite;

struct SpriteInternal<'a> {
    surface: Surface<'a>,
    texture: Texture<'a>,
}

impl<'a> SpriteInternal<'a> {
    fn new<'b, 'c, T>(
        surface: Surface<'b>,
        texture_creator: &'c TextureCreator<T>,
    ) -> Result<SpriteInternal<'b>>
    where
        'c: 'b,
    {
        let texture = surface.as_texture(texture_creator)?;
        Ok(SpriteInternal { surface, texture })
    }
}

pub struct SdlRenderer<'a> {
    sprites: Vec<SpriteInternal<'a>>,
    canvas: &'a mut Canvas<Window>,
    texture_creator: &'a TextureCreator<WindowContext>,
}

impl<'a> SdlRenderer<'a> {
    pub fn new(canvas: &'a mut Canvas<Window>) -> Self {
        let texture_creator = &canvas.texture_creator();
        let sprites = Vec::new();
        SdlRenderer {
            sprites,
            canvas,
            texture_creator,
        }
    }
}

impl Renderer for SdlRenderer<'_> {
    fn load_sprite(&mut self, path: &Path) -> Result<Sprite> {
        let surface = Surface::from_file(path)
            .map_err(|s: String| anyhow!("unable to load {:?}: {}", path, s))?;

        let width = surface.width();
        let height = surface.height();

        let texture = surface.as_texture(self.texture_creator)?;
        let sprite_internal = SpriteInternal { surface, texture };

        let id = self.sprites.len();
        self.sprites.push(sprite_internal);

        Ok(Sprite { id, width, height })
    }

    fn render(&self, context: &RenderContext) -> Result<()> {
        let canvas = self.canvas;

        let pixel_format = canvas.default_pixel_format();

        let texture_creator = canvas.texture_creator();
        let mut player_texture =
            texture_creator.create_texture_target(pixel_format, context.width, context.height)?;

        canvas.set_blend_mode(BlendMode::Blend);
        canvas.with_texture_canvas(&mut player_texture, |canvas| {
            for entry in context.player_batch.entries.iter() {
                match entry {
                    SpriteBatchEntry::Sprite {
                        sprite,
                        source,
                        destination,
                        reversed,
                    } => {
                        let sprite_internal = self
                            .sprites
                            .get(sprite.id)
                            .expect(format!("invalid sprite: {:?}", sprite).as_str());
                        canvas
                            .copy_ex(
                                &sprite_internal.texture,
                                *source,
                                *destination,
                                0.0,
                                None,
                                *reversed,
                                false,
                            )
                            .map_err(|s| anyhow!("unable to copy sprite: {}", s))
                            .expect("must succeed");
                    }
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

        canvas
            .copy(&player_texture, None, None)
            .map_err(|s| anyhow!("unable to copy player texture to window canvas: {}", s))?;

        Ok(())
    }
}
