"""Validate report string-array schema diagnostics."""

from __future__ import annotations

from typing import Any


def validate_string_array_schema_diagnostics(label: str, value: Any) -> list[str]:
    if not isinstance(value, list):
        return [f"{label} must be a string array"]
    return [
        f"{label}[{index}] must be a string"
        for index, item in enumerate(value)
        if not isinstance(item, str)
    ]
