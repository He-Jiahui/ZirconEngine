use crate::core::framework::render::{
    ShaderFeatureBits, ShaderQualityTier, ShaderVariantPrewarmSource, SHADING_MODEL_ID_BLINN_PHONG,
};
use crate::dynamic_api::builtin_standard_material_shader_prewarm_manifest_for_geometry;
use crate::graphics::backend::RenderBackend;

use super::{
    create_mesh_prewarm_validation_pipeline_layout, validate_mesh_prewarm_request_render_pipeline,
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
