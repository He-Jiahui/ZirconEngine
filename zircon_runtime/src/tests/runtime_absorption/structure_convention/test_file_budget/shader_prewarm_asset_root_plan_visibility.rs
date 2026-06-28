use super::*;

const STATUS: &str =
    "render_plan08_build_tool_shader_asset_root_plan_visibility_python_passed_cargo_deferred";
const PLAN_TEST: &str = "test_prewarm_plan_lists_asset_roots_for_registry_export";
const FALLBACK_HANDOFF_TEST: &str = "test_prewarm_plan_lists_runtime_fallback_handoff_paths";
const COMMAND_TEST: &str = "test_build_command_auto_export_registry_scans_all_asset_roots";

#[test]
fn runtime_15_shader_prewarm_asset_root_plan_visibility_is_wired() {
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let build_prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "build prewarm plan prints the same asset-root set used by registry export",
        &build_prewarm,
        &[
            "def print_shader_prewarm_plan(config) -> None:",
            "\"  shader asset roots: \"",
            "\"  shader prewarm cache root: \"",
            "\"  shader prewarm report: \"",
            "\"  shader runtime fallback root: \"",
            "config.shader_prewarm_cache_root",
            "config.shader_prewarm_report_path",
            "shader_asset_root_paths_for_prewarm(config)",
            "\"  shader resource registry export: \"",
        ],
    );
    assert_contains_all(
        "python regressions cover plan visibility and all-root auto export",
        &build_prewarm_tests,
        &[
            PLAN_TEST,
            FALLBACK_HANDOFF_TEST,
            COMMAND_TEST,
            "with redirect_stdout(output):",
            "print_shader_prewarm_plan(config)",
            "command.index(\"--export-resource-registry\")",
            "\"  shader asset roots: \"",
            "\"  shader prewarm cache root: \"",
            "\"  shader runtime fallback root: \"",
        ],
    );

    for (path, source) in [
        (
            "tools/zircon_build_shader_prewarm.py",
            build_prewarm.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm.py",
            build_prewarm_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_asset_root_plan_visibility.rs",
            include_str!("shader_prewarm_asset_root_plan_visibility.rs"),
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
        ("render session doc", session_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Build-tool shader asset-root plan visibility",
                STATUS,
                PLAN_TEST,
                FALLBACK_HANDOFF_TEST,
                COMMAND_TEST,
                "shader runtime fallback root",
                "runtime_15_shader_prewarm_asset_root_plan_visibility_is_wired",
            ],
        );
    }
}
