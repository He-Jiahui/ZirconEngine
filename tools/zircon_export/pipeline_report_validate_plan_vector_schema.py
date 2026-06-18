"""Validate report plan_summary vector schema diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_validate_identifier_schema import (
    validate_native_dynamic_package_id_array_schema_diagnostics,
    validate_project_plugin_package_id_array_schema_diagnostics,
    validate_project_runtime_crate_name_array_schema_diagnostics,
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
            diagnostics.extend(
                validate_project_plugin_package_id_array_schema_diagnostics(
                    f"validate report plan_summary.{field}",
                    plan_summary.get(field),
                )
            )
    for field in VALIDATE_PLAN_SUMMARY_RUNTIME_CRATE_NAME_ARRAY_FIELDS:
        if field in plan_summary:
            diagnostics.extend(
                validate_project_runtime_crate_name_array_schema_diagnostics(
                    f"validate report plan_summary.{field}",
                    plan_summary.get(field),
                )
            )
    for field in VALIDATE_PLAN_SUMMARY_NATIVE_DYNAMIC_PACKAGE_ID_ARRAY_FIELDS:
        if field in plan_summary:
            diagnostics.extend(
                validate_native_dynamic_package_id_array_schema_diagnostics(
                    f"validate report plan_summary.{field}",
                    plan_summary.get(field),
                )
            )
    return diagnostics
