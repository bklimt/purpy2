mod constants;
mod slope;
mod tilemap;
mod utils;

use crate::tilemap::TileMapXml;
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
    let xml = quick_xml::de::from_str::<TileMapXml>(&text)?;
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
