use super::*;

const STATUS: &str =
    "render_plan08_project_plugin_registry_production_command_python_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_project_plugin_registry_production_command_is_wired() {
    let plugin_manifest = read_repo("zircon_plugins/native_dynamic_fixture/plugin.toml");
    let plugin_meta = read_repo("zircon_plugins/native_dynamic_fixture/assets/shader.wgsl.zmeta");
    let build_tool = read_repo("tools/zircon_build.py");
    let build_plugin_assets = read_repo("tools/zircon_build_plugin_assets.py");
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let build_prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let plan_08 = read_repo(
        "docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo(
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let session_doc = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "native dynamic fixture exposes distribution assets",
        &plugin_manifest,
        &[
            "id = \"native_dynamic_fixture\"",
            "assets = [\"assets/**\"]",
        ],
    );
    assert_contains_all(
        "native dynamic fixture shader sidecar is a ready shader",
        &plugin_meta,
        &[
            "url = \"package://native_dynamic_fixture/shaders/shader\"",
            "asset_kind = \"Shader\"",
            "preview_state = \"ready\"",
        ],
    );
    assert_contains_all(
        "build helper command includes selected plugin production asset roots",
        &build_prewarm_tests,
        &[
            "test_build_command_auto_export_registry_uses_native_dynamic_fixture_assets",
            "zircon_build.discover_plugins(repo_root)",
            "packages[\"native_dynamic_fixture\"]",
            "plugin_asset_root",
            "shader.wgsl.zmeta",
            "--export-resource-registry",
        ],
    );
    assert_contains_all(
        "production prewarm command still uses shared helper seams",
        &build_prewarm,
        &[
            "build_shader_prewarm_command",
            "shader_asset_root_paths_for_prewarm",
            "getattr(plugin, \"asset_roots\", ())",
            "validate_shader_prewarm_command_contract",
        ],
    );
    assert_contains_all(
        "build tool discovers selected plugin assets from distribution metadata through the plugin-assets owner",
        &(build_tool + &build_plugin_assets),
        &[
            "collect_plugin_asset_roots",
            "append_plugin_asset_roots_from_distribution_assets",
            "distribution.assets",
        ],
    );

    for (path, source) in [
        (
            "tools/tests/test_zircon_build_shader_prewarm.py",
            build_prewarm_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_command.rs",
            include_str!("shader_prewarm_project_plugin_registry_production_command.rs"),
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
        ("render session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Project/plugin registry production command handoff",
                STATUS,
                "runtime_15_shader_prewarm_project_plugin_registry_production_command_is_wired",
                "test_build_command_auto_export_registry_uses_native_dynamic_fixture_assets",
            ],
        );
    }
}
