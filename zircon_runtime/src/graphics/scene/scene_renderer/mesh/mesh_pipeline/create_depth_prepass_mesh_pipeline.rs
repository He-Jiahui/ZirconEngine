use crate::graphics::scene::resources::{GpuMeshVertex, PipelineKey};

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_depth_prepass_mesh_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    key: &PipelineKey,
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
        cache: None,
    })
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::sync::Arc;

    use crate::core::framework::render::ShaderPassType;
    use crate::core::framework::render::GEOMETRY_SOURCE_ID_STATIC_MESH;
    use crate::graphics::scene::gpu_scene::GpuScene;
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::resources::GPU_MATERIAL_UNIFORM_MIN_SIZE;
    use crate::graphics::scene::scene_renderer::environment::scene_bind_group_layout_entries;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::mesh_pipeline_depth_prepass_template_source_for_geometry;

    use super::create_depth_prepass_mesh_pipeline;

    const TEST_SKINNED_JOINT_MATRIX_COUNT: u64 = 256;
    const TEST_SKINNED_JOINT_MATRIX_BYTES: u64 = 64;
    const TEST_SKINNED_JOINT_PARAMS_BYTES: u64 = 16;

    #[test]
    fn depth_prepass_mesh_pipeline_declares_depth_only_template_entries_and_static_layout() {
        let source = include_str!("create_depth_prepass_mesh_pipeline.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source section should exist");

        assert!(source.contains("GpuMeshVertex::layout()"));
        assert!(!source.contains("GpuMeshVertex::previous_position_layout()"));
        assert!(!source.contains("NORMAL_FORMAT"));
        assert!(source.contains("entry_point: Some(\"vs_main\")"));
        assert!(source.contains("key.is_alpha_mask()"));
        assert!(source.contains("targets: &[]"));
        assert!(source.contains("depth_write_enabled: Some(true)"));

        let key = default_pipeline_key();
        let variant_key = key.shader_variant_key(ShaderPassType::DepthPrepass, "wgpu-runtime");
        let source = mesh_pipeline_depth_prepass_template_source_for_geometry(
            &key,
            variant_key.geometry_source,
        )
        .expect("depth prepass depth-only template source should assemble");

        assert!(source.wgsl_source.contains("zr_template_depth.wgsl"));
        assert!(source.wgsl_source.contains("fn vs_main("));
        assert!(!source.wgsl_source.contains("fn fs_main("));
        assert!(!source.wgsl_source.contains("surface.normal_ws * 0.5"));
    }

    #[test]
    fn depth_prepass_mesh_pipeline_creates_on_wgpu_device_with_template_shader() {
        let Ok(backend) = crate::graphics::backend::RenderBackend::new_offscreen() else {
            return;
        };
        let device = &backend.device;
        let key = default_pipeline_key();
        let shader_source = mesh_pipeline_depth_prepass_template_source_for_geometry(
            &key,
            GEOMETRY_SOURCE_ID_STATIC_MESH,
        )
        .expect("depth prepass depth-only template source should assemble");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("zircon-test-depth-prepass-template-shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source.wgsl_source)),
        });
        let scene_layout = create_test_scene_layout(device);
        let shadow_receiver_layout = create_empty_shadow_receiver_layout(device);
        let material_layout = create_test_material_layout(device);
        let gpu_scene = create_test_gpu_scene(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-test-depth-prepass-mesh-layout"),
            bind_group_layouts: &[
                Some(&scene_layout),
                Some(&shadow_receiver_layout),
                Some(&material_layout),
                Some(gpu_scene.scene_bind_group_layout()),
            ],
            immediate_size: 0,
        });

        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let _pipeline = create_depth_prepass_mesh_pipeline(device, &pipeline_layout, &shader, &key);
        let error = pollster::block_on(error_scope.pop());

        assert!(
            error.is_none(),
            "depth prepass depth-only template pipeline should pass WGPU validation: {error:?}"
        );
    }

    fn create_test_scene_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        let scene_layout_entries = scene_bind_group_layout_entries();
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-test-depth-scene-layout"),
            entries: &scene_layout_entries,
        })
    }

    fn create_empty_shadow_receiver_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-test-depth-empty-shadow-receiver-layout"),
            entries: &[],
        })
    }

    fn create_test_material_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-test-depth-material-set-layout"),
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
            label: Some("zircon-test-depth-skinned-joint-palette-buffer"),
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
