use crate::graphics::scene::resources::{GpuMeshVertex, PipelineKey};

use super::super::mesh_pass::MeshPassPipelineKind;

const SHADOW_DEPTH_BIAS_CONSTANT: i32 = 2;
const SHADOW_DEPTH_BIAS_SLOPE_SCALE: f32 = 2.0;
const SHADOW_DEPTH_BIAS_CLAMP: f32 = 0.0;

/// 供阴影 atlas 的深度 pass 使用；alpha-mask 增加材质裁剪，其余变体只运行顶点阶段。
pub(in crate::graphics::scene::scene_renderer::mesh) fn create_shadow_mesh_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    kind: MeshPassPipelineKind,
    key: &PipelineKey,
    pipeline_cache: Option<&wgpu::PipelineCache>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(shadow_pipeline_label(kind)),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[GpuMeshVertex::layout()],
        },
        primitive: shadow_primitive_state(key),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: super::super::super::core::DEPTH_FORMAT,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState {
                constant: SHADOW_DEPTH_BIAS_CONSTANT,
                slope_scale: SHADOW_DEPTH_BIAS_SLOPE_SCALE,
                clamp: SHADOW_DEPTH_BIAS_CLAMP,
            },
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: shadow_fragment_state(shader, kind),
        multiview_mask: None,
        cache: pipeline_cache,
    })
}

fn shadow_primitive_state(key: &PipelineKey) -> wgpu::PrimitiveState {
    wgpu::PrimitiveState {
        front_face: super::mesh_front_face(key),
        cull_mode: (!key.double_sided).then_some(wgpu::Face::Back),
        ..wgpu::PrimitiveState::default()
    }
}

fn shadow_fragment_state(
    shader: &wgpu::ShaderModule,
    kind: MeshPassPipelineKind,
) -> Option<wgpu::FragmentState<'_>> {
    if kind != MeshPassPipelineKind::ShadowDepthAlphaMask {
        return None;
    }
    Some(wgpu::FragmentState {
        module: shader,
        entry_point: Some("fs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        targets: &[],
    })
}

fn shadow_pipeline_label(kind: MeshPassPipelineKind) -> &'static str {
    match kind {
        MeshPassPipelineKind::ShadowDepth => "zircon-shadow-depth-mesh-pipeline",
        MeshPassPipelineKind::ShadowDepthAlphaMask => "zircon-shadow-alpha-mask-mesh-pipeline",
        _ => "zircon-shadow-mesh-pipeline",
    }
}

#[cfg(test)]
#[path = "tests/create_shadow_mesh_pipeline.rs"]
mod tests;
