"""NativeDynamic build-execution packages schema diagnostics."""

from __future__ import annotations

from typing import Any, Callable

from .export_template_manifest import is_safe_relative_path, normalize_relative_path
from .pipeline_report_native_dynamic_build_audit_common import (
    string_array_unique_entries_schema_diagnostics,
)
from .pipeline_report_schema_string_array import (
    non_empty_string_array_schema_diagnostics,
    string_array_no_blank_entries_schema_diagnostics,
    string_array_trimmed_non_empty_entries_schema_diagnostics,
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
            elif (
                field in NATIVE_DYNAMIC_BUILD_EXECUTION_PACKAGE_NON_EMPTY_STRING_FIELDS
                and isinstance(package.get(field), str)
                and package.get(field).strip()
                and package.get(field).strip() != package.get(field)
            ):
                diagnostics.append(
                    f"{package_label}.{field} must be a non-empty trimmed string"
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
                field_label = f"{package_label}.{field}"
                if field == "command":
                    diagnostics.extend(
                        native_dynamic_build_execution_command_array_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                elif field == "copied_sidecars":
                    diagnostics.extend(
                        native_dynamic_build_execution_copied_sidecars_array_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                else:
                    diagnostics.extend(
                        validate_string_array_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                if field == "command":
                    diagnostics.extend(
                        non_empty_string_array_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                    diagnostics.extend(
                        string_array_trimmed_non_empty_entries_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                if field == "copied_sidecars":
                    diagnostics.extend(
                        string_array_no_blank_entries_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                    diagnostics.extend(
                        string_array_trimmed_non_empty_entries_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                    diagnostics.extend(
                        string_array_unique_entries_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                    diagnostics.extend(
                        native_dynamic_build_execution_safe_relative_path_array_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                    diagnostics.extend(
                        native_dynamic_build_execution_package_path_scope_array_diagnostics(
                            field_label,
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


def native_dynamic_build_execution_command_array_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, list):
        return [f"{label} must be a string array"]
    return [
        f"{label}[{index}] must be a string"
        for index, entry in enumerate(value)
        if not isinstance(entry, str)
    ]


def native_dynamic_build_execution_copied_sidecars_array_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, list):
        return [f"{label} must be a string array"]
    return [
        f"{label}[{index}] must be a string"
        for index, entry in enumerate(value)
        if not isinstance(entry, str)
    ]


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
    if not native_dynamic_build_execution_trimmed_non_empty_string_is_schema_clean(
        package_id
    ):
        return None
    return f"plugins/{package_id}/"


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
    if not native_dynamic_build_execution_trimmed_non_empty_string_is_schema_clean(
        value
    ):
        return []
    if native_dynamic_build_execution_normalized_safe_relative_path(value) is None:
        return [f"{label} must be a safe relative path"]
    return []


def native_dynamic_build_execution_normalized_safe_relative_path(
    value: Any,
) -> str | None:
    if not native_dynamic_build_execution_trimmed_non_empty_string_is_schema_clean(
        value
    ):
        return None
    normalized = normalize_relative_path(value)
    if (
        native_dynamic_build_execution_has_drive_prefix(normalized)
        or not is_safe_relative_path(normalized)
    ):
        return None
    return normalized


def native_dynamic_build_execution_trimmed_non_empty_string_is_schema_clean(
    value: object,
) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value


def native_dynamic_build_execution_has_drive_prefix(value: str) -> bool:
    first_segment = value.split("/", maxsplit=1)[0]
    return len(first_segment) == 2 and first_segment[1] == ":"
