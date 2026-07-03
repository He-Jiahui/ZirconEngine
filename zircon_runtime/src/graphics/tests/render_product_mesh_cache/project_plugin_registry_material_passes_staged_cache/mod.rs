use std::fs;
use std::path::PathBuf;

use crate::dynamic_api::prewarm_shader_variants;

use assertions::{
    assert_registry_material_pass_first_frame_shader_cache_hit,
    assert_registry_material_pass_prewarm_dimensions_written,
    assert_registry_material_pass_prewarm_written,
    assert_registry_material_pass_velocity_frame_shader_cache_hit,
};
use case::registry_shader_cases;
use fixture::submit_registry_material_passes_with_staged_cache;
use manifest::{
    registry_material_pass_product_prewarm_manifest, registry_material_pass_runtime_surface_source,
};

mod assertions;
mod case;
mod custom_second_launch;
mod custom_shading_model;
mod fixture;
mod manifest;
mod pipeline;
mod product_png;
mod second_launch;

#[test]
fn render_product_project_plugin_registry_material_passes_use_staged_prewarm_without_compile_miss()
{
    let cache_roots = shader_cache_test_roots(
        "zircon_product_project_plugin_registry_material_passes_staged_prewarm",
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
        let launch = submit_registry_material_passes_with_staged_cache(
            case,
            registry_shader_source.as_str(),
            4_201 + index as u64,
            &cache_roots.runtime_root,
            &cache_roots.staged_root,
        );
        assert_registry_material_pass_first_frame_shader_cache_hit(
            &launch.first_frame,
            case,
            &prewarm_report,
        );
        assert_registry_material_pass_velocity_frame_shader_cache_hit(
            &launch.velocity_frame,
            case,
            &prewarm_report,
        );
    }

    let _ = fs::remove_dir_all(&cache_roots.root);
}

struct ShaderCacheTestRoots {
    root: PathBuf,
    runtime_root: PathBuf,
    staged_root: PathBuf,
}

fn shader_cache_test_roots(label: &str) -> ShaderCacheTestRoots {
    let root = std::env::temp_dir().join(format!("{label}_{}", std::process::id()));
    ShaderCacheTestRoots {
        runtime_root: root.join("runtime").join("shader_variants"),
        staged_root: root.join("staged").join("cache").join("shader_variants"),
        root,
    }
}
