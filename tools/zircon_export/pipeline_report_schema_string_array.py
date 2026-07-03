"""Reusable string-array schema diagnostics for export reports."""

from __future__ import annotations

from typing import Any


def non_empty_string_array_schema_diagnostics(label: str, value: Any) -> list[str]:
    if (
        isinstance(value, list)
        and all(isinstance(item, str) for item in value)
        and (not value or any(not item.strip() for item in value))
    ):
        return [f"{label} must be a non-empty string array"]
    return []


def string_array_no_blank_entries_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if (
        isinstance(value, list)
        and all(isinstance(item, str) for item in value)
        and any(not item.strip() for item in value)
    ):
        return [f"{label} must not contain blank entries"]
    return []


def string_array_trimmed_non_empty_entries_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not (
        isinstance(value, list)
        and all(isinstance(item, str) for item in value)
    ):
        return []
    diagnostics: list[str] = []
    for index, item in enumerate(value):
        if item.strip() and item.strip() != item:
            diagnostics.append(
                f"{label}[{index}] must be a non-empty trimmed string"
            )
    return diagnostics


def string_array_unique_entries_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not (
        isinstance(value, list)
        and all(isinstance(item, str) for item in value)
    ):
        return []
    diagnostics: list[str] = []
    seen: set[str] = set()
    for item in value:
        if not item.strip() or item.strip() != item:
            continue
        if item in seen:
            diagnostics.append(f"{label} duplicate entry {item}")
            continue
        seen.add(item)
    return diagnostics


def string_array_duplicate_entry_index_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not (
        isinstance(value, list)
        and all(isinstance(item, str) for item in value)
    ):
        return []
    diagnostics: list[str] = []
    seen: dict[str, int] = {}
    for index, item in enumerate(value):
        if not item.strip() or item.strip() != item:
            continue
        previous_index = seen.get(item)
        if previous_index is None:
            seen[item] = index
            continue
        diagnostics.append(f"{label}[{index}] duplicates entry {previous_index}")
    return diagnostics
