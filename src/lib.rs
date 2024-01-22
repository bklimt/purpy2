mod constants;
mod door;
mod font;
mod imagemanager;
mod inputmanager;
mod killscreen;
mod level;
mod levelselect;
mod platform;
mod player;
mod properties;
mod rendercontext;
mod renderer;
mod scene;
mod sdlmain;
mod sdlrenderer;
mod slope;
mod smallintmap;
mod smallintset;
mod soundmanager;
mod sprite;
mod stagemanager;
mod star;
mod switchstate;
mod tilemap;
mod tileset;
mod utils;
mod wgpu;

use clap::Parser;

pub use sdlmain::sdl_main;
pub use wgpu::wgpumain::run as wgpu_main;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(long)]
    pub fullscreen: bool,

    #[arg(long)]
    pub winit: bool,
}