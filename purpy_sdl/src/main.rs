use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use clap::Parser;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

use purpy::{
    FileManager, ImageManager, InputManager, RecordOption, RenderContext, SdlRenderer,
    SoundManager, StageManager, FRAME_RATE, RENDER_HEIGHT, RENDER_WIDTH,
};

pub const WINDOW_WIDTH: u32 = 1600;
pub const WINDOW_HEIGHT: u32 = 900;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long)]
    pub fullscreen: bool,

    #[arg(long)]
    pub record: Option<String>,

    #[arg(long)]
    pub playback: Option<String>,

    #[arg(long)]
    pub speed_test: bool,
}

impl Args {
    pub fn record_option(&self) -> Result<RecordOption> {
        if self.record.is_some() && self.playback.is_some() {
            bail!("either --record or --playback or neither, but not both")
        }
        Ok(if let Some(record) = &self.record {
            RecordOption::Record(Path::new(&record).to_owned())
        } else if let Some(playback) = &self.playback {
            RecordOption::Playback(Path::new(&playback).to_owned())
        } else {
            RecordOption::None
        })
    }
}

fn sdl_main(args: Args) -> Result<()> {
    let sdl_context = sdl2::init().expect("failed to init SDL");
    let video_subsystem = sdl_context.video().expect("failed to get video context");
    let audio_subsystem = sdl_context.audio().expect("failed to get audio context");

    let file_manager = FileManager::from_fs()?;

    // We create a window.
    let title = "purpy2";
    let mut window = video_subsystem.window(title, WINDOW_WIDTH, WINDOW_HEIGHT);
    if args.fullscreen {
        window.fullscreen_desktop();
    }
    let window = window.resizable().build().expect("failed to build window");
    sdl_context.mouse().show_cursor(false);

    // We get the canvas from which we can get the `TextureCreator`.
    let mut canvas: Canvas<Window> = window
        .into_canvas()
        .build()
        .expect("failed to build window's canvas");
    let texture_creator = canvas.texture_creator();
    let renderer = SdlRenderer::new(&texture_creator);

    canvas.set_logical_size(RENDER_WIDTH, RENDER_HEIGHT)?;
    canvas.set_draw_color(Color::RGB(40, 40, 40));
    canvas.clear();
    canvas.present();

    let mut image_manager = ImageManager::new(renderer)?;
    image_manager.load_texture_atlas(
        Path::new("assets/textures.png"),
        Path::new("assets/textures_index.txt"),
        &file_manager,
    )?;
    let font = image_manager.load_font(&file_manager)?;

    let mut input_manager = InputManager::with_options(
        WINDOW_WIDTH as i32,
        WINDOW_HEIGHT as i32,
        false,
        args.record_option()?,
        &file_manager,
    )?;

    let mut stage_manager = StageManager::new(&file_manager, &mut image_manager)?;
    let mut sound_manager = SoundManager::with_sdl(&audio_subsystem)?;
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut frame = 0;
    let speed_test_start_time = Instant::now();

    'running: loop {
        let start_time = Instant::now();

        canvas.set_draw_color(Color::RGB(40, 40, 40));
        canvas.clear();

        for event in event_pump.poll_iter() {
            input_manager.handle_sdl_event(&event);
            if let Event::Quit { .. } = event {
                break 'running;
            }
        }

        let input_snapshot = input_manager.update(frame);

        let (width, height) = canvas.logical_size();
        let mut context = RenderContext::new(width, height, frame)?;

        if !stage_manager.update(
            &context,
            &input_snapshot,
            &file_manager,
            &mut image_manager,
            &mut sound_manager,
        )? {
            break 'running;
        }

        context.clear();
        stage_manager.draw(&mut context, &font);
        image_manager.renderer().render(&mut canvas, &context)?;
        canvas.present();

        frame += 1;
        let target_duration = Duration::new(0, 1_000_000_000u32 / FRAME_RATE);
        let actual_duration = start_time.elapsed();
        if actual_duration > target_duration {
            continue;
        }
        let remaining = target_duration - actual_duration;
        if !args.speed_test {
            ::std::thread::sleep(remaining);
        }
    }

    let speed_test_end_time = Instant::now();
    let speed_test_duration = speed_test_end_time - speed_test_start_time;
    let fps = frame as f64 / speed_test_duration.as_secs_f64();
    if args.speed_test {
        println!("{} fps: {} frames in {:?}", fps, frame, speed_test_duration);
    }

    Ok(())
}

fn main() {
    env_logger::init();
    let args = Args::parse();

    match sdl_main(args) {
        Ok(_) => {}
        Err(e) => panic!("{}", e),
    }
}
