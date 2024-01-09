mod constants;
mod font;
mod imagemanager;
mod inputmanager;
mod level;
mod platform;
mod player;
mod properties;
mod slope;
mod smallintset;
mod soundmanager;
mod sprite;
mod switchstate;
mod tilemap;
mod tileset;
mod utils;

use std::{fs, path::Path, time::Duration};

use anyhow::Result;
use clap::Parser;
use imagemanager::ImageManager;
use inputmanager::InputManager;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sprite::{AnimationStateMachine, SpriteBatch};
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
    let space = image_manager.load_sprite(Path::new("../purpy/assets/space.png"))?;

    let mut animation =
        image_manager.load_animation(Path::new("../purpy/assets/sprites/skelly2.png"), 24, 24)?;

    let player_sprite =
        image_manager.load_spritesheet(Path::new("../purpy/assets/sprites/skelly2.png"), 24, 24)?;
    let animation_state_machine = AnimationStateMachine::new(&fs::read_to_string(
        "../purpy/assets/sprites/skelly2_states.txt",
    )?)?;
    let mut current_frame = 0;

    canvas.set_logical_size(space.width(), space.height())?;
    canvas.set_draw_color(Color::RGB(40, 40, 40));
    canvas.clear();
    canvas.present();

    let mut input_manager = InputManager::new();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        canvas.set_draw_color(Color::RGB(40, 40, 40));
        canvas.clear();

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

        let mut batch = SpriteBatch::new(&mut canvas);
        batch.draw(&space, None, None);

        let dest = Rect {
            x: (space.width() / 2 - 24) as i32,
            y: (space.height() / 2 - 24) as i32,
            w: 24,
            h: 24,
        };
        animation.update();
        animation.blit(&mut batch, dest, false);

        let dest = Rect {
            x: (space.width() / 2) as i32,
            y: (space.height() / 2) as i32,
            w: 24,
            h: 24,
        };

        current_frame = animation_state_machine.next_frame(current_frame, "RUNNING")?;
        player_sprite.blit(&mut batch, dest, current_frame, 0, false);

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
