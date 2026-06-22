"""SourceTemplate string-array schema diagnostics."""

from __future__ import annotations

from typing import Any


def source_template_non_empty_string_array_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, list) or not value:
        return [f"{label} must be a non-empty string array"]
    entry_type_diagnostics = [
        f"{label}[{index}] must be a string"
        for index, item in enumerate(value)
        if not isinstance(item, str)
    ]
    if entry_type_diagnostics:
        return entry_type_diagnostics
    if any(not item.strip() for item in value):
        return [f"{label} must be a non-empty string array"]
    return []
