use super::*;

const STATUS: &str =
    "render_plan08_project_shader_asset_roots_auto_export_python_static_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_project_asset_roots_auto_export_is_wired() {
    let build_tool = read_zircon_build_sources();
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let build_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let plan_08 = read_repo(
        "docs/plans/_archive/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "build config carries explicit project shader asset roots",
        &build_tool,
        &[
            "shader_asset_roots: tuple[Path, ...]",
            "\"--shader-asset-root\"",
            "Project shader asset root to scan during --prewarm-shaders",
            "shader_asset_roots=resolve_optional_paths(args.shader_asset_root)",
        ],
    );
    assert_contains_all(
        "prewarm command merges staged engine, project, and selected plugin roots",
        &build_prewarm,
        &[
            "shader_asset_root_paths_for_prewarm",
            "Path(config.engine_root) / \"assets\"",
            "getattr(config, \"shader_asset_roots\", ())",
            "getattr(plugin, \"asset_roots\", ())",
            "command.extend([\"--asset-root\", str(asset_root)])",
        ],
    );
    assert_contains_all(
        "python tests cover project root command forwarding and config parsing",
        &build_tests,
        &[
            "test_zircon_build_resolves_project_shader_asset_roots_for_prewarm",
            "\"--shader-asset-root\"",
            "Project/assets",
            "Project/generated/shaders",
            "test_build_command_auto_export_registry_scans_all_asset_roots",
        ],
    );

    for (path, source) in [
        (
            "tools/zircon_build_shader_prewarm.py",
            build_prewarm.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm.py",
            build_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_asset_roots_auto_export.rs",
            include_str!("shader_prewarm_project_asset_roots_auto_export.rs"),
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
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Project shader asset roots auto-export",
                STATUS,
                "--shader-asset-root",
                "test_zircon_build_resolves_project_shader_asset_roots_for_prewarm",
                "runtime_15_shader_prewarm_project_asset_roots_auto_export_is_wired",
            ],
        );
    }
}
