"""NativeDynamic package export schema diagnostics."""

from __future__ import annotations

from typing import Any

from .native_dynamic_contract import (
    NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS,
    NATIVE_DYNAMIC_PACKAGE_REPORT_FILE,
    native_dynamic_package_directory,
)
from .pipeline_report_schema_primitives import (
    validate_integer_schema_diagnostics,
    validate_object_schema_diagnostics,
    validate_string_schema_diagnostics,
)
from .pipeline_report_validate_identifier_schema import (
    validate_native_dynamic_package_id_schema_diagnostics,
    validate_non_empty_trimmed_string_schema_diagnostics,
)

NATIVE_DYNAMIC_PACKAGE_EXPORT_FIELDS = (
    "abi",
    "directory",
    "manifest",
    "package_id",
    "package_report",
    "path",
)
NATIVE_DYNAMIC_PACKAGE_EXPORT_STRING_FIELDS = (
    "directory",
    "manifest",
    "package_id",
    "package_report",
    "path",
)
NATIVE_DYNAMIC_PACKAGE_EXPORT_REQUIRED_FIELDS = (
    "abi",
    *NATIVE_DYNAMIC_PACKAGE_EXPORT_STRING_FIELDS,
)
NATIVE_DYNAMIC_ABI_FIELDS = (
    "abi_version",
    "behavior_contract",
    "bridge_method_table",
    "descriptor_contract",
    "descriptor_symbol",
    "editor_entry_source",
    "entry_report_contract",
    "host_function_table",
    "runtime_entry_source",
    "state_snapshot_contract",
)
NATIVE_DYNAMIC_ABI_INTEGER_FIELDS = (
    "abi_version",
)
NATIVE_DYNAMIC_ABI_STRING_FIELDS = (
    "behavior_contract",
    "bridge_method_table",
    "descriptor_contract",
    "descriptor_symbol",
    "editor_entry_source",
    "entry_report_contract",
    "host_function_table",
    "runtime_entry_source",
    "state_snapshot_contract",
)
NATIVE_DYNAMIC_ABI_REQUIRED_FIELDS = (
    *NATIVE_DYNAMIC_ABI_INTEGER_FIELDS,
    *NATIVE_DYNAMIC_ABI_STRING_FIELDS,
)
VALIDATE_NATIVE_DYNAMIC_PACKAGE_EXPORTS_LABEL = (
    "validate report plan_summary.native_dynamic_package_exports"
)


def validate_native_dynamic_package_exports_schema_diagnostics(
    package_exports: Any,
) -> list[str]:
    if not isinstance(package_exports, list):
        return [f"{VALIDATE_NATIVE_DYNAMIC_PACKAGE_EXPORTS_LABEL} must be a list"]

    diagnostics: list[str] = []
    for index, package_export in enumerate(package_exports):
        if not isinstance(package_export, dict):
            diagnostics.append(
                f"{VALIDATE_NATIVE_DYNAMIC_PACKAGE_EXPORTS_LABEL}[{index}] "
                "must be an object"
            )
    diagnostics.extend(
        native_dynamic_package_export_schema_diagnostics(
            VALIDATE_NATIVE_DYNAMIC_PACKAGE_EXPORTS_LABEL,
            package_exports,
        )
    )
    return diagnostics


def native_dynamic_package_export_schema_diagnostics(
    label: str,
    package_exports: list[Any],
) -> list[str]:
    diagnostics: list[str] = []
    known_package_export_fields = set(NATIVE_DYNAMIC_PACKAGE_EXPORT_FIELDS)
    known_abi_fields = set(NATIVE_DYNAMIC_ABI_FIELDS)
    for index, package_export in enumerate(package_exports):
        if not isinstance(package_export, dict):
            continue
        diagnostics.extend(
            f"{label}[{index}] unknown field {field}"
            for field in sorted(package_export)
            if field not in known_package_export_fields
        )
        diagnostics.extend(
            native_dynamic_package_export_required_field_diagnostics(
                label,
                index,
                package_export,
            )
        )
        diagnostics.extend(
            native_dynamic_package_export_field_schema_diagnostics(
                label,
                index,
                package_export,
            )
        )
        abi = package_export.get("abi")
        if "abi" in package_export:
            diagnostics.extend(
                validate_object_schema_diagnostics(
                    f"{label}[{index}].abi",
                    abi,
                )
            )
        if not isinstance(abi, dict):
            continue
        diagnostics.extend(
            f"{label}[{index}].abi unknown field {field}"
            for field in sorted(abi)
            if field not in known_abi_fields
        )
        diagnostics.extend(
            native_dynamic_abi_schema_diagnostics(
                f"{label}[{index}].abi",
                abi,
            )
        )
    return diagnostics


def native_dynamic_package_export_required_field_diagnostics(
    label: str,
    index: int,
    package_export: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    for field in NATIVE_DYNAMIC_PACKAGE_EXPORT_REQUIRED_FIELDS:
        if field in package_export:
            continue
        field_label = f"{label}[{index}].{field}"
        if field == "abi":
            diagnostics.extend(
                validate_object_schema_diagnostics(
                    field_label,
                    package_export.get(field),
                )
            )
        else:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    field_label,
                    package_export.get(field),
                )
            )
    return diagnostics


def native_dynamic_package_export_field_schema_diagnostics(
    label: str,
    index: int,
    package_export: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    for field in NATIVE_DYNAMIC_PACKAGE_EXPORT_STRING_FIELDS:
        if field in package_export:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"{label}[{index}].{field}",
                    package_export.get(field),
                )
            )
    package_id = package_export.get("package_id")
    if isinstance(package_id, str):
        diagnostics.extend(
            validate_native_dynamic_package_id_schema_diagnostics(
                f"{label}[{index}].package_id",
                package_id,
            )
        )
    for field in ("directory", "path", "manifest", "package_report"):
        value = package_export.get(field)
        if isinstance(value, str):
            diagnostics.extend(
                validate_non_empty_trimmed_string_schema_diagnostics(
                    f"{label}[{index}].{field}",
                    value,
                )
            )
    diagnostics.extend(
        native_dynamic_package_export_path_schema_diagnostics(
            label,
            index,
            package_export,
        )
    )
    return diagnostics


def native_dynamic_package_export_path_schema_diagnostics(
    label: str,
    index: int,
    package_export: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    package_id = package_export.get("package_id")
    directory = package_export.get("directory")
    path = package_export.get("path")
    manifest = package_export.get("manifest")
    package_report = package_export.get("package_report")
    if (
        isinstance(package_id, str)
        and package_id.strip()
        and package_id.strip() == package_id
        and isinstance(directory, str)
        and directory
    ):
        expected_directory = native_dynamic_package_directory(package_id)
        if directory != expected_directory:
            diagnostics.append(
                f"{label}[{index}].directory must be {expected_directory} "
                f"for package_id {package_id}"
            )
    if not isinstance(directory, str) or not directory.strip():
        return diagnostics
    expected_path = f"plugins/{directory}"
    expected_manifest = f"{expected_path}/plugin.toml"
    expected_package_report = f"{expected_path}/{NATIVE_DYNAMIC_PACKAGE_REPORT_FILE}"
    if isinstance(path, str) and path and path != expected_path:
        diagnostics.append(
            f"{label}[{index}].path must be {expected_path} "
            f"for directory {directory}"
        )
    if isinstance(manifest, str) and manifest and manifest != expected_manifest:
        diagnostics.append(
            f"{label}[{index}].manifest must be {expected_manifest} "
            f"for directory {directory}"
        )
    if (
        isinstance(package_report, str)
        and package_report
        and package_report != expected_package_report
    ):
        diagnostics.append(
            f"{label}[{index}].package_report must be {expected_package_report} "
            f"for directory {directory}"
        )
    return diagnostics


def native_dynamic_abi_schema_diagnostics(
    label: str,
    abi: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    diagnostics.extend(
        native_dynamic_abi_required_field_diagnostics(
            label,
            abi,
        )
    )
    for field in NATIVE_DYNAMIC_ABI_INTEGER_FIELDS:
        if field in abi:
            diagnostics.extend(
                validate_integer_schema_diagnostics(
                    f"{label}.{field}",
                    abi.get(field),
                )
            )
            if abi.get(field) != 3:
                diagnostics.append(f"{label}.{field} must be 3")
    for field in NATIVE_DYNAMIC_ABI_STRING_FIELDS:
        if field in abi:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"{label}.{field}",
                    abi.get(field),
                )
            )
            value = abi.get(field)
            if isinstance(value, str):
                diagnostics.extend(
                    validate_non_empty_trimmed_string_schema_diagnostics(
                        f"{label}.{field}",
                        value,
                    )
                )
                expected_value = NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS[field]
                if value != expected_value:
                    diagnostics.append(
                        f"{label}.{field} must be {expected_value}"
    )
    return diagnostics


def native_dynamic_abi_required_field_diagnostics(
    label: str,
    abi: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    for field in NATIVE_DYNAMIC_ABI_REQUIRED_FIELDS:
        if field in abi:
            continue
        field_label = f"{label}.{field}"
        if field in NATIVE_DYNAMIC_ABI_INTEGER_FIELDS:
            diagnostics.extend(
                validate_integer_schema_diagnostics(
                    field_label,
                    abi.get(field),
                )
            )
        else:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    field_label,
                    abi.get(field),
                )
            )
    return diagnostics
