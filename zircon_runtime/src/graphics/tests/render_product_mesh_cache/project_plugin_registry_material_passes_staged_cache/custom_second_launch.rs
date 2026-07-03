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
use super::custom_shading_model::custom_toon_plugin_shading_model;
use super::fixture::submit_registry_material_passes_with_plugin_shading_model;
use super::manifest::{
    registry_material_pass_product_prewarm_manifest_with_plugin_shading_models,
    registry_material_pass_runtime_surface_source,
};
use super::shader_cache_test_roots;

#[test]
fn render_product_custom_shading_model_second_launch_uses_staged_prewarm_without_compile_miss() {
    let cache_roots = shader_cache_test_roots("zircon_product_custom_shading_model_second_launch");
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
    .expect("custom shading-model second-launch prewarm manifest");
    let registry_shader_source = registry_material_pass_runtime_surface_source();

    let prewarm_report =
        prewarm_shader_variants_with_wgpu_pipeline_validation(&manifest, &cache_roots.staged_root);
    assert_eq!(prewarm_report.requested_count, manifest.variants.len());
    assert_eq!(
        prewarm_report.written_count,
        manifest.variants.len(),
        "custom toon second-launch prewarm should write every requested pass variant; report={prewarm_report:#?}"
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
        "prewarm custom toon shading model",
    );
    assert_registry_material_pass_prewarm_written_for_shading_model(
        &manifest,
        &prewarm_report,
        case,
        descriptor.id,
    );

    let first_launch = submit_registry_material_passes_with_plugin_shading_model(
        case,
        registry_shader_source.as_str(),
        8_201,
        &cache_roots.runtime_root,
        &cache_roots.staged_root,
        plugin_shading_model,
    );
    assert_registry_material_pass_first_frame_shader_cache_hit_for_shading_model(
        &first_launch.first_frame,
        case,
        &prewarm_report,
        descriptor.id,
        "custom toon shading model first launch",
    );
    assert_registry_material_pass_velocity_frame_shader_cache_hit_for_shading_model(
        &first_launch.velocity_frame,
        case,
        &prewarm_report,
        descriptor.id,
        "custom toon shading model first launch",
    );
    assert_runtime_shader_cache_root_empty(
        &cache_roots.runtime_root,
        "first custom shading-model product launch should stay read-only against staged cache",
    );

    let second_launch = submit_registry_material_passes_with_plugin_shading_model(
        case,
        registry_shader_source.as_str(),
        9_201,
        &cache_roots.runtime_root,
        &cache_roots.staged_root,
        custom_toon_plugin_shading_model(),
    );
    assert_registry_material_pass_first_frame_shader_cache_hit_for_shading_model(
        &second_launch.first_frame,
        case,
        &prewarm_report,
        descriptor.id,
        "custom toon shading model second launch",
    );
    assert_registry_material_pass_velocity_frame_shader_cache_hit_for_shading_model(
        &second_launch.velocity_frame,
        case,
        &prewarm_report,
        descriptor.id,
        "custom toon shading model second launch",
    );
    assert_runtime_shader_cache_root_empty(
        &cache_roots.runtime_root,
        "second custom shading-model product launch should still stay read-only against staged cache",
    );

    let _ = fs::remove_dir_all(&cache_roots.root);
}
