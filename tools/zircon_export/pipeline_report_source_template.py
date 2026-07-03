"""SourceTemplate final report diagnostics for the Zircon export pipeline."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .pipeline_report_source_template_build_handoff import (
    source_template_build_validation_diagnostics,
    source_template_validate_build_plan_diagnostics,
)
from .pipeline_report_source_template_generated_files import (
    source_template_generated_file_diagnostics,
)
from .pipeline_report_source_template_path_semantics import (
    resolve_source_template_path_or_diagnostic,
    source_template_is_non_empty_trimmed_string,
)
from .pipeline_report_source_template_stage_schema import (
    source_template_report_schema_diagnostics,
)


def source_template_project_diagnostics(
    stage_reports: list[dict[str, Any]],
    validate_report: dict[str, Any] | None = None,
    validate_report_path: Path | None = None,
) -> list[str]:
    diagnostics: list[str] = []
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "source_template":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        diagnostics.extend(source_template_report_schema_diagnostics(report))
        expected_project_dir = source_template_expected_project_dir(
            stage_report,
            diagnostics,
        )
        project_dir = source_template_project_path(report, diagnostics, expected_project_dir)
        if project_dir is None:
            continue
        diagnostics.extend(
            source_template_validate_report_path_diagnostics(
                report,
                validate_report_path,
            )
        )
        diagnostics.extend(
            source_template_validate_build_plan_diagnostics(
                validate_report,
                project_dir,
            )
        )
        diagnostics.extend(
            source_template_build_validation_diagnostics(report, validate_report)
        )
        diagnostics.extend(
            source_template_generated_file_diagnostics(
                report,
                project_dir,
                validate_report,
            )
        )
    return diagnostics


def source_template_project_path(
    report: dict[str, Any],
    diagnostics: list[str],
    expected_project_dir: Path | None,
) -> Path | None:
    project = report.get("project")
    if not isinstance(project, str) or not project.strip():
        diagnostics.append("SourceTemplate report project must be a non-empty string")
        return None
    if not source_template_is_non_empty_trimmed_string(project):
        return None
    project_dir = resolve_source_template_path_or_diagnostic(
        project,
        diagnostics,
        "SourceTemplate report project",
    )
    if project_dir is None:
        return None
    if expected_project_dir is not None:
        expected_dir = resolve_source_template_path_or_diagnostic(
            expected_project_dir,
            diagnostics,
            "SourceTemplate report expected project",
        )
        if expected_dir is None:
            return None
    else:
        expected_dir = None
    if expected_dir is not None and project_dir != expected_dir:
        diagnostics.append(
            "SourceTemplate report project must match current SourceTemplate stage project"
        )
        return None
    if not project_dir.exists():
        diagnostics.append(f"SourceTemplate project {project_dir} does not exist")
        return None
    if not project_dir.is_dir():
        diagnostics.append(f"SourceTemplate project {project_dir} is not a directory")
        return None
    return project_dir


def source_template_expected_project_dir(
    stage_report: dict[str, Any],
    diagnostics: list[str],
) -> Path | None:
    report_path = stage_report.get("path")
    if not isinstance(report_path, str) or not report_path:
        return None
    resolved_report_path = resolve_source_template_path_or_diagnostic(
        report_path,
        diagnostics,
        "SourceTemplate stage report path",
    )
    if resolved_report_path is None:
        return None
    return resolved_report_path.parent / "project"


def source_template_validate_report_path_diagnostics(
    report: dict[str, Any],
    expected_path: Path | None,
) -> list[str]:
    diagnostics: list[str] = []
    validate_report = report.get("validate_report")
    if not isinstance(validate_report, str) or not validate_report.strip():
        return ["SourceTemplate report validate_report must be a non-empty string"]
    if not source_template_is_non_empty_trimmed_string(validate_report):
        return []
    if expected_path is None:
        return []
    actual_path = resolve_source_template_path_or_diagnostic(
        validate_report,
        diagnostics,
        "SourceTemplate report validate_report",
    )
    if actual_path is None:
        return diagnostics
    expected_validate_report = resolve_source_template_path_or_diagnostic(
        expected_path,
        diagnostics,
        "SourceTemplate expected Validate report",
    )
    if expected_validate_report is None:
        return diagnostics
    if actual_path != expected_validate_report:
        return ["SourceTemplate report validate_report must match current Validate report"]
    return diagnostics
