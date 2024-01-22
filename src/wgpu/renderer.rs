use std::mem::zeroed;
use std::path::Path;

use anyhow::Result;
use bytemuck::Zeroable;
use log::error;
use wgpu::util::DeviceExt;
use wgpu::SurfaceError;
use winit::{dpi::PhysicalSize, window::Window};

use crate::rendercontext::{RenderContext, SpriteBatch, SpriteBatchEntry};
use crate::renderer::Renderer;
use crate::sprite::Sprite;

use super::{shader::Vertex, texture::Texture};

pub struct WgpuRenderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_groups: Vec<wgpu::BindGroup>,

    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: Window,
}

fn create_vertex_buffer(batch: &SpriteBatch, device: &wgpu::Device) -> (wgpu::Buffer, usize) {
    const MAX_ENTRIES: usize = 1024;
    const MAX_VERTICES: usize = MAX_ENTRIES * 6;

    if batch.entries.len() > MAX_ENTRIES {
        error!("sprite batch is too large: {}", batch.entries.len());
    }

    let mut vertices = [Vertex::zeroed(); MAX_VERTICES];
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

        let st = source.y as f32;
        let sb = source.bottom() as f32;
        let sl = source.x as f32;
        let sr = source.right() as f32;

        let i = vertex_count;
        vertex_count += 6;

        vertices[i] = Vertex {
            position: [dl, dt, 0.0],
            tex_coords: [sl, st],
        };
        vertices[i + 1] = Vertex {
            position: [dl, db, 0.0],
            tex_coords: [sl, sb],
        };
        vertices[i + 2] = Vertex {
            position: [dr, dt, 0.0],
            tex_coords: [sr, st],
        };
        vertices[i + 3] = Vertex {
            position: [dr, dt, 0.0],
            tex_coords: [sr, st],
        };
        vertices[i + 4] = Vertex {
            position: [dl, db, 0.0],
            tex_coords: [sl, sb],
        };
        vertices[i + 5] = Vertex {
            position: [dr, db, 0.0],
            tex_coords: [sr, sb],
        };
    }

    let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(&mut vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    (vertex_buffer, vertex_count)
}

impl WgpuRenderer {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // The surface needs to live as long as the window that created it.
        // State owns the window, so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

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
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
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

        let texture_bind_groups = Vec::new();

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
                    blend: Some(wgpu::BlendState::REPLACE),
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

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            texture_bind_group_layout,
            texture_bind_groups,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn recreate_surface(&mut self) {
        self.resize(self.size)
    }

    pub fn resize(&mut self, _size: PhysicalSize<u32>) {}

    pub fn render(&self, context: &RenderContext) -> Result<(), SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

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
            for entry in context.player_batch.entries.iter() {
                match entry {
                    SpriteBatchEntry::Sprite {
                        sprite,
                        source,
                        destination,
                        reversed,
                    } => {
                        let texture_bind_group = &self.texture_bind_groups[sprite.id];

                        // TODO: Redo this with an index buffer.
                        let vertices: &[Vertex; 3] = &[
                            Vertex {
                                position: [destination.x as f32, destination.y as f32, 0.0],
                                tex_coords: [source.x as f32, source.y as f32],
                            },
                            Vertex {
                                position: [destination.x as f32, destination.bottom() as f32, 0.0],
                                tex_coords: [source.x as f32, source.bottom() as f32],
                            },
                            Vertex {
                                position: [
                                    destination.right() as f32,
                                    destination.top() as f32,
                                    0.0,
                                ],
                                tex_coords: [source.right() as f32, source.top() as f32],
                            },
                        ];

                        let vertex_buffer =
                            self.device
                                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                    label: Some("Vertex Buffer"),
                                    contents: bytemuck::cast_slice(vertices),
                                    usage: wgpu::BufferUsages::VERTEX,
                                });

                        render_pass.set_bind_group(0, texture_bind_group, &[]);
                        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                        render_pass.draw(0..3, 0..1);
                    }
                    _ => {}
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

impl Renderer for WgpuRenderer {
    fn load_sprite(&mut self, path: &Path) -> Result<Sprite> {
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

        let index = self.texture_bind_groups.len();
        self.texture_bind_groups.push(texture_bind_group);

        Ok(Sprite {
            id: index,
            width: texture.width,
            height: texture.height,
        })
    }
}
