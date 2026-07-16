use super::*;

const STATUS: &str =
    "render_plan08_project_plugin_registry_production_cli_selection_python_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_project_plugin_registry_production_cli_selection_is_wired() {
    let plugin_manifest = read_repo("zircon_plugins/native_dynamic_fixture/plugin.toml");
    let plugin_meta = read_repo("zircon_plugins/native_dynamic_fixture/assets/shader.wgsl.zmeta");
    let build_tool = read_repo("tools/zircon_build.py");
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let build_prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "native dynamic fixture keeps production distribution assets",
        &plugin_manifest,
        &[
            "id = \"native_dynamic_fixture\"",
            "assets = [\"assets/**\"]",
        ],
    );
    assert_contains_all(
        "native dynamic fixture shader sidecar is available to selected CLI package",
        &plugin_meta,
        &[
            "url = \"package://native_dynamic_fixture/shaders/shader\"",
            "asset_kind = \"Shader\"",
        ],
    );
    assert_contains_all(
        "CLI config selects real plugin assets for production prewarm command",
        &build_prewarm_tests,
        &[
            "test_cli_selects_native_dynamic_fixture_assets_for_prewarm_command",
            "--plugins",
            "native_dynamic_fixture",
            "zircon_build.resolve_config",
            "build_shader_prewarm_command(config)",
            "--export-resource-registry",
            "--resource-registry",
        ],
    );
    assert_contains_all(
        "build config selection feeds selected plugin packages",
        &build_tool,
        &[
            "selected_plugins = tuple(select_plugins(candidates, args.plugins))",
            "plugins=selected_plugins",
            "collect_plugin_asset_roots",
        ],
    );
    assert_contains_all(
        "prewarm command consumes selected plugin asset roots",
        &build_prewarm,
        &[
            "shader_asset_root_paths_for_prewarm",
            "getattr(plugin, \"asset_roots\", ())",
            "--asset-root",
            "--export-resource-registry",
        ],
    );

    for (path, source) in [
        (
            "tools/tests/test_zircon_build_shader_prewarm.py",
            build_prewarm_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_cli_selection.rs",
            include_str!("shader_prewarm_project_plugin_registry_production_cli_selection.rs"),
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
                "Project/plugin registry production CLI selection handoff",
                STATUS,
                "runtime_15_shader_prewarm_project_plugin_registry_production_cli_selection_is_wired",
                "test_cli_selects_native_dynamic_fixture_assets_for_prewarm_command",
            ],
        );
    }
}
