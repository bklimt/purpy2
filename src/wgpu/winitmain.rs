use std::path::Path;

use anyhow::{bail, Result};
use log::error;
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use crate::constants::{RENDER_HEIGHT, RENDER_WIDTH};
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
    images: ImageManager<WgpuRenderer<'window, Window>>,
    sounds: SoundManager,
    inputs: InputManager,
    font: Font,
    frame: u32,
}

impl<'window> GameState<'window> {
    fn new(renderer: WgpuRenderer<'window, Window>) -> Result<Self> {
        let sdl_context = sdl2::init().expect("failed to init SDL");
        let audio_subsystem = sdl_context.audio().expect("failed to get audio context");

        let mut images = ImageManager::new(renderer)?;
        let inputs = InputManager::new()?;
        let stage_manager = StageManager::new(&images)?;
        let sounds = SoundManager::new(&audio_subsystem)?;

        images.load_texture_atlas(
            Path::new("assets/textures.png"),
            Path::new("assets/textures_index.txt"),
        )?;
        let font = images.load_font()?;
        let frame = 0;

        Ok(Self {
            stage_manager,
            images,
            sounds,
            inputs,
            font,
            frame,
        })
    }

    fn run_one_frame(&mut self) -> Result<bool> {
        let inputs = self.inputs.update();
        if !self
            .stage_manager
            .update(&inputs, &mut self.images, &mut self.sounds)?
        {
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

pub async fn run(_args: crate::Args) -> Result<()> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let PhysicalSize { width, height } = window.inner_size();
    let renderer = WgpuRenderer::new(&window, width, height).await;
    let mut game = match GameState::new(renderer) {
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
                    game.images.renderer_mut().resize(*width, *height);
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
        Event::AboutToWait => match game.run_one_frame() {
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
        _ => {}
    })?;

    Ok(())
}
