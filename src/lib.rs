mod args;
mod constants;
mod door;
mod font;
mod geometry;
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
mod warp;
mod wgpu;

pub use args::Args;
pub use wgpu::winitmain::run as winit_main;

#[cfg(not(target_arch = "wasm32"))]
mod sdl;
#[cfg(not(target_arch = "wasm32"))]
pub use sdl::sdlmain::sdl_main;
#[cfg(not(target_arch = "wasm32"))]
pub use wgpu::wgpumain::run as wgpu_main;
