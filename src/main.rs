mod constants;
mod door;
mod font;
mod imagemanager;
mod inputmanager;
mod level;
mod levelselect;
mod platform;
mod player;
mod properties;
mod rendercontext;
mod scene;
mod slope;
mod smallintset;
mod soundmanager;
mod sprite;
mod stagemanager;
mod star;
mod switchstate;
mod tilemap;
mod tileset;
mod utils;

use std::{fs, path::Path, time::Duration};

use anyhow::Result;
use clap::Parser;
use constants::{RENDER_HEIGHT, RENDER_WIDTH};
use imagemanager::ImageManager;
use inputmanager::InputManager;
use rendercontext::{RenderContext, RenderLayer};
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use soundmanager::SoundManager;
use sprite::AnimationStateMachine;
use stagemanager::StageManager;
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

    let image_manager = ImageManager::new(&texture_creator)?;
    let mut frame = 0;

    canvas.set_logical_size(RENDER_WIDTH, RENDER_HEIGHT)?;
    canvas.set_draw_color(Color::RGB(40, 40, 40));
    canvas.clear();
    canvas.present();

    let mut input_manager = InputManager::new();
    let mut stage_manager = StageManager::new(&image_manager)?;
    let mut sound_manager = SoundManager {};

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.set_draw_color(Color::RGB(40, 40, 40));
        canvas.clear();

        let (width, height) = canvas.logical_size();
        let pixel_format = canvas.default_pixel_format();
        let mut context = RenderContext::new(width, height, pixel_format, frame)?;

        for event in event_pump.poll_iter() {
            input_manager.handle_event(&event);
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        input_manager.update();

        if input_manager.is_on(inputmanager::BinaryInput::Cancel) {
            break 'running;
        }

        if !stage_manager.update(&input_manager, &image_manager, &sound_manager)? {
            break 'running;
        }

        stage_manager.draw(&mut context, &image_manager);

        context.render(&mut canvas)?;
        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        frame += 1;
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
