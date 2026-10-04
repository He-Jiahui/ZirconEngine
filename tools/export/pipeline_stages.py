"""Pipeline stage selection for resumable Zircon exports."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .pipeline_report import pipeline_execution_stage_keys
from .stage_handoff import (
    stage_report_diagnostics_diagnostic,
    stage_report_fatal_diagnostic,
    stage_report_identity_diagnostic,
)
from .stage_handoff_strategy import (
    export_strategy_list_is_empty,
    export_strategy_list_is_invalid,
    export_strategies_from_validate_report,
    unsupported_export_strategies_from_validate_report,
)

REPORT_FILE_NAME = "report.json"
LIBRARY_EMBED_EXECUTION_STAGES = (
    "compile_host",
    "cook_assets",
    "pack",
    "platform_bundle",
)
FALLBACK_RESUME_STAGES = (
    "source_template",
    "native_dynamic",
    *LIBRARY_EMBED_EXECUTION_STAGES,
    "report",
)
INVALID_VALIDATE_REPORT: dict[str, Any] = {
    "stage": "Validate",
    "profile": "",
    "fatal": True,
    "diagnostics": ["validate report metadata is invalid"],
}


def pipeline_stages_after_validate(
    out_root: Path,
    profile: str,
) -> tuple[str, ...]:
    return (*pipeline_stages_from_validate_report(out_root, profile), "report")


def pipeline_stages_from_resume(
    out_root: Path,
    profile: str,
    resume_from: str,
) -> tuple[str, ...]:
    if resume_from == "report":
        return ("report",)
    report = load_validated_report_or_invalid(out_root, profile)
    if report is None:
        if resume_from in FALLBACK_RESUME_STAGES:
            return FALLBACK_RESUME_STAGES[FALLBACK_RESUME_STAGES.index(resume_from) :]
        return (resume_from, "report")
    if report is INVALID_VALIDATE_REPORT:
        return ("report",)
    stages = (*pipeline_stages_from_validated_report(report), "report")
    if resume_from in stages:
        return stages[stages.index(resume_from):]
    return ("report",)


def pipeline_stages_from_validate_report(
    out_root: Path,
    profile: str,
) -> tuple[str, ...]:
    report = load_validated_report_or_invalid(out_root, profile)
    if report is None:
        return LIBRARY_EMBED_EXECUTION_STAGES
    if report is INVALID_VALIDATE_REPORT:
        return ()
    return pipeline_stages_from_validated_report(report)


def pipeline_stages_from_validated_report(
    report: dict[str, Any],
) -> tuple[str, ...]:
    if export_strategy_list_is_invalid(report):
        return ()
    if export_strategy_list_is_empty(report):
        return ()
    if unsupported_export_strategies_from_validate_report(report):
        return ()
    strategies = export_strategies_from_validate_report(report)
    if not strategies:
        return LIBRARY_EMBED_EXECUTION_STAGES
    return pipeline_execution_stage_keys(strategies)


def load_validated_report(out_root: Path, profile: str) -> dict[str, Any] | None:
    report = load_validated_report_or_invalid(out_root, profile)
    if report is INVALID_VALIDATE_REPORT:
        return None
    return report


def load_validated_report_or_invalid(
    out_root: Path,
    profile: str,
) -> dict[str, Any] | None:
    report_path = out_root / "stages" / "validate" / REPORT_FILE_NAME
    if not report_path.exists():
        return None
    if not report_path.is_file():
        return INVALID_VALIDATE_REPORT
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except OSError:
        return INVALID_VALIDATE_REPORT
    except json.JSONDecodeError:
        return INVALID_VALIDATE_REPORT
    if (
        not isinstance(report, dict)
        or stage_report_identity_diagnostic(report, "validate")
        or stage_report_fatal_diagnostic(report, "validate")
        or stage_report_diagnostics_diagnostic(report, "validate")
        or report.get("fatal")
        or report.get("profile") != profile
    ):
        return INVALID_VALIDATE_REPORT
    return report
