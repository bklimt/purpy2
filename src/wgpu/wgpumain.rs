use std::path::Path;

use anyhow::{bail, Result};
use log::error;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

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

use super::renderer::RenderError;

struct GameState {
    stage_manager: StageManager,
    images: ImageManager<WgpuRenderer>,
    sounds: SoundManager,
    inputs: InputManager,
    font: Font,
    frame: u32,
}

impl GameState {
    fn new(renderer: WgpuRenderer) -> Result<Self> {
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

        match self.images.renderer().render(&context) {
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

pub async fn run() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let renderer = WgpuRenderer::new(window).await;
    let mut game = match GameState::new(renderer) {
        Ok(game) => game,
        Err(e) => {
            error!("unable to initialize game: {:?}", e);
            return;
        }
    };

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == game.images.renderer().window().id() => {
            game.inputs.handle_winit_event(event);
        }
        Event::RedrawRequested(window_id) if window_id == game.images.renderer().window().id() => {
            match game.run_one_frame() {
                Ok(running) => {
                    if !running {
                        *control_flow = ControlFlow::Exit;
                    }
                }
                Err(e) => {
                    error!("{:?}", e);
                    *control_flow = ControlFlow::ExitWithCode(-1);
                }
            }
        }
        Event::MainEventsCleared => {
            game.images.renderer().window().request_redraw();
        }
        _ => {}
    });
}