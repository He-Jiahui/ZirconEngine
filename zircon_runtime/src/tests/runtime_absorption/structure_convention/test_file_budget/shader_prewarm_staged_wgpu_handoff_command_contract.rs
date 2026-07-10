use super::*;

const STATUS: &str =
    "render_plan08_build_tool_staged_wgpu_handoff_command_contract_python_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_staged_wgpu_handoff_command_contract_is_wired() {
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let command_contract_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_command_contract.py");
    let plan_08 = read_repo(
        "docs/plans/zircon_runtime/render/08/2026-07-09-material-shader-permutation-output-records.md",
    );
    let render_index =
        read_repo("docs/plans/zircon_runtime/render/08/2026-07-09-index-output-records.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo(
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let session_doc = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "build prewarm helper validates the complete staged WGPU handoff command",
        &build_prewarm,
        &[
            "validate_shader_prewarm_command_contract(config, command)",
            "def validate_shader_prewarm_command_contract",
            "\"--validate-wgpu-modules\"",
            "shader_asset_root_paths_for_prewarm(config)",
            "shader_permutation_registry_paths_for_prewarm(config)",
            "\"--export-resource-registry\"",
            "\"--cache-dir\"",
            "\"--report\"",
        ],
    );
    assert_contains_all(
        "python tests lock the full staged WGPU handoff and failure mode",
        &command_contract_tests,
        &[
            "test_full_staged_wgpu_handoff_keeps_generated_registries_and_roots",
            "test_command_contract_rejects_missing_wgpu_validation_flag",
            "validate_shader_prewarm_command_contract(config, command)",
            "\"--validate-wgpu-modules\"",
            "\"--shader-permutation-registry\"",
            "\"--export-resource-registry\"",
        ],
    );

    for (path, source) in [
        ("tools/zircon_build_shader_prewarm.py", build_prewarm.as_str()),
        (
            "tools/tests/test_zircon_build_shader_prewarm_command_contract.py",
            command_contract_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_staged_wgpu_handoff_command_contract.rs",
            include_str!("shader_prewarm_staged_wgpu_handoff_command_contract.rs"),
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
                "Build-tool staged WGPU handoff command contract",
                STATUS,
                "test_full_staged_wgpu_handoff_keeps_generated_registries_and_roots",
                "runtime_15_shader_prewarm_staged_wgpu_handoff_command_contract_is_wired",
            ],
        );
    }
}
