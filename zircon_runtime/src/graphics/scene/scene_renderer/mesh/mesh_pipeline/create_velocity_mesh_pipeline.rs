use crate::graphics::scene::resources::{GpuMeshVertex, PipelineKey};

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_velocity_mesh_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    target_format: wgpu::TextureFormat,
    key: &PipelineKey,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-velocity-mesh-pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_velocity_object"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[
                GpuMeshVertex::layout(),
                GpuMeshVertex::previous_position_layout(),
            ],
        },
        primitive: wgpu::PrimitiveState {
            cull_mode: (!key.double_sided).then_some(wgpu::Face::Back),
            ..Default::default()
        },
        depth_stencil: Some(wgpu::DepthStencilState {
            format: super::super::super::core::DEPTH_FORMAT,
            depth_write_enabled: Some(false),
            depth_compare: Some(wgpu::CompareFunction::LessEqual),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_velocity_object"),
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
    use super::super::FALLBACK_MESH_SHADER;
    use crate::graphics::scene::resources::GpuMeshVertex;

    #[test]
    fn velocity_mesh_pipeline_declares_previous_position_vertex_slot() {
        let source = include_str!("create_velocity_mesh_pipeline.rs");

        assert!(source.contains("GpuMeshVertex::previous_position_layout()"));
        assert_eq!(
            GpuMeshVertex::previous_position_layout().attributes[0].shader_location,
            8
        );
        assert!(FALLBACK_MESH_SHADER.contains("struct VelocityVertexInput"));
        assert!(FALLBACK_MESH_SHADER.contains("@location(8) previous_position"));
        assert!(
            FALLBACK_MESH_SHADER.contains("skin_previous_vertex_position(input.previous_position")
        );
    }
}
