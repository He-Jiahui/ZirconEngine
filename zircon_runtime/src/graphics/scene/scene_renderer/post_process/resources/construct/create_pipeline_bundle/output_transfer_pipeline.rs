use super::super::super::shader_sources::OUTPUT_TRANSFER_SHADER;

/// 最终输出转换只依赖终端采样和目标格式；资源精简模式也必须拥有此管线。
pub(in crate::graphics::scene::scene_renderer::post_process::resources::construct) fn output_transfer_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    output_transfer_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-output-transfer-shader"),
        source: wgpu::ShaderSource::Wgsl(OUTPUT_TRANSFER_SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-output-transfer-pipeline-layout"),
        bind_group_layouts: &[Some(output_transfer_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-output-transfer-pipeline"),
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
                format: target_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(test)]
#[path = "tests/output_transfer_pipeline.rs"]
mod tests;
