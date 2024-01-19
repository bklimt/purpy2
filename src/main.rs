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

use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use constants::{FRAME_RATE, RENDER_HEIGHT, RENDER_WIDTH, WINDOW_HEIGHT, WINDOW_WIDTH};
use imagemanager::ImageManager;
use inputmanager::InputManager;
use rendercontext::RenderContext;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use soundmanager::SoundManager;
use stagemanager::StageManager;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    fullscreen: bool,
}

fn run_game(args: Args) -> Result<()> {
    env_logger::init();

    let sdl_context = sdl2::init().expect("failed to init SDL");
    let video_subsystem = sdl_context.video().expect("failed to get video context");
    let audio_subsystem = sdl_context.audio().expect("failed to get audio context");

    // We create a window.
    let title = "purpy2";
    let mut window = video_subsystem.window(title, WINDOW_WIDTH, WINDOW_HEIGHT);
    if args.fullscreen {
        window.fullscreen_desktop();
    }
    let window = window.resizable().build().expect("failed to build window");

    // We get the canvas from which we can get the `TextureCreator`.
    let mut canvas: Canvas<Window> = window
        .into_canvas()
        .build()
        .expect("failed to build window's canvas");
    let texture_creator = canvas.texture_creator();

    let image_manager = ImageManager::new(&texture_creator)?;
    let mut frame = 0;

    canvas.set_logical_size(RENDER_WIDTH, RENDER_HEIGHT)?;
    canvas.set_draw_color(Color::RGB(40, 40, 40));
    canvas.clear();
    canvas.present();

    let mut input_manager = InputManager::new()?;
    let mut stage_manager = StageManager::new(&image_manager)?;

    let mut sound_manager = SoundManager::new(&audio_subsystem)?;

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        let start_time = Instant::now();

        canvas.set_draw_color(Color::RGB(40, 40, 40));
        canvas.clear();

        let (width, height) = canvas.logical_size();
        let pixel_format = canvas.default_pixel_format();
        let mut context = RenderContext::new(width, height, pixel_format, frame)?;

        for event in event_pump.poll_iter() {
            input_manager.handle_sdl_event(&event);
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        let input_snapshot = input_manager.update();

        if !stage_manager.update(&input_snapshot, &image_manager, &mut sound_manager)? {
            break 'running;
        }

        context.clear();
        stage_manager.draw(&mut context, &image_manager);
        context.render(&mut canvas)?;
        canvas.present();

        frame += 1;
        let target_duration = Duration::new(0, 1_000_000_000u32 / FRAME_RATE);
        let actual_duration = start_time.elapsed();
        if actual_duration > target_duration {
            continue;
        }
        let remaining = target_duration - actual_duration;
        ::std::thread::sleep(remaining);
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
