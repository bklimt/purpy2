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

    texture_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_groups: Vec<Option<wgpu::BindGroup>>,
}

impl Pipeline {
    pub fn new(
        label: &str,
        device: &wgpu::Device,
        shader: &wgpu::ShaderModule,
        vertex_shader_entry_point: &str,
        fragment_shader_entry_point: &str,
        vertex_buffer_layout: wgpu::VertexBufferLayout,
        texture_count: usize,
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
            &device,
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
            format!("{} Fragment", label).as_str(),
            &device,
            default_uniform,
            &fragment_uniform_bind_group_layout,
        );

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(format!("[{}] Texture Bind Group Layout", label).as_str()),
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
            });

        let mut bind_group_layouts = Vec::new();
        bind_group_layouts.push(&vertex_uniform_bind_group_layout);
        bind_group_layouts.push(&fragment_uniform_bind_group_layout);
        for _ in 0..texture_count {
            bind_group_layouts.push(&texture_bind_group_layout);
        }

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(format!("[{}] Render Pipeline Layout", label).as_str()),
                bind_group_layouts: &bind_group_layouts,
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some(format!("[{}] Render Pipeline", label).as_str()),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: vertex_shader_entry_point,
                buffers: &[vertex_buffer_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: fragment_shader_entry_point,
                targets: &[Some(wgpu::ColorTargetState {
                    format: format,
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

        let label = label.to_owned();
        let mut texture_bind_groups = Vec::new();
        texture_bind_groups.resize_with(texture_count, || None);

        Ok(Self {
            label,
            render_pipeline,
            vertex_uniform_bind_group_layout,
            vertex_uniform_bind_group,
            fragment_uniform_bind_group_layout,
            fragment_uniform_bind_group,
            texture_bind_group_layout,
            texture_bind_groups,
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
        let fragment_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(format!("[{}] Fragment Uniform Buffer", self.label).as_str()),
                contents: bytemuck::cast_slice(&[fragment_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        self.fragment_uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("[{}] Fragment Uniform_Bind_Group"),
            layout: &self.fragment_uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: fragment_uniform_buffer.as_entire_binding(),
            }],
        });
    }

    pub fn set_texture(&mut self, device: &wgpu::Device, index: usize, texture: &Texture) {
        self.texture_bind_groups[index] =
            Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(format!("[{}] Texture Bind Group", self.label).as_str()),
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
            }));
    }

    pub fn render(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        destination: &wgpu::TextureView,
        clear_color: Color,
        vertex_buffer: wgpu::BufferSlice,
        vertex_count: u32,
    ) {
        let texture_bind_groups: Vec<&wgpu::BindGroup> = self
            .texture_bind_groups
            .iter()
            .map(|group: &Option<wgpu::BindGroup>| group.as_ref().expect("Texture was not set."))
            .collect();

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &destination,
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
        for (i, texture_bind_group) in texture_bind_groups.iter().enumerate() {
            render_pass.set_bind_group(2 + i as u32, texture_bind_group, &[]);
        }
        render_pass.set_vertex_buffer(0, vertex_buffer);
        render_pass.draw(0..vertex_count, 0..1);
    }
}
