use std::path::Path;

use anyhow::{bail, Result};
use log::error;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

use crate::imagemanager::ImageManager;
use crate::inputmanager::InputManager;
use crate::rendercontext::RenderContext;
use crate::soundmanager::SoundManager;
use crate::stagemanager::StageManager;
use crate::wgpu::renderer::WgpuRenderer;
use crate::{
    constants::{RENDER_HEIGHT, RENDER_WIDTH},
    font::Font,
};

use super::renderer::{RenderError, RendererCanvas};

impl RendererCanvas for Window {
    fn canvas_size(&self) -> (u32, u32) {
        let size = self.inner_size();
        (size.width, size.height)
    }
}

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
            Err(RenderError::SurfaceError(wgpu::SurfaceError::Outdated)) => {
                self.images.renderer_mut().recreate_surface();
            }
            Err(RenderError::SurfaceError(wgpu::SurfaceError::Lost)) => {
                self.images.renderer_mut().recreate_surface();
            }
            Err(RenderError::SurfaceError(wgpu::SurfaceError::OutOfMemory)) => {
                bail!("out of memory");
            }
            Err(e) => error!("{:?}", e),
        }

        self.frame += 1;
        Ok(true)
    }
}

pub async fn run() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let renderer = WgpuRenderer::new(&window).await;
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
                WindowEvent::Resized(_) => {
                    game.images.renderer_mut().recreate_surface();
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
