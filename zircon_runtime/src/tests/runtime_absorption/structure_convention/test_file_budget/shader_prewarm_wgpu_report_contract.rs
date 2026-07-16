use super::*;

const STATUS: &str = "render_plan08_build_tool_wgpu_report_contract_python_passed_cargo_deferred";
const TOTALS_MATCH_STATUS: &str =
    "render_plan08_build_tool_wgpu_validation_totals_match_python_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_wgpu_report_contract_is_wired() {
    let build = read_repo("tools/zircon_build.py");
    let acceptance_helper = read_repo("tools/zircon_build_shader_prewarm_acceptance.py");
    let report_contract = read_repo("tools/zircon_build_shader_prewarm_report_contract.py");
    let build_prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let wgpu_report_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_wgpu_report_contract.py");
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
        "build helper validates successful WGPU module validation reports",
        &report_contract,
        &[
            "def validate_shader_prewarm_report_contract(",
            "require_wgpu_module_validation: bool = False",
            "require_wgpu_pipeline_validation: bool = False",
            "shader prewarm report did not confirm WGPU module validation",
            "shader prewarm report did not confirm WGPU render pipeline validation",
            "_count_value(validation, \"requested\")",
            "_count_value(validation, \"validated\")",
            "validated != requested",
            "shader prewarm WGPU {label} validation did not validate every",
            "shader prewarm WGPU {label} validation counts did not match",
            "report_requested = _count_value(report, \"requested\")",
            "requested != report_requested",
        ],
    );
    assert_contains_all(
        "staged build enforces the report contract after a successful prewarm run",
        &(build + &acceptance_helper),
        &[
            "validate_shader_prewarm_report_contract",
            "if result.returncode == 0:",
            "require_wgpu_module_validation=getattr(",
            "\"validate_wgpu_shaders\"",
        ],
    );
    assert_contains_all(
        "python tests cover report contract enforcement",
        &wgpu_report_tests,
        &[
            "class ZirconBuildShaderPrewarmWgpuReportContractTests",
            "test_validate_report_contract_requires_wgpu_validation_when_requested",
            "test_validate_report_contract_accepts_wgpu_validation_counts",
            "test_validate_report_contract_rejects_wgpu_validation_total_mismatch",
            "WGPU module validation counts did not match report totals",
        ],
    );
    assert_contains_all(
        "general prewarm tests still cover staged success behavior",
        &build_prewarm_tests,
        &[
            "test_prewarm_shaders_validates_staged_acceptance_after_success",
            "nonzero prewarm should not validate",
        ],
    );
    assert!(
        !build_prewarm_tests
            .contains("test_validate_report_contract_rejects_wgpu_validation_total_mismatch"),
        "WGPU report-contract regressions should live in the dedicated WGPU report test owner"
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
            "tools/tests/test_zircon_build_shader_prewarm_wgpu_report_contract.py",
            wgpu_report_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_wgpu_report_contract.rs",
            include_str!("shader_prewarm_wgpu_report_contract.rs"),
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
                "Build-tool WGPU validation report contract",
                STATUS,
                "Build-tool WGPU validation totals match contract",
                TOTALS_MATCH_STATUS,
                "test_zircon_build_shader_prewarm_wgpu_report_contract.py",
                "test_validate_report_contract_requires_wgpu_validation_when_requested",
                "test_validate_report_contract_rejects_wgpu_validation_total_mismatch",
                "runtime_15_shader_prewarm_wgpu_report_contract_is_wired",
            ],
        );
    }
}
