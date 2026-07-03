use crate::graphics::scene::resources::GpuMeshVertex;

use super::super::mesh_pass::MeshPassPipelineKind;

const SHADOW_DEPTH_BIAS_CONSTANT: i32 = 2;
const SHADOW_DEPTH_BIAS_SLOPE_SCALE: f32 = 2.0;
const SHADOW_DEPTH_BIAS_CLAMP: f32 = 0.0;

pub(in crate::graphics::scene::scene_renderer::mesh) fn create_shadow_mesh_pipeline(
    device: &wgpu::Device,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    kind: MeshPassPipelineKind,
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
        primitive: wgpu::PrimitiveState::default(),
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
        cache: None,
    })
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
mod tests {
    use std::borrow::Cow;
    use std::sync::Arc;

    use super::create_shadow_mesh_pipeline;
    use crate::core::framework::render::ShaderPassType;
    use crate::core::framework::render::GEOMETRY_SOURCE_ID_STATIC_MESH;
    use crate::graphics::scene::gpu_scene::GpuScene;
    use crate::graphics::scene::resources::default_pipeline_key;
    use crate::graphics::scene::resources::GPU_MATERIAL_UNIFORM_MIN_SIZE;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshPassPipelineKind;
    use crate::graphics::scene::scene_renderer::mesh::mesh_pipeline_cache::mesh_pipeline_shadow_template_source_for_geometry;

    const TEST_SKINNED_JOINT_MATRIX_COUNT: u64 = 256;
    const TEST_SKINNED_JOINT_MATRIX_BYTES: u64 = 64;
    const TEST_SKINNED_JOINT_PARAMS_BYTES: u64 = 16;

    #[test]
    fn shadow_mesh_pipeline_declares_template_entries_static_layout_and_depth_bias() {
        let source = include_str!("create_shadow_mesh_pipeline.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("production source section should exist");

        assert!(source.contains("GpuMeshVertex::layout()"));
        assert!(!source.contains("GpuMeshVertex::previous_position_layout()"));
        assert!(source.contains("entry_point: Some(\"vs_main\")"));
        assert!(source.contains("entry_point: Some(\"fs_main\")"));
        assert!(source.contains("targets: &[]"));
        assert!(source.contains("SHADOW_DEPTH_BIAS_CONSTANT"));
        assert!(source.contains("SHADOW_DEPTH_BIAS_SLOPE_SCALE"));
        assert!(source.contains("SHADOW_DEPTH_BIAS_CLAMP"));

        let key = default_pipeline_key();
        let variant_key = key.shader_variant_key(ShaderPassType::Shadow, "wgpu-runtime");
        let source =
            mesh_pipeline_shadow_template_source_for_geometry(&key, variant_key.geometry_source)
                .expect("shadow template source should assemble");

        assert!(source.wgsl_source.contains("fn vs_main("));
        assert!(!source.wgsl_source.contains("fn fs_main("));
        assert!(!source
            .wgsl_source
            .contains("GpuMeshVertex::previous_position_layout()"));
    }

    #[test]
    fn shadow_mesh_pipeline_creates_on_wgpu_device_with_template_shader() {
        let Ok(backend) = crate::graphics::backend::RenderBackend::new_offscreen() else {
            return;
        };
        let device = &backend.device;
        let opaque_key = default_pipeline_key();
        let mut alpha_key = default_pipeline_key();
        alpha_key.alpha_mask = true;
        alpha_key.alpha_cutoff_bits = Some(0.5f32.to_bits());

        let scene_layout = create_test_scene_layout(device);
        let shadow_receiver_layout = create_empty_shadow_receiver_layout(device);
        let material_layout = create_test_material_layout(device);
        let gpu_scene = create_test_gpu_scene(device);
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-test-shadow-mesh-layout"),
            bind_group_layouts: &[
                Some(&scene_layout),
                Some(&shadow_receiver_layout),
                Some(&material_layout),
                Some(gpu_scene.scene_bind_group_layout()),
            ],
            immediate_size: 0,
        });

        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        let opaque_shader = create_shadow_template_shader(device, &opaque_key, "opaque");
        let alpha_shader = create_shadow_template_shader(device, &alpha_key, "alpha-mask");
        let _opaque_pipeline = create_shadow_mesh_pipeline(
            device,
            &pipeline_layout,
            &opaque_shader,
            MeshPassPipelineKind::ShadowDepth,
        );
        let _alpha_pipeline = create_shadow_mesh_pipeline(
            device,
            &pipeline_layout,
            &alpha_shader,
            MeshPassPipelineKind::ShadowDepthAlphaMask,
        );
        let error = pollster::block_on(error_scope.pop());

        assert!(
            error.is_none(),
            "shadow depth and alpha-mask template pipelines should pass WGPU validation: {error:?}"
        );
    }

    fn create_shadow_template_shader(
        device: &wgpu::Device,
        key: &crate::graphics::scene::resources::PipelineKey,
        label_suffix: &str,
    ) -> wgpu::ShaderModule {
        let shader_source =
            mesh_pipeline_shadow_template_source_for_geometry(key, GEOMETRY_SOURCE_ID_STATIC_MESH)
                .expect("shadow template source should assemble");
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&format!(
                "zircon-test-shadow-template-shader-{label_suffix}"
            )),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(shader_source.wgsl_source)),
        })
    }

    fn create_test_scene_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-test-shadow-scene-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX
                    | wgpu::ShaderStages::FRAGMENT
                    | wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        })
    }

    fn create_empty_shadow_receiver_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-test-shadow-empty-shadow-receiver-layout"),
            entries: &[],
        })
    }

    fn create_test_material_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("zircon-test-shadow-material-set-layout"),
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
            label: Some("zircon-test-shadow-skinned-joint-palette-buffer"),
            size: test_skinned_joint_palette_min_binding_size().get(),
            usage: wgpu::BufferUsages::UNIFORM,
            mapped_at_creation: false,
        }))
    }

    fn test_skinned_joint_palette_min_binding_size() -> wgpu::BufferSize {
        wgpu::BufferSize::new(
            TEST_SKINNED_JOINT_MATRIX_COUNT * TEST_SKINNED_JOINT_MATRIX_BYTES
                + TEST_SKINNED_JOINT_PARAMS_BYTES,
        )
        .expect("test skinned joint palette uniform size is non-zero")
    }
}
