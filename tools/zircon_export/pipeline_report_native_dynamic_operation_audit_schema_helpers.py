"""Helper diagnostics for NativeDynamic operation-audit schema checks."""

from __future__ import annotations

from typing import Any

from .export_template_manifest import (
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
)
from .native_signing import native_dynamic_signing_platform_allowed


def table_required_non_empty_string_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = table.get(field)
        if isinstance(value, str) and not value.strip():
            diagnostics.append(f"{label}.{field} must be a non-empty string")
    return diagnostics


def table_required_trimmed_non_empty_string_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = table.get(field)
        if isinstance(value, str) and value.strip() and value.strip() != value:
            diagnostics.append(
                f"{label}.{field} must be a non-empty trimmed string"
            )
    return diagnostics


def operation_audit_platform_allowed_schema_diagnostics(
    label: str,
    audit: dict[str, Any],
) -> list[str]:
    if audit.get("enabled") is not True:
        return []
    target_platform = audit.get("target_platform")
    allowed_platforms = audit.get("allowed_platforms")
    platform_allowed = audit.get("platform_allowed")
    if (
        not isinstance(target_platform, str)
        or not target_platform.strip()
        or target_platform.strip() != target_platform
        or not isinstance(allowed_platforms, list)
        or not all(
            isinstance(platform, str)
            and platform.strip()
            and platform.strip() == platform
            for platform in allowed_platforms
        )
        or type(platform_allowed) is not bool
    ):
        return []
    computed_platform_allowed = native_dynamic_signing_platform_allowed(
        target_platform,
        allowed_platforms,
    )
    if platform_allowed == computed_platform_allowed:
        return []
    return [
        f"{label}.platform_allowed does not match target_platform "
        "and allowed_platforms"
    ]


def operation_audit_artifact_command_schema_diagnostics(
    label: str,
    artifact: dict[str, Any],
) -> list[str]:
    field = "command"
    value = artifact.get(field)
    if not isinstance(value, list):
        return [f"{label}.{field} must be a string array"]

    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, str):
            diagnostics.append(f"{label}.{field}[{index}] must be a string")
    return diagnostics


def object_array_required_non_empty_string_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    fields: tuple[str, ...],
) -> list[str]:
    value = table.get(field)
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_required_non_empty_string_schema_diagnostics(
                f"{label} {field}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics


def object_array_required_trimmed_non_empty_string_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    fields: tuple[str, ...],
) -> list[str]:
    value = table.get(field)
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_required_trimmed_non_empty_string_schema_diagnostics(
                f"{label} {field}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics


def table_sha256_hex_string_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = table.get(field)
        if (
            isinstance(value, str)
            and value.strip()
            and value.strip() == value
            and not is_sha256_hex(value)
        ):
            diagnostics.append(f"{label}.{field} must be a SHA-256 hex digest")
    return diagnostics


def object_array_sha256_hex_string_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    fields: tuple[str, ...],
) -> list[str]:
    value = table.get(field)
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_sha256_hex_string_schema_diagnostics(
                f"{label} {field}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics


def string_array_unique_entries_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
        return []
    seen: set[str] = set()
    for item in value:
        if not item.strip() or item.strip() != item:
            continue
        if item in seen:
            return [f"{label} must not contain duplicate entries"]
        seen.add(item)
    return []


def object_array_unique_string_field_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    value_field: str,
) -> list[str]:
    value = table.get(field)
    if not isinstance(value, list):
        return []

    entries: list[str] = []
    for entry in value:
        if not isinstance(entry, dict):
            return []
        field_value = entry.get(value_field)
        if not isinstance(field_value, str):
            return []
        if not field_value.strip() or field_value.strip() != field_value:
            continue
        entries.append(field_value)

    return string_array_unique_entries_schema_diagnostics(
        f"{label} {field}.{value_field}",
        entries,
    )


def artifact_safe_relative_path_schema_diagnostics(
    label: str,
    artifact: dict[str, Any],
) -> list[str]:
    value = artifact.get("package_relative_artifact")
    if not isinstance(value, str):
        return []
    if not value.strip():
        return []
    if is_safe_relative_path(normalize_relative_path(value)):
        return []
    return [f"{label}.package_relative_artifact must be a safe relative path"]


def artifact_exit_code_success_schema_diagnostics(
    label: str,
    artifact: dict[str, Any],
) -> list[str]:
    exit_code = artifact.get("exit_code")
    if type(exit_code) is not int or exit_code == 0:
        return []
    return [f"{label}.exit_code must be 0 for non-fatal operation audit"]


def table_non_negative_integer_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = table.get(field)
        if type(value) is int and value < 0:
            diagnostics.append(f"{label}.{field} must be non-negative")
    return diagnostics


def object_array_non_negative_integer_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    fields: tuple[str, ...],
) -> list[str]:
    value = table.get(field)
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_non_negative_integer_schema_diagnostics(
                f"{label} {field}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics
