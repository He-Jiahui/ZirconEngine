"""NativeDynamic build-plan schema helper diagnostics."""

from __future__ import annotations

from typing import Any, Callable

from .pipeline_report_native_dynamic_build_audit_common import (
    string_array_unique_entries_schema_diagnostics,
)
from .pipeline_report_native_dynamic_build_plan_commands import (
    native_dynamic_build_plan_package_command_semantics_diagnostics,
)
from .pipeline_report_schema_string_array import (
    non_empty_string_array_schema_diagnostics,
    string_array_no_blank_entries_schema_diagnostics,
    string_array_trimmed_non_empty_entries_schema_diagnostics,
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

SchemaDiagnostic = Callable[[str, Any], list[str]]


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
            if field not in package or package.get(field) is None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
            elif (
                isinstance(package.get(field), str)
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
                diagnostics.extend(
                    native_dynamic_build_plan_trimmed_non_empty_string_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_BOOL_FIELDS:
            if field not in package or package.get(field) is None:
                diagnostics.extend(
                    validate_bool_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
            if field in package and package.get(field) is not None:
                diagnostics.extend(
                    validate_bool_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_STRING_ARRAY_FIELDS:
            if field not in package or package.get(field) is None:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
            if field in package and package.get(field) is not None:
                field_label = f"{package_label}.{field}"
                if field == "features":
                    diagnostics.extend(
                        native_dynamic_build_plan_feature_array_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                elif field == "command":
                    diagnostics.extend(
                        native_dynamic_build_plan_command_array_schema_diagnostics(
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
                if field == "features":
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
            native_dynamic_build_plan_package_command_semantics_diagnostics(
                package_label,
                package,
            )
        )
    return diagnostics


def native_dynamic_build_plan_trimmed_non_empty_string_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if isinstance(value, str) and value.strip() and value.strip() != value:
        return [f"{label} must be a non-empty trimmed string"]
    return []


def native_dynamic_build_plan_feature_array_schema_diagnostics(
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


def native_dynamic_build_plan_command_array_schema_diagnostics(
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


def native_dynamic_build_plan_diagnostics_array_schema_diagnostics(
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


def native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
    value: Any,
) -> bool:
    return (
        isinstance(value, str)
        and bool(value.strip())
        and value.strip() == value
    )


def native_dynamic_build_plan_string_array_is_trimmed_non_empty(
    value: Any,
) -> bool:
    return (
        isinstance(value, list)
        and bool(value)
        and all(
            isinstance(entry, str) and entry.strip() and entry.strip() == entry
            for entry in value
        )
    )


def native_dynamic_build_plan_string_array_is_schema_clean(
    value: Any,
    *,
    allow_empty: bool,
    require_unique: bool,
) -> bool:
    if not (
        isinstance(value, list)
        and (allow_empty or bool(value))
        and all(
            native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
                entry
            )
            for entry in value
        )
    ):
        return False
    return not require_unique or len(set(value)) == len(value)
