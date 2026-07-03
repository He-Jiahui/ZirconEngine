use super::*;

const STATUS: &str =
    "render_plan08_build_tool_resource_registry_report_correlation_python_passed_cargo_deferred";
const WRITTEN_VARIANT_STATUS: &str =
    "render_plan08_build_tool_resource_registry_written_source_label_python_passed_cargo_deferred";
const USABLE_RECORD_STATUS: &str =
    "render_plan08_build_tool_resource_registry_usable_shader_revision_python_passed_cargo_deferred";
const READY_RECORD_STATUS: &str =
    "render_plan08_resource_registry_ready_shader_revision_contract_python_static_passed_cargo_deferred";
const OWNER_SPLIT_STATUS: &str =
    "render_plan08_build_tool_resource_registry_contract_tests_owner_split_python_passed_cargo_deferred";
const RECORD_SHAPE_STATUS: &str =
    "render_plan08_build_tool_resource_registry_record_shape_python_passed_cargo_deferred";
const ENUM_SHAPE_STATUS: &str =
    "render_plan08_build_tool_resource_registry_enum_wire_shape_python_passed_cargo_deferred";
const NUMERIC_WIDTH_STATUS: &str =
    "render_plan08_build_tool_resource_registry_numeric_width_python_passed_cargo_deferred";
const LOCATOR_SHAPE_STATUS: &str =
    "render_plan08_build_tool_resource_registry_locator_wire_shape_python_passed_cargo_deferred";
const REGISTRY_BACKED_LOCATOR_STATUS: &str =
    "render_plan08_build_tool_resource_registry_backed_locator_correlation_python_passed_cargo_deferred";

#[test]
fn runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired() {
    let build_prewarm = read_repo("tools/zircon_build_shader_prewarm.py");
    let acceptance_helper = read_repo("tools/zircon_build_shader_prewarm_acceptance.py");
    let resource_registry = read_repo("tools/zircon_build_shader_resource_registry.py");
    let build_prewarm_tests = read_repo("tools/tests/test_zircon_build_shader_prewarm.py");
    let acceptance_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py");
    let registry_tests =
        read_repo("tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py");
    let registry_report_test_sources = format!("{acceptance_tests}\n{registry_tests}");
    let plan_08 = read_repo("docs/plans/zircon_runtime/render/08-material-shader-permutation.md");
    let render_index = read_repo("docs/plans/zircon_runtime/render/index.md");
    let shader_doc = read_repo("docs/zircon_runtime/core/framework/render/shader.md");
    let build_tool_doc = read_repo("docs/cli-and-tooling/zircon-build-tool.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let session = read_repo(".codex/sessions/20260628-0141-render-plan08-continuation.md");

    assert_contains_all(
        "build helper exposes the resource registry export contract",
        &build_prewarm,
        &["validate_shader_resource_registry_export_contract"],
    );
    assert_contains_all(
        "resource registry helper correlates exported locators with report source provenance",
        &resource_registry,
        &[
            "report_path: Path | None = None",
            "_RESOURCE_RECORD_REQUIRED_FIELDS",
            "\"id\"",
            "\"kind\"",
            "\"primary_locator\"",
            "\"artifact_locator\"",
            "\"state\"",
            "\"dependency_ids\"",
            "\"diagnostics\"",
            "\"importer_version\"",
            "_U32_MAX = 2**32 - 1",
            "_U64_MAX = 2**64 - 1",
            "_RESOURCE_LOCATOR_SCHEMES = frozenset",
            "_RESOURCE_REGISTRY_BACKED_LOCATOR_SCHEMES = frozenset",
            "def _validate_resource_record_shape(",
            "_is_unsigned_int_within(revision, _U64_MAX)",
            "_is_unsigned_int_within(importer_version, _U32_MAX)",
            "def _validate_registry_export_matches_report_sources(",
            "def _report_resource_source_labels(",
            "def _append_report_resource_source_label(",
            "def _is_resource_id_string(",
            "UUID(value)",
            "def _is_resource_locator_string(",
            "scheme, separator, remainder = value.partition(\"://\")",
            "scheme not in _RESOURCE_LOCATOR_SCHEMES",
            "def _is_registry_backed_resource_locator_string(",
            "scheme in _RESOURCE_REGISTRY_BACKED_LOCATOR_SCHEMES",
            "def _split_resource_locator_label(",
            "def _is_package_resource_locator_path(",
            "def _is_resource_locator_relative_path(",
            "def _is_plain_resource_locator_segment(",
            "def _contains_resource_locator_drive_prefix(",
            "def _is_resource_diagnostic_record(",
            "def _is_unsigned_int_within(",
            "0 <= value <= max_value",
            "def _resource_record_kind_is_known(",
            "return isinstance(kind, str) and kind in _RESOURCE_RECORD_KINDS",
            "def _resource_record_state_is_known(",
            "return isinstance(state, str) and state in _RESOURCE_RECORD_STATES",
            "written_variants = report.get(\"written_variants\")",
            "def _resource_record_locators(",
            "def _usable_shader_resource_record_locators(",
            "def _is_usable_shader_record(",
            "def _resource_record_kind_is_shader(",
            "record.get(\"state\") == \"Ready\"",
            "revision > 0",
            "_is_registry_backed_resource_locator_string(source_label)",
            "\"primary_locator\"",
            "\"artifact_locator\"",
            "incomplete ResourceRecord entry at index",
            "missing ResourceRecord locators for report sources",
            "missing usable shader ResourceRecord revisions for report sources",
        ],
    );
    assert_contains_all(
        "staged acceptance passes the report to the registry validator",
        &acceptance_helper,
        &[
            "validate_shader_resource_registry_export_contract(",
            "report_path=config.shader_prewarm_report_path",
            "or config.shader_prewarm_resource_registry_path",
        ],
    );
    assert_contains_all(
        "python regressions cover registry/report correlation",
        &registry_report_test_sources,
        &[
            "class ZirconBuildShaderPrewarmResourceRegistryContractTests",
            "test_validate_registry_export_contract_rejects_missing_report_source_locator",
            "test_validate_registry_export_contract_rejects_missing_written_variant_locator",
            "test_validate_registry_export_contract_accepts_report_source_locator",
            "test_validate_registry_export_contract_rejects_non_shader_report_source_record",
            "test_validate_registry_export_contract_rejects_zero_revision_report_source_record",
            "test_validate_registry_export_contract_rejects_non_ready_report_source_record",
            "test_validate_registry_export_contract_rejects_incomplete_resource_record",
            "test_validate_registry_export_contract_rejects_tagged_enum_resource_record",
            "test_validate_registry_export_contract_rejects_u64_revision_overflow",
            "test_validate_registry_export_contract_rejects_u32_importer_version_overflow",
            "test_validate_registry_export_contract_rejects_invalid_locator_wire_shape",
            "test_validate_registry_export_contract_accepts_locator_wire_shape_variants",
            "test_validate_registry_export_contract_rejects_invalid_artifact_locator",
            "test_validate_registry_export_contract_accepts_registry_backed_source_locators",
            "test_validate_registry_export_contract_rejects_missing_registry_backed_source_locator",
            "test_validate_registry_export_contract_ignores_builtin_report_sources",
            "def _write_report_written_variant(",
            "def _resource_record(",
            "kind: object = \"Shader\"",
            "state: object = \"Ready\"",
            "_resource_record(state=\"Error\")",
            "kind={\"type\": \"Shader\"}",
            "state={\"state\": \"Ready\"}",
            "revision=2**64",
            "record[\"importer_version\"] = 2**32",
            "file://shaders/example",
            "res:///absolute",
            "res://C:/outside",
            "res://shaders/example#",
            "package://package-only",
            "package://zircon//shader",
            "lib://render/shaders/example",
            "package://zircon/shaders/example",
            "lib://plugin/shaders/example",
            "package://demo/shaders/example",
            "mem://generated/shader",
            "\"kind\": \"Shader\"",
            "\"artifact_locator\": None",
            "\"revision\": 1",
            "\"state\": \"Ready\"",
            "\"dependency_ids\": []",
            "\"diagnostics\": []",
            "res://shaders/example",
            "builtin://shader/pbr.wgsl",
            "registry:{config.shader_prewarm_resource_registry_path}:",
        ],
    );
    assert!(
        !build_prewarm_tests.contains(
            "test_validate_registry_export_contract_rejects_missing_report_source_locator"
        ),
        "general build-helper owner no longer carries registry report contract tests"
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
            "tools/tests/test_zircon_build_shader_prewarm.py",
            build_prewarm_tests.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm_acceptance_contract.py",
            acceptance_tests.as_str(),
        ),
        (
            "tools/zircon_build_shader_resource_registry.py",
            resource_registry.as_str(),
        ),
        (
            "tools/tests/test_zircon_build_shader_prewarm_resource_registry_contract.py",
            registry_tests.as_str(),
        ),
        (
            "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/shader_prewarm_resource_registry_report_correlation.rs",
            include_str!("shader_prewarm_resource_registry_report_correlation.rs"),
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
                "Build-tool shader resource registry report correlation",
                STATUS,
                WRITTEN_VARIANT_STATUS,
                USABLE_RECORD_STATUS,
                READY_RECORD_STATUS,
                OWNER_SPLIT_STATUS,
                RECORD_SHAPE_STATUS,
                ENUM_SHAPE_STATUS,
                NUMERIC_WIDTH_STATUS,
                LOCATOR_SHAPE_STATUS,
                REGISTRY_BACKED_LOCATOR_STATUS,
                "test_validate_registry_export_contract_rejects_missing_report_source_locator",
                "test_validate_registry_export_contract_rejects_missing_written_variant_locator",
                "test_validate_registry_export_contract_rejects_non_shader_report_source_record",
                "test_validate_registry_export_contract_rejects_zero_revision_report_source_record",
                "test_validate_registry_export_contract_rejects_non_ready_report_source_record",
                "test_validate_registry_export_contract_rejects_incomplete_resource_record",
                "test_validate_registry_export_contract_rejects_tagged_enum_resource_record",
                "test_validate_registry_export_contract_rejects_u64_revision_overflow",
                "test_validate_registry_export_contract_rejects_u32_importer_version_overflow",
                "test_validate_registry_export_contract_rejects_invalid_locator_wire_shape",
                "test_validate_registry_export_contract_accepts_locator_wire_shape_variants",
                "test_validate_registry_export_contract_rejects_invalid_artifact_locator",
                "test_validate_registry_export_contract_accepts_registry_backed_source_locators",
                "test_validate_registry_export_contract_rejects_missing_registry_backed_source_locator",
                "runtime_15_shader_prewarm_resource_registry_report_correlation_is_wired",
            ],
        );
    }
}
