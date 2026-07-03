use super::*;

const STATUS: &str =
    "render_plan08_build_tool_report_dimension_contract_python_passed_cargo_deferred";
const PERMUTATION_ID_STATUS: &str =
    "render_plan08_build_tool_permutation_id_report_dimension_contract_python_passed_cargo_deferred";
const PRODUCT_PASS_STATUS: &str =
    "render_plan08_build_tool_product_base_pass_acceptance_contract_python_passed_cargo_deferred";
const COMPLETE_DIMENSION_STATUS: &str =
    "render_plan08_build_tool_report_dimension_complete_counts_python_passed_cargo_deferred";
const DIMENSION_TOTALS_STATUS: &str =
    "render_plan08_build_tool_report_dimension_totals_match_python_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_report_dimension_contract_is_wired() {
    let build = read_repo("tools/zircon_build.py");
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let acceptance_helper = read_repo("tools/zircon_build_shader_prewarm_acceptance.py");
    let report_contract = read_repo("tools/zircon_build_shader_prewarm_report_contract.py");
    let build_prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let acceptance_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py");
    let dimension_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py");
    let permutation_registry_tests =
        read_repo("tools/tests/test_zircon_build_shader_permutation_registry_contract.py");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "build helper validates requested prewarm dimensions against report summary",
        &report_contract,
        &[
            "expected_quality_tiers: Sequence[str] = ()",
            "expected_pass_types: Sequence[str] = ()",
            "expected_geometry_sources: Sequence[str] = ()",
            "expected_geometry_source_ids: Sequence[str] = ()",
            "expected_shading_model_ids: Sequence[str] = ()",
            "def _validate_expected_dimension_contract(",
            "def _validate_expected_pass_types(",
            "def _validate_expected_quality_tiers(",
            "def _validate_expected_geometry_sources(",
            "def _validate_expected_shader_dimension_ids(",
            "def _validate_dimension_summary_totals_match_report(",
            "def _dimension_group_totals(",
            "def _shader_dimension_id_records(",
            "def _dimension_has_requested_count(",
            "def _incomplete_dimension_counts(",
            "def _geometry_source_dimension_id(",
            "missing requested pass types",
            "missing requested quality tiers",
            "missing requested geometry sources",
            "missing requested shader geometry source ids",
            "missing requested shader shading model ids",
            "did not fully write requested pass types",
            "did not fully write requested quality tiers",
            "did not fully write requested geometry sources",
            "did not fully write requested shader geometry source ids",
            "did not fully write requested shader shading model ids",
            "counts did not match report totals",
        ],
    );
    assert_contains_all(
        "staged build and acceptance helper route report dimensions through dedicated owner",
        &[
            build.as_str(),
            build_prewarm.as_str(),
            acceptance_helper.as_str(),
        ]
        .join("\n"),
        &[
            "from .zircon_build_shader_prewarm_report_contract import (",
            "validate_staged_shader_prewarm_acceptance_contract(config)",
            "expected_geometry_source_ids = shader_geometry_source_id_specs(config)",
            "expected_shading_model_ids = shader_shading_model_id_specs(config)",
            "_PRODUCT_MATERIAL_MESH_PASS_TYPES = (",
            "\"taa_reactive_mask\"",
            "expected_pass_types=_PRODUCT_MATERIAL_MESH_PASS_TYPES",
            "expected_quality_tiers=config.shader_quality_tiers",
            "expected_geometry_sources=config.shader_geometry_sources",
            "expected_geometry_source_ids=expected_geometry_source_ids",
            "expected_shading_model_ids=expected_shading_model_ids",
        ],
    );
    let combined_prewarm_tests = [
        build_prewarm_tests.as_str(),
        acceptance_tests.as_str(),
        dimension_tests.as_str(),
        permutation_registry_tests.as_str(),
    ]
    .join("\n");
    assert_contains_all(
        "python regressions cover report dimension contract and acceptance handoff",
        &combined_prewarm_tests,
        &[
            "test_prewarm_shaders_validates_staged_acceptance_after_success",
            "expected_pass_types",
            "expected_quality_tiers",
            "expected_geometry_sources",
            "test_validate_report_contract_requires_requested_pass_types",
            "test_validate_report_contract_requires_requested_quality_tiers",
            "test_validate_report_contract_requires_requested_geometry_sources",
            "test_validate_report_contract_rejects_incomplete_requested_dimension_counts",
            "test_validate_report_contract_rejects_dimension_count_total_mismatch",
            "test_validate_report_contract_requires_requested_geometry_source_ids",
            "test_validate_report_contract_requires_requested_shading_model_ids",
            "test_validate_report_contract_accepts_requested_dimensions",
            "test_acceptance_contract_validates_report_cache_and_exported_registry",
            "test_prewarm_shaders_passes_selected_custom_ids_to_acceptance_contract",
        ],
    );

    for (path, source) in [
        (
            "tools/zircon_build_shader_prewarm.py",
            build_prewarm.as_str(),
        ),
        (
            "tools/zircon_build_shader_prewarm_acceptance.py",
            acceptance_helper.as_str(),
        ),
        (
            "tools/zircon_build_shader_prewarm_report_contract.py",
            report_contract.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm.py",
            build_prewarm_tests.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py",
            acceptance_tests.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm_dimension_contract.py",
            dimension_tests.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_permutation_registry_contract.py",
            permutation_registry_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_report_dimension_contract.rs",
            include_str!("shader_prewarm_report_dimension_contract.rs"),
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
                "Build-tool shader prewarm report dimension contract",
                STATUS,
                "Build-tool shader permutation id report dimension contract",
                PERMUTATION_ID_STATUS,
                "Build-tool product Base pass acceptance contract",
                PRODUCT_PASS_STATUS,
                "Build-tool shader prewarm report dimension complete-count contract",
                COMPLETE_DIMENSION_STATUS,
                "Build-tool shader prewarm report dimension totals match contract",
                DIMENSION_TOTALS_STATUS,
                "test_validate_report_contract_requires_requested_pass_types",
                "test_validate_report_contract_requires_requested_quality_tiers",
                "test_validate_report_contract_rejects_incomplete_requested_dimension_counts",
                "test_validate_report_contract_rejects_dimension_count_total_mismatch",
                "test_validate_report_contract_requires_requested_geometry_source_ids",
                "runtime_15_shader_prewarm_report_dimension_contract_is_wired",
            ],
        );
    }
}
