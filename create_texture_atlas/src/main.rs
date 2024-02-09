use std::fs::{self};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, ValueEnum};
use image::{DynamicImage, GenericImage, ImageBuffer, ImageFormat};
use log::error;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    texture_list: String,

    #[arg(long)]
    texture_atlas_image: String,

    #[arg(long)]
    texture_atlas_index: String,

    #[arg(long)]
    score_func: ScoreFunc,

    #[arg(long)]
    try_all_pairs: bool,
}

#[derive(Debug)]
struct ImageFile {
    path: PathBuf,
    img: DynamicImage,
}

#[derive(Debug)]
struct AtlasEntry {
    location: (u32, u32, u32, u32),
    path: PathBuf,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
enum ScoreFunc {
    Multiply,
    Harmonic,
    Average,
}

impl ScoreFunc {
    fn apply(&self, a: u32, b: u32) -> u32 {
        match self {
            ScoreFunc::Harmonic => (2 * a * b) / (a + b),
            ScoreFunc::Multiply => a * b,
            ScoreFunc::Average => (a + b) / 2,
        }
    }
}

fn horizontal_score(img1: &MergedImages, img2: &MergedImages, f: ScoreFunc) -> u32 {
    let height = img1.height().max(img2.height());
    let width = img1.width() + img2.width();
    f.apply(width, height)
}

fn vertical_score(img1: &MergedImages, img2: &MergedImages, f: ScoreFunc) -> u32 {
    let height = img1.height() + img2.height();
    let width = img1.width().max(img2.width());
    f.apply(width, height)
}

#[derive(Debug)]
enum MergedImages {
    Vertical {
        top: Box<MergedImages>,
        bottom: Box<MergedImages>,
    },
    Horizontal {
        left: Box<MergedImages>,
        right: Box<MergedImages>,
    },
    Single(ImageFile),
}

impl MergedImages {
    fn width(&self) -> u32 {
        match self {
            MergedImages::Vertical { top, bottom } => top.width().max(bottom.width()),
            MergedImages::Horizontal { left, right } => left.width() + right.width(),
            MergedImages::Single(img) => img.img.width(),
        }
    }

    fn height(&self) -> u32 {
        match self {
            MergedImages::Vertical { top, bottom } => top.height() + bottom.height(),
            MergedImages::Horizontal { left, right } => left.height().max(right.height()),
            MergedImages::Single(img) => img.img.height(),
        }
    }

    fn dimensions(&self) -> (u32, u32) {
        (self.width(), self.height())
    }

    fn score(&self, f: ScoreFunc) -> u32 {
        f.apply(self.width(), self.height())
    }

    fn render_into(
        &self,
        dest: &mut DynamicImage,
        x: u32,
        y: u32,
        index: &mut Vec<AtlasEntry>,
    ) -> Result<()> {
        match self {
            MergedImages::Vertical { top, bottom } => {
                top.render_into(dest, x, y, index)?;
                bottom.render_into(dest, x, y + top.height(), index)?;
            }
            MergedImages::Horizontal { left, right } => {
                left.render_into(dest, x, y, index)?;
                right.render_into(dest, x + left.width(), y, index)?;
            }
            MergedImages::Single(img) => {
                let w = img.img.width();
                let h = img.img.height();
                index.push(AtlasEntry {
                    location: (x, y, w, h),
                    path: img.path.to_owned(),
                });
                dest.copy_from(&img.img, x, y)?;
            }
        }
        Ok(())
    }

    fn render(&self, index: &mut Vec<AtlasEntry>) -> Result<DynamicImage> {
        let (width, height) = self.dimensions();
        let img = ImageBuffer::new(width, height);
        let mut img = DynamicImage::ImageRgba8(img);
        self.render_into(&mut img, 0, 0, index)?;
        Ok(img)
    }
}

fn process(args: &Args) -> Result<()> {
    let path = Path::new(&args.texture_list);
    let root = path
        .parent()
        .context("parent of file must be a directory")?;
    let data = fs::read_to_string(path)?;
    let lines: std::str::Split<'_, char> = data.split('\n');
    let mut images = Vec::new();
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let relative_path = Path::new(line).to_owned();
        let absolute_path = root.join(&relative_path);

        let img = image::open(&absolute_path)?;
        let img = ImageFile {
            path: relative_path,
            img,
        };
        let img = MergedImages::Single(img);

        images.push(img);
    }

    if images.is_empty() {
        bail!("no images found");
    }

    let f = args.score_func;

    while images.len() > 1 {
        // TODO: Sort options by (wasted area/percent, total area, squareness abs(w-h)).
        // TODO: Fill in the wasted space for each with the largest thing that fits it.

        let mut best_pair = (0, 0);
        if args.try_all_pairs {
            let mut min_score = None;
            for i in 0..images.len() {
                for j in (i + 1)..images.len() {
                    let vertical_score = vertical_score(&images[i], &images[j], f);
                    let horizontal_score = horizontal_score(&images[i], &images[j], f);
                    let score = vertical_score.min(horizontal_score);
                    if match min_score {
                        Some(best) => score < best,
                        None => true,
                    } {
                        best_pair = (i, j);
                        min_score = Some(score);
                    }
                }
            }
        } else {
            images.sort_by_cached_key(|img| img.score(f));
            best_pair = (0, 1);
        }

        let img2 = images.swap_remove(best_pair.1);
        let img1 = images.swap_remove(best_pair.0);
        let vertical_area = vertical_score(&img1, &img2, f);
        let horizontal_area = horizontal_score(&img1, &img2, f);
        let new_img = if vertical_area < horizontal_area {
            MergedImages::Vertical {
                top: Box::new(img2),
                bottom: Box::new(img1),
            }
        } else {
            MergedImages::Horizontal {
                left: Box::new(img2),
                right: Box::new(img1),
            }
        };
        images.push(new_img);
    }

    let mut index = Vec::new();
    let texture_atlas_image = images[0].render(&mut index)?;
    let texture_atlas_image_path = Path::new(&args.texture_atlas_image);
    texture_atlas_image.save_with_format(texture_atlas_image_path, ImageFormat::Png)?;

    let texture_atlas_index_path = Path::new(&args.texture_atlas_index);
    let mut index_lines: Vec<String> = index
        .iter()
        .map(|entry| {
            format!(
                "{},{},{},{},{}",
                entry.location.0,
                entry.location.1,
                entry.location.2,
                entry.location.3,
                entry.path.to_string_lossy()
            )
        })
        .collect();
    index_lines.push("".to_owned());
    let index_text = index_lines.join("\n");
    fs::write(texture_atlas_index_path, index_text)?;

    println!(
        "Total size: {}",
        texture_atlas_image.width() * texture_atlas_image.height()
    );

    Ok(())
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    if let Err(e) = process(&args) {
        error!("error: {}", e);
    }
}
