"""Reusable field diagnostics for NativeDynamic package report schema."""

from __future__ import annotations

from typing import Any

from .export_template_manifest import (
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
)


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
        if (
            isinstance(value, str)
            and value.strip()
            and value.strip() != value
        ):
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


def table_safe_relative_path_string_schema_diagnostics(
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
            and not is_safe_relative_path(normalize_relative_path(value))
        ):
            diagnostics.append(f"{label}.{field} must be a safe relative path")
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
        diagnostics.extend(
            table_safe_relative_path_string_schema_diagnostics(
                f"{label} {field}[{index}]",
                entry,
                fields,
            )
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
        diagnostics.extend(
            table_non_negative_integer_schema_diagnostics(
                f"{label} {field}[{index}]",
                entry,
                fields,
            )
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
    value: list[str],
) -> list[str]:
    seen: set[str] = set()
    for entry in value:
        if entry in seen:
            return [f"{label} must not contain duplicate entries"]
        seen.add(entry)
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
