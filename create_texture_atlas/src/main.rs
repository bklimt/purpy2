use std::fs::{self};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;
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
}

#[derive(Debug)]
struct ImageFile {
    path: PathBuf,
    img: DynamicImage,
}

#[derive(Debug)]
struct AtlasEntry {
    path: PathBuf,
    img: DynamicImage,
    location: (u32, u32, u32, u32),
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

        images.push(img);
    }

    if images.is_empty() {
        bail!("no images found");
    }

    // Take the tallest images first.
    images.sort_by_key(|img| img.img.height());
    images.reverse();

    let max_width = images
        .iter()
        .max_by_key(|img| img.img.width())
        .unwrap()
        .img
        .width();
    let mut x: u32 = 0u32;
    let mut y = 0u32;
    let mut row_height = 0u32;
    let mut index = Vec::new();

    while !images.is_empty() {
        let next = images
            .iter()
            .position(|img| x + img.img.width() <= max_width);
        if let Some(next) = next {
            // Put next on the current row.
            let img = images.remove(next);
            let ImageFile { path, img } = img;
            let location = (x, y, img.width(), img.height());
            let entry = AtlasEntry {
                path,
                img,
                location,
            };

            row_height = row_height.max(entry.img.height());
            x += entry.img.width();

            index.push(entry);
        } else {
            // Nothing else will fit on this row. Move to the next one.
            y += row_height;
            x = 0;
            row_height = 0;
        }
    }

    y += row_height;
    let height = y;
    let width = max_width;
    let texture_atlas_image = ImageBuffer::new(width, height);
    let mut texture_atlas_image = DynamicImage::ImageRgba8(texture_atlas_image);

    for entry in index.iter() {
        texture_atlas_image.copy_from(&entry.img, entry.location.0, entry.location.1)?;
    }

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
