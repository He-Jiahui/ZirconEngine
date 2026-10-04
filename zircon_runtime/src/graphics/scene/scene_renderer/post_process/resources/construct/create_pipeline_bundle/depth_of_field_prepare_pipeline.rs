//! 创建景深预处理管线；后端深度模式会在编译前替换输入声明和采样表达式。
use super::super::super::depth_sampling_mode::PostProcessDepthSamplingMode;

const DEPTH_OF_FIELD_PREPARE_SHADER: &str =
    include_str!("../../../shaders/depth_of_field_prepare.wgsl");
const DEPTH_OF_FIELD_COC_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// 构建同时输出 CoC 与散景预滤波纹理的景深预处理管线，并保持深度回退布局一致。
pub(super) fn depth_of_field_prepare_pipeline(
    device: &wgpu::Device,
    bokeh_target_format: wgpu::TextureFormat,
    depth_of_field_prepare_bind_group_layout: &wgpu::BindGroupLayout,
    depth_sampling_mode: PostProcessDepthSamplingMode,
) -> wgpu::RenderPipeline {
    let shader_source =
        depth_sampling_mode.depth_of_field_prepare_shader_source(DEPTH_OF_FIELD_PREPARE_SHADER);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-depth-of-field-prepare-shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-depth-of-field-prepare-pipeline-layout"),
        bind_group_layouts: &[Some(depth_of_field_prepare_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-depth-of-field-prepare-pipeline"),
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
            targets: &[
                Some(wgpu::ColorTargetState {
                    format: DEPTH_OF_FIELD_COC_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: bokeh_target_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
            ],
        }),
        multiview_mask: None,
        cache: None,
    })
}

#[cfg(test)]
#[path = "tests/depth_of_field_prepare_pipeline.rs"]
mod tests;
