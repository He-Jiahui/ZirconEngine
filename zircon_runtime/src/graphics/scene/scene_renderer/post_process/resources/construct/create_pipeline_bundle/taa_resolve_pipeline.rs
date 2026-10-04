//! 创建 TAA resolve 管线；后端深度模式决定 shader 的深度绑定类型和读取实现。
use super::super::super::depth_sampling_mode::PostProcessDepthSamplingMode;
use crate::graphics::scene::scene_renderer::post_process::POST_PROCESS_INTERMEDIATE_HDR_FORMAT;

const TAA_RESOLVE_SHADER: &str = include_str!("../../../../temporal/taa/shaders/taa_resolve.wgsl");
const TAA_OUTPUT_FORMAT: wgpu::TextureFormat = POST_PROCESS_INTERMEDIATE_HDR_FORMAT;
const TAA_HISTORY_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

/// 同时写入当前 HDR 结果与历史纹理；两项附件分别须匹配本管线声明的输出与历史格式。
pub(super) fn taa_resolve_pipeline(
    device: &wgpu::Device,
    taa_resolve_bind_group_layout: &wgpu::BindGroupLayout,
    depth_sampling_mode: PostProcessDepthSamplingMode,
) -> wgpu::RenderPipeline {
    let shader_source = depth_sampling_mode.taa_resolve_shader_source(TAA_RESOLVE_SHADER);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-taa-resolve-shader"),
        source: wgpu::ShaderSource::Wgsl(shader_source),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-taa-resolve-pipeline-layout"),
        bind_group_layouts: &[Some(taa_resolve_bind_group_layout)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-taa-resolve-pipeline"),
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
            entry_point: Some("fs_taa_resolve"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[
                Some(wgpu::ColorTargetState {
                    format: TAA_OUTPUT_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: TAA_HISTORY_FORMAT,
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
#[path = "tests/taa_resolve_pipeline.rs"]
mod tests;
