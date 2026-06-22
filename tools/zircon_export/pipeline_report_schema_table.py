"""Reusable table and sequence schema diagnostics for export reports."""

from __future__ import annotations

from collections.abc import Callable
from typing import Any

from .pipeline_report_schema_primitives import (
    validate_bool_schema_diagnostics,
    validate_integer_schema_diagnostics,
    validate_object_array_schema_diagnostics,
    validate_object_schema_diagnostics,
    validate_string_schema_diagnostics,
)
from .pipeline_report_validate_string_array_schema import (
    validate_string_array_schema_diagnostics,
)


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


def object_array_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    allowed_fields: tuple[str, ...],
    *,
    string_fields: tuple[str, ...] = (),
    integer_fields: tuple[str, ...] = (),
    string_array_fields: tuple[str, ...] = (),
    required_string_fields: tuple[str, ...] = (),
    required_integer_fields: tuple[str, ...] = (),
    required_string_array_fields: tuple[str, ...] = (),
    required_object_array_fields: tuple[str, ...] = (),
    require_present: bool = False,
) -> list[str]:
    value = table.get(field)
    if value is None and not require_present:
        return []
    field_label = f"{label} {field}"
    if not isinstance(value, list):
        return [f"{field_label} must be an object array"]
    diagnostics: list[str] = []
    diagnostics.extend(validate_object_array_schema_diagnostics(field_label, value))
    diagnostics.extend(
        sequence_unknown_field_diagnostics(
            field_label,
            value,
            allowed_fields,
        )
    )
    diagnostics.extend(
        sequence_required_string_schema_diagnostics(
            field_label,
            value,
            required_string_fields,
        )
    )
    diagnostics.extend(
        sequence_required_integer_schema_diagnostics(
            field_label,
            value,
            required_integer_fields,
        )
    )
    diagnostics.extend(
        sequence_required_string_array_schema_diagnostics(
            field_label,
            value,
            required_string_array_fields,
        )
    )
    diagnostics.extend(
        sequence_required_object_array_schema_diagnostics(
            field_label,
            value,
            required_object_array_fields,
        )
    )
    diagnostics.extend(
        sequence_string_schema_diagnostics(
            field_label,
            value,
            optional_fields(string_fields, required_string_fields),
        )
    )
    diagnostics.extend(
        sequence_integer_schema_diagnostics(
            field_label,
            value,
            optional_fields(integer_fields, required_integer_fields),
        )
    )
    diagnostics.extend(
        sequence_string_array_schema_diagnostics(
            field_label,
            value,
            optional_fields(string_array_fields, required_string_array_fields),
        )
    )
    return diagnostics


def table_unknown_field_diagnostics(
    label: str,
    table: dict[str, Any],
    known_fields: tuple[str, ...],
) -> list[str]:
    known_field_set = set(known_fields)
    return [
        f"{label} unknown field {field}"
        for field in sorted(table)
        if field not in known_field_set
    ]


def sequence_unknown_field_diagnostics(
    label: str,
    value: object,
    known_fields: tuple[str, ...],
) -> list[str]:
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_unknown_field_diagnostics(
                f"{label}[{index}]",
                entry,
                known_fields,
            )
        )
    return diagnostics


def table_string_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    *,
    require_present: bool = False,
) -> list[str]:
    return table_field_schema_diagnostics(
        label,
        table,
        fields,
        validate_string_schema_diagnostics,
        require_present=require_present,
    )


def table_integer_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    *,
    require_present: bool = False,
) -> list[str]:
    return table_field_schema_diagnostics(
        label,
        table,
        fields,
        validate_integer_schema_diagnostics,
        require_present=require_present,
    )


def table_bool_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    *,
    require_present: bool = False,
) -> list[str]:
    return table_field_schema_diagnostics(
        label,
        table,
        fields,
        validate_bool_schema_diagnostics,
        require_present=require_present,
    )


def table_object_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    *,
    require_present: bool = False,
) -> list[str]:
    return table_field_schema_diagnostics(
        label,
        table,
        fields,
        validate_object_schema_diagnostics,
        require_present=require_present,
    )


def table_string_array_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    *,
    require_present: bool = False,
) -> list[str]:
    return table_field_schema_diagnostics(
        label,
        table,
        fields,
        validate_string_array_schema_diagnostics,
        require_present=require_present,
    )


def table_field_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    validate_schema: Callable[[str, Any], list[str]],
    *,
    require_present: bool = False,
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = table.get(field)
        if value is not None or require_present:
            diagnostics.extend(validate_schema(typed_field_label(label, field), value))
    return diagnostics


def typed_field_label(label: str, field: str) -> str:
    return f"{label}.{field}"


def optional_fields(
    fields: tuple[str, ...],
    required_fields: tuple[str, ...],
) -> tuple[str, ...]:
    required_field_set = set(required_fields)
    return tuple(field for field in fields if field not in required_field_set)


def sequence_string_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_string_schema_diagnostics(
                f"{label}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics


def sequence_required_string_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
) -> list[str]:
    return sequence_required_field_schema_diagnostics(
        label,
        value,
        fields,
        validate_string_schema_diagnostics,
    )


def sequence_integer_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_integer_schema_diagnostics(
                f"{label}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics


def sequence_required_integer_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
) -> list[str]:
    return sequence_required_field_schema_diagnostics(
        label,
        value,
        fields,
        validate_integer_schema_diagnostics,
    )


def sequence_required_string_array_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
) -> list[str]:
    return sequence_required_field_schema_diagnostics(
        label,
        value,
        fields,
        validate_string_array_schema_diagnostics,
    )


def sequence_required_object_array_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
) -> list[str]:
    return sequence_required_field_schema_diagnostics(
        label,
        value,
        fields,
        validate_object_array_schema_diagnostics,
    )


def sequence_required_field_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
    validate_schema: Callable[[str, Any], list[str]],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        for field in fields:
            diagnostics.extend(
                validate_schema(
                    typed_field_label(f"{label}[{index}]", field),
                    entry.get(field),
                )
            )
    return diagnostics


def sequence_string_array_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_string_array_schema_diagnostics(
                f"{label}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics
