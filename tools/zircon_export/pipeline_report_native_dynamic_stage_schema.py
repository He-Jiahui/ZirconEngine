"""NativeDynamic stage report schema diagnostics."""

from __future__ import annotations

from typing import Any, Callable

from .pipeline_report_native_dynamic_build_audit_schema import (
    native_dynamic_build_execution_report_fatal_diagnostics,
    native_dynamic_build_execution_schema_diagnostics,
    native_dynamic_build_plan_execution_state_diagnostics,
    native_dynamic_build_plan_schema_diagnostics,
)
from .pipeline_report_native_dynamic_operation_audit_schema import (
    NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS,
    native_dynamic_operation_audit_stage_schema_diagnostics,
)
from .pipeline_report_native_dynamic_payload_schema import (
    native_dynamic_file_manifest_schema_diagnostics,
    native_dynamic_materialized_packages_schema_diagnostics,
)
from .pipeline_report_schema_table import (
    string_array_no_blank_entries_schema_diagnostics,
)

NATIVE_DYNAMIC_REPORT_FIELDS = (
    "artifact_extensions",
    "cleanup_reason",
    "content_hash",
    "diagnostics",
    "fatal",
    "file_manifest",
    "loader_manifest",
    "materialized_packages",
    "native_build_execution",
    "native_build_plan",
    "native_dynamic_packages",
    "native_notarization",
    "native_plugin_root",
    "native_signing",
    "package_count",
    "package_exports",
    "payload_cleaned",
    "plugins_dir",
    "profile",
    "stage",
    "stage_output",
    "target_platform",
    "validate_report",
)
NATIVE_DYNAMIC_REPORT_STRING_FIELDS = (
    "cleanup_reason",
    "content_hash",
    "loader_manifest",
    "native_plugin_root",
    "plugins_dir",
    "stage_output",
    "target_platform",
    "validate_report",
)
NATIVE_DYNAMIC_REPORT_STRING_ARRAY_FIELDS = (
    "artifact_extensions",
    "native_dynamic_packages",
)
NATIVE_DYNAMIC_REPORT_INTEGER_FIELDS = ("package_count",)
NATIVE_DYNAMIC_REPORT_NON_NEGATIVE_INTEGER_FIELDS = ("package_count",)
NATIVE_DYNAMIC_REPORT_BOOL_FIELDS = ("payload_cleaned",)
NATIVE_DYNAMIC_REPORT_OBJECT_FIELDS = (
    "native_build_execution",
    "native_build_plan",
    "native_notarization",
    "native_signing",
)
NATIVE_DYNAMIC_REPORT_OBJECT_ARRAY_FIELDS = ("package_exports",)
NATIVE_DYNAMIC_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS = (
    "content_hash",
    "loader_manifest",
    "native_plugin_root",
    "plugins_dir",
    "stage_output",
    "target_platform",
    "validate_report",
)
NATIVE_DYNAMIC_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS = (
    "artifact_extensions",
    "native_dynamic_packages",
)
NATIVE_DYNAMIC_REPORT_REQUIRED_NON_FATAL_INTEGER_FIELDS = (
    "package_count",
)
NATIVE_DYNAMIC_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS = (
    "native_build_execution",
    "native_build_plan",
    "native_notarization",
    "native_signing",
)
NATIVE_DYNAMIC_REPORT_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS = (
    "file_manifest",
    "materialized_packages",
    "package_exports",
)
SchemaDiagnostic = Callable[[str, Any], list[str]]


def native_dynamic_report_schema_diagnostics(
    report: dict[str, Any],
    *,
    validate_bool_schema_diagnostics: SchemaDiagnostic,
    validate_integer_schema_diagnostics: SchemaDiagnostic,
    validate_string_schema_diagnostics: SchemaDiagnostic,
    validate_string_array_schema_diagnostics: SchemaDiagnostic,
    validate_object_schema_diagnostics: SchemaDiagnostic,
    validate_object_array_schema_diagnostics: SchemaDiagnostic,
    native_dynamic_package_export_schema_diagnostics: Callable[
        [str, list[Any]], list[str]
    ],
) -> list[str]:
    diagnostics: list[str] = []
    for field in NATIVE_DYNAMIC_REPORT_STRING_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"native_dynamic report {field}",
                    report.get(field),
                )
            )
    for field in NATIVE_DYNAMIC_REPORT_STRING_ARRAY_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_string_array_schema_diagnostics(
                    f"native_dynamic report {field}",
                    report.get(field),
                )
            )
            diagnostics.extend(
                string_array_no_blank_entries_schema_diagnostics(
                    f"native_dynamic report {field}",
                    report.get(field),
                )
            )
    for field in NATIVE_DYNAMIC_REPORT_INTEGER_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_integer_schema_diagnostics(
                    f"native_dynamic report {field}",
                    report.get(field),
                )
            )
    for field in NATIVE_DYNAMIC_REPORT_NON_NEGATIVE_INTEGER_FIELDS:
        value = report.get(field)
        if isinstance(value, int) and value < 0:
            diagnostics.append(
                f"native_dynamic report {field} must be non-negative"
            )
    for field in NATIVE_DYNAMIC_REPORT_BOOL_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_bool_schema_diagnostics(
                    f"native_dynamic report {field}",
                    report.get(field),
                )
            )
    for field in NATIVE_DYNAMIC_REPORT_OBJECT_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_object_schema_diagnostics(
                    f"native_dynamic report {field}",
                    report.get(field),
                )
            )
    if report.get("fatal") is False:
        for field in NATIVE_DYNAMIC_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS:
            if field not in report or report.get(field) is None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"native_dynamic report {field}",
                        report.get(field),
                    )
                )
            elif isinstance(report.get(field), str) and not report.get(field).strip():
                diagnostics.append(
                    f"native_dynamic report {field} must be a non-empty string"
                )
        for field in NATIVE_DYNAMIC_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS:
            if field not in report or report.get(field) is None:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"native_dynamic report {field}",
                        report.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_REPORT_REQUIRED_NON_FATAL_INTEGER_FIELDS:
            if field not in report or report.get(field) is None:
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        f"native_dynamic report {field}",
                        report.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS:
            if field not in report or report.get(field) is None:
                diagnostics.extend(
                    validate_object_schema_diagnostics(
                        f"native_dynamic report {field}",
                        report.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_REPORT_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_object_array_schema_diagnostics(
                        f"native_dynamic report {field}",
                        report.get(field),
                    )
                )
    diagnostics.extend(
        native_dynamic_file_manifest_schema_diagnostics(
            "native_dynamic report",
            report,
        )
    )
    diagnostics.extend(
        native_dynamic_materialized_packages_schema_diagnostics(
            "native_dynamic report",
            report,
        )
    )
    for field in NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS:
        audit = report.get(field)
        if isinstance(audit, dict):
            diagnostics.extend(
                native_dynamic_operation_audit_stage_schema_diagnostics(
                    f"native_dynamic report {field}",
                    audit,
                )
            )
    native_build_plan = report.get("native_build_plan")
    if isinstance(native_build_plan, dict):
        diagnostics.extend(
            native_dynamic_build_plan_schema_diagnostics(
                "native_dynamic report native_build_plan",
                native_build_plan,
                validate_bool_schema_diagnostics=validate_bool_schema_diagnostics,
                validate_integer_schema_diagnostics=(
                    validate_integer_schema_diagnostics
                ),
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_string_array_schema_diagnostics=(
                    validate_string_array_schema_diagnostics
                ),
                validate_object_array_schema_diagnostics=(
                    validate_object_array_schema_diagnostics
                ),
                require_release_evidence=report.get("fatal") is False,
                target_platform=report.get("target_platform"),
            )
        )
    native_build_execution = report.get("native_build_execution")
    if isinstance(native_build_execution, dict):
        diagnostics.extend(
            native_dynamic_build_execution_schema_diagnostics(
                "native_dynamic report native_build_execution",
                native_build_execution,
                validate_bool_schema_diagnostics=validate_bool_schema_diagnostics,
                validate_integer_schema_diagnostics=(
                    validate_integer_schema_diagnostics
                ),
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_string_array_schema_diagnostics=(
                    validate_string_array_schema_diagnostics
                ),
                validate_object_array_schema_diagnostics=(
                    validate_object_array_schema_diagnostics
                ),
                require_release_evidence=report.get("fatal") is False,
            )
        )
        diagnostics.extend(
            native_dynamic_build_execution_report_fatal_diagnostics(
                "native_dynamic report native_build_execution",
                native_build_execution,
                report.get("fatal"),
            )
        )
    diagnostics.extend(
        native_dynamic_build_plan_execution_state_diagnostics(
            native_build_plan,
            native_build_execution,
            report.get("fatal"),
        )
    )
    for field in NATIVE_DYNAMIC_REPORT_OBJECT_ARRAY_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_object_array_schema_diagnostics(
                    f"native_dynamic report {field}",
                    report.get(field),
                )
            )
    package_exports = report.get("package_exports")
    if isinstance(package_exports, list):
        diagnostics.extend(
            native_dynamic_package_export_schema_diagnostics(
                "native_dynamic report package_exports",
                package_exports,
            )
        )
    return diagnostics
