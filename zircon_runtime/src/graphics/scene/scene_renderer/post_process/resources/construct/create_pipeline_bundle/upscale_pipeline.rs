use crate::graphics::scene::scene_renderer::post_process::POST_PROCESS_TONEMAPPED_FORMAT;

use super::super::super::shader_sources::UPSCALE_SHADER;

/// 把逻辑视口放大到中间色调映射目标；真正的显示设备格式转换由终端管线完成。
pub(super) fn upscale_pipeline(
    device: &wgpu::Device,
    upscale_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-upscale-shader"),
        source: wgpu::ShaderSource::Wgsl(UPSCALE_SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-upscale-pipeline-layout"),
        bind_group_layouts: &[Some(upscale_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-upscale-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: POST_PROCESS_TONEMAPPED_FORMAT,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(test)]
#[path = "tests/upscale_pipeline.rs"]
mod tests;
