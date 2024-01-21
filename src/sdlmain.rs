use std::time::{Duration, Instant};

use anyhow::Result;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::constants::{FRAME_RATE, RENDER_HEIGHT, RENDER_WIDTH, WINDOW_HEIGHT, WINDOW_WIDTH};
use crate::imagemanager::ImageManager;
use crate::inputmanager::InputManager;
use crate::rendercontext::RenderContext;
use crate::sdlrenderer::SdlRenderer;
use crate::soundmanager::SoundManager;
use crate::stagemanager::StageManager;
use crate::Args;

pub fn sdl_main(args: Args) -> Result<()> {
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
    let renderer = SdlRenderer::new(&texture_creator);

    canvas.set_logical_size(RENDER_WIDTH, RENDER_HEIGHT)?;
    canvas.set_draw_color(Color::RGB(40, 40, 40));
    canvas.clear();
    canvas.present();

    let mut image_manager = ImageManager::new(renderer)?;
    let mut input_manager = InputManager::new()?;
    let mut stage_manager = StageManager::new(&image_manager)?;
    let mut sound_manager = SoundManager::new(&audio_subsystem)?;
    let mut event_pump = sdl_context.event_pump().unwrap();

    let font = image_manager.load_font()?;

    let mut frame = 0;

    'running: loop {
        let start_time = Instant::now();

        canvas.set_draw_color(Color::RGB(40, 40, 40));
        canvas.clear();

        let (width, height) = canvas.logical_size();
        let mut context = RenderContext::new(width, height, frame)?;

        for event in event_pump.poll_iter() {
            input_manager.handle_sdl_event(&event);
            match event {
                Event::Quit { .. } => break 'running,
                _ => {}
            }
        }

        let input_snapshot = input_manager.update();

        if !stage_manager.update(&input_snapshot, &mut image_manager, &mut sound_manager)? {
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
        ::std::thread::sleep(remaining);
    }

    Ok(())
}
