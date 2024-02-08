#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use std::path::Path;
use std::time::Instant;

use anyhow::{anyhow, bail, Result};
use log::error;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use crate::args::Args;
use crate::constants::{RENDER_HEIGHT, RENDER_WIDTH};
use crate::filemanager::FileManager;
use crate::font::Font;
use crate::imagemanager::ImageManager;
use crate::inputmanager::InputManager;
use crate::rendercontext::RenderContext;
use crate::soundmanager::SoundManager;
use crate::stagemanager::StageManager;
use crate::wgpu::renderer::WgpuRenderer;

use super::renderer::WindowHandle;

impl WindowHandle for Window {}

struct GameState<'window> {
    stage_manager: StageManager,
    files: FileManager,
    images: ImageManager<WgpuRenderer<'window, Window>>,
    sounds: SoundManager,
    inputs: InputManager,
    font: Font,
    frame: u64,
    start_time: Instant,
    speed_test: bool,
}

impl<'window> GameState<'window> {
    fn new(
        args: Args,
        renderer: WgpuRenderer<'window, Window>,
        files: FileManager,
    ) -> Result<Self> {
        let mut sounds = None;
        cfg_if::cfg_if! {
            if #[cfg(target_arch = "wasm32")] {
                sounds = Some(SoundManager::noop_manager());
            } else {
                let sdl_context = sdl2::init().expect("failed to init SDL");
                let audio_subsystem = sdl_context.audio().expect("failed to get audio context");
                sounds = Some(SoundManager::with_sdl(&audio_subsystem)?);
            }
        }
        let sounds = sounds.unwrap();

        let file_manager =
            FileManager::new().map_err(|e| anyhow!("unable to create file manager: {}", e))?;

        let mut images = ImageManager::new(renderer)?;
        let inputs = InputManager::new(&args)?;
        let stage_manager = StageManager::new(&files, &images)?;

        images.load_texture_atlas(
            &files,
            Path::new("assets/textures.png"),
            Path::new("assets/textures_index.txt"),
        )?;
        let font = images.load_font(&file_manager)?;
        let frame = 0;
        let start_time = Instant::now();
        let speed_test = args.speed_test;

        Ok(Self {
            stage_manager,
            files,
            images,
            sounds,
            inputs,
            font,
            frame,
            start_time,
            speed_test,
        })
    }

    fn run_one_frame(&mut self) -> Result<bool> {
        if self.frame == 0 {
            self.start_time = Instant::now();
        }

        let inputs = self.inputs.update(self.frame);
        if !self
            .stage_manager
            .update(&inputs, &self.files, &mut self.images, &mut self.sounds)?
        {
            let finish_time = Instant::now();
            if self.speed_test {
                let elapsed = finish_time - self.start_time;
                let fps = self.frame as f64 / elapsed.as_secs_f64();
                println!("{} fps: {} frames in {:?}", fps, self.frame, elapsed);
            }
            return Ok(false);
        }

        let width = RENDER_WIDTH; //self.images.renderer().width();
        let height = RENDER_HEIGHT; //self.images.renderer().height();
        let mut context = RenderContext::new(width, height, self.frame)?;
        self.stage_manager.draw(&mut context, &self.font);

        match self.images.renderer_mut().render(&context) {
            Ok(_) => {}
            Err(e) => error!("{:?}", e),
        }

        self.frame += 1;
        Ok(true)
    }
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn wasm_main() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        }
    }

    let args = Args {
        fullscreen: false,
        record: None,
        playback: None,
        speed_test: false,
    };
    if let Err(e) = run(args).await {
        panic!("error: {}", e);
    }
}

pub async fn run(args: Args) -> Result<()> {
    let file_manager =
        FileManager::new().map_err(|e| anyhow!("unable to create file manager: {}", e))?;

    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        window.request_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas().unwrap());
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    let PhysicalSize { width, height } = window.inner_size();
    let texture_atlas_path = Path::new("assets/textures.png");
    let renderer = WgpuRenderer::new(&window, &file_manager, width, height, texture_atlas_path)
        .await
        .map_err(|e| anyhow!("unable to create renderer: {}", e))?;
    let mut game = GameState::new(args, renderer, file_manager)
        .map_err(|e| anyhow!("unable to initialize game: {:?}", e))?;

    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run(move |event, elwt| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == game.images.renderer().window().id() => {
            game.inputs.handle_winit_event(event);
            match event {
                WindowEvent::Resized(new_size) => {
                    let PhysicalSize { width, height } = new_size;
                    game.images.renderer_mut().resize(*width, *height);
                }
                WindowEvent::RedrawRequested => match game.run_one_frame() {
                    Ok(running) => {
                        if !running {
                            elwt.exit();
                        }
                    }
                    Err(e) => {
                        error!("error stepping game: {:?}", e);
                        elwt.exit();
                    }
                },
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                _ => {}
            }
        }
        Event::AboutToWait => match game.run_one_frame() {
            Ok(running) => {
                if !running {
                    elwt.exit();
                }
            }
            Err(e) => {
                error!("error stepping game: {:?}", e);
                elwt.exit();
            }
        },
        _ => {}
    })?;

    Ok(())
}
