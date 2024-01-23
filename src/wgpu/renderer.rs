use std::mem;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bytemuck::Zeroable;
use log::{error, info};
use thiserror::Error;
use wgpu::util::DeviceExt;
use wgpu::WindowHandle;

use crate::constants::{RENDER_HEIGHT, RENDER_WIDTH};
use crate::rendercontext::{RenderContext, SpriteBatch, SpriteBatchEntry};
use crate::renderer::Renderer;
use crate::sprite::Sprite;

use super::{shader::Vertex, texture::Texture};

const MAX_ENTRIES: usize = 4096;
const MAX_VERTICES: usize = MAX_ENTRIES * 6;

pub trait RendererCanvas
where
    Self: WindowHandle,
{
    fn canvas_size(&self) -> (u32, u32);
}

pub struct WgpuRenderer<'window, T: RendererCanvas> {
    window: &'window T,
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: (u32, u32),
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: Option<wgpu::BindGroup>,
    texture_width: u32,
    texture_height: u32,
    vertices: Vec<Vertex>,
}

#[derive(Error, Debug)]
pub enum RenderError {
    #[error("render surface error")]
    SurfaceError(#[from] wgpu::SurfaceError),
    #[error("other rendering error")]
    Other(#[from] anyhow::Error),
}

impl<'window, T> WgpuRenderer<'window, T>
where
    T: RendererCanvas,
{
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'window T) -> Self {
        let size = window.canvas_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // The surface needs to live as long as the window that created it.
        // State owns the window, so this should be safe.
        let surface = instance.create_surface(window).unwrap();

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
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.is_srgb())
            .next()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.0,
            height: size.1,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        // This should match the filterable field of the
                        // corresponding Texture entry above.
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let texture_bind_group = None;
        let texture_width = 0;
        let texture_height = 0;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let mut vertices = Vec::new();
        vertices.resize_with(MAX_VERTICES, Vertex::zeroed);
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&mut vertices),
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            texture_bind_group_layout,
            texture_bind_group,
            texture_width,
            texture_height,
            vertices,
            window,
        }
    }

    pub fn window(&self) -> &T {
        &self.window
    }

    pub fn recreate_surface(&mut self) {
        self.resize(self.window.canvas_size())
    }

    pub fn resize(&mut self, new_size: (u32, u32)) {
        if new_size.0 > 0 && new_size.1 > 0 {
            self.size = new_size;
            self.config.width = new_size.0;
            self.config.height = new_size.1;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn fill_vertex_buffer(&mut self, batch: &SpriteBatch) -> u32 {
        if batch.entries.len() > MAX_ENTRIES {
            error!("sprite batch is too large: {}", batch.entries.len());
        }

        let mut vertex_count = 0;

        for entry in batch.entries.iter() {
            if vertex_count >= MAX_VERTICES {
                break;
            }

            let SpriteBatchEntry::Sprite {
                sprite,
                source,
                destination,
                reversed,
            } = entry
            else {
                continue;
            };

            let dt = destination.y as f32;
            let db = destination.bottom() as f32;
            let dl = destination.x as f32;
            let dr = destination.right() as f32;

            let st = (source.y + sprite.y as i32) as f32;
            let sb = (source.bottom() + sprite.y as i32) as f32;
            let mut sl = (source.x + sprite.x as i32) as f32;
            let mut sr = (source.right() + sprite.x as i32) as f32;

            if *reversed {
                mem::swap(&mut sl, &mut sr);
            }

            // TODO: Consider moving this scaling into the shader.
            let xscale = RENDER_WIDTH as f32;
            let yscale = RENDER_HEIGHT as f32;
            let dt = dt / yscale;
            let db = db / yscale;
            let dl = dl / xscale;
            let dr = dr / xscale;

            let xscale = self.texture_width as f32;
            let yscale = self.texture_height as f32;
            let st = st / yscale;
            let sb = sb / yscale;
            let sl = sl / xscale;
            let sr = sr / xscale;

            let i = vertex_count;
            vertex_count += 6;

            self.vertices[i] = Vertex {
                position: [dl, dt, 0.0],
                tex_coords: [sl, st],
            };
            self.vertices[i + 1] = Vertex {
                position: [dl, db, 0.0],
                tex_coords: [sl, sb],
            };
            self.vertices[i + 2] = Vertex {
                position: [dr, dt, 0.0],
                tex_coords: [sr, st],
            };
            self.vertices[i + 3] = Vertex {
                position: [dr, dt, 0.0],
                tex_coords: [sr, st],
            };
            self.vertices[i + 4] = Vertex {
                position: [dl, db, 0.0],
                tex_coords: [sl, sb],
            };
            self.vertices[i + 5] = Vertex {
                position: [dr, db, 0.0],
                tex_coords: [sr, sb],
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

    pub fn render(&mut self, context: &RenderContext) -> Result<(), RenderError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let vertex_count = self.fill_vertex_buffer(&context.player_batch);

        let texture_bind_group = self
            .texture_bind_group
            .as_ref()
            .context("texture atlas not loaded")?;

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, texture_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw(0..vertex_count, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

impl<'window, T> Renderer for WgpuRenderer<'window, T>
where
    T: RendererCanvas,
{
    fn load_sprite(&mut self, path: &Path) -> Result<Sprite> {
        if self.texture_bind_group.is_some() {
            bail!("wgpu renderer requires a single texture atlas, but a second texture was loaded");
        }

        info!("Reading texture atlas from {:?}", path);

        let texture = Texture::from_file(&self.device, &self.queue, path)?;
        let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        self.texture_bind_group = Some(texture_bind_group);
        self.texture_width = texture.width;
        self.texture_height = texture.height;

        Ok(Sprite {
            id: 0,
            x: 0,
            y: 0,
            width: texture.width,
            height: texture.height,
        })
    }
}
