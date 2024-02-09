use wasm_bindgen::prelude::*;

use std::path::Path;

use anyhow::{bail, Result};
use log::{error, info};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use purpy::{
    FileManager, Font, ImageManager, InputManager, RecordOption, RenderContext, SoundManager,
    StageManager, WgpuRenderer, RENDER_HEIGHT, RENDER_WIDTH,
};

pub const CANVAS_WIDTH: u32 = 800;
pub const CANVAS_HEIGHT: u32 = 450;

const ASSETS_ARCHIVE_BYTES: &[u8] = include_bytes!("../../assets.tar.gz");

struct GameState<'window> {
    stage_manager: StageManager,
    file_manager: FileManager,
    images: ImageManager<WgpuRenderer<'window, Window>>,
    sounds: SoundManager,
    inputs: InputManager,
    font: Font,
    frame: u64,
}

impl<'window> GameState<'window> {
    fn new(file_manager: FileManager, renderer: WgpuRenderer<'window, Window>) -> Result<Self> {
        let mut images = ImageManager::new(renderer)?;
        let inputs = InputManager::with_options(RecordOption::None, &file_manager)?;
        let stage_manager = StageManager::new(&file_manager, &images)?;
        let sounds = SoundManager::noop_manager();

        images.load_texture_atlas(
            Path::new("assets/textures.png"),
            Path::new("assets/textures_index.txt"),
            &file_manager,
        )?;
        let font = images.load_font(&file_manager)?;
        let frame = 0;

        Ok(Self {
            stage_manager,
            file_manager,
            images,
            sounds,
            inputs,
            font,
            frame,
        })
    }

    fn run_one_frame(&mut self) -> Result<bool> {
        let inputs = self.inputs.update(self.frame);
        if !self.stage_manager.update(
            &inputs,
            &self.file_manager,
            &mut self.images,
            &mut self.sounds,
        )? {
            return Ok(false);
        }

        let width = RENDER_WIDTH;
        let height = RENDER_HEIGHT;
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
pub async fn run_or_die() {
    if let Err(err) = run().await {
        panic!("error: {}", err);
    }
}

pub async fn run() -> Result<()> {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");

    let event_loop = EventLoop::new()?;

    let file_manager = FileManager::from_archive_bytes(&ASSETS_ARCHIVE_BYTES[..])?;

    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let _ = window.request_inner_size(PhysicalSize::new(CANVAS_WIDTH, CANVAS_HEIGHT));

    {
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
    let width = if width == 0 { CANVAS_WIDTH } else { width };
    let height = if height == 0 { CANVAS_HEIGHT } else { height };

    let texture_atlas_path = Path::new("assets/textures.png");
    let vsync = true;
    let renderer = WgpuRenderer::new(
        &window,
        width,
        height,
        vsync,
        texture_atlas_path,
        &file_manager,
    )
    .await?;
    let mut game = match GameState::new(file_manager, renderer) {
        Ok(game) => game,
        Err(e) => {
            bail!("unable to initialize game: {:?}", e);
        }
    };

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
                    info!("window resized to {width}, {height}");
                    //game.images.renderer_mut().resize(*width, *height);
                }
                WindowEvent::RedrawRequested => match game.run_one_frame() {
                    Ok(running) => {
                        if !running {
                            elwt.exit();
                        }
                    }
                    Err(e) => {
                        error!("{:?}", e);
                        elwt.exit();
                    }
                },
                WindowEvent::CloseRequested => {
                    elwt.exit();
                }
                _ => {}
            }
        }
        Event::AboutToWait => game.images.renderer().window().request_redraw(),
        _ => {}
    })?;

    Ok(())
}
