use crate::scene::resources::GpuMeshVertex;

use super::super::constants::NORMAL_FORMAT;
use super::super::normal_prepass_shader_source::NORMAL_PREPASS_SHADER;
use super::normal_prepass_pipeline::NormalPrepassPipeline;

impl NormalPrepassPipeline {
    pub(crate) fn new(
        device: &wgpu::Device,
        scene_layout: &wgpu::BindGroupLayout,
        model_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-normal-prepass-shader"),
            source: wgpu::ShaderSource::Wgsl(NORMAL_PREPASS_SHADER.into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-normal-prepass-layout"),
            bind_group_layouts: &[Some(scene_layout), Some(model_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("zircon-normal-prepass-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[GpuMeshVertex::layout()],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: super::super::super::core::DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::LessEqual),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: NORMAL_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });

        Self { pipeline }
    }
}
