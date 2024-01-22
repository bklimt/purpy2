use anyhow::{bail, Result};
use log::error;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::font::Font;
use crate::imagemanager::ImageManager;
use crate::inputmanager::InputManager;
use crate::rendercontext::RenderContext;
use crate::soundmanager::SoundManager;
use crate::stagemanager::StageManager;
use crate::wgpu::renderer::WgpuRenderer;

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
        let mut inputs = InputManager::new()?;
        let mut stage_manager = StageManager::new(&images)?;
        let mut sounds = SoundManager::new(&audio_subsystem)?;

        let font = images.load_font()?;
        let mut frame = 0;

        Ok(Self {
            stage_manager,
            images,
            sounds,
            inputs,
            font,
            frame,
        })
    }

    fn run_one_frame(&mut self) -> Result<()> {
        let inputs = self.inputs.update();
        self.stage_manager
            .update(&inputs, &mut self.images, &mut self.sounds)?;

        let width = self.images.renderer().width();
        let height = self.images.renderer().height();
        let mut context = RenderContext::new(width, height, self.frame)?;
        self.stage_manager.draw(&mut context, &self.font);

        match self.images.renderer().render(&context) {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => self.images.renderer_mut().recreate_surface(),
            Err(wgpu::SurfaceError::OutOfMemory) => {
                bail!("out of memory");
            }
            Err(e) => error!("{:?}", e),
        }

        self.frame += 1;
        Ok(())
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
            match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                _ => {}
            }
        }
        Event::RedrawRequested(window_id) if window_id == game.images.renderer().window().id() => {
            if let Err(e) = game.run_one_frame() {
                error!("{:?}", e);
                *control_flow = ControlFlow::ExitWithCode(-1);
            }
        }
        Event::MainEventsCleared => {
            game.images.renderer().window().request_redraw();
        }
        _ => {}
    });
}
