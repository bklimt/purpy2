mod constants;
mod image_manager;
mod properties;
mod slope;
mod sprite;
mod tilemap;
mod tileset;
mod utils;

use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use image_manager::ImageManager;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sprite::SpriteBatch;
use utils::Rect;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

fn run_game(_args: Args) -> Result<()> {
    let sdl_context = sdl2::init().expect("failed to init SDL");
    let video_subsystem = sdl_context.video().expect("failed to get video context");

    // We create a window.
    let window = video_subsystem
        .window("sdl2 demo", 800, 600)
        .resizable()
        .fullscreen_desktop()
        .build()
        .expect("failed to build window");

    // We get the canvas from which we can get the `TextureCreator`.
    let mut canvas: Canvas<Window> = window
        .into_canvas()
        .build()
        .expect("failed to build window's canvas");
    let texture_creator = canvas.texture_creator();

    let image_manager = ImageManager::new(&texture_creator);
    let sprite = image_manager.load_sprite("../purpy/assets/space.png")?;
    let mut animation =
        image_manager.load_animation("../purpy/assets/sprites/skelly2.png", 24, 24)?;

    canvas.set_logical_size(sprite.width(), sprite.height())?;
    canvas.set_draw_color(Color::RGB(40, 40, 40));
    canvas.clear();
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.set_draw_color(Color::RGB(40, 40, 40));
        canvas.clear();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        let mut batch = SpriteBatch::new(&mut canvas);
        batch.draw(&sprite, None, None);
        let dest = Rect {
            x: (sprite.width() / 2 - 12) as i32,
            y: (sprite.height() / 2 - 12) as i32,
            w: 24,
            h: 24,
        };
        animation.update();
        animation.blit(&mut batch, dest, false);

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
    }

    Ok(())
}

fn main() {
    let args = Args::parse();
    match run_game(args) {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    }
}
