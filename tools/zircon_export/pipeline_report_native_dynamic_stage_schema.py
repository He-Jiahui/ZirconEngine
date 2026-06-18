"""NativeDynamic stage report schema diagnostics."""

from __future__ import annotations

from typing import Any, Callable

from .pipeline_report_native_dynamic_payload_schema import (
    NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS,
    native_dynamic_file_manifest_schema_diagnostics,
    native_dynamic_materialized_packages_schema_diagnostics,
    native_dynamic_operation_audit_stage_schema_diagnostics,
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
NATIVE_DYNAMIC_BUILD_PLAN_FIELDS = (
    "build_features",
    "cargo_profile",
    "diagnostics",
    "fatal",
    "package_count",
    "packages",
    "release",
    "target_dir",
    "workspace_manifest",
)
NATIVE_DYNAMIC_BUILD_PLAN_STRING_FIELDS = (
    "cargo_profile",
    "target_dir",
    "workspace_manifest",
)
NATIVE_DYNAMIC_BUILD_PLAN_BOOL_FIELDS = ("fatal", "release")
NATIVE_DYNAMIC_BUILD_PLAN_INTEGER_FIELDS = ("package_count",)
NATIVE_DYNAMIC_BUILD_PLAN_STRING_ARRAY_FIELDS = (
    "build_features",
    "diagnostics",
)
NATIVE_DYNAMIC_BUILD_PLAN_OBJECT_ARRAY_FIELDS = ("packages",)
NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_STRING_FIELDS = (
    "cargo_profile",
    "target_dir",
    "workspace_manifest",
)
NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_BOOL_FIELDS = (
    "fatal",
    "release",
)
NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_INTEGER_FIELDS = (
    "package_count",
)
NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS = (
    "build_features",
    "diagnostics",
)
NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS = (
    "packages",
)
NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_FIELDS = (
    "cargo_profile",
    "command",
    "crate_name",
    "expected_loadable_artifact",
    "features",
    "manifest_path",
    "package_id",
    "release",
    "target_dir",
    "workspace_manifest",
)
NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_STRING_FIELDS = (
    "cargo_profile",
    "crate_name",
    "expected_loadable_artifact",
    "manifest_path",
    "package_id",
    "target_dir",
    "workspace_manifest",
)
NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_BOOL_FIELDS = ("release",)
NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_STRING_ARRAY_FIELDS = (
    "command",
    "features",
)
NATIVE_DYNAMIC_BUILD_EXECUTION_FIELDS = (
    "diagnostics",
    "enabled",
    "fatal",
    "package_count",
    "packages",
    "skip_reason",
    "skipped",
)
NATIVE_DYNAMIC_BUILD_EXECUTION_BOOL_FIELDS = ("enabled", "fatal", "skipped")
NATIVE_DYNAMIC_BUILD_EXECUTION_STRING_FIELDS = ("skip_reason",)
NATIVE_DYNAMIC_BUILD_EXECUTION_INTEGER_FIELDS = ("package_count",)
NATIVE_DYNAMIC_BUILD_EXECUTION_STRING_ARRAY_FIELDS = ("diagnostics",)
NATIVE_DYNAMIC_BUILD_EXECUTION_OBJECT_ARRAY_FIELDS = ("packages",)
NATIVE_DYNAMIC_BUILD_EXECUTION_REQUIRED_NON_FATAL_BOOL_FIELDS = (
    "enabled",
    "fatal",
)
NATIVE_DYNAMIC_BUILD_EXECUTION_REQUIRED_NON_FATAL_INTEGER_FIELDS = (
    "package_count",
)
NATIVE_DYNAMIC_BUILD_EXECUTION_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS = (
    "diagnostics",
)
NATIVE_DYNAMIC_BUILD_EXECUTION_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS = (
    "packages",
)
NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_FIELDS = (
    "command",
    "copied_loadable_artifact",
    "copied_sidecars",
    "crate_name",
    "exit_code",
    "expected_loadable_artifact",
    "package_id",
    "stderr",
    "stdout",
)
NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_STRING_FIELDS = (
    "copied_loadable_artifact",
    "crate_name",
    "expected_loadable_artifact",
    "package_id",
    "stderr",
    "stdout",
)
NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_INTEGER_FIELDS = ("exit_code",)
NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_STRING_ARRAY_FIELDS = (
    "command",
    "copied_sidecars",
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
    for field in NATIVE_DYNAMIC_REPORT_INTEGER_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_integer_schema_diagnostics(
                    f"native_dynamic report {field}",
                    report.get(field),
                )
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


def native_dynamic_build_plan_schema_diagnostics(
    label: str,
    build_plan: dict[str, Any],
    *,
    validate_bool_schema_diagnostics: SchemaDiagnostic,
    validate_integer_schema_diagnostics: SchemaDiagnostic,
    validate_string_schema_diagnostics: SchemaDiagnostic,
    validate_string_array_schema_diagnostics: SchemaDiagnostic,
    validate_object_array_schema_diagnostics: SchemaDiagnostic,
    require_release_evidence: bool = False,
) -> list[str]:
    diagnostics: list[str] = []
    diagnostics.extend(
        f"{label} unknown field {field}"
        for field in sorted(build_plan)
        if field not in NATIVE_DYNAMIC_BUILD_PLAN_FIELDS
    )
    if require_release_evidence:
        for field in NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_STRING_FIELDS:
            if field not in build_plan or build_plan.get(field) is None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"{label}.{field}",
                        build_plan.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_BOOL_FIELDS:
            if field not in build_plan or build_plan.get(field) is None:
                diagnostics.extend(
                    validate_bool_schema_diagnostics(
                        f"{label}.{field}",
                        build_plan.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_INTEGER_FIELDS:
            if field not in build_plan or build_plan.get(field) is None:
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        f"{label}.{field}",
                        build_plan.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS:
            if field not in build_plan or build_plan.get(field) is None:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"{label}.{field}",
                        build_plan.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS:
            if field not in build_plan or build_plan.get(field) is None:
                diagnostics.extend(
                    validate_object_array_schema_diagnostics(
                        f"{label}.{field}",
                        build_plan.get(field),
                    )
                )
    for field in NATIVE_DYNAMIC_BUILD_PLAN_STRING_FIELDS:
        if field in build_plan and build_plan.get(field) is not None:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"{label}.{field}",
                    build_plan.get(field),
                )
            )
    for field in NATIVE_DYNAMIC_BUILD_PLAN_BOOL_FIELDS:
        if field in build_plan and build_plan.get(field) is not None:
            diagnostics.extend(
                validate_bool_schema_diagnostics(
                    f"{label}.{field}",
                    build_plan.get(field),
                )
            )
    for field in NATIVE_DYNAMIC_BUILD_PLAN_INTEGER_FIELDS:
        if field in build_plan and build_plan.get(field) is not None:
            diagnostics.extend(
                validate_integer_schema_diagnostics(
                    f"{label}.{field}",
                    build_plan.get(field),
                )
            )
    for field in NATIVE_DYNAMIC_BUILD_PLAN_STRING_ARRAY_FIELDS:
        if field in build_plan and build_plan.get(field) is not None:
            diagnostics.extend(
                validate_string_array_schema_diagnostics(
                    f"{label}.{field}",
                    build_plan.get(field),
                )
            )
    packages = build_plan.get("packages")
    if "packages" in build_plan and packages is not None:
        diagnostics.extend(
            validate_object_array_schema_diagnostics(
                f"{label}.packages",
                packages,
            )
        )
    if isinstance(packages, list):
        diagnostics.extend(
            native_dynamic_build_plan_packages_schema_diagnostics(
                f"{label}.packages",
                packages,
                validate_bool_schema_diagnostics=validate_bool_schema_diagnostics,
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_string_array_schema_diagnostics=(
                    validate_string_array_schema_diagnostics
                ),
            )
        )
    return diagnostics


def native_dynamic_build_plan_packages_schema_diagnostics(
    label: str,
    packages: list[Any],
    *,
    validate_bool_schema_diagnostics: SchemaDiagnostic,
    validate_string_schema_diagnostics: SchemaDiagnostic,
    validate_string_array_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    diagnostics: list[str] = []
    known_fields = set(NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_FIELDS)
    for index, package in enumerate(packages):
        if not isinstance(package, dict):
            continue
        package_label = f"{label}[{index}]"
        diagnostics.extend(
            f"{package_label} unknown field {field}"
            for field in sorted(package)
            if field not in known_fields
        )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_STRING_FIELDS:
            if field in package and package.get(field) is not None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_BOOL_FIELDS:
            if field in package and package.get(field) is not None:
                diagnostics.extend(
                    validate_bool_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_STRING_ARRAY_FIELDS:
            if field in package and package.get(field) is not None:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
    return diagnostics


def native_dynamic_build_execution_schema_diagnostics(
    label: str,
    execution: dict[str, Any],
    *,
    validate_bool_schema_diagnostics: SchemaDiagnostic,
    validate_integer_schema_diagnostics: SchemaDiagnostic,
    validate_string_schema_diagnostics: SchemaDiagnostic,
    validate_string_array_schema_diagnostics: SchemaDiagnostic,
    validate_object_array_schema_diagnostics: SchemaDiagnostic,
    require_release_evidence: bool = False,
) -> list[str]:
    diagnostics: list[str] = []
    diagnostics.extend(
        f"{label} unknown field {field}"
        for field in sorted(execution)
        if field not in NATIVE_DYNAMIC_BUILD_EXECUTION_FIELDS
    )
    if require_release_evidence:
        for field in NATIVE_DYNAMIC_BUILD_EXECUTION_REQUIRED_NON_FATAL_BOOL_FIELDS:
            if field not in execution or execution.get(field) is None:
                diagnostics.extend(
                    validate_bool_schema_diagnostics(
                        f"{label}.{field}",
                        execution.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_EXECUTION_REQUIRED_NON_FATAL_INTEGER_FIELDS:
            if field not in execution or execution.get(field) is None:
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        f"{label}.{field}",
                        execution.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_EXECUTION_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS:
            if field not in execution or execution.get(field) is None:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"{label}.{field}",
                        execution.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_EXECUTION_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS:
            if field not in execution or execution.get(field) is None:
                diagnostics.extend(
                    validate_object_array_schema_diagnostics(
                        f"{label}.{field}",
                        execution.get(field),
                    )
                )
    for field in NATIVE_DYNAMIC_BUILD_EXECUTION_BOOL_FIELDS:
        if field in execution and execution.get(field) is not None:
            diagnostics.extend(
                validate_bool_schema_diagnostics(
                    f"{label}.{field}",
                    execution.get(field),
                )
            )
    for field in NATIVE_DYNAMIC_BUILD_EXECUTION_STRING_FIELDS:
        if field in execution and execution.get(field) is not None:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"{label}.{field}",
                    execution.get(field),
                )
            )
    for field in NATIVE_DYNAMIC_BUILD_EXECUTION_INTEGER_FIELDS:
        if field in execution and execution.get(field) is not None:
            diagnostics.extend(
                validate_integer_schema_diagnostics(
                    f"{label}.{field}",
                    execution.get(field),
                )
            )
    for field in NATIVE_DYNAMIC_BUILD_EXECUTION_STRING_ARRAY_FIELDS:
        if field in execution and execution.get(field) is not None:
            diagnostics.extend(
                validate_string_array_schema_diagnostics(
                    f"{label}.{field}",
                    execution.get(field),
                )
            )
    packages = execution.get("packages")
    if "packages" in execution and packages is not None:
        diagnostics.extend(
            validate_object_array_schema_diagnostics(
                f"{label}.packages",
                packages,
            )
        )
    if isinstance(packages, list):
        diagnostics.extend(
            native_dynamic_build_execution_packages_schema_diagnostics(
                f"{label}.packages",
                packages,
                validate_integer_schema_diagnostics=(
                    validate_integer_schema_diagnostics
                ),
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_string_array_schema_diagnostics=(
                    validate_string_array_schema_diagnostics
                ),
            )
        )
    return diagnostics


def native_dynamic_build_execution_packages_schema_diagnostics(
    label: str,
    packages: list[Any],
    *,
    validate_integer_schema_diagnostics: SchemaDiagnostic,
    validate_string_schema_diagnostics: SchemaDiagnostic,
    validate_string_array_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    diagnostics: list[str] = []
    known_fields = set(NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_FIELDS)
    for index, package in enumerate(packages):
        if not isinstance(package, dict):
            continue
        package_label = f"{label}[{index}]"
        diagnostics.extend(
            f"{package_label} unknown field {field}"
            for field in sorted(package)
            if field not in known_fields
        )
        for field in NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_STRING_FIELDS:
            if field in package and package.get(field) is not None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_INTEGER_FIELDS:
            if field in package and package.get(field) is not None:
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_STRING_ARRAY_FIELDS:
            if field in package and package.get(field) is not None:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
    return diagnostics
