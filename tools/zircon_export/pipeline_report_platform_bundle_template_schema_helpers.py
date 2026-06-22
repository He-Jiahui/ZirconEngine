"""Field-level helpers for PlatformBundle template schema diagnostics."""

from __future__ import annotations

from collections.abc import Callable
from typing import Any

from .export_template import (
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
)
from .pipeline_report_schema_primitives import (
    validate_bool_schema_diagnostics,
    validate_integer_schema_diagnostics,
    validate_object_array_schema_diagnostics,
    validate_object_schema_diagnostics,
    validate_string_schema_diagnostics,
)
from .pipeline_report_schema_table import (
    string_array_no_blank_entries_schema_diagnostics,
    string_array_unique_entries_schema_diagnostics,
)
from .pipeline_report_validate_string_array_schema import (
    validate_string_array_schema_diagnostics,
)


def sequence_object_schema_diagnostics(
    label: str,
    value: object,
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            diagnostics.append(f"{label}[{index}] must be an object")
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
) -> list[str]:
    return table_field_schema_diagnostics(
        label, table, fields, validate_string_schema_diagnostics
    )


def table_whitespace_only_string_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = table.get(field)
        if isinstance(value, str) and value and not value.strip():
            diagnostics.append(f"{label}.{field} must be a non-empty string")
    return diagnostics


def sequence_present_non_blank_string_diagnostics(
    label: str,
    value: object,
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        for field in fields:
            field_value = entry.get(field)
            if isinstance(field_value, str) and field in entry and not field_value.strip():
                diagnostics.append(f"{label}[{index}].{field} must be non-empty when present")
    return diagnostics


def sequence_present_trimmed_non_empty_string_diagnostics(
    label: str,
    value: object,
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        for field in fields:
            field_value = entry.get(field)
            if (
                isinstance(field_value, str)
                and field in entry
                and field_value.strip()
                and field_value.strip() != field_value
            ):
                diagnostics.append(
                    f"{label}[{index}].{field} "
                    "must be a non-empty trimmed string"
                )
    return diagnostics


def sequence_string_array_entries_trimmed_non_empty_diagnostics(
    label: str,
    value: object,
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_string_array_entries_trimmed_non_empty_diagnostics(
                f"{label}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics


def table_required_non_empty_string_diagnostics(
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


def table_present_trimmed_non_empty_string_diagnostics(
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


def table_enum_string_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: dict[str, set[str]],
) -> list[str]:
    diagnostics: list[str] = []
    for field, allowed_values in fields.items():
        value = table.get(field)
        if isinstance(value, str) and value.strip() and value not in allowed_values:
            diagnostics.append(
                f"{label}.{field}={value!r} is not one of "
                f"{', '.join(sorted(allowed_values))}"
            )
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


def table_integer_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    return table_field_schema_diagnostics(
        label, table, fields, validate_integer_schema_diagnostics
    )


def table_integer_equals_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    expected: int,
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = table.get(field)
        if isinstance(value, int) and not isinstance(value, bool) and value != expected:
            diagnostics.append(f"{label}.{field} must be {expected}")
    return diagnostics


def table_bool_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    return table_field_schema_diagnostics(
        label, table, fields, validate_bool_schema_diagnostics
    )


def table_string_array_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics = table_field_schema_diagnostics(
        label, table, fields, validate_string_array_schema_diagnostics
    )
    diagnostics.extend(
        table_field_schema_diagnostics(
            label,
            table,
            fields,
            string_array_no_blank_entries_schema_diagnostics,
        )
    )
    return diagnostics


def table_string_array_entries_trimmed_non_empty_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = table.get(field)
        if not (
            isinstance(value, list)
            and all(isinstance(item, str) for item in value)
        ):
            continue
        for index, item in enumerate(value):
            if item.strip() and item.strip() != item:
                diagnostics.append(
                    f"{label}.{field}[{index}] "
                    "must be a non-empty trimmed string"
                )
    return diagnostics


def table_unique_string_array_entries_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    return table_field_schema_diagnostics(
        label, table, fields, string_array_unique_entries_schema_diagnostics
    )


def table_object_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    return table_field_schema_diagnostics(
        label, table, fields, validate_object_schema_diagnostics
    )


def table_object_array_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    return table_field_schema_diagnostics(
        label, table, fields, validate_object_array_schema_diagnostics
    )


def table_field_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    validate_schema: Callable[[str, Any], list[str]],
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        if field in table and table.get(field) is not None:
            diagnostics.extend(validate_schema(f"{label}.{field}", table.get(field)))
    return diagnostics


def sequence_string_schema_diagnostics(
    label: str,
    value: object,
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


def sequence_required_non_empty_string_diagnostics(
    label: str,
    value: object,
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_required_non_empty_string_diagnostics(
                f"{label}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics


def sequence_string_array_schema_diagnostics(
    label: str,
    value: object,
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


def sequence_unique_string_array_entries_schema_diagnostics(
    label: str,
    value: object,
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_unique_string_array_entries_schema_diagnostics(
                f"{label}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics
