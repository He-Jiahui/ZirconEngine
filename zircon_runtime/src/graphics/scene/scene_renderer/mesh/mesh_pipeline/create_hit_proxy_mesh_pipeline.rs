use crate::graphics::scene::resources::{GpuMeshVertex, PipelineKey};

pub(crate) const HIT_PROXY_TOKEN_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::R32Uint;
pub(crate) const HIT_PROXY_WORLD_POSITION_DEPTH_FORMAT: wgpu::TextureFormat =
    wgpu::TextureFormat::Rgba32Float;
pub(crate) const HIT_PROXY_WORLD_NORMAL_FORMAT: wgpu::TextureFormat =
    wgpu::TextureFormat::Rgba16Float;

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_hit_proxy_mesh_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    key: &PipelineKey,
    pipeline_cache: Option<&wgpu::PipelineCache>,
) -> wgpu::RenderPipeline {
    let targets = [
        Some(wgpu::ColorTargetState {
            format: HIT_PROXY_TOKEN_FORMAT,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        }),
        Some(wgpu::ColorTargetState {
            format: HIT_PROXY_WORLD_POSITION_DEPTH_FORMAT,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        }),
        Some(wgpu::ColorTargetState {
            format: HIT_PROXY_WORLD_NORMAL_FORMAT,
            blend: None,
            write_mask: wgpu::ColorWrites::ALL,
        }),
    ];
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-hit-proxy-mesh-pipeline"),
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
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &targets,
        }),
        multiview_mask: None,
        cache: pipeline_cache,
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn hit_proxy_pipeline_owns_exact_unblended_products_and_depth_visibility() {
        let source = include_str!("create_hit_proxy_mesh_pipeline.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source section should exist");

        assert!(source.contains("TextureFormat::R32Uint"));
        assert!(source.contains("TextureFormat::Rgba32Float"));
        assert!(source.contains("TextureFormat::Rgba16Float"));
        assert_eq!(source.matches("blend: None").count(), 3);
        assert!(source.contains("depth_write_enabled: Some(true)"));
        assert!(source.contains("CompareFunction::LessEqual"));
        assert!(source.contains("front_face: super::mesh_front_face(key)"));
        assert!(source.contains("(!key.double_sided).then_some(wgpu::Face::Back)"));
        assert!(source.contains("entry_point: Some(\"fs_main\")"));
    }
}
