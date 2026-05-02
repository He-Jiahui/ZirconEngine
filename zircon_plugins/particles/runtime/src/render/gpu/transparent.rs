use zircon_runtime::core::math::Vec3;

use super::program::{
    ParticleGpuTransparentShaderEntries, PARTICLE_GPU_TRANSPARENT_RENDER_PARAMS_BYTES,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParticleGpuTransparentRenderConfig {
    pub target_format: wgpu::TextureFormat,
    pub depth_format: wgpu::TextureFormat,
}

impl ParticleGpuTransparentRenderConfig {
    pub const fn new(
        target_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
        Self {
            target_format,
            depth_format,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParticleGpuTransparentRenderParams {
    pub camera_right: Vec3,
    pub camera_up: Vec3,
    pub intensity: f32,
}

impl ParticleGpuTransparentRenderParams {
    pub const ENCODED_SIZE: usize = PARTICLE_GPU_TRANSPARENT_RENDER_PARAMS_BYTES as usize;

    pub fn new(camera_right: Vec3, camera_up: Vec3, intensity: f32) -> Self {
        Self {
            camera_right,
            camera_up,
            intensity,
        }
    }

    pub fn encode(self) -> [u8; Self::ENCODED_SIZE] {
        let mut bytes = [0u8; Self::ENCODED_SIZE];
        write_f32s(
            &mut bytes,
            0,
            &[
                self.camera_right.x,
                self.camera_right.y,
                self.camera_right.z,
                self.intensity,
                self.camera_up.x,
                self.camera_up.y,
                self.camera_up.z,
                0.0,
            ],
        );
        bytes
    }
}

pub(super) struct ParticleGpuTransparentRenderer {
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
    render_params_buffer: wgpu::Buffer,
    render_bind_groups: Vec<wgpu::BindGroup>,
}

impl ParticleGpuTransparentRenderer {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        device: &wgpu::Device,
        scene_layout: &wgpu::BindGroupLayout,
        config: ParticleGpuTransparentRenderConfig,
        shader_wgsl: &str,
        entries: &ParticleGpuTransparentShaderEntries,
        particle_buffers: &[wgpu::Buffer],
        alive_indices_buffer: &wgpu::Buffer,
    ) -> Self {
        let bind_group_layout = create_render_bind_group_layout(device);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-particle-gpu-transparent-shader"),
            source: wgpu::ShaderSource::Wgsl(shader_wgsl.into()),
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-particle-gpu-transparent-pipeline-layout"),
            bind_group_layouts: &[Some(scene_layout), Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-particle-gpu-transparent-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some(entries.vertex),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: config.depth_format,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some(entries.fragment),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.target_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let render_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-particle-gpu-transparent-render-params"),
            size: PARTICLE_GPU_TRANSPARENT_RENDER_PARAMS_BYTES,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });
        let render_bind_groups = particle_buffers
            .iter()
            .enumerate()
            .map(|(index, particle_buffer)| {
                create_render_bind_group(
                    device,
                    &bind_group_layout,
                    match index {
                        0 => "zircon-particle-gpu-transparent-a",
                        _ => "zircon-particle-gpu-transparent-b",
                    },
                    particle_buffer,
                    alive_indices_buffer,
                    &render_params_buffer,
                )
            })
            .collect();

        Self {
            bind_group_layout,
            pipeline,
            render_params_buffer,
            render_bind_groups,
        }
    }

    pub(super) fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn record(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        depth_view: &wgpu::TextureView,
        scene_bind_group: &wgpu::BindGroup,
        active_buffer_index: usize,
        indirect_draw_args_buffer: &wgpu::Buffer,
        params: ParticleGpuTransparentRenderParams,
    ) {
        queue.write_buffer(&self.render_params_buffer, 0, &params.encode());

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("ParticleGpuTransparentPass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, scene_bind_group, &[]);
        pass.set_bind_group(1, &self.render_bind_groups[active_buffer_index], &[]);
        pass.draw_indirect(indirect_draw_args_buffer, 0);
    }
}

fn create_render_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-particle-gpu-transparent-bind-group-layout"),
        entries: &[
            render_storage_entry(0),
            render_storage_entry(1),
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

fn render_storage_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn create_render_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    label: &'static str,
    particles: &wgpu::Buffer,
    alive_indices: &wgpu::Buffer,
    render_params: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: particles.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: alive_indices.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: render_params.as_entire_binding(),
            },
        ],
    })
}

fn write_f32s(bytes: &mut [u8], start: usize, values: &[f32]) {
    for (index, value) in values.iter().enumerate() {
        let byte_start = start + index * std::mem::size_of::<f32>();
        bytes[byte_start..byte_start + std::mem::size_of::<f32>()]
            .copy_from_slice(&value.to_le_bytes());
    }
}
