use super::*;

const STATUS: &str = "render_plan08_project_plugin_registry_production_wrapper_orchestration_passed_cargo_proxy_product_renderdoc_deferred";

#[test]
fn runtime_15_shader_prewarm_project_plugin_registry_wrapper_orchestration_is_wired() {
    let wrapper_test =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_wrapper_orchestration.py");
    let build_tool = read_repo("tools/zircon_build.py");
    let command_helper = read_repo("tools/zircon_build_shader_prewarm.py");
    let acceptance = read_repo("tools/zircon_build_shader_prewarm_acceptance.py");
    let plan_08 = read_repo(
        "docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "public wrapper test runs zircon_build.main through a cargo proxy and real shader prewarm",
        &wrapper_test,
        &[
            "test_public_runtime_wrapper_exports_project_plugin_registry_with_live_wgpu",
            "zircon_build.main",
            "fake_cargo.cmd",
            "PREWARM_EXE",
            "--validate-wgpu-shaders",
            "--plugins",
            "native_dynamic_fixture",
            "requested_count\"])",
            "package://native_dynamic_fixture/shaders/shader",
            "res://project/shaders/project_shader",
        ],
    );
    assert_contains_all(
        "zircon_build public runtime target stages assets then validates shader prewarm acceptance",
        &build_tool,
        &[
            "build_runtime(config, config.runtime_feature_arg, include_preview=True)",
            "stage_engine_assets(config)",
            "if config.prewarm_shaders:",
            "prewarm_shaders(config)",
            "validate_staged_shader_prewarm_acceptance_contract(config)",
        ],
    );
    assert_contains_all(
        "shader prewarm command helper exports project/plugin registry through public cargo run args",
        &command_helper,
        &[
            "build_shader_prewarm_command",
            "shader_asset_root_paths_for_prewarm",
            "--validate-wgpu-modules",
            "--export-resource-registry",
            "validate_shader_prewarm_command_contract",
        ],
    );
    assert_contains_all(
        "acceptance still checks the report, cache, WGPU validation, and registry-backed sources",
        &acceptance,
        &[
            "validate_staged_shader_prewarm_acceptance_contract",
            "require_wgpu_module_validation",
            "validate_shader_prewarm_cache_artifact_contract",
            "require_report_registry_backed_sources=requires_project_plugin_auto_export",
        ],
    );

    for (path, source) in [
        (
            "tools/tests/test_zircon_build_shader_prewarm_wrapper_orchestration.py",
            wrapper_test.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_wrapper_orchestration.rs",
            include_str!(
                "shader_prewarm_project_plugin_registry_production_wrapper_orchestration.rs"
            ),
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
                "Project/plugin registry production wrapper orchestration",
                STATUS,
                "runtime_15_shader_prewarm_project_plugin_registry_wrapper_orchestration_is_wired",
                "cargo proxy",
                "18/18",
            ],
        );
    }
}
