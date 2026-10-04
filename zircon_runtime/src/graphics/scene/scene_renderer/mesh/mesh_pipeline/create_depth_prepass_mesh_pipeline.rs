use crate::graphics::scene::resources::{GpuMeshVertex, PipelineKey};

/// 为颜色 pass 之前的深度预通过创建管线；alpha-mask 复用材质裁剪，同时保持深度专用附件契约。
pub(in crate::graphics::scene::scene_renderer::mesh) fn create_depth_prepass_mesh_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    key: &PipelineKey,
    pipeline_cache: Option<&wgpu::PipelineCache>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-depth-prepass-mesh-pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[GpuMeshVertex::layout()],
        },
        primitive: wgpu::PrimitiveState {
            front_face: super::mesh_front_face(key),
            cull_mode: (!key.double_sided).then_some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: super::super::super::core::DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: if key.is_alpha_mask() {
            Some(wgpu::FragmentState {
                module: shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[],
            })
        } else {
            None
        },
        multiview_mask: None,
        cache: pipeline_cache,
    })
}

#[cfg(test)]
#[path = "tests/create_depth_prepass_mesh_pipeline.rs"]
mod tests;
