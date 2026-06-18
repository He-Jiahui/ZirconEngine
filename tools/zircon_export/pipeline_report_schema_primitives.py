"""Shared schema primitive diagnostics for pipeline reports."""

from __future__ import annotations

from typing import Any


def validate_bool_schema_diagnostics(label: str, value: Any) -> list[str]:
    if not isinstance(value, bool):
        return [f"{label} must be a boolean"]
    return []


def validate_integer_schema_diagnostics(label: str, value: Any) -> list[str]:
    if not isinstance(value, int) or isinstance(value, bool):
        return [f"{label} must be an integer"]
    return []


def validate_string_schema_diagnostics(label: str, value: Any) -> list[str]:
    if not isinstance(value, str):
        return [f"{label} must be a string"]
    return []


def validate_string_array_schema_diagnostics(label: str, value: Any) -> list[str]:
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        return [f"{label} must be a string array"]
    return []


def validate_object_array_schema_diagnostics(label: str, value: Any) -> list[str]:
    if not isinstance(value, list):
        return [f"{label} must be an object array"]
    diagnostics: list[str] = []
    for index, item in enumerate(value):
        if not isinstance(item, dict):
            diagnostics.append(f"{label}[{index}] must be an object")
    return diagnostics


def validate_object_schema_diagnostics(label: str, value: Any) -> list[str]:
    if not isinstance(value, dict):
        return [f"{label} must be an object"]
    return []
