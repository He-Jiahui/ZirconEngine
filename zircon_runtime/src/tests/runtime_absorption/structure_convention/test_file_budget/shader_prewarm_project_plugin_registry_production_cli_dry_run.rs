use super::*;

const STATUS: &str =
    "render_plan08_project_plugin_registry_production_cli_dry_run_python_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_project_plugin_registry_production_cli_dry_run_is_wired() {
    let plugin_meta = read_repo("zircon_plugins/native_dynamic_fixture/assets/shader.wgsl.zmeta");
    let build_tool = read_repo("tools/zircon_build.py");
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let build_prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let plan_08 = read_repo(
        "docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "native dynamic fixture shader sidecar reaches dry-run evidence",
        &plugin_meta,
        &[
            "url = \"package://native_dynamic_fixture/shaders/shader\"",
            "asset_kind = \"Shader\"",
        ],
    );
    assert_contains_all(
        "public CLI dry-run prints selected plugin prewarm command",
        &build_prewarm_tests,
        &[
            "test_cli_dry_run_prints_native_dynamic_fixture_prewarm_command",
            "zircon_build.main",
            "--dry-run",
            "zircon_shader_prewarm",
            "--export-resource-registry",
            "--resource-registry ",
        ],
    );
    assert_contains_all(
        "dry-run build path reaches prewarm owner without subprocess execution",
        &build_tool,
        &[
            "if config.dry_run:",
            "prewarm_shaders(config)",
            "print(\"DRY-RUN\", quote_command(command))",
        ],
    );
    assert_contains_all(
        "prewarm command owner still emits automatic registry export",
        &build_prewarm,
        &[
            "build_shader_prewarm_command",
            "--export-resource-registry",
            "shader_prewarm_resource_registry_path",
        ],
    );

    for (path, source) in [
        (
            "tools/tests/test_zircon_build_shader_prewarm.py",
            build_prewarm_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_project_plugin_registry_production_cli_dry_run.rs",
            include_str!("shader_prewarm_project_plugin_registry_production_cli_dry_run.rs"),
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
                "Project/plugin registry production CLI dry-run handoff",
                STATUS,
                "runtime_15_shader_prewarm_project_plugin_registry_production_cli_dry_run_is_wired",
                "test_cli_dry_run_prints_native_dynamic_fixture_prewarm_command",
            ],
        );
    }
}
