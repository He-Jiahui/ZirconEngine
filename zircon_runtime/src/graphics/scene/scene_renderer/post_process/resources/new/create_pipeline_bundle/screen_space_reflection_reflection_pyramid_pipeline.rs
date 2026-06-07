use super::super::super::depth_sampling_mode::PostProcessDepthSamplingMode;
use super::super::super::shader_sources::POST_PROCESS_SHADER;
use crate::graphics::scene::scene_renderer::post_process::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_FORMAT;

pub(super) fn screen_space_reflection_reflection_pyramid_pipeline(
    device: &wgpu::Device,
    post_process_bind_group_layout: &wgpu::BindGroupLayout,
    depth_sampling_mode: PostProcessDepthSamplingMode,
) -> wgpu::RenderPipeline {
    let shader_source = depth_sampling_mode.post_process_shader_source(POST_PROCESS_SHADER);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-screen-space-reflection-reflection-pyramid-shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-screen-space-reflection-reflection-pyramid-pipeline-layout"),
        bind_group_layouts: &[Some(post_process_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-screen-space-reflection-reflection-pyramid-pipeline"),
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
            entry_point: Some("fs_screen_space_reflection_reflection_pyramid"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_FORMAT,
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
    fn post_process_shader_exposes_split_ssr_reflection_pyramid_entry_point() {
        assert!(POST_PROCESS_SHADER.contains("fn fs_screen_space_reflection_reflection_pyramid"));
        assert!(POST_PROCESS_SHADER.contains("resolve_screen_space_reflection_reflection_pyramid"));
        assert!(POST_PROCESS_SHADER.contains("screen_space_reflection_reflection_pyramid_tex"));
        assert!(POST_PROCESS_SHADER.contains("@group(0) @binding(24)"));
    }
}
