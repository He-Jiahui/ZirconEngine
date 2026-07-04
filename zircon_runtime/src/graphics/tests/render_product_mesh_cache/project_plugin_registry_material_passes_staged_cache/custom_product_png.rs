use std::fs;
use std::sync::Arc;

use crate::asset::ProjectAssetManager;
use crate::dynamic_api::prewarm_shader_variants_with_wgpu_pipeline_validation;

use super::assertions::{
    assert_registry_material_pass_first_frame_shader_cache_hit_for_shading_model,
    assert_registry_material_pass_prewarm_dimensions_written_for_shading_model,
    assert_registry_material_pass_prewarm_written_for_shading_model,
    assert_registry_material_pass_velocity_frame_shader_cache_hit_for_shading_model,
    assert_runtime_shader_cache_root_empty,
};
use super::case::registry_shader_cases;
use super::custom_shading_model::{
    assert_custom_toon_deferred_lighting_readback, custom_toon_plugin_shading_model,
};
use super::fixture::submit_registry_material_passes_with_plugin_shading_model_capture;
use super::manifest::{
    registry_material_pass_product_prewarm_manifest_with_plugin_shading_models,
    registry_material_pass_runtime_surface_source,
};
use super::product_png::{
    assert_visible_frame, render_test_output_dir, save_side_by_side_product_frames,
};
use super::shader_cache_test_roots;

const CUSTOM_DEFERRED_PRODUCT_PNG_STATUS: &str =
    "render_plan08_custom_shading_model_deferred_lighting_product_readback_png_passed_renderdoc_deferred";

#[test]
#[ignore = "manual product PNG export for Plan 08 custom shading-model deferred-lighting readback"]
fn export_custom_shading_model_deferred_lighting_product_png() {
    assert!(!CUSTOM_DEFERRED_PRODUCT_PNG_STATUS.is_empty());

    let cache_roots =
        shader_cache_test_roots("zircon_product_custom_shading_model_deferred_lighting_png");
    let _ = fs::remove_dir_all(&cache_roots.root);
    fs::create_dir_all(&cache_roots.root).expect("shader cache test root");

    let case = registry_shader_cases()[0];
    let plugin_shading_model = custom_toon_plugin_shading_model();
    let descriptor = plugin_shading_model.descriptor.clone();
    let prewarm_asset_manager = Arc::new(ProjectAssetManager::default());
    (plugin_shading_model.register_shader_includes)(&prewarm_asset_manager);
    let manifest = registry_material_pass_product_prewarm_manifest_with_plugin_shading_models(
        &prewarm_asset_manager,
        &[case],
        &[descriptor.clone()],
    )
    .expect("custom shading-model deferred-lighting PNG prewarm manifest");
    let registry_shader_source = registry_material_pass_runtime_surface_source();

    let prewarm_report =
        prewarm_shader_variants_with_wgpu_pipeline_validation(&manifest, &cache_roots.staged_root);
    assert_eq!(prewarm_report.requested_count, manifest.variants.len());
    assert_eq!(
        prewarm_report.written_count,
        manifest.variants.len(),
        "custom toon PNG prewarm should write every requested pass variant; report={prewarm_report:#?}"
    );
    assert_eq!(prewarm_report.failed_count, 0);
    assert!(prewarm_report.failures.is_empty());
    assert!(prewarm_report.wgpu_pipeline_validation.enabled);
    assert_eq!(
        prewarm_report.wgpu_pipeline_validation.validated_count,
        manifest.variants.len()
    );
    assert_registry_material_pass_prewarm_dimensions_written_for_shading_model(
        &prewarm_report,
        descriptor.id,
        "prewarm custom toon deferred-lighting PNG",
    );
    assert_registry_material_pass_prewarm_written_for_shading_model(
        &manifest,
        &prewarm_report,
        case,
        descriptor.id,
    );

    let launch = submit_registry_material_passes_with_plugin_shading_model_capture(
        case,
        registry_shader_source.as_str(),
        7_401,
        &cache_roots.runtime_root,
        &cache_roots.staged_root,
        plugin_shading_model,
    );
    assert_registry_material_pass_first_frame_shader_cache_hit_for_shading_model(
        &launch.first_frame,
        case,
        &prewarm_report,
        descriptor.id,
        "custom toon deferred-lighting PNG",
    );
    assert_registry_material_pass_velocity_frame_shader_cache_hit_for_shading_model(
        &launch.velocity_frame,
        case,
        &prewarm_report,
        descriptor.id,
        "custom toon deferred-lighting PNG",
    );
    assert_runtime_shader_cache_root_empty(
        &cache_roots.runtime_root,
        "custom toon deferred-lighting PNG export should stay read-only against staged cache",
    );

    let first = launch
        .first_capture
        .as_ref()
        .expect("custom toon first-frame capture");
    let velocity = launch
        .velocity_capture
        .as_ref()
        .expect("custom toon velocity-frame capture");
    assert_visible_frame(first, "custom toon first frame");
    assert_visible_frame(velocity, "custom toon velocity frame");
    assert_custom_toon_deferred_lighting_readback(first);
    assert_custom_toon_deferred_lighting_readback(velocity);

    let output_path = render_test_output_dir()
        .join("runtime_render_plan08_custom_shading_model_deferred_lighting_20260704.png");
    save_side_by_side_product_frames(first, velocity, &output_path);

    let _ = fs::remove_dir_all(&cache_roots.root);
}
