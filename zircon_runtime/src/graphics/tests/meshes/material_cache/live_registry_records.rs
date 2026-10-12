use std::fs;
use std::path::{Path, PathBuf};

use crate::asset::project::shader_resource_records_from_asset_roots;
use crate::core::resource::{ResourceKind, ResourceRecord, ResourceState};
use crate::dynamic_api::prewarm_shader_variants_with_wgpu_pipeline_validation;

use super::assertions::{
    assert_registry_material_pass_first_frame_shader_cache_hit,
    assert_registry_material_pass_prewarm_dimensions_written,
    assert_registry_material_pass_velocity_frame_shader_cache_hit,
    assert_runtime_shader_cache_root_empty,
};
use super::case::{registry_shader_cases_from_live_records, RegistryShaderCase};
use super::fixture::submit_registry_material_passes_with_staged_cache;
use super::manifest::{
    raw_wgsl_hash, registry_material_pass_live_source_label_prewarm_manifest,
    registry_material_pass_runtime_surface_source,
};
use super::shader_cache_test_roots;

const PROJECT_SHADER_LOCATOR: &str = "res://project/shaders/project_shader";
const PLUGIN_SHADER_LOCATOR: &str = "package://native_dynamic_fixture/shaders/shader";

#[test]
fn render_product_project_plugin_registry_material_passes_asset_root_records_hit_staged_cache() {
    let cache_roots = shader_cache_test_roots(
        "zircon_product_project_plugin_registry_material_passes_asset_root_records",
    );
    let _ = fs::remove_dir_all(&cache_roots.root);
    fs::create_dir_all(&cache_roots.root).expect("shader cache test root");

    let registry_shader_source = registry_material_pass_runtime_surface_source();
    let project_root = cache_roots.root.join("project_shader_assets");
    write_project_shader_asset_root(&project_root, registry_shader_source.as_str());
    let records = shader_resource_records_from_asset_roots(&[
        project_root,
        native_dynamic_fixture_asset_root(),
    ])
    .expect("project/plugin asset-root shader resource records");
    assert_live_registry_records_cover_project_and_plugin(&records);

    let registry_cases = registry_shader_cases_from_live_records(&records);
    assert_eq!(
        registry_cases.len(),
        2,
        "project/plugin asset-root records should become two product registry cases"
    );
    assert_cases_use_live_record_revisions(&registry_cases, &records);

    let manifest = registry_material_pass_live_source_label_prewarm_manifest(&registry_cases);
    let prewarm_report =
        prewarm_shader_variants_with_wgpu_pipeline_validation(&manifest, &cache_roots.staged_root);
    assert_eq!(prewarm_report.requested_count, manifest.variants.len());
    assert_eq!(
        prewarm_report.written_count,
        manifest.variants.len(),
        "asset-root record prewarm should write every requested pass variant; report={prewarm_report:#?}"
    );
    assert_eq!(prewarm_report.failed_count, 0);
    assert!(prewarm_report.failures.is_empty());
    assert!(prewarm_report.wgpu_pipeline_validation.enabled);
    assert_eq!(
        prewarm_report.wgpu_pipeline_validation.validated_count,
        manifest.variants.len()
    );
    assert_registry_material_pass_prewarm_dimensions_written(&prewarm_report);

    for (index, case) in registry_cases.iter().copied().enumerate() {
        let launch = submit_registry_material_passes_with_staged_cache(
            case,
            registry_shader_source.as_str(),
            12_401 + index as u64,
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
            "asset-root record product launch should stay read-only against staged cache",
        );
    }

    let _ = fs::remove_dir_all(&cache_roots.root);
}

fn write_project_shader_asset_root(asset_root: &Path, source: &str) {
    let shader_dir = asset_root.join("shaders");
    fs::create_dir_all(&shader_dir).expect("project shader asset dir");
    fs::write(shader_dir.join("project_shader.wgsl"), source).expect("project shader source");
    fs::write(
        shader_dir.join("project_shader.wgsl.zmeta"),
        format!(
            r#"format_version = 7
uuid = "00000000-0000-0000-0000-000000000062"
url = "{PROJECT_SHADER_LOCATOR}"
asset_kind = "Shader"
unit = "single"
source_digest = "{}"
preview_state = "ready"
importer_id = "zircon.plan08.project_shader"
importer_version = 1
config_hash = "plan08-project-shader-record"
dependencies = []
"#,
            raw_wgsl_hash(source)
        ),
    )
    .expect("project shader zmeta");
}

fn native_dynamic_fixture_asset_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .join("zircon_plugins")
        .join("native_dynamic_fixture")
        .join("assets")
}

fn assert_live_registry_records_cover_project_and_plugin(records: &[ResourceRecord]) {
    let mut locators = records
        .iter()
        .map(|record| record.primary_locator.to_string())
        .collect::<Vec<_>>();
    locators.sort();
    assert_eq!(
        locators,
        vec![
            PLUGIN_SHADER_LOCATOR.to_string(),
            PROJECT_SHADER_LOCATOR.to_string()
        ]
    );
    assert!(records.iter().all(|record| {
        record.kind == ResourceKind::Shader
            && record.state == ResourceState::Ready
            && record.revision != 0
    }));
}

fn assert_cases_use_live_record_revisions(
    cases: &[RegistryShaderCase],
    records: &[ResourceRecord],
) {
    for record in records {
        let label = record.primary_locator.to_string();
        let case = cases
            .iter()
            .find(|case| case.locator == label)
            .unwrap_or_else(|| panic!("product registry case for live record {label}"));
        assert_eq!(
            case.revision, record.revision,
            "product prewarm request should use exported ResourceRecord revision for {label}"
        );
    }
}
