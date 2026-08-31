use crate::graphics::scene::resources::{GpuMeshVertex, PipelineKey};

pub(crate) const MESH_VELOCITY_TARGET_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rg16Float;

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_velocity_mesh_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    key: &PipelineKey,
    pipeline_cache: Option<&wgpu::PipelineCache>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-velocity-mesh-pipeline"),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[
                GpuMeshVertex::layout(),
                GpuMeshVertex::previous_position_layout(),
            ],
        },
        primitive: wgpu::PrimitiveState {
            front_face: super::mesh_front_face(key),
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
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: MESH_VELOCITY_TARGET_FORMAT,
                blend: Some(wgpu::BlendState::REPLACE),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: pipeline_cache,
    })
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::super::test_support::create_standard_mesh_pipeline_layout;
    use super::create_velocity_mesh_pipeline;
    use crate::core::framework::render::GEOMETRY_SOURCE_ID_STATIC_MESH;
    use crate::graphics::scene::resources::GpuMeshVertex;
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::mesh_pipeline_velocity_template_source_for_geometry;

    #[test]
    fn velocity_mesh_pipeline_declares_template_entries_and_previous_position_vertex_slot() {
        let source = include_str!("create_velocity_mesh_pipeline.rs");

        assert!(source.contains("GpuMeshVertex::previous_position_layout()"));
        assert!(source.contains("entry_point: Some(\"vs_main\")"));
        assert!(source.contains("entry_point: Some(\"fs_main\")"));
        assert_eq!(
            GpuMeshVertex::previous_position_layout().attributes[0].shader_location,
            8
        );
        let key = default_pipeline_key();
        let source = mesh_pipeline_velocity_template_source_for_geometry(
            &key,
            key.shader_variant_key(
                crate::core::framework::render::ShaderPassType::Velocity,
                "wgpu-runtime",
            )
            .geometry_source,
        )
        .expect("velocity template source should assemble");

        assert!(source.wgsl_source.contains("struct ZrVelocityVertexInput"));
        assert!(
            source
                .wgsl_source
                .contains("@location(8) previous_position")
        );
        assert!(source.wgsl_source.contains("fn vs_main("));
        assert!(source.wgsl_source.contains("fn fs_main("));
    }

    #[test]
    fn velocity_mesh_pipeline_creates_on_wgpu_device_with_template_shader() {
        let Ok(backend) = crate::graphics::backend::RenderBackend::new_offscreen() else {
            return;
        };
        let device = &backend.device;
        let key = default_pipeline_key();
        let shader_source = mesh_pipeline_velocity_template_source_for_geometry(
            &key,
            GEOMETRY_SOURCE_ID_STATIC_MESH,
        )
        .expect("velocity template source should assemble");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-test-velocity-template-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source.wgsl_source)),
        });
        let pipeline_layout = create_standard_mesh_pipeline_layout(device, "velocity");

        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let _pipeline =
            create_velocity_mesh_pipeline(device, &pipeline_layout, &shader, &key, None);
        let error = pollster::block_on(error_scope.pop());

        assert!(
            error.is_none(),
            "velocity template pipeline should pass WGPU validation: {error:?}"
        );
    }
}
