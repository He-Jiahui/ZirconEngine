//! 创建屏幕空间反射粗粒度反射金字塔管线；输出格式必须与对应中间纹理分配一致。
use crate::graphics::scene::scene_renderer::post_process::SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE_FORMAT;

/// 复用共享后处理 shader module 和 pipeline layout，生成粗粒度反射金字塔的渲染管线。
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
#[path = "tests/screen_space_reflection_reflection_pyramid_coarse_pipeline.rs"]
mod tests;
