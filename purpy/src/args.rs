use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub fullscreen: bool,

    #[arg(long)]
    pub record: Option<String>,

    #[arg(long)]
    pub playback: Option<String>,

    #[arg(long)]
    pub speed_test: bool,
}
