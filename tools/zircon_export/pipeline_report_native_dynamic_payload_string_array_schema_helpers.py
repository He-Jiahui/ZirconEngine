"""String-array helper diagnostics for NativeDynamic payload schema checks."""

from __future__ import annotations

from typing import Any

from .export_template_manifest import (
    is_safe_relative_path,
    normalize_relative_path,
)
from .pipeline_report_schema_string_array import (
    string_array_trimmed_non_empty_entries_schema_diagnostics,
)


def materialized_package_loadable_artifact_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    integer_field: str,
    string_array_field: str,
) -> tuple[list[str], list[str]]:
    value = table.get(field)
    if not isinstance(value, list):
        return [], []
    type_diagnostics: list[str] = []
    blank_diagnostics: list[str] = []
    trimmed_diagnostics: list[str] = []
    path_diagnostics: list[str] = []
    duplicate_diagnostics: list[str] = []
    length_diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        item_value = entry.get(string_array_field)
        if not isinstance(item_value, list):
            type_diagnostics.append(
                f"{label} {field}[{index}].{string_array_field} "
                "must be a string array"
            )
            continue
        all_strings = True
        seen: set[str] = set()
        has_duplicate = False
        item_label = f"{label} {field}[{index}].{string_array_field}"
        for item_index, item in enumerate(item_value):
            if not isinstance(item, str):
                all_strings = False
                type_diagnostics.append(
                    f"{item_label}[{item_index}] must be a string"
                )
                continue
            stripped = item.strip()
            if not stripped:
                continue
            if stripped != item:
                trimmed_diagnostics.append(
                    f"{item_label}[{item_index}] must be a non-empty trimmed string"
                )
            normalized = normalize_relative_path(item)
            path_is_safe = is_safe_relative_path(normalized)
            if not path_is_safe:
                path_diagnostics.append(
                    f"{item_label}[{item_index}] must be a safe relative path"
                )
            if stripped == item and path_is_safe:
                if normalized in seen:
                    has_duplicate = True
                else:
                    seen.add(normalized)
        if all_strings:
            if any(not item.strip() for item in item_value):
                blank_diagnostics.append(
                    f"{item_label} must not contain blank entries"
                )
            if has_duplicate:
                duplicate_diagnostics.append(
                    f"{item_label} must not contain duplicate entries"
                )
            integer_value = entry.get(integer_field)
            if (
                type(integer_value) is int
                and integer_value >= 0
                and integer_value != len(item_value)
            ):
                length_diagnostics.append(
                    f"{label} {field}[{index}].{integer_field} "
                    f"must match {string_array_field} length"
                )
    return type_diagnostics, (
        blank_diagnostics
        + trimmed_diagnostics
        + path_diagnostics
        + duplicate_diagnostics
        + length_diagnostics
    )


def object_array_loadable_artifacts_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
) -> list[str]:
    value = table.get(field)
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    item_field = "loadable_artifacts"
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        item_value = entry.get(item_field)
        if not isinstance(item_value, list):
            diagnostics.append(
                f"{label} {field}[{index}].{item_field} "
                "must be a string array"
            )
            continue
        for item_index, item in enumerate(item_value):
            if not isinstance(item, str):
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field}[{item_index}] "
                    "must be a string"
                )
    return diagnostics


def object_array_string_array_no_blank_entries_schema_diagnostics(
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
                isinstance(item_value, list)
                and all(isinstance(item, str) for item in item_value)
                and any(not item.strip() for item in item_value)
            ):
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field} "
                    "must not contain blank entries"
                )
    return diagnostics


def object_array_string_array_trimmed_non_empty_entries_schema_diagnostics(
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
            if not (
                isinstance(item_value, list)
                and all(isinstance(item, str) for item in item_value)
            ):
                continue
            diagnostics.extend(
                string_array_trimmed_non_empty_entries_schema_diagnostics(
                    f"{label} {field}[{index}].{item_field}",
                    item_value,
                )
            )
    return diagnostics


def object_array_string_array_safe_relative_path_schema_diagnostics(
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
            if not (
                isinstance(item_value, list)
                and all(isinstance(item, str) for item in item_value)
            ):
                continue
            for item_index, item in enumerate(item_value):
                if (
                    item.strip()
                    and not is_safe_relative_path(normalize_relative_path(item))
                ):
                    diagnostics.append(
                        f"{label} {field}[{index}].{item_field}[{item_index}] "
                        "must be a safe relative path"
                    )
    return diagnostics


def object_array_string_array_unique_entries_schema_diagnostics(
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
            if not (
                isinstance(item_value, list)
                and all(isinstance(item, str) for item in item_value)
            ):
                continue
            seen: set[str] = set()
            has_duplicate = False
            for item in item_value:
                if not item.strip() or item.strip() != item:
                    continue
                normalized = normalize_relative_path(item)
                if not is_safe_relative_path(normalized):
                    continue
                if normalized in seen:
                    has_duplicate = True
                    break
                seen.add(normalized)
            if has_duplicate:
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field} "
                    "must not contain duplicate entries"
                )
    return diagnostics


def object_array_integer_matches_string_array_length_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    integer_field: str,
    string_array_field: str,
) -> list[str]:
    value = table.get(field)
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        integer_value = entry.get(integer_field)
        array_value = entry.get(string_array_field)
        if (
            type(integer_value) is int
            and integer_value >= 0
            and isinstance(array_value, list)
            and all(isinstance(item, str) for item in array_value)
            and integer_value != len(array_value)
        ):
            diagnostics.append(
                f"{label} {field}[{index}].{integer_field} "
                f"must match {string_array_field} length"
            )
    return diagnostics
