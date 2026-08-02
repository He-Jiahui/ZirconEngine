use std::borrow::Cow;
use std::sync::Arc;

use crate::core::framework::render::{
    ShaderPassType, ShaderVariantPrewarmRequest, ShaderVariantPrewarmSource,
};
use crate::graphics::scene::resources::PipelineKey;
use crate::graphics::scene::scene_renderer::environment::scene_bind_group_layout_entries;

use super::super::mesh_pipeline::{
    create_depth_prepass_mesh_pipeline, create_gbuffer_mesh_pipeline, create_mesh_pipeline,
    create_shadow_mesh_pipeline, create_taa_reactive_mask_mesh_pipeline,
    create_velocity_mesh_pipeline,
};
use super::create_forward_shadow_receiver_layout;
use super::prewarm_manifest::{
    pipeline_key_from_prewarm_request, pipeline_kind_from_prewarm_request,
};

pub(crate) fn validate_mesh_prewarm_request_render_pipeline(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
    request: &ShaderVariantPrewarmRequest,
    source: &ShaderVariantPrewarmSource,
) -> Result<(), String> {
    let pipeline_key = pipeline_key_from_prewarm_request(request).map_err(str::to_string)?;
    let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-shader-prewarm-pipeline-validation-module"),
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(source.wgsl_source.as_str())),
    });
    create_validation_pipeline(device, pipeline_layout, &shader, &pipeline_key, request);
    match pollster::block_on(error_scope.pop()) {
        Some(error) => Err(error.to_string()),
        None => Ok(()),
    }
}

pub(crate) fn create_mesh_prewarm_validation_pipeline_layout(
    device: &wgpu::Device,
) -> wgpu::PipelineLayout {
    let scene_layout = create_validation_scene_layout(device);
    let shadow_receiver_layout = create_forward_shadow_receiver_layout(device);
    let material_layout = create_validation_material_layout(device);
    let gpu_scene_layout = create_validation_gpu_scene_layout(device);
    device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-shader-prewarm-pipeline-validation-layout"),
        bind_group_layouts: &[
            Some(&scene_layout),
            Some(&shadow_receiver_layout),
            Some(&material_layout),
            Some(&gpu_scene_layout),
        ],
        immediate_size: 0,
    })
}

fn create_validation_pipeline(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    pipeline_key: &PipelineKey,
    request: &ShaderVariantPrewarmRequest,
) -> wgpu::RenderPipeline {
    match request.key.pass_type {
        ShaderPassType::Forward => create_mesh_pipeline(
            device,
            pipeline_layout,
            shader,
            wgpu::TextureFormat::Bgra8UnormSrgb,
            pipeline_key,
            None,
        ),
        ShaderPassType::GBuffer => {
            create_gbuffer_mesh_pipeline(device, pipeline_layout, shader, pipeline_key, None)
        }
        ShaderPassType::DepthPrepass => {
            create_depth_prepass_mesh_pipeline(device, pipeline_layout, shader, pipeline_key, None)
        }
        ShaderPassType::Shadow => create_shadow_mesh_pipeline(
            device,
            pipeline_layout,
            shader,
            pipeline_kind_from_prewarm_request(request),
            None,
        ),
        ShaderPassType::Velocity => create_velocity_mesh_pipeline(
            device,
            pipeline_layout,
            shader,
            wgpu::TextureFormat::Rg16Float,
            pipeline_key,
            None,
        ),
        ShaderPassType::TaaReactiveMask => create_taa_reactive_mask_mesh_pipeline(
            device,
            pipeline_layout,
            shader,
            wgpu::TextureFormat::R8Unorm,
            pipeline_key,
            None,
        ),
    }
}

fn create_validation_scene_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let scene_layout_entries = scene_bind_group_layout_entries();
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-shader-prewarm-validation-scene-layout"),
        entries: &scene_layout_entries,
    })
}

fn create_validation_material_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-shader-prewarm-validation-material-layout"),
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

fn create_validation_gpu_scene_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    use crate::graphics::scene::gpu_scene::GpuScene;

    let joint_palette_buffer = Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-shader-prewarm-validation-joint-palette"),
        size: 256 * 64 + 16,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    }));
    let min_binding_size =
        wgpu::BufferSize::new(256 * 64 + 16).expect("prewarm validation joint palette size");
    let gpu_scene = GpuScene::new(device, joint_palette_buffer, min_binding_size);
    gpu_scene.scene_bind_group_layout().clone()
}

fn material_uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    use crate::graphics::scene::resources::GPU_MATERIAL_UNIFORM_MIN_SIZE;

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

#[cfg(all(test, feature = "dynamic-api"))]
mod tests {
    use crate::core::framework::render::{
        ShaderFeatureBits, ShaderQualityTier, ShaderVariantPrewarmSource,
        SHADING_MODEL_ID_BLINN_PHONG,
    };
    use crate::dynamic_api::builtin_standard_material_shader_prewarm_manifest_for_geometry;
    use crate::graphics::backend::RenderBackend;

    use super::{
        create_mesh_prewarm_validation_pipeline_layout,
        validate_mesh_prewarm_request_render_pipeline,
    };

    #[test]
    fn mesh_prewarm_pipeline_validation_creates_all_builtin_pass_pipelines() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let device = &backend.device;
        let layout = create_mesh_prewarm_validation_pipeline_layout(device);
        let manifest = builtin_standard_material_shader_prewarm_manifest_for_geometry(
            ShaderFeatureBits::new(
                ShaderFeatureBits::ALPHA_TEST
                    | ShaderFeatureBits::DOUBLE_SIDED
                    | ShaderFeatureBits::RECEIVE_SHADOWS,
            ),
            SHADING_MODEL_ID_BLINN_PHONG,
            Some(0.42),
            crate::core::framework::render::GEOMETRY_SOURCE_ID_SKINNED_MESH,
            &[ShaderQualityTier::Medium],
        );

        assert_eq!(manifest.variants.len(), 6);
        for request in &manifest.variants {
            let source = manifest
                .source_for(request)
                .expect("builtin prewarm source");
            validate_mesh_prewarm_request_render_pipeline(device, &layout, request, source)
                .unwrap_or_else(|error| {
                    panic!(
                        "prewarm {} pipeline should validate: {error}",
                        request.key.pass_type.token()
                    )
                });
        }
    }

    #[test]
    fn mesh_prewarm_pipeline_validation_rejects_raw_surface_only_wgsl() {
        let Ok(backend) = RenderBackend::new_offscreen() else {
            return;
        };
        let device = &backend.device;
        let layout = create_mesh_prewarm_validation_pipeline_layout(device);
        let manifest = builtin_standard_material_shader_prewarm_manifest_for_geometry(
            ShaderFeatureBits::new(0),
            SHADING_MODEL_ID_BLINN_PHONG,
            None,
            crate::core::framework::render::GEOMETRY_SOURCE_ID_STATIC_MESH,
            &[ShaderQualityTier::Medium],
        );
        let request = manifest.variants.first().expect("prewarm request");
        let raw_surface_source = ShaderVariantPrewarmSource::new(
            "test://raw-surface-only.wgsl",
            "fn zr_material_surface() {}",
            Vec::new(),
            "test-template",
            "test-naga",
            "test-wgpu",
        );

        let error = validate_mesh_prewarm_request_render_pipeline(
            device,
            &layout,
            request,
            &raw_surface_source,
        )
        .expect_err("raw surface-only WGSL must not pass render-pipeline validation");

        assert!(
            error.contains("vs_main") || error.contains("Entry point"),
            "expected missing pipeline entry point validation error, got {error}"
        );
    }
}
