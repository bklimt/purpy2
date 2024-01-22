use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use log::info;

use crate::font::Font;
use crate::renderer::Renderer;
use crate::sprite::{Animation, Sprite, SpriteSheet};
use crate::utils::{normalize_path, Rect};

pub trait ImageLoader {
    fn load_sprite(&mut self, path: &Path) -> Result<Sprite>;

    fn load_spritesheet(
        &mut self,
        path: &Path,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Result<SpriteSheet>;

    fn load_animation(
        &mut self,
        path: &Path,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Result<Animation>;
}

pub struct ImageManager<T: Renderer> {
    path_to_sprite: HashMap<PathBuf, Sprite>,
    renderer: T,
    locked: bool, // once it's locked, it can't read more images
}

impl<T> ImageManager<T>
where
    T: Renderer,
{
    pub fn new(renderer: T) -> Result<Self> {
        let path_to_sprite = HashMap::new();
        let locked = false;
        Ok(ImageManager {
            path_to_sprite,
            renderer,
            locked,
        })
    }

    pub fn load_font(&mut self) -> Result<Font> {
        Font::new(Path::new("assets/8bitfont.tsx"), self)
    }

    pub fn renderer(&self) -> &T {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut T {
        &mut self.renderer
    }

    pub fn load_texture_atlas(&mut self, image_path: &Path, index_path: &Path) -> Result<()> {
        let base_path = index_path.parent().unwrap();
        let base_sprite = self.load_sprite(image_path)?;

        let file = File::open(index_path)
            .with_context(|| format!("unable to open texture atlas index {:?}", index_path))?;
        let mut r = BufReader::new(file);
        loop {
            let mut line = String::new();
            let n = r.read_line(&mut line).unwrap();
            let line = line.trim();

            if line == "" {
                if n == 0 {
                    break;
                }
                continue;
            }

            let parts: Vec<&str> = line.split(",").collect();
            if parts.len() != 5 {
                bail!("invalid texture atlas index entry: {}", line);
            }
            let x = parts[0].parse()?;
            let y = parts[1].parse()?;
            let w = parts[2].parse()?;
            let h = parts[3].parse()?;
            let area = Rect { x, y, w, h };
            let sprite = base_sprite.subview(area);

            let path = base_path.join(parts[4]);
            info!("loaded image from texture atlas: {:?}", path);

            self.path_to_sprite.insert(path, sprite);
        }

        self.locked = true;
        Ok(())
    }
}

impl<T> ImageLoader for ImageManager<T>
where
    T: Renderer,
{
    fn load_sprite(&mut self, path: &Path) -> Result<Sprite> {
        info!("loading sprite from path: {:?}", path);
        let path = normalize_path(path)?;
        info!("loading sprite from normalized path: {:?}", path);
        if let Some(existing) = self.path_to_sprite.get(&path) {
            info!("sprite already exists at {}, {}", existing.x, existing.y);
            return Ok(*existing);
        }
        if self.locked {
            bail!("image manager is locked while loading: {:?}", path);
        }
        let sprite = self.renderer.load_sprite(&path)?;
        self.path_to_sprite.insert(path.to_owned(), sprite);
        Ok(sprite)
    }

    fn load_spritesheet(
        &mut self,
        path: &Path,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Result<SpriteSheet> {
        let sprite = self.load_sprite(path)?;
        SpriteSheet::new(sprite, sprite_width, sprite_height)
    }

    fn load_animation(
        &mut self,
        path: &Path,
        sprite_width: u32,
        sprite_height: u32,
    ) -> Result<Animation> {
        let sprite = self.load_sprite(path)?;
        Animation::new(sprite, sprite_width, sprite_height)
    }
}
