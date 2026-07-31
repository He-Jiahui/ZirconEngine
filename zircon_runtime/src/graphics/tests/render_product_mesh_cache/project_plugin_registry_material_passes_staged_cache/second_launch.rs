use std::fs;

use crate::dynamic_api::prewarm_shader_variants;

use super::assertions::{
    assert_registry_material_pass_first_frame_shader_cache_hit,
    assert_registry_material_pass_prewarm_dimensions_written,
    assert_registry_material_pass_prewarm_written,
    assert_registry_material_pass_velocity_frame_shader_cache_hit,
    assert_runtime_shader_cache_root_empty,
};
use super::case::registry_shader_cases;
use super::fixture::submit_registry_material_passes_with_staged_cache;
use super::manifest::{
    registry_material_pass_product_prewarm_manifest, registry_material_pass_runtime_surface_source,
};
use super::shader_cache_test_roots;

#[test]
fn render_product_project_plugin_registry_material_passes_second_launch_use_staged_prewarm_without_compile_miss()
 {
    let cache_roots = shader_cache_test_roots(
        "zircon_product_project_plugin_registry_material_passes_second_launch",
    );
    let _ = fs::remove_dir_all(&cache_roots.root);
    fs::create_dir_all(&cache_roots.root).expect("shader cache test root");

    let registry_cases = registry_shader_cases();
    let manifest = registry_material_pass_product_prewarm_manifest(&registry_cases);
    let registry_shader_source = registry_material_pass_runtime_surface_source();
    let prewarm_report = prewarm_shader_variants(&manifest, &cache_roots.staged_root);
    assert_eq!(prewarm_report.requested_count, manifest.variants.len());
    assert_eq!(prewarm_report.written_count, manifest.variants.len());
    assert_eq!(prewarm_report.failed_count, 0);
    assert!(prewarm_report.failures.is_empty());
    assert_registry_material_pass_prewarm_dimensions_written(&prewarm_report);
    for case in registry_cases.iter().copied() {
        assert_registry_material_pass_prewarm_written(&manifest, &prewarm_report, case);
    }

    for (index, case) in registry_cases.iter().copied().enumerate() {
        let first_launch = submit_registry_material_passes_with_staged_cache(
            case,
            registry_shader_source.as_str(),
            5_201 + index as u64,
            &cache_roots.runtime_root,
            &cache_roots.staged_root,
        );
        assert_registry_material_pass_first_frame_shader_cache_hit(
            &first_launch.first_frame,
            case,
            &prewarm_report,
        );
        assert_registry_material_pass_velocity_frame_shader_cache_hit(
            &first_launch.velocity_frame,
            case,
            &prewarm_report,
        );
        assert_runtime_shader_cache_root_empty(
            &cache_roots.runtime_root,
            "first product launch should stay read-only against staged cache",
        );

        let second_launch = submit_registry_material_passes_with_staged_cache(
            case,
            registry_shader_source.as_str(),
            6_201 + index as u64,
            &cache_roots.runtime_root,
            &cache_roots.staged_root,
        );
        assert_registry_material_pass_first_frame_shader_cache_hit(
            &second_launch.first_frame,
            case,
            &prewarm_report,
        );
        assert_registry_material_pass_velocity_frame_shader_cache_hit(
            &second_launch.velocity_frame,
            case,
            &prewarm_report,
        );
        assert_runtime_shader_cache_root_empty(
            &cache_roots.runtime_root,
            "second product launch should still stay read-only against staged cache",
        );
    }

    let _ = fs::remove_dir_all(&cache_roots.root);
}
