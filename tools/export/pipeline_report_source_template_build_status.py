"""SourceTemplate build validation status diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_source_template_path_semantics import (
    source_template_is_non_empty_trimmed_string,
)


def source_template_build_status_diagnostics(
    *,
    requested: Any,
    executed: Any,
    status: Any,
    exit_code: Any,
    build_executed: Any,
) -> list[str]:
    diagnostics: list[str] = []
    if not isinstance(requested, bool):
        diagnostics.append("SourceTemplate build_validation requested must be a boolean")
    if not isinstance(executed, bool):
        diagnostics.append("SourceTemplate build_validation executed must be a boolean")
    if (
        source_template_is_non_empty_trimmed_string(status)
        and status not in {"skipped", "passed", "failed", "blocked"}
    ):
        diagnostics.append(
            "SourceTemplate build_validation status must be skipped, passed, failed, or blocked"
        )
    if not isinstance(build_executed, bool):
        diagnostics.append("SourceTemplate report build_executed must be a boolean")
    elif isinstance(executed, bool) and executed != build_executed:
        diagnostics.append(
            "SourceTemplate build_validation executed must match "
            "SourceTemplate report build_executed"
        )

    if status in {"failed", "blocked"}:
        diagnostics.append(
            f"SourceTemplate build_validation status {status} is not publishable"
        )
    if status == "skipped":
        diagnostics.append(
            "SourceTemplate build_validation skipped status is not publishable"
        )
    if status == "passed" and exit_code != 0:
        diagnostics.append(
            "SourceTemplate build_validation passed status requires exit_code 0"
        )
    if status == "failed" and not isinstance(exit_code, int):
        diagnostics.append(
            "SourceTemplate build_validation failed status requires an integer exit_code"
        )
    if status == "skipped" and executed is True:
        diagnostics.append("SourceTemplate build_validation skipped status cannot be executed")
    if status == "skipped" and requested is True:
        diagnostics.append("SourceTemplate build_validation requested build cannot be skipped")
    if status == "skipped" and exit_code is not None:
        diagnostics.append("SourceTemplate build_validation skipped status requires exit_code null")
    if executed is True and requested is not True:
        diagnostics.append("SourceTemplate build_validation executed build must be requested")
    if status == "passed" and executed is not True:
        diagnostics.append("SourceTemplate build_validation passed status requires executed=true")
    return diagnostics
