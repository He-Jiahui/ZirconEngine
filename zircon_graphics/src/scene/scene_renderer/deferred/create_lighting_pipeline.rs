use super::deferred_lighting_shader_source::DEFERRED_LIGHTING_SHADER;

pub(in crate::scene::scene_renderer::deferred) fn create_lighting_pipeline(
    device: &wgpu::Device,
    scene_layout: &wgpu::BindGroupLayout,
    lighting_bind_group_layout: &wgpu::BindGroupLayout,
    target_format: wgpu::TextureFormat,
) -> wgpu::RenderPipeline {
    let lighting_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-deferred-lighting-shader"),
        source: wgpu::ShaderSource::Wgsl(DEFERRED_LIGHTING_SHADER.into()),
    });
    let lighting_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-deferred-lighting-layout"),
        bind_group_layouts: &[scene_layout, lighting_bind_group_layout],
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-deferred-lighting-pipeline"),
        layout: Some(&lighting_layout),
        vertex: wgpu::VertexState {
            module: &lighting_shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &lighting_shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    })
}
