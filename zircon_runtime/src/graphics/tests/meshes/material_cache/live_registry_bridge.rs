use std::fs;

use crate::core::framework::render::{
    ShaderPassType, ShaderVariantPrewarmManifest, ShaderVariantPrewarmReport,
    ShaderVariantPrewarmRequest,
};
use crate::dynamic_api::prewarm_shader_variants_with_wgpu_pipeline_validation;

use super::assertions::{
    assert_registry_material_pass_first_frame_shader_cache_hit,
    assert_registry_material_pass_prewarm_dimensions_written,
    assert_registry_material_pass_velocity_frame_shader_cache_hit,
    assert_runtime_shader_cache_root_empty,
};
use super::case::{registry_shader_cases, RegistryShaderCase};
use super::fixture::submit_registry_material_passes_with_staged_cache;
use super::manifest::{
    registry_material_pass_live_source_label_prewarm_manifest,
    registry_material_pass_runtime_surface_source, REGISTRY_MATERIAL_PASS_TYPES,
};
use super::shader_cache_test_roots;

#[test]
fn render_product_project_plugin_registry_material_passes_live_registry_source_labels_hit_staged_cache(
) {
    let cache_roots = shader_cache_test_roots(
        "zircon_product_project_plugin_registry_material_passes_live_registry_labels",
    );
    let _ = fs::remove_dir_all(&cache_roots.root);
    fs::create_dir_all(&cache_roots.root).expect("shader cache test root");

    let registry_cases = registry_shader_cases();
    let manifest = registry_material_pass_live_source_label_prewarm_manifest(&registry_cases);
    let registry_shader_source = registry_material_pass_runtime_surface_source();
    let prewarm_report =
        prewarm_shader_variants_with_wgpu_pipeline_validation(&manifest, &cache_roots.staged_root);
    assert_eq!(prewarm_report.requested_count, manifest.variants.len());
    assert_eq!(
        prewarm_report.written_count,
        manifest.variants.len(),
        "live registry label prewarm should write every requested pass variant; report={prewarm_report:#?}"
    );
    assert_eq!(prewarm_report.failed_count, 0);
    assert!(prewarm_report.failures.is_empty());
    assert!(prewarm_report.wgpu_pipeline_validation.enabled);
    assert_eq!(
        prewarm_report.wgpu_pipeline_validation.validated_count,
        manifest.variants.len()
    );
    assert_registry_material_pass_prewarm_dimensions_written(&prewarm_report);
    for case in registry_cases.iter().copied() {
        assert_live_registry_material_pass_prewarm_written(&manifest, &prewarm_report, case);
    }

    for (index, case) in registry_cases.iter().copied().enumerate() {
        let launch = submit_registry_material_passes_with_staged_cache(
            case,
            registry_shader_source.as_str(),
            10_201 + index as u64,
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
        assert_runtime_shader_cache_root_empty(
            &cache_roots.runtime_root,
            "live registry source-label product launch should stay read-only against staged cache",
        );
    }

    let _ = fs::remove_dir_all(&cache_roots.root);
}

fn assert_live_registry_material_pass_prewarm_written(
    manifest: &ShaderVariantPrewarmManifest,
    report: &ShaderVariantPrewarmReport,
    case: RegistryShaderCase,
) {
    for pass_type in REGISTRY_MATERIAL_PASS_TYPES {
        let request = live_registry_material_pass_request(manifest, case, pass_type);
        let source = manifest
            .source_for(request)
            .expect("live registry material-pass prewarm source");
        assert_eq!(
            source.source_label,
            case.locator,
            "live registry product bridge should preserve asset-root source label for {}",
            pass_type.token()
        );
        assert!(
            !source.source_label.contains("::"),
            "live registry source label should not depend on test-only pass suffixes"
        );

        let written = report
            .written_variants
            .iter()
            .find(|variant| variant.canonical_string == request.key.canonical_string())
            .unwrap_or_else(|| {
                panic!(
                    "live registry written variant for {} pass {}",
                    case.locator,
                    pass_type.token()
                )
            });
        assert_eq!(written.source_label, case.locator);
        assert!(
            written
                .canonical_string
                .contains(&format!("|pass={}", pass_type.token())),
            "written live registry cache key should retain pass dimension; canonical={}",
            written.canonical_string
        );
    }
}

fn live_registry_material_pass_request(
    manifest: &ShaderVariantPrewarmManifest,
    case: RegistryShaderCase,
    pass_type: ShaderPassType,
) -> &ShaderVariantPrewarmRequest {
    manifest
        .variants
        .iter()
        .find(|request| {
            request.key.material_shader == case.shader_id()
                && request.key.material_revision == case.revision
                && request.key.pass_type == pass_type
        })
        .unwrap_or_else(|| {
            panic!(
                "live registry material-pass prewarm request for {} pass {}",
                case.locator,
                pass_type.token()
            )
        })
}
