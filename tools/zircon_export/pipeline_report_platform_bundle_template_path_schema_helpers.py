"""Path/hash helpers for PlatformBundle template schema diagnostics."""

from __future__ import annotations

from typing import Any

from .export_template_manifest import (
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
)


def table_sha256_hex_string_diagnostics(
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


def sequence_sha256_hex_string_diagnostics(
    label: str,
    value: object,
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_sha256_hex_string_diagnostics(
                f"{label}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics


def table_safe_relative_path_string_diagnostics(
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
            and not is_safe_relative_path(normalize_relative_path(value))
        ):
            diagnostics.append(f"{label}.{field} must be a safe relative path")
    return diagnostics


def table_bundle_path_string_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = table.get(field)
        if not isinstance(value, str) or not value.strip():
            continue
        normalized = normalize_relative_path(value)
        if normalized == ".":
            continue
        if not is_safe_relative_path(normalized):
            diagnostics.append(f"{label}.{field} must be a safe relative path")
    return diagnostics


def sequence_safe_relative_path_string_diagnostics(
    label: str,
    value: object,
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_safe_relative_path_string_diagnostics(
                f"{label}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics


def sequence_unique_path_diagnostics(
    label: str,
    value: object,
) -> list[str]:
    return sequence_unique_relative_path_field_diagnostics(label, value, "path")


def sequence_unique_relative_path_field_diagnostics(
    label: str,
    value: object,
    field: str,
) -> list[str]:
    diagnostics: list[str] = []
    seen: dict[str, int] = {}
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        path = entry.get(field)
        if not isinstance(path, str) or not path.strip():
            continue
        normalized = normalize_relative_path(path)
        if normalized in seen:
            diagnostics.append(
                f"{label}[{index}].{field} duplicates {label}[{seen[normalized]}].{field}"
            )
            continue
        seen[normalized] = index
    return diagnostics
