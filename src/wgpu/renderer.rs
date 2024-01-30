use std::mem;
use std::path::Path;

use anyhow::Result;
use bytemuck::Zeroable;
use log::{error, info};
use num_traits::Zero;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use wgpu::util::DeviceExt;

use crate::constants::{RENDER_HEIGHT, RENDER_WIDTH};
use crate::geometry::{Pixels, Rect};
use crate::rendercontext::{RenderContext, SpriteBatch, SpriteBatchEntry};
use crate::renderer::Renderer;
use crate::sprite::Sprite;
use crate::utils::Color;
use crate::wgpu::pipeline::Pipeline;
use crate::wgpu::shader::PostprocessVertex;
use crate::wgpu::shader::PostprocessVertexUniform;
use crate::wgpu::shader::RenderVertexUniform;
use crate::wgpu::shader::Vertex;
use crate::wgpu::texture::Texture;

const MAX_ENTRIES: usize = 4096;
const MAX_VERTICES: usize = MAX_ENTRIES * 6;

const RECT_VERTICES: &[PostprocessVertex] = &[
    PostprocessVertex {
        position: [1.0, 1.0],
        tex_coords: [1.0, 0.0],
    },
    PostprocessVertex {
        position: [-1.0, 1.0],
        tex_coords: [0.0, 0.0],
    },
    PostprocessVertex {
        position: [-1.0, -1.0],
        tex_coords: [0.0, 1.0],
    },
    PostprocessVertex {
        position: [1.0, 1.0],
        tex_coords: [1.0, 0.0],
    },
    PostprocessVertex {
        position: [-1.0, -1.0],
        tex_coords: [0.0, 1.0],
    },
    PostprocessVertex {
        position: [1.0, -1.0],
        tex_coords: [1.0, 1.0],
    },
];

pub trait WindowHandle
where
    Self: HasRawWindowHandle + HasRawDisplayHandle,
{
}

pub struct WgpuRenderer<'window, T: WindowHandle> {
    window: &'window T,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    width: u32,
    height: u32,

    render_pipeline: Pipeline,

    texture_width: u32,
    texture_height: u32,

    vertices: Vec<Vertex>,
    vertex_buffer: wgpu::Buffer,

    framebuffer: Texture,
    postprocess_pipeline: Pipeline,
    postprocess_vertex_buffer: wgpu::Buffer,
}

impl<'window, T> WgpuRenderer<'window, T>
where
    T: WindowHandle,
{
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'window T, width: u32, height: u32) -> Result<Self> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // The surface needs to live as long as the window that created it.
        // State owns the window, so this should be safe.
        let surface = unsafe { instance.create_surface(window).unwrap() };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        for format in surface_caps.formats.iter() {
            info!("available texture format: {:?}", format);
        }

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| !f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);
        info!("using texture format: {:?}", surface_format);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoNoVsync, //surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let texture_width = 0;
        let texture_height = 0;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let mut vertices = Vec::new();
        vertices.resize_with(MAX_VERTICES, Vertex::zeroed);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&mut vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let postprocess_vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Postprocess Vertex Buffer"),
                contents: bytemuck::cast_slice(RECT_VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            });

        let mut render_pipeline = Pipeline::new(
            "Render Pipeline",
            &device,
            &shader,
            "vs_main",
            "fs_main",
            Vertex::desc(),
            config.format,
        )?;

        let vertex_uniform = RenderVertexUniform::new(RENDER_WIDTH, RENDER_HEIGHT);
        render_pipeline.set_vertex_uniform(&device, vertex_uniform);

        let mut postprocess_pipeline = Pipeline::new(
            "Postprocess Pipeline",
            &device,
            &shader,
            "vs_main2",
            "fs_main2",
            PostprocessVertex::desc(),
            config.format,
        )?;

        let postprocess_uniform = PostprocessVertexUniform::new();
        postprocess_pipeline.set_vertex_uniform(&device, postprocess_uniform);

        let framebuffer = Texture::frame_buffer(&device)?;
        postprocess_pipeline.set_texture(&device, &framebuffer);

        Ok(Self {
            surface,
            device,
            queue,
            config,
            width,
            height,
            render_pipeline,
            postprocess_pipeline,
            vertices,
            vertex_buffer,
            postprocess_vertex_buffer,
            texture_width,
            texture_height,
            framebuffer,
            window,
        })
    }

    pub fn window(&self) -> &T {
        &self.window
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.width = new_width;
            self.height = new_height;
            self.config.width = new_width;
            self.config.height = new_height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn fill_vertex_buffer(&mut self, batch: &SpriteBatch) -> u32 {
        if batch.entries.len() > MAX_ENTRIES {
            error!("sprite batch is too large: {}", batch.entries.len());
        }

        let mut vertex_count = 0;
        let one_pixel = Pixels::new(1);

        for entry in batch.entries.iter() {
            if vertex_count >= MAX_VERTICES {
                break;
            }

            let (destination, source, color, reversed) = match entry {
                SpriteBatchEntry::FillRect { destination, color } => (
                    *destination,
                    Rect {
                        x: Pixels::zero(),
                        y: Pixels::zero(),
                        w: Pixels::zero(),
                        h: Pixels::zero(),
                    },
                    *color,
                    false,
                ),
                SpriteBatchEntry::Sprite {
                    sprite,
                    source,
                    destination,
                    reversed,
                } => {
                    let source = Rect {
                        x: sprite.area.x + source.x,
                        y: sprite.area.y + source.y,
                        w: source.w,
                        h: source.h,
                    };
                    let color = Color {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 0,
                    };
                    (*destination, source, color, *reversed)
                }
            };

            let dt = (destination.y / one_pixel) as f32;
            let db = (destination.bottom() / one_pixel) as f32;
            let dl = (destination.x / one_pixel) as f32;
            let dr = (destination.right() / one_pixel) as f32;

            let st = (source.y / one_pixel) as f32;
            let sb = (source.bottom() / one_pixel) as f32;
            let mut sl = (source.x / one_pixel) as f32;
            let mut sr = (source.right() / one_pixel) as f32;

            if reversed {
                mem::swap(&mut sl, &mut sr);
            }

            // TODO: Consider moving this scaling into the shader.
            let xscale = self.texture_width as f32;
            let yscale = self.texture_height as f32;
            let st = st / yscale;
            let sb = sb / yscale;
            let sl = sl / xscale;
            let sr = sr / xscale;

            let color: [f32; 4] = color.into();

            let i = vertex_count;
            vertex_count += 6;

            self.vertices[i] = Vertex {
                position: [dl, dt],
                tex_coords: [sl, st],
                color,
            };
            self.vertices[i + 1] = Vertex {
                position: [dl, db],
                tex_coords: [sl, sb],
                color,
            };
            self.vertices[i + 2] = Vertex {
                position: [dr, dt],
                tex_coords: [sr, st],
                color,
            };
            self.vertices[i + 3] = Vertex {
                position: [dr, dt],
                tex_coords: [sr, st],
                color,
            };
            self.vertices[i + 4] = Vertex {
                position: [dl, db],
                tex_coords: [sl, sb],
                color,
            };
            self.vertices[i + 5] = Vertex {
                position: [dr, db],
                tex_coords: [sr, sb],
                color,
            };
        }
        //info!("created {} vertices", vertex_count);

        self.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.vertices[0..vertex_count]),
        );

        vertex_count as u32
    }

    pub fn render(&mut self, context: &RenderContext) -> Result<()> {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let vertex_count = self.fill_vertex_buffer(&context.player_batch);

        let output = self.surface.get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.render_pipeline.render(
            &mut encoder,
            &self.framebuffer.view,
            context.player_batch.clear_color,
            self.vertex_buffer.slice(..),
            vertex_count,
        );

        self.postprocess_pipeline.render(
            &mut encoder,
            &output_view,
            context.player_batch.clear_color.into(),
            self.postprocess_vertex_buffer.slice(..),
            6,
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        output.present();

        Ok(())
    }
}

impl<'window, T> Renderer for WgpuRenderer<'window, T>
where
    T: WindowHandle,
{
    fn load_sprite(&mut self, path: &Path) -> Result<Sprite> {
        info!("Reading texture atlas from {:?}", path);

        let texture = Texture::from_file(&self.device, &self.queue, path)?;
        self.render_pipeline.set_texture(&self.device, &texture);
        self.texture_width = texture.width;
        self.texture_height = texture.height;

        Ok(Sprite {
            id: 0,
            area: Rect {
                x: Pixels::zero(),
                y: Pixels::zero(),
                w: Pixels::new(texture.width as i32),
                h: Pixels::new(texture.height as i32),
            },
        })
    }
}
