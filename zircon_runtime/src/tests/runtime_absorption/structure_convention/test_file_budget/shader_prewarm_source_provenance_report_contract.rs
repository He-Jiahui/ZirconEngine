use super::*;

const STATUS: &str =
    "render_plan08_build_tool_source_provenance_report_contract_python_passed_cargo_deferred";
const COUNT_TOTALS_STATUS: &str =
    "render_plan08_build_tool_source_provenance_totals_match_python_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_source_provenance_report_contract_is_wired() {
    let build = read_repo("tools/zircon_build.py");
    let report_contract = read_repo("tools/zircon_build_shader_prewarm_report_contract.py");
    let build_prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let source_provenance_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_source_provenance_contract.py");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "build helper validates source provenance reports",
        &report_contract,
        &[
            "require_source_provenance: bool = False",
            "def _validate_source_provenance_contract(",
            "shader prewarm report did not confirm shader source provenance",
            "_count_value(provenance, \"source\")",
            "_count_value(provenance, \"variant\")",
            "source_entries={entry_requested_count}",
            "source provenance counts did not match report totals",
            "entry_written_count",
            "entry_failed_count",
        ],
    );
    assert_contains_all(
        "staged build requires provenance after a successful prewarm run",
        &build,
        &[
            "validate_shader_prewarm_report_contract",
            "if result.returncode == 0:",
            "require_source_provenance=True",
        ],
    );
    assert_contains_all(
        "python regressions cover source provenance contract enforcement",
        &source_provenance_tests,
        &[
            "class ZirconBuildShaderPrewarmSourceProvenanceContractTests",
            "test_validate_report_contract_requires_source_provenance_when_requested",
            "test_validate_report_contract_accepts_source_provenance_counts",
            "test_validate_report_contract_rejects_source_provenance_count_mismatch",
            "require_source_provenance",
        ],
    );
    assert!(
        !build_prewarm_tests
            .contains("test_validate_report_contract_rejects_source_provenance_count_mismatch"),
        "source provenance report-contract regressions should live in the dedicated source provenance test owner"
    );

    for (path, source) in [
        (
            "tools/zircon_build_shader_prewarm_report_contract.py",
            report_contract.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm.py",
            build_prewarm_tests.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm_source_provenance_contract.py",
            source_provenance_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_source_provenance_report_contract.rs",
            include_str!("shader_prewarm_source_provenance_report_contract.rs"),
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
        ("session note", session.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                "Build-tool source provenance report contract",
                STATUS,
                "Build-tool source provenance totals match contract",
                COUNT_TOTALS_STATUS,
                "test_zircon_build_shader_prewarm_source_provenance_contract.py",
                "test_validate_report_contract_requires_source_provenance_when_requested",
                "test_validate_report_contract_rejects_source_provenance_count_mismatch",
                "runtime_15_shader_prewarm_source_provenance_report_contract_is_wired",
                "66/66",
            ],
        );
    }
}
