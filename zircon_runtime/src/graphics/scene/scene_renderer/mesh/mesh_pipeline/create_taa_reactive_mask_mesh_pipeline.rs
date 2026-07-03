use crate::graphics::scene::resources::{GpuMeshVertex, PipelineKey};

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_taa_reactive_mask_mesh_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    target_format: wgpu::TextureFormat,
    key: &PipelineKey,
) -> wgpu::RenderPipeline {
    create_taa_reactive_mask_pipeline_with_fragment_entry(
        device,
        layout,
        shader,
        target_format,
        key,
        "zircon-taa-reactive-mask-mesh-pipeline",
        "fs_taa_reactive_mask",
    )
}

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_taa_reactive_material_mask_mesh_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    target_format: wgpu::TextureFormat,
    key: &PipelineKey,
) -> wgpu::RenderPipeline {
    create_taa_reactive_mask_pipeline_with_fragment_entry(
        device,
        layout,
        shader,
        target_format,
        key,
        "zircon-taa-reactive-material-mask-mesh-pipeline",
        "fs_taa_reactive_material_mask",
    )
}

fn create_taa_reactive_mask_pipeline_with_fragment_entry(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    target_format: wgpu::TextureFormat,
    key: &PipelineKey,
    label: &'static str,
    fragment_entry_point: &'static str,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[GpuMeshVertex::layout()],
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
            entry_point: Some(fragment_entry_point),
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
    use std::borrow::Cow;

    use super::super::test_support::create_standard_mesh_pipeline_layout;
    use super::{
        create_taa_reactive_mask_mesh_pipeline, create_taa_reactive_material_mask_mesh_pipeline,
    };
    use crate::core::framework::render::GEOMETRY_SOURCE_ID_STATIC_MESH;
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::mesh_pipeline_taa_reactive_mask_template_source_for_geometry;

    #[test]
    fn taa_reactive_mask_pipeline_declares_template_entries_and_static_vertex_layout() {
        let source = include_str!("create_taa_reactive_mask_mesh_pipeline.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source section should exist");

        assert!(source.contains("GpuMeshVertex::layout()"));
        assert!(!source.contains("GpuMeshVertex::previous_position_layout()"));
        assert!(source.contains("entry_point: Some(\"vs_main\")"));
        assert!(source.contains("\"fs_taa_reactive_mask\""));
        assert!(source.contains("\"fs_taa_reactive_material_mask\""));

        let key = default_pipeline_key();
        let source = mesh_pipeline_taa_reactive_mask_template_source_for_geometry(
            &key,
            key.shader_variant_key(
                crate::core::framework::render::ShaderPassType::Forward,
                "wgpu-runtime",
            )
            .geometry_source,
        )
        .expect("TAA reactive mask template source should assemble");

        assert!(source.wgsl_source.contains("fn vs_main("));
        assert!(source.wgsl_source.contains("fn fs_taa_reactive_mask("));
        assert!(source
            .wgsl_source
            .contains("fn fs_taa_reactive_material_mask("));
        assert!(source.wgsl_source.contains("surface.custom0.x"));
    }

    #[test]
    fn taa_reactive_mask_mesh_pipeline_creates_on_wgpu_device_with_template_shader() {
        let Ok(backend) = crate::graphics::backend::RenderBackend::new_offscreen() else {
            return;
        };
        let device = &backend.device;
        let key = default_pipeline_key();
        let shader_source = mesh_pipeline_taa_reactive_mask_template_source_for_geometry(
            &key,
            GEOMETRY_SOURCE_ID_STATIC_MESH,
        )
        .expect("TAA reactive mask template source should assemble");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-test-taa-reactive-mask-template-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source.wgsl_source)),
        });
        let pipeline_layout = create_standard_mesh_pipeline_layout(device, "taa-reactive-mask");

        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let _reactive_pipeline = create_taa_reactive_mask_mesh_pipeline(
            device,
            &pipeline_layout,
            &shader,
            wgpu::TextureFormat::R8Unorm,
            &key,
        );
        let _material_pipeline = create_taa_reactive_material_mask_mesh_pipeline(
            device,
            &pipeline_layout,
            &shader,
            wgpu::TextureFormat::R8Unorm,
            &key,
        );
        let error = pollster::block_on(error_scope.pop());

        assert!(
            error.is_none(),
            "TAA reactive mask template pipelines should pass WGPU validation: {error:?}"
        );
    }
}
