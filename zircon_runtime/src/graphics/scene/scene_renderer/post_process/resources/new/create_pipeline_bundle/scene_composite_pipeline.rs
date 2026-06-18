use super::super::super::depth_sampling_mode::PostProcessDepthSamplingMode;
use super::super::super::shader_sources::POST_PROCESS_SHADER;

pub(super) fn scene_composite_pipeline(
    device: &wgpu::Device,
    target_format: wgpu::TextureFormat,
    post_process_bind_group_layout: &wgpu::BindGroupLayout,
    depth_sampling_mode: PostProcessDepthSamplingMode,
) -> wgpu::RenderPipeline {
    let shader_source = depth_sampling_mode.post_process_shader_source(POST_PROCESS_SHADER);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-scene-composite-shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-scene-composite-pipeline-layout"),
        bind_group_layouts: &[Some(post_process_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-scene-composite-pipeline"),
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
            entry_point: Some("fs_scene_composite"),
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
    fn scene_composite_shader_entry_is_split_from_uber() {
        assert!(POST_PROCESS_SHADER.contains("fn fs_scene_composite"));
        assert!(POST_PROCESS_SHADER.contains("apply_scene_composite"));
    }
}
