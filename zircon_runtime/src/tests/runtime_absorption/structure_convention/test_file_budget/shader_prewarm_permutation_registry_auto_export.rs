use super::*;

const STATUS: &str = "render_plan08_build_tool_shader_permutation_registry_auto_export_focused_tests_passed_renderdoc_deferred";

#[test]
fn runtime_15_shader_prewarm_permutation_registry_auto_export_is_wired() {
    let build_tool = read_zircon_build_sources();
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let build_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let plan_08 = read_repo(
        "docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");

    assert_contains_all(
        "zircon build owns the staged permutation registry export path",
        &build_tool,
        &[
            "write_generated_shader_permutation_registry",
            "def shader_prewarm_permutation_registry_path(self) -> Path:",
            "\"shader_permutation_registry.json\"",
            "if not config.dry_run:",
            "write_generated_shader_permutation_registry(config)",
        ],
    );
    assert_contains_all(
        "shader prewarm helper writes generated permutation registries",
        &build_prewarm,
        &[
            "shader_permutation_registry_paths_for_prewarm",
            "generated_shader_permutation_registry_path",
            "write_generated_shader_permutation_registry",
            "generated_shader_permutation_registry_document",
            "\"geometry_source_ids\"",
            "\"shading_model_ids\"",
            "json.dumps(document, indent=2)",
            "shader permutation registry export",
        ],
    );
    assert_contains_all(
        "python focused tests cover auto-export and explicit override",
        &build_tests,
        &[
            "test_build_command_uses_generated_shader_permutation_registry_for_custom_ids",
            "test_build_command_prefers_explicit_shader_permutation_registry",
            "test_generated_shader_permutation_registry_document_groups_custom_ids",
            "test_write_generated_shader_permutation_registry_writes_json",
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
                "Shader permutation registry auto-export",
                STATUS,
                "shader_prewarm_permutation_registry_path",
                "test_write_generated_shader_permutation_registry_writes_json",
                "runtime_15_shader_prewarm_permutation_registry_auto_export_is_wired",
            ],
        );
    }
}
