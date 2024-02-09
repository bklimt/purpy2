use anyhow::Result;
use bytemuck::Pod;
use wgpu::util::DeviceExt;

use crate::utils::Color;

use super::{shader::DefaultUniform, texture::Texture};

pub fn create_uniform<T>(
    label: &str,
    device: &wgpu::Device,
    uniform: T,
    layout: &wgpu::BindGroupLayout,
) -> wgpu::BindGroup
where
    T: Pod,
{
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(format!("[{}] Uniform Buffer", label).as_str()),
        contents: bytemuck::cast_slice(&[uniform]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    return device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("[{}] Uniform Bind Group"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });
}

pub struct Pipeline {
    label: String,
    render_pipeline: wgpu::RenderPipeline,

    vertex_uniform_bind_group_layout: wgpu::BindGroupLayout,
    vertex_uniform_bind_group: wgpu::BindGroup,

    fragment_uniform_bind_group_layout: wgpu::BindGroupLayout,
    fragment_uniform_bind_group: wgpu::BindGroup,
    fragment_uniform_buffer: Option<wgpu::Buffer>,

    texture_bind_group: wgpu::BindGroup,
}

impl Pipeline {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        label: &str,
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        vertex_shader_entry_point: &str,
        fragment_shader_entry_point: &str,
        vertex_buffer_layout: wgpu::VertexBufferLayout,
        textures: &[&Texture],
        format: wgpu::TextureFormat,
    ) -> Result<Self> {
        let vertex_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(format!("[{}] Vertex Uniform Bind Group Layout", label).as_str()),
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
            });

        let default_uniform = DefaultUniform::new();

        let vertex_uniform_bind_group = create_uniform(
            format!("{} Vertex", label).as_str(),
            device,
            default_uniform,
            &vertex_uniform_bind_group_layout,
        );

        let fragment_uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(format!("[{}] Fragment Uniform Bind Group Layout", label).as_str()),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let fragment_uniform_bind_group = create_uniform(
            format!("{} Fragment Uniform Bind Group", label).as_str(),
            device,
            default_uniform,
            &fragment_uniform_bind_group_layout,
        );

        let mut texture_bind_group_layout_entries = Vec::new();
        for i in 0..textures.len() {
            texture_bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: i as u32 * 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            });
            texture_bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: i as u32 * 2 + 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                // This should match the filterable field of the
                // corresponding Texture entry above.
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            });
        }

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(format!("[{}] Texture Bind Group Layout", label).as_str()),
                entries: &texture_bind_group_layout_entries,
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(format!("[{}] Render Pipeline Layout", label).as_str()),
                bind_group_layouts: &[
                    &vertex_uniform_bind_group_layout,
                    &fragment_uniform_bind_group_layout,
                    &texture_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(format!("[{}] Render Pipeline", label).as_str()),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: shader,
                entry_point: vertex_shader_entry_point,
                buffers: &[vertex_buffer_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: shader,
                entry_point: fragment_shader_entry_point,
                targets: &[Some(wgpu::ColorTargetState {
                    format,
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

        let mut texture_bind_group_entries = Vec::new();
        for (i, texture) in textures.iter().enumerate() {
            texture_bind_group_entries.push(wgpu::BindGroupEntry {
                binding: i as u32 * 2,
                resource: wgpu::BindingResource::TextureView(&texture.view),
            });
            texture_bind_group_entries.push(wgpu::BindGroupEntry {
                binding: i as u32 * 2 + 1,
                resource: wgpu::BindingResource::Sampler(&texture.sampler),
            });
        }

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(format!("[{}] Texture Bind Group", label).as_str()),
            layout: &texture_bind_group_layout,
            entries: &texture_bind_group_entries,
        });

        let label = label.to_owned();

        let fragment_uniform_buffer = None;

        Ok(Self {
            label,
            render_pipeline,
            vertex_uniform_bind_group_layout,
            vertex_uniform_bind_group,
            fragment_uniform_bind_group_layout,
            fragment_uniform_bind_group,
            fragment_uniform_buffer,
            texture_bind_group,
        })
    }

    pub fn set_vertex_uniform<T>(&mut self, device: &wgpu::Device, vertex_uniform: T)
    where
        T: Pod,
    {
        let vertex_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(format!("[{}] Vertex Uniform Buffer", self.label).as_str()),
            contents: bytemuck::cast_slice(&[vertex_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        self.vertex_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("[{}] Vertex Uniform Bind Group"),
            layout: &self.vertex_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: vertex_uniform_buffer.as_entire_binding(),
            }],
        });
    }

    pub fn set_fragment_uniform<T>(&mut self, device: &wgpu::Device, fragment_uniform: T)
    where
        T: Pod,
    {
        self.fragment_uniform_buffer = Some(device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(format!("[{}] Fragment Uniform Buffer", self.label).as_str()),
                contents: bytemuck::cast_slice(&[fragment_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            },
        ));

        self.fragment_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("[{}] Fragment Uniform_Bind_Group"),
            layout: &self.fragment_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self
                    .fragment_uniform_buffer
                    .as_ref()
                    .unwrap()
                    .as_entire_binding(),
            }],
        });
    }

    pub fn update_fragment_uniform<T>(&mut self, queue: &wgpu::Queue, fragment_uniform: T)
    where
        T: Pod,
    {
        queue.write_buffer(
            self.fragment_uniform_buffer
                .as_ref()
                .expect("fragment uniform must be set before update"),
            0,
            bytemuck::cast_slice(&[fragment_uniform]),
        );
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        destination: &wgpu::TextureView,
        clear_color: Color,
        vertex_buffer: wgpu::BufferSlice,
        vertex_count: u32,
    ) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: destination,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(clear_color.into()),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            occlusion_query_set: None,
            timestamp_writes: None,
        });

        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_bind_group(0, &self.vertex_uniform_bind_group, &[]);
        render_pass.set_bind_group(1, &self.fragment_uniform_bind_group, &[]);
        render_pass.set_bind_group(2, &self.texture_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer);
        render_pass.draw(0..vertex_count, 0..1);
    }
}
