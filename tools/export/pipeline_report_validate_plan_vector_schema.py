"""Validate report plan_summary vector schema diagnostics."""

from __future__ import annotations

from collections.abc import Callable
from typing import Any

from .pipeline_report_validate_identifier_schema import (
    validate_native_dynamic_package_id_array_schema_diagnostics,
    validate_native_dynamic_package_id_schema_diagnostics,
    validate_project_plugin_package_id_array_schema_diagnostics,
    validate_project_plugin_package_id_schema_diagnostics,
    validate_project_runtime_crate_name_array_schema_diagnostics,
    validate_project_runtime_crate_name_schema_diagnostics,
)

VALIDATE_PLAN_SUMMARY_PROJECT_PLUGIN_ID_ARRAY_FIELDS = ("enabled_runtime_plugins",)
VALIDATE_PLAN_SUMMARY_RUNTIME_CRATE_NAME_ARRAY_FIELDS = ("linked_runtime_crates",)
VALIDATE_PLAN_SUMMARY_NATIVE_DYNAMIC_PACKAGE_ID_ARRAY_FIELDS = (
    "native_dynamic_packages",
)


def validate_plan_summary_vector_schema_diagnostics(
    plan_summary: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    for field in VALIDATE_PLAN_SUMMARY_PROJECT_PLUGIN_ID_ARRAY_FIELDS:
        if field in plan_summary:
            label = f"validate report plan_summary.{field}"
            value = plan_summary.get(field)
            diagnostics.extend(
                validate_project_plugin_package_id_array_schema_diagnostics(
                    label,
                    value,
                )
            )
            diagnostics.extend(
                validate_unique_plan_vector_entries_schema_diagnostics(
                    label,
                    value,
                    validate_project_plugin_package_id_schema_diagnostics,
                )
            )
    for field in VALIDATE_PLAN_SUMMARY_RUNTIME_CRATE_NAME_ARRAY_FIELDS:
        if field in plan_summary:
            label = f"validate report plan_summary.{field}"
            value = plan_summary.get(field)
            diagnostics.extend(
                validate_project_runtime_crate_name_array_schema_diagnostics(
                    label,
                    value,
                )
            )
            diagnostics.extend(
                validate_unique_plan_vector_entries_schema_diagnostics(
                    label,
                    value,
                    validate_project_runtime_crate_name_schema_diagnostics,
                )
            )
    for field in VALIDATE_PLAN_SUMMARY_NATIVE_DYNAMIC_PACKAGE_ID_ARRAY_FIELDS:
        if field in plan_summary:
            label = f"validate report plan_summary.{field}"
            value = plan_summary.get(field)
            diagnostics.extend(
                validate_native_dynamic_package_id_array_schema_diagnostics(
                    label,
                    value,
                )
            )
            diagnostics.extend(
                validate_unique_plan_vector_entries_schema_diagnostics(
                    label,
                    value,
                    validate_native_dynamic_package_id_schema_diagnostics,
                )
            )
    return diagnostics


def validate_required_plan_summary_vector_schema_diagnostics(
    plan_summary: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    required_fields = (
        (
            VALIDATE_PLAN_SUMMARY_PROJECT_PLUGIN_ID_ARRAY_FIELDS,
            validate_project_plugin_package_id_array_schema_diagnostics,
        ),
        (
            VALIDATE_PLAN_SUMMARY_RUNTIME_CRATE_NAME_ARRAY_FIELDS,
            validate_project_runtime_crate_name_array_schema_diagnostics,
        ),
        (
            VALIDATE_PLAN_SUMMARY_NATIVE_DYNAMIC_PACKAGE_ID_ARRAY_FIELDS,
            validate_native_dynamic_package_id_array_schema_diagnostics,
        ),
    )
    for fields, field_validator in required_fields:
        for field in fields:
            if field in plan_summary:
                continue
            diagnostics.extend(
                field_validator(
                    f"validate report plan_summary.{field}",
                    plan_summary.get(field),
                )
            )
    return diagnostics


def validate_unique_plan_vector_entries_schema_diagnostics(
    label: str,
    value: Any,
    entry_validator: Callable[[str, Any], list[str]],
) -> list[str]:
    if not isinstance(value, list) or any(not isinstance(item, str) for item in value):
        return []
    diagnostics: list[str] = []
    seen: dict[str, int] = {}
    for index, item in enumerate(value):
        if entry_validator(f"{label}[{index}]", item):
            continue
        previous_index = seen.get(item)
        if previous_index is None:
            seen[item] = index
            continue
        diagnostics.append(f"{label}[{index}] duplicates entry {previous_index}")
    return diagnostics
