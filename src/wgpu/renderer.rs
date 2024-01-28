use std::mem;
use std::path::Path;

use anyhow::{bail, Context, Result};
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

use super::shader::ShaderUniform;
use super::{shader::Vertex, texture::Texture};

const MAX_ENTRIES: usize = 4096;
const MAX_VERTICES: usize = MAX_ENTRIES * 6;

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
    render_pipeline: wgpu::RenderPipeline,

    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group: Option<wgpu::BindGroup>,
    texture_width: u32,
    texture_height: u32,

    _shader_uniform: ShaderUniform,
    _shader_uniform_buffer: wgpu::Buffer,
    _uniform_bind_group_layout: wgpu::BindGroupLayout,
    uniform_bind_group: wgpu::BindGroup,

    vertex_buffer: wgpu::Buffer,
    vertices: Vec<Vertex>,
}

impl<'window, T> WgpuRenderer<'window, T>
where
    T: WindowHandle,
{
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'window T, width: u32, height: u32) -> Self {
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
            width,
            height,
            present_mode: wgpu::PresentMode::AutoNoVsync, //surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
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

        let shader_uniform = ShaderUniform::new(RENDER_WIDTH, RENDER_HEIGHT);
        let shader_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shader Uniform Buffer"),
            contents: bytemuck::cast_slice(&[shader_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });
        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: shader_uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&texture_bind_group_layout, &uniform_bind_group_layout],
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
            width,
            height,
            render_pipeline,
            vertex_buffer,
            texture_bind_group_layout,
            texture_bind_group,
            texture_width,
            texture_height,
            _shader_uniform: shader_uniform,
            _shader_uniform_buffer: shader_uniform_buffer,
            _uniform_bind_group_layout: uniform_bind_group_layout,
            uniform_bind_group,
            vertices,
            window,
        }
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
                        load: wgpu::LoadOp::Clear(context.player_batch.clear_color.into()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, texture_bind_group, &[]);
            render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
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
    T: WindowHandle,
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
            area: Rect {
                x: Pixels::zero(),
                y: Pixels::zero(),
                w: Pixels::new(texture.width as i32),
                h: Pixels::new(texture.height as i32),
            },
        })
    }
}
