use super::*;

const STATUS: &str = "render_plan08_project_plugin_registry_production_direct_wgpu_export_passed_product_renderdoc_deferred";

#[test]
fn runtime_15_shader_prewarm_project_plugin_registry_production_live_wgpu_is_wired() {
    let run = read_repo("zircon_runtime/src/bin/zircon_shader_prewarm/run.rs");
    let acceptance = read_repo("tools/zircon_build_shader_prewarm_acceptance.py");
    let registry_helper = read_repo("tools/zircon_build_shader_resource_registry.py");
    let plugin_meta = read_repo("zircon_plugins/native_dynamic_fixture/assets/shader.wgsl.zmeta");
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "direct prewarm run exports registry then consumes it for asset-root manifest scans",
        &run,
        &[
            "export_shader_resource_registry_for_asset_roots",
            "ShaderPrewarmResourceRegistryOverlay::from_records",
            "asset_root_manifest_with_resource_registry_revisions",
            "prewarm_shader_variants_with_wgpu_module_validation",
            "write_shader_prewarm_report",
        ],
    );
    assert_contains_all(
        "staged acceptance validates WGPU report, cache artifacts, and registry-backed sources",
        &acceptance,
        &[
            "validate_staged_shader_prewarm_acceptance_contract",
            "require_wgpu_module_validation",
            "validate_shader_prewarm_cache_artifact_contract",
            "require_usable_shader_records=requires_project_plugin_auto_export",
            "require_report_registry_backed_sources=requires_project_plugin_auto_export",
        ],
    );
    assert_contains_all(
        "registry helper still requires ready shader records and report-visible locators",
        &registry_helper,
        &[
            "validate_shader_resource_registry_export_contract",
            "require_usable_shader_records",
            "require_report_registry_backed_sources",
            "\"Ready\"",
            "_RESOURCE_REGISTRY_BACKED_LOCATOR_SCHEMES",
        ],
    );
    assert_contains_all(
        "native dynamic fixture sidecar is the real selected-plugin live export input",
        &plugin_meta,
        &[
            "url = \"package://native_dynamic_fixture/shaders/shader\"",
            "asset_kind = \"Shader\"",
            "preview_state = \"ready\"",
        ],
    );

    for (path, source) in [
        (
            "tools/zircon_build_shader_prewarm_acceptance.py",
            acceptance.as_str(),
        ),
        (
            "tools/zircon_build_shader_resource_registry.py",
            registry_helper.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_live_wgpu.rs",
            include_str!("shader_prewarm_project_plugin_registry_production_live_wgpu.rs"),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 owner budget; got {line_count} lines"
        );
    }

    for (label, source) in [
        ("Plan 08", plan_08.as_str()),
        ("render index", render_index.as_str()),
        ("shader doc", shader_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Project/plugin registry production direct WGPU export",
                STATUS,
                "runtime_15_shader_prewarm_project_plugin_registry_production_live_wgpu_is_wired",
                "18/18",
                "Ready Shader records",
            ],
        );
    }
}
