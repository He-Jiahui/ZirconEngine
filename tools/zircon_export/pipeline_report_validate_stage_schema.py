"""Validate stage report schema diagnostics for Zircon export final reports."""

from __future__ import annotations

from typing import Any

from .export_strategy_contract import normalize_export_strategy
from .pipeline_report_native_dynamic_package_export_schema import (
    validate_native_dynamic_package_exports_schema_diagnostics,
)
from .pipeline_report_source_template_validate_schema import (
    source_template_validate_build_plan_schema_diagnostics,
    source_template_validate_generated_files_schema_diagnostics,
)
from .pipeline_report_schema_primitives import (
    validate_bool_schema_diagnostics,
    validate_object_schema_diagnostics,
    validate_string_schema_diagnostics,
)
from .pipeline_report_schema_table import (
    string_array_no_blank_entries_schema_diagnostics,
    string_array_trimmed_non_empty_entries_schema_diagnostics,
)
from .pipeline_report_validate_compile_host_schema import (
    validate_library_embed_compile_host_schema_diagnostics,
)
from .pipeline_report_validate_plan_vector_schema import (
    validate_plan_summary_vector_schema_diagnostics,
    validate_required_plan_summary_vector_schema_diagnostics,
)
from .pipeline_report_validate_profile_summary_schema import (
    validate_profile_summary_schema_diagnostics,
)
from .pipeline_report_validate_runtime_availability_schema import (
    validate_runtime_plugin_availability_schema_diagnostics,
)
from .pipeline_report_validate_string_array_schema import (
    validate_string_array_schema_diagnostics,
)

VALIDATE_REPORT_FIELDS = (
    "diagnostics",
    "fatal",
    "fatal_diagnostics",
    "plan_summary",
    "profile",
    "profile_found",
    "profile_summary",
    "project_manifest",
    "stage",
    "stage_output",
)
VALIDATE_REPORT_STRING_FIELDS = (
    "project_manifest",
    "stage_output",
)
VALIDATE_REPORT_STRING_ARRAY_FIELDS = ("fatal_diagnostics",)
VALIDATE_REPORT_BOOL_FIELDS = ("profile_found",)
VALIDATE_REPORT_OBJECT_FIELDS = (
    "plan_summary",
    "profile_summary",
)
VALIDATE_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS = (
    "project_manifest",
    "stage_output",
)
VALIDATE_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS = ("fatal_diagnostics",)
VALIDATE_REPORT_REQUIRED_NON_FATAL_BOOL_FIELDS = ("profile_found",)
VALIDATE_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS = (
    "plan_summary",
    "profile_summary",
)
VALIDATE_PLAN_SUMMARY_FIELDS = (
    "enabled_runtime_plugins",
    "generated_files",
    "library_embed_compile_host",
    "linked_runtime_crates",
    "native_dynamic_package_exports",
    "native_dynamic_packages",
    "runtime_plugin_availability",
    "source_template_build",
)


def validate_report_schema_diagnostics(report: dict[str, Any]) -> list[str]:
    diagnostics: list[str] = []
    for field in VALIDATE_REPORT_STRING_FIELDS:
        if field in report:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"validate report {field}",
                    report.get(field),
                )
            )
    for field in VALIDATE_REPORT_STRING_ARRAY_FIELDS:
        if field in report:
            diagnostics.extend(
                validate_string_array_schema_diagnostics(
                    f"validate report {field}",
                    report.get(field),
                )
            )
            diagnostics.extend(
                string_array_no_blank_entries_schema_diagnostics(
                    f"validate report {field}",
                    report.get(field),
                )
            )
            diagnostics.extend(
                string_array_trimmed_non_empty_entries_schema_diagnostics(
                    f"validate report {field}",
                    report.get(field),
                )
            )
    for field in VALIDATE_REPORT_BOOL_FIELDS:
        if field in report:
            diagnostics.extend(
                validate_bool_schema_diagnostics(
                    f"validate report {field}",
                    report.get(field),
                )
            )
    for field in VALIDATE_REPORT_OBJECT_FIELDS:
        if field in report:
            diagnostics.extend(
                validate_object_schema_diagnostics(
                    f"validate report {field}",
                    report.get(field),
                )
            )
    if report.get("fatal") is False:
        for field in VALIDATE_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS:
            if field not in report or report.get(field) is None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"validate report {field}",
                        report.get(field),
                    )
                )
        for field in VALIDATE_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"validate report {field}",
                        report.get(field),
                    )
                )
        for field in VALIDATE_REPORT_REQUIRED_NON_FATAL_BOOL_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_bool_schema_diagnostics(
                        f"validate report {field}",
                        report.get(field),
                    )
                )
        for field in VALIDATE_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS:
            if field not in report or report.get(field) is None:
                diagnostics.extend(
                    validate_object_schema_diagnostics(
                        f"validate report {field}",
                        report.get(field),
                    )
                )
    profile_summary = report.get("profile_summary")
    if isinstance(profile_summary, dict):
        diagnostics.extend(
            validate_profile_summary_schema_diagnostics(profile_summary)
        )
    plan_summary = report.get("plan_summary")
    if isinstance(plan_summary, dict):
        diagnostics.extend(
            validate_plan_summary_schema_diagnostics(
                plan_summary,
                profile_summary=profile_summary,
                require_release_evidence=report.get("fatal") is False,
            )
        )
    return diagnostics


def validate_plan_summary_schema_diagnostics(
    plan_summary: dict[str, Any],
    *,
    profile_summary: dict[str, Any] | None = None,
    require_release_evidence: bool = False,
) -> list[str]:
    known_plan_summary_fields = set(VALIDATE_PLAN_SUMMARY_FIELDS)
    diagnostics = [
        f"validate report plan_summary unknown field {field}"
        for field in sorted(plan_summary)
        if field not in known_plan_summary_fields
    ]
    if require_release_evidence:
        diagnostics.extend(
            validate_required_plan_summary_release_evidence_schema_diagnostics(
                plan_summary
            )
        )
    diagnostics.extend(validate_plan_summary_vector_schema_diagnostics(plan_summary))
    library_embed_compile_host = plan_summary.get("library_embed_compile_host")
    if "library_embed_compile_host" in plan_summary:
        diagnostics.extend(
            validate_library_embed_compile_host_schema_diagnostics(
                library_embed_compile_host,
            )
        )
    source_template_build = plan_summary.get("source_template_build")
    if "source_template_build" in plan_summary:
        diagnostics.extend(
            source_template_validate_build_plan_schema_diagnostics(
                source_template_build,
            )
        )
    generated_files = plan_summary.get("generated_files")
    if "generated_files" in plan_summary:
        diagnostics.extend(
            source_template_validate_generated_files_schema_diagnostics(
                generated_files
            )
        )
    native_dynamic_package_exports = plan_summary.get(
        "native_dynamic_package_exports"
    )
    if (
        native_dynamic_release_evidence_required(
            plan_summary,
            profile_summary=profile_summary,
        )
        and "native_dynamic_package_exports" not in plan_summary
    ):
        diagnostics.extend(
            validate_native_dynamic_package_exports_schema_diagnostics(
                native_dynamic_package_exports
            )
        )
    if "native_dynamic_package_exports" in plan_summary:
        diagnostics.extend(
            validate_native_dynamic_package_exports_schema_diagnostics(
                native_dynamic_package_exports
            )
        )
    runtime_plugin_availability = plan_summary.get("runtime_plugin_availability")
    if "runtime_plugin_availability" in plan_summary:
        diagnostics.extend(
            validate_runtime_plugin_availability_schema_diagnostics(
                runtime_plugin_availability,
            )
        )
    return diagnostics


def validate_required_plan_summary_release_evidence_schema_diagnostics(
    plan_summary: dict[str, Any],
) -> list[str]:
    diagnostics = validate_required_plan_summary_vector_schema_diagnostics(
        plan_summary
    )
    if "generated_files" not in plan_summary:
        diagnostics.extend(
            source_template_validate_generated_files_schema_diagnostics(
                plan_summary.get("generated_files")
            )
        )
    if "runtime_plugin_availability" not in plan_summary:
        diagnostics.extend(
            validate_runtime_plugin_availability_schema_diagnostics(
                plan_summary.get("runtime_plugin_availability"),
            )
        )
    return diagnostics


def native_dynamic_release_evidence_required(
    plan_summary: dict[str, Any],
    *,
    profile_summary: dict[str, Any] | None,
) -> bool:
    packages = plan_summary.get("native_dynamic_packages")
    if isinstance(packages, list) and packages:
        return True
    if not isinstance(profile_summary, dict):
        return False
    strategies = profile_summary.get("strategies")
    if not isinstance(strategies, list):
        return False
    return any(
        normalize_export_strategy(strategy) == "native_dynamic"
        for strategy in strategies
    )
