"""Loadable-artifact row diagnostics for NativeDynamic payloads."""

from __future__ import annotations

from typing import Any


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
