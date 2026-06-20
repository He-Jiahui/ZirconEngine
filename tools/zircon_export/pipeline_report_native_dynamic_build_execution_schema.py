"""NativeDynamic build-execution schema diagnostics."""

from __future__ import annotations

from typing import Any, Callable

from .export_template import is_safe_relative_path, normalize_relative_path
from .pipeline_report_native_dynamic_build_audit_common import (
    native_dynamic_build_audit_package_count_diagnostics,
    native_dynamic_build_audit_package_id_uniqueness_diagnostics,
    native_dynamic_fatal_report_diagnostics,
    native_dynamic_non_fatal_report_diagnostics,
    string_array_unique_entries_schema_diagnostics,
    table_non_negative_integer_schema_diagnostics,
)
from .pipeline_report_schema_table import (
    non_empty_string_array_schema_diagnostics,
    string_array_no_blank_entries_schema_diagnostics,
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
NATIVE_DYNAMIC_BUILD_EXECUTION_NON_EMPTY_STRING_FIELDS = ("skip_reason",)
NATIVE_DYNAMIC_BUILD_EXECUTION_INTEGER_FIELDS = ("package_count",)
NATIVE_DYNAMIC_BUILD_EXECUTION_NON_NEGATIVE_INTEGER_FIELDS = ("package_count",)
NATIVE_DYNAMIC_BUILD_EXECUTION_STRING_ARRAY_FIELDS = ("diagnostics",)
NATIVE_DYNAMIC_BUILD_EXECUTION_OBJECT_ARRAY_FIELDS = ("packages",)
NATIVE_DYNAMIC_BUILD_EXECUTION_REQUIRED_NON_FATAL_BOOL_FIELDS = (
    "enabled",
    "fatal",
    "skipped",
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
NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_NON_EMPTY_STRING_FIELDS = (
    "copied_loadable_artifact",
    "crate_name",
    "expected_loadable_artifact",
    "package_id",
)
NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_SAFE_RELATIVE_STRING_FIELDS = (
    "copied_loadable_artifact",
)
NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_INTEGER_FIELDS = ("exit_code",)
NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_NON_NEGATIVE_INTEGER_FIELDS = (
    "exit_code",
)
NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_STRING_ARRAY_FIELDS = (
    "command",
    "copied_sidecars",
)

SchemaDiagnostic = Callable[[str, Any], list[str]]

def native_dynamic_build_execution_report_fatal_diagnostics(
    label: str,
    execution: dict[str, Any],
    report_fatal: Any,
) -> list[str]:
    if report_fatal is not False:
        return []

    diagnostics: list[str] = []
    if execution.get("fatal") is True:
        diagnostics.append(
            f"{label}.fatal must be False when NativeDynamic report fatal is False"
        )
    if execution.get("skipped") is True:
        diagnostics.append(
            f"{label}.skipped must be False when NativeDynamic report fatal is False"
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
            if (
                field in NATIVE_DYNAMIC_BUILD_EXECUTION_NON_EMPTY_STRING_FIELDS
                and isinstance(execution.get(field), str)
                and not execution.get(field).strip()
            ):
                diagnostics.append(f"{label}.{field} must be a non-empty string")
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
    diagnostics.extend(
        table_non_negative_integer_schema_diagnostics(
            label,
            execution,
            NATIVE_DYNAMIC_BUILD_EXECUTION_NON_NEGATIVE_INTEGER_FIELDS,
        )
    )
    diagnostics.extend(
        native_dynamic_build_execution_enabled_table_diagnostics(
            label,
            execution,
        )
    )
    diagnostics.extend(
        native_dynamic_build_execution_skip_state_diagnostics(
            label,
            execution,
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
            if field == "diagnostics":
                diagnostics.extend(
                    string_array_no_blank_entries_schema_diagnostics(
                        f"{label}.{field}",
                        execution.get(field),
                    )
                )
    diagnostics.extend(native_dynamic_fatal_report_diagnostics(label, execution))
    diagnostics.extend(native_dynamic_non_fatal_report_diagnostics(label, execution))
    packages = execution.get("packages")
    if "packages" in execution and packages is not None:
        diagnostics.extend(
            validate_object_array_schema_diagnostics(
                f"{label}.packages",
                packages,
            )
        )
    diagnostics.extend(
        native_dynamic_build_audit_package_count_diagnostics(label, execution)
    )
    if isinstance(packages, list):
        diagnostics.extend(
            native_dynamic_build_audit_package_id_uniqueness_diagnostics(
                f"{label}.packages",
                packages,
            )
        )
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
                require_exit_success=execution.get("fatal") is False,
            )
        )
    return diagnostics


def native_dynamic_build_execution_skip_state_diagnostics(
    label: str,
    execution: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    skipped = execution.get("skipped")
    skip_reason = execution.get("skip_reason")
    has_skip_reason = "skip_reason" in execution and skip_reason is not None
    if skipped is False and has_skip_reason:
        return [f"{label}.skip_reason must be absent when skipped is False"]
    if skipped is True and (
        not isinstance(skip_reason, str) or not skip_reason.strip()
    ):
        diagnostics.append(
            f"{label}.skip_reason must be a non-empty string when skipped is True"
        )
    if skipped is True:
        if execution.get("enabled") is False:
            diagnostics.append(f"{label}.enabled must be True when skipped is True")
        if execution.get("fatal") is True:
            diagnostics.append(f"{label}.fatal must be False when skipped is True")
        package_count = execution.get("package_count")
        if type(package_count) is int and package_count != 0:
            diagnostics.append(
                f"{label}.package_count must be 0 when skipped is True"
            )
        packages = execution.get("packages")
        if isinstance(packages, list) and packages != []:
            diagnostics.append(f"{label}.packages must be empty when skipped is True")
    return diagnostics


def native_dynamic_build_execution_enabled_table_diagnostics(
    label: str,
    execution: dict[str, Any],
) -> list[str]:
    if execution.get("enabled") is not False:
        return []

    diagnostics: list[str] = []
    package_count = execution.get("package_count")
    if type(package_count) is int and package_count != 0:
        diagnostics.append(
            f"{label}.package_count must be 0 when enabled is False"
        )
    packages = execution.get("packages")
    if isinstance(packages, list) and packages != []:
        diagnostics.append(f"{label}.packages must be empty when enabled is False")
    return diagnostics


def native_dynamic_build_execution_packages_schema_diagnostics(
    label: str,
    packages: list[Any],
    *,
    validate_integer_schema_diagnostics: SchemaDiagnostic,
    validate_string_schema_diagnostics: SchemaDiagnostic,
    validate_string_array_schema_diagnostics: SchemaDiagnostic,
    require_exit_success: bool = False,
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
            if field not in package or package.get(field) is None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
            elif (
                field in NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_NON_EMPTY_STRING_FIELDS
                and isinstance(package.get(field), str)
                and not package.get(field).strip()
            ):
                diagnostics.append(
                    f"{package_label}.{field} must be a non-empty string"
                )
            if field in package and package.get(field) is not None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
                if field in NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_SAFE_RELATIVE_STRING_FIELDS:
                    diagnostics.extend(
                        native_dynamic_build_execution_safe_relative_path_diagnostics(
                            f"{package_label}.{field}",
                            package.get(field),
                        )
                    )
                    diagnostics.extend(
                        native_dynamic_build_execution_package_path_scope_diagnostics(
                            f"{package_label}.{field}",
                            package,
                            package.get(field),
                        )
                    )
        for field in NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_INTEGER_FIELDS:
            if field not in package or package.get(field) is None:
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
            if field in package and package.get(field) is not None:
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
                if (
                    field in NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_NON_NEGATIVE_INTEGER_FIELDS
                    and type(package.get(field)) is int
                    and package.get(field) < 0
                ):
                    diagnostics.append(
                        f"{package_label}.{field} must be non-negative"
                    )
                if (
                    require_exit_success
                    and field == "exit_code"
                    and type(package.get(field)) is int
                    and package.get(field) != 0
                ):
                    diagnostics.append(
                        f"{package_label}.{field} must be 0 for "
                        "non-fatal build execution"
                    )
        for field in NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_STRING_ARRAY_FIELDS:
            if field not in package or package.get(field) is None:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
            if field in package and package.get(field) is not None:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
                if field == "command":
                    diagnostics.extend(
                        non_empty_string_array_schema_diagnostics(
                            f"{package_label}.{field}",
                            package.get(field),
                        )
                    )
                if field == "copied_sidecars":
                    diagnostics.extend(
                        string_array_no_blank_entries_schema_diagnostics(
                            f"{package_label}.{field}",
                            package.get(field),
                        )
                    )
                    diagnostics.extend(
                        string_array_unique_entries_schema_diagnostics(
                            f"{package_label}.{field}",
                            package.get(field),
                        )
                    )
                    diagnostics.extend(
                        native_dynamic_build_execution_safe_relative_path_array_diagnostics(
                            f"{package_label}.{field}",
                            package.get(field),
                        )
                    )
                    diagnostics.extend(
                        native_dynamic_build_execution_package_path_scope_array_diagnostics(
                            f"{package_label}.{field}",
                            package,
                            package.get(field),
                        )
                    )
    return diagnostics


def native_dynamic_build_execution_package_path_scope_array_diagnostics(
    label: str,
    package: dict[str, Any],
    values: Any,
) -> list[str]:
    if not isinstance(values, list):
        return []
    diagnostics: list[str] = []
    for index, value in enumerate(values):
        diagnostics.extend(
            native_dynamic_build_execution_package_path_scope_diagnostics(
                f"{label}[{index}]",
                package,
                value,
            )
        )
    return diagnostics


def native_dynamic_build_execution_package_path_scope_diagnostics(
    label: str,
    package: dict[str, Any],
    value: Any,
) -> list[str]:
    package_prefix = native_dynamic_build_execution_package_path_prefix(package)
    normalized = native_dynamic_build_execution_normalized_safe_relative_path(value)
    if package_prefix is None or normalized is None:
        return []
    if not normalized.startswith(package_prefix):
        return [f"{label} must be inside {package_prefix}"]
    return []


def native_dynamic_build_execution_package_path_prefix(
    package: dict[str, Any],
) -> str | None:
    package_id = package.get("package_id")
    if not isinstance(package_id, str) or not package_id.strip():
        return None
    return f"plugins/{package_id.strip()}/"


def native_dynamic_build_execution_safe_relative_path_array_diagnostics(
    label: str,
    values: Any,
) -> list[str]:
    if not isinstance(values, list):
        return []
    diagnostics: list[str] = []
    for index, value in enumerate(values):
        diagnostics.extend(
            native_dynamic_build_execution_safe_relative_path_diagnostics(
                f"{label}[{index}]",
                value,
            )
        )
    return diagnostics


def native_dynamic_build_execution_safe_relative_path_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, str) or not value.strip():
        return []
    if native_dynamic_build_execution_normalized_safe_relative_path(value) is None:
        return [f"{label} must be a safe relative path"]
    return []


def native_dynamic_build_execution_normalized_safe_relative_path(
    value: Any,
) -> str | None:
    if not isinstance(value, str) or not value.strip():
        return None
    normalized = normalize_relative_path(value)
    if (
        native_dynamic_build_execution_has_drive_prefix(normalized)
        or not is_safe_relative_path(normalized)
    ):
        return None
    return normalized


def native_dynamic_build_execution_has_drive_prefix(value: str) -> bool:
    first_segment = value.split("/", maxsplit=1)[0]
    return len(first_segment) == 2 and first_segment[1] == ":"
