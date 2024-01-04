mod constants;
mod properties;
mod slope;
mod sprite;
mod tilemap;
mod tileset;
mod utils;

use crate::tileset::TileSetXml;
use anyhow::Result;
use clap::Parser;
use std::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    path: String,
}

fn test_xml(path: &str) -> Result<()> {
    let text = fs::read_to_string(path)?;
    let xml = quick_xml::de::from_str::<TileSetXml>(&text)?;
    println!("{:?}", xml);
    Ok(())
}

fn main() {
    let args = Args::parse();
    match test_xml(&args.path) {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    }
}
