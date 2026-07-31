pub(super) fn screen_space_reflection_resolve_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    pipeline_layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-screen-space-reflection-resolve-pipeline"),
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_screen_space_reflection_resolve"),
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
mod tests {
    use super::super::super::super::shader_sources::POST_PROCESS_SHADER;

    #[test]
    fn post_process_shader_exposes_split_ssr_resolve_entry_point() {
        assert!(POST_PROCESS_SHADER.contains("fn fs_screen_space_reflection_resolve"));
        assert!(POST_PROCESS_SHADER.contains("-> @location(0) vec4<f32>"));
        assert!(POST_PROCESS_SHADER.contains("resolve_screen_space_reflection_history"));
    }
}
