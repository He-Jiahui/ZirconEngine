use super::*;

const STATUS: &str =
    "render_plan08_project_plugin_registry_production_fixture_static_passed_cargo_timeout_no_result";

#[test]
fn runtime_15_shader_prewarm_project_plugin_registry_production_fixture_is_wired() {
    let plugin_manifest = read_repo("zircon_plugins/native_dynamic_fixture/plugin.toml");
    let plugin_shader = read_repo("zircon_plugins/native_dynamic_fixture/assets/shader.wgsl");
    let plugin_meta = read_repo("zircon_plugins/native_dynamic_fixture/assets/shader.wgsl.zmeta");
    let build_tool = read_repo("tools/zircon_build.py");
    let build_plugin_assets = read_repo("tools/zircon_build_plugin_assets.py");
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "native dynamic fixture exposes a real plugin asset root",
        &plugin_manifest,
        &[
            "id = \"native_dynamic_fixture\"",
            "assets = [\"assets/**\"]",
        ],
    );
    assert_contains_all(
        "native dynamic fixture shader has production zmeta identity",
        &plugin_meta,
        &[
            "url = \"package://native_dynamic_fixture/shaders/shader\"",
            "asset_kind = \"Shader\"",
            "unit = \"single\"",
            "source_hash = \"8269800411942cc72d0a20c5bddc4ce19fcaeca1642bbda00c4801cce04b1ed4\"",
            "preview_state = \"ready\"",
        ],
    );
    assert_contains_all(
        "prewarm build helper carries selected plugin asset roots into registry export",
        &build_prewarm,
        &[
            "shader_asset_root_paths_for_prewarm",
            "getattr(plugin, \"asset_roots\", ())",
            "--export-resource-registry",
            "shader_prewarm_resource_registry_path",
        ],
    );
    assert_contains_all(
        "build tool discovers distribution assets for selected plugin roots through the plugin-assets owner",
        &(build_tool + &build_plugin_assets),
        &[
            "collect_plugin_asset_roots",
            "append_plugin_asset_roots_from_distribution_assets",
            "distribution.assets",
            "existing_roots",
        ],
    );

    for (path, source) in [
        (
            "zircon_plugins/native_dynamic_fixture/assets/shader.wgsl",
            plugin_shader.as_str(),
        ),
        (
            "zircon_plugins/native_dynamic_fixture/assets/shader.wgsl.zmeta",
            plugin_meta.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_fixture.rs",
            include_str!("shader_prewarm_project_plugin_registry_production_fixture.rs"),
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
                "Project/plugin registry production fixture prewarm",
                STATUS,
                "runtime_15_shader_prewarm_project_plugin_registry_production_fixture_is_wired",
                "package://native_dynamic_fixture/shaders/shader",
            ],
        );
    }
}
