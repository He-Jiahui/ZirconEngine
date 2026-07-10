use super::*;

const STATUS: &str =
    "render_plan08_plugin_shader_asset_roots_auto_export_focused_tests_passed_cargo_deferred_renderdoc_deferred";

#[test]
fn runtime_15_shader_prewarm_plugin_asset_roots_auto_export_is_wired() {
    let package_manifest = read_runtime_src("plugin/package_manifest/plugin_package_manifest.rs");
    let native_fixture_manifest = read_repo("zircon_plugins/native_dynamic_fixture/plugin.toml");
    let native_fixture_shader =
        read_repo("zircon_plugins/native_dynamic_fixture/assets/shader.wgsl");
    let build_tool = read_repo("tools/zircon_build.py");
    let build_plugin_assets = read_repo("tools/zircon_build_plugin_assets.py");
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let build_plugin_tests = read_repo("tools/tests/test_zircon_build_plugin_carriers.py");
    let build_prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let plugin_manifest_doc = read_repo("docs/zircon_runtime/plugin/package_manifest.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "package manifest exposes runtime asset root defaults",
        &package_manifest,
        &[
            "pub asset_roots: Vec<String>",
            "pub fn asset_roots_or_default(&self) -> Vec<String>",
            "vec![\"assets\".to_string()]",
        ],
    );
    assert_contains_all(
        "native dynamic fixture keeps legacy distribution assets",
        &native_fixture_manifest,
        &[
            "id = \"native_dynamic_fixture\"",
            "[distribution]",
            "assets = [\"assets/**\"]",
        ],
    );
    assert_contains_all(
        "native dynamic fixture has a shader payload under its package assets",
        &native_fixture_shader,
        &["@fragment", "native_dynamic_fixture_fragment"],
    );
    assert_contains_all(
        "build tool discovers package asset roots for selected plugins through the plugin-assets owner",
        &(build_tool + &build_plugin_assets),
        &[
            "asset_roots: tuple[Path, ...]",
            "collect_plugin_asset_roots",
            "append_plugin_asset_roots_from_distribution_assets",
            "distribution_asset_root_text",
            "normalized_plugin_asset_root",
            "asset_roots=asset_roots",
        ],
    );
    assert_contains_all(
        "build prewarm helper forwards selected plugin roots as asset roots",
        &build_prewarm,
        &[
            "shader_asset_root_paths_for_prewarm",
            "Path(config.engine_root) / \"assets\"",
            "getattr(plugin, \"asset_roots\", ())",
            "command.extend([\"--asset-root\", str(asset_root)])",
        ],
    );
    assert_contains_all(
        "python tests cover asset root discovery and command forwarding",
        &(build_plugin_tests + &build_prewarm_tests),
        &[
            "test_build_command_includes_selected_plugin_asset_roots",
            "test_zircon_build_discovers_plugin_asset_roots_for_shader_prewarm",
            "test_zircon_build_discovers_distribution_assets_as_plugin_asset_roots",
            "test_zircon_build_uses_existing_default_plugin_assets_root",
        ],
    );

    for (path, source) in [
        (
            "tools/zircon_build_plugin_assets.py",
            build_plugin_assets.as_str(),
        ),
        (
            "tools/zircon_build_shader_prewarm.py",
            build_prewarm.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm.py",
            build_prewarm_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_plugin_asset_roots_auto_export.rs",
            include_str!("shader_prewarm_plugin_asset_roots_auto_export.rs"),
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
        ("build tool doc", build_tool_doc.as_str()),
        ("plugin manifest doc", plugin_manifest_doc.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Plugin shader asset roots auto-export",
                STATUS,
                "test_build_command_includes_selected_plugin_asset_roots",
                "runtime_15_shader_prewarm_plugin_asset_roots_auto_export_is_wired",
            ],
        );
    }
}
