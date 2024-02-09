use std::fs::{self, File};
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use flate2::{Compression, GzBuilder};
use glob::glob;
use log::{debug, error};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    manifest: String,

    #[arg(long)]
    output: String,
}

fn process(args: &Args) -> Result<()> {
    let manifest_path = Path::new(&args.manifest);

    let parent = manifest_path
        .parent()
        .with_context(|| anyhow!("not a file: {:?}", manifest_path))?;
    let parent = parent
        .to_str()
        .with_context(|| anyhow!("cannot be represented as a string: {:?}", parent))?;

    let output_path = Path::new(&args.output);
    let output = File::options()
        .write(true)
        .create_new(true)
        .open(output_path)
        .map_err(|e| anyhow!("unable to create output file at {:?}: {}", output_path, e))?;

    let mut gz_builder = GzBuilder::new()
        .filename("assets.tar")
        .write(output, Compression::default());

    {
        let mut tar_builder = tar::Builder::new(&mut gz_builder);

        let manifest = fs::read_to_string(manifest_path)?;
        for line in manifest.lines() {
            let line = line.trim();
            if line.len() == 0 {
                continue;
            }

            let line = format!("{}/{}", parent, line);

            debug!("processing glob {}", &line);
            let expanded = glob(&line).with_context(|| format!("invalid glob: {}", &line))?;
            for path in expanded {
                let path = path.map_err(|e| anyhow!("unable to expand {}: {}", &line, e))?;
                println!("{:?}", &path);

                tar_builder
                    .append_path(&path)
                    .map_err(|e| anyhow!("unable to add {:?} to {:?}: {}", path, output_path, e))?;
            }
        }

        tar_builder
            .finish()
            .map_err(|e| anyhow!("unable to finish tar file: {}", e))?;
    }

    gz_builder
        .finish()
        .map_err(|e| anyhow!("unable to finish gz file: {}", e))?;

    Ok(())
}

fn main() {
    env_logger::init();
    let args = Args::parse();
    if let Err(e) = process(&args) {
        error!("error: {}", e);
    }
}
