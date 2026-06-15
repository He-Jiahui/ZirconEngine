"""Final report aggregation for the Zircon export pipeline."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

REPORT_FILE_NAME = "report.json"
REPORT_STAGE_NAME = "report"
LIBRARY_EMBED_REPORT_STAGES = (
    "compile_host",
    "cook_assets",
    "pack",
    "platform_bundle",
)
NATIVE_DYNAMIC_BUNDLE_REPORT_STAGES = (
    "native_dynamic",
    *LIBRARY_EMBED_REPORT_STAGES,
)
DEFAULT_REPORT_OUTPUT_STAGES = (
    "validate",
    *LIBRARY_EMBED_REPORT_STAGES,
)


def run_report(args: argparse.Namespace) -> int:
    out_root = resolve_user_path(args.out)
    stage_dir = out_root / "stages" / REPORT_STAGE_NAME
    report_path = stage_dir / REPORT_FILE_NAME
    pipeline_report_path = out_root / REPORT_FILE_NAME

    print(f"zircon_export stage=Report profile={args.profile}")
    print(f"pipeline_report={pipeline_report_path}")
    print(f"report={report_path}")
    if args.dry_run:
        return 0

    stage_dir.mkdir(parents=True, exist_ok=True)
    report = build_pipeline_report(out_root, args.profile)
    encoded_report = json.dumps(report, indent=2)
    report_path.write_text(encoded_report, encoding="utf-8")
    pipeline_report_path.write_text(encoded_report, encoding="utf-8")
    print(encoded_report)
    return 2 if report["fatal"] else 0


def build_pipeline_report(out_root: Path, profile: str) -> dict[str, Any]:
    diagnostics: list[str] = []
    stage_reports: list[dict[str, Any]] = []

    validate_report = load_stage_report(
        "validate",
        stage_report_path(out_root, "validate"),
        profile,
        diagnostics,
    )
    if validate_report is not None:
        stage_reports.append(validate_report)

    validated_stage_requirements = None
    if validate_report is not None and validate_report.get("fatal") is not True:
        validated_stage_requirements = validate_report["report"]
    required_stages = report_required_stage_keys(validated_stage_requirements)
    for stage in required_stages:
        if stage == "validate":
            continue
        report_path = stage_report_path(out_root, stage)
        report = load_stage_report(stage, report_path, profile, diagnostics)
        if report is not None:
            stage_reports.append(report)

    fatal_stages = [
        report["stage"]
        for report in stage_reports
        if report.get("fatal") is True
    ]
    if fatal_stages:
        diagnostics.append(
            "pipeline contains fatal stage reports: "
            + ", ".join(fatal_stages)
        )
    diagnostics.extend(delta_verification_diagnostics(stage_reports))

    missing_stages = [
        stage
        for stage in required_stages
        if stage not in {report["stage_key"] for report in stage_reports}
    ]

    report = {
        "stage": "Report",
        "profile": profile,
        "fatal": bool(diagnostics),
        "diagnostics": diagnostics,
        "out": str(out_root),
        "missing_stages": missing_stages,
        "fatal_stages": fatal_stages,
        "stages": stage_reports,
    }
    native_plugins_payload = pipeline_native_plugins_payload(stage_reports)
    if native_plugins_payload is not None:
        report["native_plugins_payload"] = native_plugins_payload
    return report


def report_required_stage_keys(validate_report: dict[str, Any] | None) -> tuple[str, ...]:
    strategies = export_strategies_from_validate_report(validate_report)
    if not strategies:
        return DEFAULT_REPORT_OUTPUT_STAGES
    return ("validate", *pipeline_execution_stage_keys(strategies))


def pipeline_execution_stage_keys(strategies: set[str]) -> tuple[str, ...]:
    stages: list[str] = []
    if "source_template" in strategies:
        stages.append("source_template")
    if "native_dynamic" in strategies:
        stages.extend(NATIVE_DYNAMIC_BUNDLE_REPORT_STAGES)
    if "library_embed" in strategies:
        stages.extend(LIBRARY_EMBED_REPORT_STAGES)
    return tuple(dedupe(stages))


def export_strategies_from_validate_report(
    validate_report: dict[str, Any] | None,
) -> set[str]:
    if not isinstance(validate_report, dict):
        return set()
    profile_summary = validate_report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return set()
    strategies = profile_summary.get("strategies")
    if not isinstance(strategies, list):
        return set()
    normalized = {
        normalized_strategy
        for strategy in strategies
        if (normalized_strategy := normalize_export_strategy(strategy))
    }
    return normalized


def normalize_export_strategy(value: object) -> str | None:
    if not isinstance(value, str):
        return None
    normalized = value.strip().replace("-", "_").replace(" ", "_")
    aliases = {
        "SourceTemplate": "source_template",
        "LibraryEmbed": "library_embed",
        "NativeDynamic": "native_dynamic",
    }
    if normalized in aliases:
        return aliases[normalized]
    lowered = normalized.lower()
    if lowered in {"source_template", "library_embed", "native_dynamic"}:
        return lowered
    return None


def dedupe(values: list[str]) -> list[str]:
    deduped: list[str] = []
    for value in values:
        if value not in deduped:
            deduped.append(value)
    return deduped


def delta_verification_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "pack":
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        if report.get("delta_pack") and report.get("delta_apply_verified") is not True:
            diagnostics.append(
                "pack report delta_pack is present but delta_apply_verified is not true"
            )
    return diagnostics


def pipeline_native_plugins_payload(
    stage_reports: list[dict[str, Any]],
) -> dict[str, Any] | None:
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "platform_bundle":
            continue
        if stage_report.get("fatal") is True:
            return None
        report = stage_report.get("report")
        if not isinstance(report, dict):
            return None
        native_plugins_payload = report.get("native_plugins_payload")
        if isinstance(native_plugins_payload, dict):
            return native_plugins_payload
        return None
    return None


def load_stage_report(
    stage_key: str,
    report_path: Path,
    profile: str,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if not report_path.exists():
        diagnostics.append(f"{stage_key} report {report_path} does not exist")
        return None
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        diagnostics.append(f"{stage_key} report {report_path} is not valid JSON: {error}")
        return None

    if not isinstance(report, dict):
        diagnostics.append(f"{stage_key} report {report_path} must be a JSON object")
        return None

    report_profile = report.get("profile")
    profile_mismatch = report_profile is not None and report_profile != profile
    if profile_mismatch:
        diagnostics.append(
            f"{stage_key} report profile {report_profile} does not match requested profile {profile}"
        )

    return {
        "stage_key": stage_key,
        "stage": report.get("stage", stage_key),
        "path": str(report_path),
        "fatal": bool(report.get("fatal", False)) or profile_mismatch,
        "diagnostics": report.get("diagnostics", []),
        "report": report,
    }


def stage_report_path(out_root: Path, stage_key: str) -> Path:
    return out_root / "stages" / stage_key / REPORT_FILE_NAME


def resolve_user_path(path: str | Path) -> Path:
    return Path(path).expanduser().resolve()
