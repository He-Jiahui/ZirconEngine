"""CookAssets stage report schema diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_schema_primitives import (
    validate_bool_schema_diagnostics,
    validate_integer_schema_diagnostics,
    validate_string_schema_diagnostics,
)

COOK_ASSETS_REPORT_FIELDS = (
    "asset_count",
    "asset_filter",
    "cooked_asset_manifest",
    "cooked_asset_manifest_sha256",
    "diagnostics",
    "fatal",
    "generated_from_project",
    "profile",
    "project_default_scene",
    "project_manifest",
    "root_count",
    "source_asset_manifest",
    "stage",
)
COOK_ASSETS_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS = (
    "cooked_asset_manifest",
    "cooked_asset_manifest_sha256",
)
COOK_ASSETS_REPORT_OPTIONAL_STRING_FIELDS = (
    "asset_filter",
    "project_default_scene",
    "project_manifest",
    "source_asset_manifest",
)
COOK_ASSETS_REPORT_STRING_FIELDS = (
    *COOK_ASSETS_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS,
    *COOK_ASSETS_REPORT_OPTIONAL_STRING_FIELDS,
)
COOK_ASSETS_REPORT_INTEGER_FIELDS = (
    "asset_count",
    "root_count",
)
COOK_ASSETS_REPORT_BOOL_FIELDS = ("generated_from_project",)
COOK_ASSETS_REPORT_REQUIRED_NON_FATAL_INTEGER_FIELDS = (
    "asset_count",
    "root_count",
)
COOK_ASSETS_REPORT_REQUIRED_NON_FATAL_BOOL_FIELDS = ("generated_from_project",)


def cook_assets_report_schema_diagnostics(report: dict[str, Any]) -> list[str]:
    diagnostics: list[str] = []
    for field in COOK_ASSETS_REPORT_STRING_FIELDS:
        value = report.get(field)
        if field in report and value is not None:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"cook_assets report {field}",
                    value,
                )
            )
            if isinstance(value, str):
                if (
                    field in COOK_ASSETS_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS
                    and (not value.strip() or value.strip() != value)
                ):
                    diagnostics.append(
                        f"cook_assets report {field} "
                        "must be a non-empty trimmed string"
                    )
                elif (
                    field in COOK_ASSETS_REPORT_OPTIONAL_STRING_FIELDS
                    and (not value.strip() or value.strip() != value)
                ):
                    diagnostics.append(
                        f"cook_assets report {field} "
                        "must be a non-empty trimmed string when present"
                    )
    for field in COOK_ASSETS_REPORT_INTEGER_FIELDS:
        if field in report:
            diagnostics.extend(
                validate_integer_schema_diagnostics(
                    f"cook_assets report {field}",
                    report.get(field),
                )
            )
    for field in COOK_ASSETS_REPORT_BOOL_FIELDS:
        if field in report:
            diagnostics.extend(
                validate_bool_schema_diagnostics(
                    f"cook_assets report {field}",
                    report.get(field),
                )
            )
    if report.get("fatal") is False:
        for field in COOK_ASSETS_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS:
            if field not in report or report.get(field) is None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"cook_assets report {field}",
                        report.get(field),
                    )
                )
        for field in COOK_ASSETS_REPORT_REQUIRED_NON_FATAL_INTEGER_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        f"cook_assets report {field}",
                        report.get(field),
                    )
                )
        for field in COOK_ASSETS_REPORT_REQUIRED_NON_FATAL_BOOL_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_bool_schema_diagnostics(
                        f"cook_assets report {field}",
                        report.get(field),
                    )
                )
    return diagnostics
