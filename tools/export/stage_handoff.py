"""Typed stage-report handoff helpers for the Zircon export pipeline."""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

REPORT_FILE_NAME = "report.json"


def compile_host_report_host_executable(out_root: Path, profile: str) -> Path | None:
    return stage_report_path_field(out_root, "compile_host", profile, "host_executable")


def cook_assets_report_asset_manifest(out_root: Path, profile: str) -> Path | None:
    return stage_report_path_field(
        out_root,
        "cook_assets",
        profile,
        "cooked_asset_manifest",
    )


def pack_report_pack_file(out_root: Path, profile: str) -> Path | None:
    return stage_report_path_field(
        out_root,
        "pack",
        profile,
        "pack",
    )


def pack_report_delta_pack_file(out_root: Path, profile: str) -> Path | None:
    return stage_report_path_field(
        out_root,
        "pack",
        profile,
        "delta_pack",
    )


def native_dynamic_report_plugins_dir(out_root: Path, profile: str) -> Path | None:
    return stage_report_path_field(
        out_root,
        "native_dynamic",
        profile,
        "plugins_dir",
    )


def validate_report_asset_filter(out_root: Path, profile: str) -> str | None:
    report = matching_stage_report(out_root, "validate", profile)
    if report is None:
        return None
    profile_summary = report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return None
    asset_filter = profile_summary.get("asset_filter")
    if not isinstance(asset_filter, str) or not asset_filter:
        return None
    return asset_filter


def validate_report_asset_filter_diagnostic(
    out_root: Path,
    profile: str,
) -> str | None:
    report_path = stage_report_path(out_root, "validate")
    if not report_path.exists():
        return None
    handoff_diagnostic = stage_report_metadata_handoff_diagnostic(
        out_root,
        "validate",
        profile,
    )
    if handoff_diagnostic:
        return handoff_diagnostic
    report = matching_stage_report(out_root, "validate", profile)
    if report is None:
        return None
    profile_summary = report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return None
    if "asset_filter" not in profile_summary:
        return None
    asset_filter = profile_summary.get("asset_filter")
    if not isinstance(asset_filter, str) or not asset_filter:
        return (
            "Validate report field profile_summary.asset_filter must be a "
            "non-empty string"
        )
    return None


def dedupe(values: list[str]) -> list[str]:
    deduped: list[str] = []
    seen: set[str] = set()
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        deduped.append(value)
    return deduped


def stage_report_path_handoff_diagnostic(
    out_root: Path,
    stage: str,
    profile: str,
    field: str,
) -> str | None:
    metadata_diagnostic = stage_report_metadata_handoff_diagnostic(
        out_root,
        stage,
        profile,
    )
    if metadata_diagnostic:
        return metadata_diagnostic
    report = matching_stage_report(out_root, stage, profile)
    if report is None:
        return None
    value = report.get(field)
    if not isinstance(value, str) or not value.strip():
        stage_label = stage_report_label(stage)
        return f"{stage_label} report field {field} must be a non-empty string"
    return field_value_path_diagnostic(report, stage, field)


def stage_report_metadata_handoff_diagnostic(
    out_root: Path,
    stage: str,
    profile: str,
) -> str | None:
    report_path = stage_report_path(out_root, stage)
    if not report_path.exists():
        return None
    stage_label = stage_report_label(stage)
    report, diagnostic = load_stage_report_object(report_path, stage_label)
    if diagnostic:
        return diagnostic
    if report is None:
        return None
    return stage_report_metadata_diagnostic(report, stage, profile)


def load_stage_report_with_diagnostics(
    report_path: Path,
    stage_label: str,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if not report_path.exists():
        diagnostics.append(f"{stage_label} report {report_path} does not exist")
        return None
    report, diagnostic = load_stage_report_object(report_path, stage_label)
    if diagnostic:
        diagnostics.append(diagnostic)
        return None
    return report


def stage_report_metadata_diagnostic(
    report: dict[str, Any],
    stage: str,
    profile: str,
) -> str | None:
    stage_label = stage_report_label(stage)
    stage_identity_diagnostic = stage_report_identity_diagnostic(report, stage)
    if stage_identity_diagnostic:
        return stage_identity_diagnostic
    fatal_diagnostic = stage_report_fatal_diagnostic(report, stage)
    if fatal_diagnostic:
        return fatal_diagnostic
    diagnostics_diagnostic = stage_report_diagnostics_diagnostic(report, stage)
    if diagnostics_diagnostic:
        return diagnostics_diagnostic
    if report["fatal"]:
        return f"{stage_label} report is fatal; run {stage_label} again before PlatformBundle"
    report_profile = report.get("profile")
    if not isinstance(report_profile, str):
        return f"{stage_label} report profile is missing or invalid"
    if report_profile != profile:
        return (
            f"{stage_label} report profile {report_profile} does not match "
            f"requested profile {profile}"
        )
    return None


def stage_report_optional_path_handoff_diagnostic(
    out_root: Path,
    stage: str,
    profile: str,
    field: str,
) -> str | None:
    report = matching_stage_report(out_root, stage, profile)
    if report is None or field not in report:
        return None
    value = report.get(field)
    if not isinstance(value, str) or not value.strip():
        stage_label = stage_report_label(stage)
        return f"{stage_label} report field {field} must be a non-empty string"
    return field_value_path_diagnostic(report, stage, field)


def stage_report_path_field(
    out_root: Path,
    stage: str,
    profile: str,
    field: str,
) -> Path | None:
    report = matching_stage_report(out_root, stage, profile)
    if report is None:
        return None
    return field_value_path(report, field)


def matching_stage_report(
    out_root: Path,
    stage: str,
    profile: str,
) -> dict[str, Any] | None:
    report_path = stage_report_path(out_root, stage)
    if not report_path.exists():
        return None
    report, diagnostic = load_stage_report_object(
        report_path,
        stage_report_label(stage),
    )
    if diagnostic or report is None:
        return None
    if stage_report_identity_diagnostic(report, stage):
        return None
    if stage_report_fatal_diagnostic(report, stage):
        return None
    if stage_report_diagnostics_diagnostic(report, stage):
        return None
    if report["fatal"]:
        return None
    report_profile = report.get("profile")
    if not isinstance(report_profile, str) or report_profile != profile:
        return None
    return report


def load_stage_report_object(
    report_path: Path,
    stage_label: str,
) -> tuple[dict[str, Any] | None, str | None]:
    if not report_path.is_file():
        return None, f"{stage_label} report {report_path} is not a file"
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except OSError as error:
        return None, f"{stage_label} report {report_path} could not be read: {error}"
    except json.JSONDecodeError:
        return None, f"{stage_label} report {report_path} is not valid JSON"
    if not isinstance(report, dict):
        return None, f"{stage_label} report {report_path} must be a JSON object"
    return report, None


def stage_report_path(out_root: Path, stage: str) -> Path:
    return out_root / "stages" / stage / REPORT_FILE_NAME


def stage_report_label(stage: str) -> str:
    return "".join(part.capitalize() for part in stage.split("_"))


def stage_report_identity_diagnostic(
    report: dict[str, Any],
    stage: str,
    *,
    label: str | None = None,
) -> str | None:
    expected_stage = stage_report_label(stage)
    stage_label = label or expected_stage
    report_stage = report.get("stage")
    if not isinstance(report_stage, str) or not report_stage:
        return f"{stage_label} report stage is missing or invalid"
    if normalized_stage_identity(report_stage) != normalized_stage_identity(expected_stage):
        return (
            f"{stage_label} report stage {report_stage} does not match "
            f"expected stage {expected_stage}"
        )
    return None


def stage_report_fatal_diagnostic(
    report: dict[str, Any],
    stage: str,
    *,
    label: str | None = None,
) -> str | None:
    stage_label = label or stage_report_label(stage)
    if not isinstance(report.get("fatal"), bool):
        return f"{stage_label} report fatal must be a boolean"
    return None


def stage_report_diagnostics_diagnostic(
    report: dict[str, Any],
    stage: str,
    *,
    label: str | None = None,
) -> str | None:
    stage_label = label or stage_report_label(stage)
    diagnostics = report.get("diagnostics")
    if not isinstance(diagnostics, list):
        return f"{stage_label} report diagnostics must be a string array"
    for index, diagnostic in enumerate(diagnostics):
        if not isinstance(diagnostic, str):
            return f"{stage_label} report diagnostics[{index}] must be a string"
    if any(not diagnostic.strip() for diagnostic in diagnostics):
        return f"{stage_label} report diagnostics must not contain blank entries"
    for index, diagnostic in enumerate(diagnostics):
        if diagnostic.strip() != diagnostic:
            return (
                f"{stage_label} report diagnostics[{index}] "
                "must be a non-empty trimmed string"
            )
    return None


def normalized_stage_identity(stage: str) -> str:
    return (
        stage.replace("_", "")
        .replace("-", "")
        .replace(" ", "")
        .lower()
    )


def field_value_path(report: dict[str, Any], field: str) -> Path | None:
    value = report.get(field)
    if not isinstance(value, str) or not value.strip():
        return None
    try:
        return resolve_user_path(value)
    except OSError:
        return None


def field_value_path_diagnostic(
    report: dict[str, Any],
    stage: str,
    field: str,
) -> str | None:
    value = report.get(field)
    if not isinstance(value, str) or not value.strip():
        stage_label = stage_report_label(stage)
        return f"{stage_label} report field {field} must be a non-empty string"
    try:
        resolve_user_path(value)
    except OSError as error:
        stage_label = stage_report_label(stage)
        return (
            f"{stage_label} report field {field} {value} "
            f"could not be resolved: {error}"
        )
    return None


def resolve_user_path(value: str | Path) -> Path:
    return Path(value).expanduser().resolve()
