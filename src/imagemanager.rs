use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use log::info;

use crate::filemanager::FileManager;
use crate::font::Font;
use crate::geometry::{Pixels, Rect};
use crate::renderer::Renderer;
use crate::sprite::{Animation, Sprite, SpriteSheet};
use crate::utils::normalize_path;

pub trait ImageLoader {
    fn load_sprite(&mut self, path: &Path) -> Result<Sprite>;

    fn load_spritesheet(
        &mut self,
        path: &Path,
        sprite_width: Pixels,
        sprite_height: Pixels,
    ) -> Result<SpriteSheet>;

    fn load_animation(
        &mut self,
        path: &Path,
        sprite_width: Pixels,
        sprite_height: Pixels,
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

    pub fn load_font(&mut self, files: &FileManager) -> Result<Font> {
        Font::new(Path::new("assets/8bitfont.tsx"), files, self)
    }

    pub fn renderer(&self) -> &T {
        &self.renderer
    }

    pub fn renderer_mut(&mut self) -> &mut T {
        &mut self.renderer
    }

    pub fn load_texture_atlas(
        &mut self,
        files: &FileManager,
        image_path: &Path,
        index_path: &Path,
    ) -> Result<()> {
        let base_path = index_path.parent().unwrap();
        let base_sprite = self.load_sprite(image_path)?;

        let index_bytes = files
            .read(index_path)
            .map_err(|e| anyhow!("unable to open texture atlas index {:?}: {}", index_path, e))?;
        let mut r = BufReader::new(&index_bytes[..]);
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
            let x = Pixels::new(parts[0].parse()?);
            let y = Pixels::new(parts[1].parse()?);
            let w = Pixels::new(parts[2].parse()?);
            let h = Pixels::new(parts[3].parse()?);
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
            info!("sprite already exists at {:?}", existing.area);
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
        sprite_width: Pixels,
        sprite_height: Pixels,
    ) -> Result<SpriteSheet> {
        let sprite = self.load_sprite(path)?;
        SpriteSheet::new(sprite, sprite_width, sprite_height)
    }

    fn load_animation(
        &mut self,
        path: &Path,
        sprite_width: Pixels,
        sprite_height: Pixels,
    ) -> Result<Animation> {
        let sprite = self.load_sprite(path)?;
        Animation::new(sprite, sprite_width, sprite_height)
    }
}
