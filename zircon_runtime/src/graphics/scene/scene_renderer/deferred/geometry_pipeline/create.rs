use crate::graphics::scene::resources::GpuMeshVertex;

use super::super::constants::GBUFFER_ALBEDO_FORMAT;
use super::shader_source::DEFERRED_GEOMETRY_SHADER;

pub(in crate::graphics::scene::scene_renderer::deferred) fn create_geometry_pipeline(
    device: &wgpu::Device,
    scene_layout: &wgpu::BindGroupLayout,
    model_layout: &wgpu::BindGroupLayout,
    texture_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let geometry_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-deferred-geometry-shader"),
        source: wgpu::ShaderSource::Wgsl(DEFERRED_GEOMETRY_SHADER.into()),
    });
    let geometry_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-deferred-geometry-layout"),
        bind_group_layouts: &[Some(scene_layout), Some(model_layout), Some(texture_layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-deferred-geometry-pipeline"),
        layout: Some(&geometry_layout),
        vertex: wgpu::VertexState {
            module: &geometry_shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[GpuMeshVertex::layout()],
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: None,
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: super::super::super::core::DEPTH_FORMAT,
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &geometry_shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: GBUFFER_ALBEDO_FORMAT,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}
