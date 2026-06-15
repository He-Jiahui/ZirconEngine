"""Pipeline stage selection for resumable Zircon exports."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .pipeline_report import (
    export_strategies_from_validate_report,
    pipeline_execution_stage_keys,
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
    report = load_validated_report(out_root, profile)
    if report is None:
        if resume_from in FALLBACK_RESUME_STAGES:
            return FALLBACK_RESUME_STAGES[FALLBACK_RESUME_STAGES.index(resume_from) :]
        return (resume_from, "report")
    stages = (*pipeline_stages_from_validated_report(report), "report")
    if resume_from in stages:
        return stages[stages.index(resume_from):]
    return ("report",)


def pipeline_stages_from_validate_report(
    out_root: Path,
    profile: str,
) -> tuple[str, ...]:
    report = load_validated_report(out_root, profile)
    if report is None:
        return LIBRARY_EMBED_EXECUTION_STAGES
    return pipeline_stages_from_validated_report(report)


def pipeline_stages_from_validated_report(
    report: dict[str, Any],
) -> tuple[str, ...]:
    strategies = export_strategies_from_validate_report(report)
    if not strategies:
        return LIBRARY_EMBED_EXECUTION_STAGES
    return pipeline_execution_stage_keys(strategies)


def load_validated_report(out_root: Path, profile: str) -> dict[str, Any] | None:
    report_path = out_root / "stages" / "validate" / REPORT_FILE_NAME
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    if (
        not isinstance(report, dict)
        or report.get("fatal")
        or report.get("profile") != profile
    ):
        return None
    return report
