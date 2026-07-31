use crate::graphics::scene::resources::{GpuMeshVertex, PipelineKey};
use crate::graphics::scene::scene_renderer::deferred::{
    GBUFFER_ALBEDO_FORMAT, GBUFFER_EMISSIVE_FORMAT, GBUFFER_MATERIAL_FORMAT,
};
use crate::graphics::scene::scene_renderer::prepass::NORMAL_FORMAT;

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_gbuffer_mesh_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    key: &PipelineKey,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("zircon-gbuffer-mesh-pipeline"),
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
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[
                Some(wgpu::ColorTargetState {
                    format: GBUFFER_ALBEDO_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: NORMAL_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: GBUFFER_MATERIAL_FORMAT,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                }),
                Some(wgpu::ColorTargetState {
                    format: GBUFFER_EMISSIVE_FORMAT,
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
mod tests {
    use std::borrow::Cow;
    use std::sync::Arc;

    use crate::core::framework::render::GEOMETRY_SOURCE_ID_STATIC_MESH;
    use crate::graphics::scene::gpu_scene::GpuScene;
    use crate::graphics::scene::resources::{GPU_MATERIAL_UNIFORM_MIN_SIZE, default_pipeline_key};
    use crate::graphics::scene::scene_renderer::environment::scene_bind_group_layout_entries;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::{
        create_forward_shadow_receiver_layout,
        mesh_pipeline_deferred_gbuffer_template_source_for_geometry,
    };

    use super::create_gbuffer_mesh_pipeline;

    const TEST_SKINNED_JOINT_MATRIX_COUNT: u64 = 256;
    const TEST_SKINNED_JOINT_MATRIX_BYTES: u64 = 64;
    const TEST_SKINNED_JOINT_PARAMS_BYTES: u64 = 16;

    #[test]
    fn gbuffer_mesh_pipeline_declares_albedo_material_targets_and_static_layout() {
        let source = include_str!("create_gbuffer_mesh_pipeline.rs");

        assert!(source.contains("zircon-gbuffer-mesh-pipeline"));
        assert!(source.contains("GBUFFER_ALBEDO_FORMAT"));
        assert!(source.contains("NORMAL_FORMAT"));
        assert!(source.contains("GBUFFER_MATERIAL_FORMAT"));
        assert!(source.contains("GBUFFER_EMISSIVE_FORMAT"));
        assert!(source.contains("depth_write_enabled: Some(false)"));
        assert!(source.contains("entry_point: Some(\"vs_main\")"));
        assert!(source.contains("entry_point: Some(\"fs_main\")"));
        assert!(source.contains("GpuMeshVertex::layout()"));
    }

    #[test]
    fn gbuffer_mesh_pipeline_creates_on_wgpu_device_with_template_shader() {
        let Ok(backend) = crate::graphics::backend::RenderBackend::new_offscreen() else {
            return;
        };
        let device = &backend.device;
        let key = default_pipeline_key();
        let shader_source = mesh_pipeline_deferred_gbuffer_template_source_for_geometry(
            &key,
            GEOMETRY_SOURCE_ID_STATIC_MESH,
        )
        .expect("deferred G-buffer template source should assemble");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-test-deferred-gbuffer-template-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source.wgsl_source)),
        });
        let scene_layout = create_test_scene_layout(device);
        let shadow_receiver_layout = create_forward_shadow_receiver_layout(device);
        let material_layout = create_test_material_layout(device);
        let gpu_scene = create_test_gpu_scene(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-test-gbuffer-mesh-layout"),
            bind_group_layouts: &[
                Some(&scene_layout),
                Some(&shadow_receiver_layout),
                Some(&material_layout),
                Some(gpu_scene.scene_bind_group_layout()),
            ],
            immediate_size: 0,
        });

        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let _pipeline = create_gbuffer_mesh_pipeline(device, &pipeline_layout, &shader, &key);
        let error = pollster::block_on(error_scope.pop());

        assert!(
            error.is_none(),
            "deferred G-buffer template pipeline should pass WGPU validation: {error:?}"
        );
    }

    fn create_test_scene_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let scene_layout_entries = scene_bind_group_layout_entries();
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-test-scene-layout"),
            entries: &scene_layout_entries,
        })
    }

    fn create_test_material_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-test-material-set-layout"),
            entries: &[
                material_uniform_entry(0),
                material_texture_entry(1),
                material_sampler_entry(2),
                material_texture_entry(3),
                material_sampler_entry(4),
                material_texture_entry(5),
                material_sampler_entry(6),
                material_texture_entry(7),
                material_sampler_entry(8),
                material_texture_entry(9),
                material_sampler_entry(10),
                material_texture_entry(11),
                material_sampler_entry(12),
            ],
        })
    }

    fn material_texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                multisampled: false,
                view_dimension: wgpu::TextureViewDimension::D2,
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
            },
            count: None,
        }
    }

    fn material_sampler_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        }
    }

    fn material_uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(GPU_MATERIAL_UNIFORM_MIN_SIZE as u64),
            },
            count: None,
        }
    }

    fn create_test_gpu_scene(device: &wgpu::Device) -> GpuScene {
        GpuScene::new(
            device,
            test_skinned_joint_palette_buffer(device),
            test_skinned_joint_palette_min_binding_size(),
        )
    }

    fn test_skinned_joint_palette_buffer(device: &wgpu::Device) -> Arc<wgpu::Buffer> {
        Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-test-gbuffer-skinned-joint-palette-buffer"),
            size: test_skinned_joint_palette_min_binding_size().get(),
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        }))
    }

    fn test_skinned_joint_palette_min_binding_size() -> wgpu::BufferSize {
        wgpu::BufferSize::new(
            TEST_SKINNED_JOINT_MATRIX_COUNT * TEST_SKINNED_JOINT_MATRIX_BYTES
                + TEST_SKINNED_JOINT_PARAMS_BYTES,
        )
        .expect("test skinned joint palette storage size is non-zero")
    }
}
