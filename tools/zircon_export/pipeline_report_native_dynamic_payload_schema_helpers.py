"""Helper diagnostics for NativeDynamic payload schema checks."""

from __future__ import annotations

from typing import Any

from .export_template_manifest import (
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
)


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
        for item_field in fields:
            item_value = entry.get(item_field)
            if isinstance(item_value, str) and not item_value.strip():
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field} "
                    "must be a non-empty string"
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
        for item_field in fields:
            item_value = entry.get(item_field)
            if (
                isinstance(item_value, str)
                and item_value.strip()
                and item_value.strip() != item_value
            ):
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field} "
                    "must be a non-empty trimmed string"
                )
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
        for item_field in fields:
            item_value = entry.get(item_field)
            if (
                isinstance(item_value, str)
                and item_value.strip()
                and item_value.strip() == item_value
                and not is_sha256_hex(item_value)
            ):
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field} "
                    "must be a SHA-256 hex digest"
                )
    return diagnostics


def object_array_safe_relative_path_string_schema_diagnostics(
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
        for item_field in fields:
            item_value = entry.get(item_field)
            if (
                isinstance(item_value, str)
                and item_value.strip()
                and not is_safe_relative_path(normalize_relative_path(item_value))
            ):
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field} "
                    "must be a safe relative path"
                )
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
        for item_field in fields:
            item_value = entry.get(item_field)
            if type(item_value) is int and item_value < 0:
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field} "
                    "must be non-negative"
                )
    return diagnostics


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


def table_non_empty_string_schema_diagnostics(
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


def table_trimmed_non_empty_string_schema_diagnostics(
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


def object_array_unique_string_field_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    item_field: str,
    *,
    normalize_path: bool = False,
) -> list[str]:
    value = table.get(field)
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    seen: set[str] = set()
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        item_value = entry.get(item_field)
        if (
            not isinstance(item_value, str)
            or not item_value.strip()
            or item_value.strip() != item_value
        ):
            continue
        normalized = normalize_relative_path(item_value) if normalize_path else item_value
        if normalize_path and not is_safe_relative_path(normalized):
            continue
        if normalized in seen:
            diagnostics.append(
                f"{label} {field}[{index}].{item_field} must be unique"
            )
            continue
        seen.add(normalized)
    return diagnostics


