use crate::graphics::scene::resources::{GpuMeshVertex, PipelineKey};

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_taa_reactive_mask_mesh_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    target_format: wgpu::TextureFormat,
    key: &PipelineKey,
) -> wgpu::RenderPipeline {
    create_taa_reactive_mask_pipeline_with_fragment_entry(
        device,
        layout,
        shader,
        target_format,
        key,
        "zircon-taa-reactive-mask-mesh-pipeline",
        "fs_taa_reactive_mask",
    )
}

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_taa_reactive_material_mask_mesh_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    target_format: wgpu::TextureFormat,
    key: &PipelineKey,
) -> wgpu::RenderPipeline {
    create_taa_reactive_mask_pipeline_with_fragment_entry(
        device,
        layout,
        shader,
        target_format,
        key,
        "zircon-taa-reactive-material-mask-mesh-pipeline",
        "fs_taa_reactive_material_mask",
    )
}

fn create_taa_reactive_mask_pipeline_with_fragment_entry(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    target_format: wgpu::TextureFormat,
    key: &PipelineKey,
    label: &'static str,
    fragment_entry_point: &'static str,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[GpuMeshVertex::layout()],
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: (!key.double_sided).then_some(wgpu::Face::Back),
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
            module: shader,
            entry_point: Some(fragment_entry_point),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}
