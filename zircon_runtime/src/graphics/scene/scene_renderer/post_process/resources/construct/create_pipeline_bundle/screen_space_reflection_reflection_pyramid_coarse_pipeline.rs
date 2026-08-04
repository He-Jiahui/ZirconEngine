use crate::graphics::scene::scene_renderer::post_process::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE_FORMAT;

pub(super) fn screen_space_reflection_reflection_pyramid_coarse_pipeline(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-screen-space-reflection-reflection-pyramid-coarse-pipeline"),
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
            entry_point: Some("fs_screen_space_reflection_reflection_pyramid_coarse"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE_FORMAT,
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
    fn post_process_shader_exposes_split_ssr_reflection_pyramid_coarse_entry_point() {
        assert!(
            POST_PROCESS_SHADER.contains("fn fs_screen_space_reflection_reflection_pyramid_coarse")
        );
        assert!(
            POST_PROCESS_SHADER
                .contains("resolve_screen_space_reflection_reflection_pyramid_coarse")
        );
        assert!(
            POST_PROCESS_SHADER.contains("screen_space_reflection_reflection_pyramid_coarse_tex")
        );
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(26)"));
    }
}
