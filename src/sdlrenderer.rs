use std::path::Path;

use anyhow::{anyhow, Result};

use sdl2::image::LoadSurface;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

use crate::rendercontext::{RenderContext, SpriteBatchEntry};
use crate::renderer::Renderer;
use crate::sprite::Sprite;
use crate::utils::Rect;

struct SpriteInternal<'a> {
    _surface: Surface<'a>,
    texture: Texture<'a>,
}

pub struct SdlRenderer<'a> {
    sprites: Vec<SpriteInternal<'a>>,
    texture_creator: &'a TextureCreator<WindowContext>,
}

impl<'a> SdlRenderer<'a> {
    pub fn new(texture_creator: &'a TextureCreator<WindowContext>) -> Self {
        let sprites = Vec::new();
        SdlRenderer {
            sprites,
            texture_creator,
        }
    }

    pub fn render(&self, canvas: &mut Canvas<Window>, context: &RenderContext) -> Result<()> {
        let pixel_format = canvas.default_pixel_format();

        let texture_creator = canvas.texture_creator();
        let mut player_texture =
            texture_creator.create_texture_target(pixel_format, context.width, context.height)?;

        canvas.set_blend_mode(BlendMode::Blend);
        canvas.with_texture_canvas(&mut player_texture, |canvas| {
            canvas.set_draw_color(context.player_batch.clear_color);
            canvas.clear();
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

                        let source = Rect {
                            x: sprite.x as i32 + source.x,
                            y: sprite.y as i32 + source.y,
                            w: source.w,
                            h: source.h,
                        };

                        canvas
                            .copy_ex(
                                &sprite_internal.texture,
                                source,
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
                }
            }
        })?;

        canvas
            .copy(&player_texture, None, None)
            .map_err(|s| anyhow!("unable to copy player texture to window canvas: {}", s))?;

        Ok(())
    }
}

impl Renderer for SdlRenderer<'_> {
    fn load_sprite(&mut self, path: &Path) -> Result<Sprite> {
        let surface = Surface::from_file(path)
            .map_err(|s: String| anyhow!("unable to load {:?}: {}", path, s))?;

        let width = surface.width();
        let height = surface.height();

        let texture = surface.as_texture(self.texture_creator)?;
        let sprite_internal = SpriteInternal {
            _surface: surface,
            texture,
        };

        let id = self.sprites.len();
        self.sprites.push(sprite_internal);

        let x = 0;
        let y = 0;

        Ok(Sprite {
            id,
            x,
            y,
            width,
            height,
        })
    }
}
