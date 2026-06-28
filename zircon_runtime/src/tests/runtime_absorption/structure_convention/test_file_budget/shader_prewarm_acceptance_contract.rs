use super::*;

const STATUS: &str =
    "render_plan08_build_tool_staged_prewarm_acceptance_contract_python_passed_cargo_deferred";
const NONEMPTY_STATUS: &str =
    "render_plan08_build_tool_staged_prewarm_nonempty_success_report_python_passed_cargo_deferred";
const WRITTEN_VARIANTS_STATUS: &str =
    "render_plan08_build_tool_staged_prewarm_written_variant_identity_python_passed_cargo_deferred";
const WRITTEN_SOURCE_LABEL_STATUS: &str =
    "render_plan08_build_tool_staged_prewarm_written_source_label_identity_python_passed_cargo_deferred";
const COMPLETE_WRITTEN_STATUS: &str =
    "render_plan08_build_tool_staged_prewarm_complete_written_count_python_passed_cargo_deferred";
const PRODUCT_PASS_STATUS: &str =
    "render_plan08_build_tool_product_base_pass_acceptance_contract_python_passed_cargo_deferred";
const PRODUCT_MATERIAL_PASS_STATUS: &str =
    "render_plan08_build_tool_product_material_mesh_pass_acceptance_contract_python_passed_cargo_deferred";
const WRITTEN_VARIANT_UNIQUENESS_STATUS: &str =
    "render_plan08_build_tool_written_variant_uniqueness_contract_python_passed_cargo_deferred";
const WRITTEN_HASH_SHAPE_STATUS: &str =
    "render_plan08_build_tool_staged_prewarm_written_cache_hash_shape_python_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_acceptance_contract_is_wired() {
    let acceptance_helper = read_repo("tools/zircon_build_shader_prewarm_acceptance.py");
    let written_variants_helper =
        read_repo("tools/zircon_build_shader_prewarm_written_variants.py");
    let build_tool = read_repo("tools/zircon_build.py");
    let acceptance_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py");
    let prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let permutation_registry_tests =
        read_repo("tools/tests/test_zircon_build_shader_permutation_registry_contract.py");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session_doc = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "staged shader prewarm acceptance helper owns the success bundle",
        &acceptance_helper,
        &[
            "def validate_staged_shader_prewarm_acceptance_contract",
            "def validate_staged_shader_prewarm_runtime_fallback_layout",
            "def validate_staged_shader_prewarm_nonempty_success_report",
            "def _validate_staged_shader_prewarm_written_variant_identity",
            "validate_unique_written_variant_identity(",
            "validate_cache_hash_shape(",
            "_reported_written_variants_for_acceptance",
            "message_prefix=\"staged shader prewarm acceptance rejects\"",
            "_PRODUCT_MATERIAL_MESH_PASS_TYPES = (",
            "\"forward\"",
            "\"gbuffer\"",
            "\"depth_prepass\"",
            "\"shadow\"",
            "\"velocity\"",
            "\"taa_reactive_mask\"",
            "expected_cache_root = engine_root / \"cache\" / \"shader_variants\"",
            "expected_report_path = engine_root / \"cache\" / \"shader_variants_report.json\"",
            "expected_resource_registry_path = (",
            "staged shader prewarm cache root must match runtime fallback root",
            "staged shader prewarm acceptance requires written variants",
            "staged shader prewarm acceptance requires zero failed variants",
            "staged shader prewarm acceptance requires all requested variants written",
            "staged shader prewarm acceptance requires written cache variants",
            "staged shader prewarm acceptance requires written cache variant ",
            "\"cache_hash\"",
            "\"canonical_string\"",
            "\"source_label\"",
            "validate_shader_prewarm_report_contract(",
            "validate_shader_prewarm_cache_artifact_contract(",
            "validate_shader_resource_registry_export_contract(",
            "require_wgpu_module_validation=getattr(",
            "require_source_provenance=True",
            "expected_pass_types=_PRODUCT_MATERIAL_MESH_PASS_TYPES",
            "expected_quality_tiers=config.shader_quality_tiers",
            "expected_geometry_sources=config.shader_geometry_sources",
            "expected_geometry_source_ids=expected_geometry_source_ids",
            "expected_shading_model_ids=expected_shading_model_ids",
        ],
    );
    assert_contains_all(
        "shared written variant helper rejects duplicate identity rows",
        &written_variants_helper,
        &[
            "class ReportedWrittenVariant",
            "def reported_written_variants(",
            "def validate_unique_written_variant_identity(",
            "def validate_cache_hash_shape(",
            "def validate_written_variant_source_labels(",
            "_BLAKE3_HEX_LENGTH = 64",
            "duplicate written cache variant identity",
            "cache_hash=",
            "canonical_string=",
        ],
    );
    assert_contains_all(
        "zircon build success path calls only the acceptance bundle",
        &build_tool,
        &[
            "validate_staged_shader_prewarm_acceptance_contract",
            "if result.returncode == 0:",
            "validate_staged_shader_prewarm_acceptance_contract(config)",
        ],
    );
    assert_contains_all(
        "python tests lock acceptance behavior and build handoff",
        &acceptance_tests,
        &[
            "test_acceptance_contract_validates_report_cache_and_exported_registry",
            "test_acceptance_contract_skips_export_validation_for_explicit_registry",
            "test_acceptance_contract_rejects_runtime_fallback_layout_drift",
            "test_acceptance_contract_accepts_runtime_fallback_layout",
            "test_acceptance_contract_rejects_empty_success_report",
            "test_acceptance_contract_rejects_failed_success_report",
            "test_acceptance_contract_rejects_partial_written_success_report",
            "test_acceptance_contract_requires_written_variant_identity",
            "test_acceptance_contract_rejects_incomplete_written_variant_identity",
            "test_acceptance_contract_requires_written_variant_source_label_identity",
            "test_acceptance_contract_rejects_duplicate_written_variant_identity",
            "test_acceptance_contract_rejects_invalid_written_variant_cache_hash_shape",
            "test_acceptance_contract_rejects_forward_only_staged_pass_report",
            "test_prewarm_shaders_runs_acceptance_bundle_after_success",
            "expected_pass_types",
            "\"taa_reactive_mask\"",
            "validate_staged_shader_prewarm_acceptance_contract(config)",
        ],
    );
    assert_contains_all(
        "existing prewarm tests now verify the acceptance entry point",
        &prewarm_tests,
        &[
            "test_prewarm_shaders_validates_staged_acceptance_after_success",
            "test_prewarm_shaders_uses_same_acceptance_entry_for_explicit_registry",
            "validate_staged_shader_prewarm_acceptance_contract",
        ],
    );
    assert_contains_all(
        "permutation registry tests keep success validation routed through acceptance",
        &permutation_registry_tests,
        &[
            "test_prewarm_shaders_validates_generated_registry_before_run",
            "test_prewarm_shaders_passes_selected_custom_ids_to_acceptance_contract",
            "validate_staged_shader_prewarm_acceptance_contract",
        ],
    );
    assert!(
        !permutation_registry_tests.contains("\"validate_shader_prewarm_report_contract\""),
        "permutation registry build tests should patch the staged acceptance entry point, not the lower-level report validator"
    );

    for (path, source) in [
        ("tools/zircon_build_shader_prewarm_acceptance.py", acceptance_helper.as_str()),
        (
            "tools/zircon_build_shader_prewarm_written_variants.py",
            written_variants_helper.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py",
            acceptance_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_acceptance_contract.rs",
            include_str!("shader_prewarm_acceptance_contract.rs"),
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
                "Build-tool staged prewarm acceptance contract",
                STATUS,
                "Build-tool staged prewarm nonempty success report acceptance",
                NONEMPTY_STATUS,
                "Build-tool staged prewarm written variant identity acceptance",
                WRITTEN_VARIANTS_STATUS,
                "Build-tool staged prewarm written source-label identity acceptance",
                WRITTEN_SOURCE_LABEL_STATUS,
                "Build-tool staged prewarm complete written count acceptance",
                COMPLETE_WRITTEN_STATUS,
                "Build-tool product Base pass acceptance contract",
                PRODUCT_PASS_STATUS,
                "Build-tool product material mesh pass acceptance contract",
                PRODUCT_MATERIAL_PASS_STATUS,
                "Build-tool written variant uniqueness contract",
                WRITTEN_VARIANT_UNIQUENESS_STATUS,
                "Build-tool staged prewarm written cache-hash shape acceptance",
                WRITTEN_HASH_SHAPE_STATUS,
                "test_acceptance_contract_validates_report_cache_and_exported_registry",
                "test_acceptance_contract_rejects_forward_only_staged_pass_report",
                "expected_pass_types",
                "taa_reactive_mask",
                "test_acceptance_contract_rejects_empty_success_report",
                "test_acceptance_contract_rejects_failed_success_report",
                "test_acceptance_contract_rejects_partial_written_success_report",
                "test_acceptance_contract_requires_written_variant_identity",
                "test_acceptance_contract_rejects_incomplete_written_variant_identity",
                "test_acceptance_contract_requires_written_variant_source_label_identity",
                "test_acceptance_contract_rejects_duplicate_written_variant_identity",
                "test_acceptance_contract_rejects_invalid_written_variant_cache_hash_shape",
                "test_validate_cache_artifact_contract_rejects_duplicate_written_variant_identity",
                "tools/zircon_build_shader_prewarm_written_variants.py",
                "duplicate written cache variant identity",
                "runtime fallback root",
                "runtime_15_shader_prewarm_acceptance_contract_is_wired",
            ],
        );
    }
}
