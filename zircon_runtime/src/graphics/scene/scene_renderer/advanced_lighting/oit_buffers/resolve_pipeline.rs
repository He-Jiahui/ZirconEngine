use wgpu::util::DeviceExt;

use crate::core::framework::render::{OitBufferPlan, OitSettings};
use crate::graphics::types::ViewportRenderRegion;

use super::OIT_RESOLVE_SHADER_SOURCE;

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct OitGpuSettings {
    viewport_width: u32,
    viewport_height: u32,
    viewport_origin_x: u32,
    viewport_origin_y: u32,
    fragments_per_pixel: u32,
    sorted_fragment_max_count: u32,
    alpha_threshold: f32,
    _padding: u32,
}

pub(super) struct OitResolvePipeline {
    target_format: wgpu::TextureFormat,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::RenderPipeline,
}

impl OitResolvePipeline {
    pub(super) fn new(device: &wgpu::Device, target_format: wgpu::TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-oit-resolve-bind-group-layout"),
            entries: &[
                storage_entry(0, true),
                storage_entry(1, true),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-oit-resolve-pipeline-layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-oit-resolve-shader"),
            source: wgpu::ShaderSource::Wgsl(OIT_RESOLVE_SHADER_SOURCE.into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-oit-resolve-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: target_format,
                    blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        Self {
            target_format,
            bind_group_layout,
            pipeline,
        }
    }

    pub(super) const fn target_format(&self) -> wgpu::TextureFormat {
        self.target_format
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        output: &wgpu::TextureView,
        layers: &wgpu::Buffer,
        counts: &wgpu::Buffer,
        render_region: ViewportRenderRegion,
        settings: OitSettings,
    ) {
        let viewport_size = render_region.physical_size();
        let plan = OitBufferPlan::for_view([viewport_size.x, viewport_size.y], settings);
        let gpu_settings = OitGpuSettings {
            viewport_width: viewport_size.x.max(1),
            viewport_height: viewport_size.y.max(1),
            viewport_origin_x: render_region.physical_position().x,
            viewport_origin_y: render_region.physical_position().y,
            fragments_per_pixel: plan.fragments_per_pixel_capacity,
            sorted_fragment_max_count: settings.sorted_fragment_max_count.clamp(1, 32),
            alpha_threshold: settings.alpha_threshold.clamp(0.0, 1.0),
            _padding: 0,
        };
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("zircon-oit-resolve-settings"),
            contents: bytemuck::bytes_of(&gpu_settings),
            usage: wgpu::BufferUsages::UNIFORM,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("zircon-oit-resolve-bind-group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: layers.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: counts.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: uniform.as_entire_binding(),
                },
            ],
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("oit.resolve"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: output,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        if render_region.apply_physical_to_render_pass(&mut pass) {
            pass.draw(0..3, 0..1);
        }
    }
}

fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}
