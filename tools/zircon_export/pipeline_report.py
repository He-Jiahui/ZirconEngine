"""Final report aggregation for the Zircon export pipeline."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from .pipeline_report_stage_schema import stage_report_schema_diagnostics
from .pipeline_report_stage_location import (
    native_dynamic_loader_manifest_location_diagnostics,
    native_dynamic_plugins_dir_location_diagnostics,
    stage_output_location_diagnostics,
)
from .pipeline_report_cook_assets import (
    cook_assets_manifest_asset_filter_diagnostics,
    cook_assets_manifest_count_diagnostics,
    cook_assets_manifest_determinism_diagnostics,
    cook_assets_manifest_hash_diagnostics,
    cook_assets_manifest_shape_diagnostics,
    cook_assets_manifest_source_diagnostics,
    cook_assets_manifest_stage_location_diagnostics,
)
from .pipeline_report_cook_assets_pack_handoff import (
    cook_assets_pack_manifest_handoff_diagnostics,
)
from .pipeline_report_cook_assets_pack_trim_closure import (
    cook_assets_pack_trim_closure_diagnostics,
)
from .pipeline_report_platform_bundle import (
    platform_bundle_manifest_diagnostics,
)
from .pipeline_report_platform_bundle_stage_handoff import (
    delta_verification_diagnostics,
    platform_bundle_delta_diagnostics,
    platform_bundle_host_diagnostics,
    platform_bundle_pack_diagnostics,
)
from .pipeline_report_native_dynamic_payload import pipeline_native_plugins_payload
from .pipeline_report_native_dynamic_stage_payload import (
    native_dynamic_stage_payload_diagnostics,
)
from .pipeline_report_source_template import source_template_project_diagnostics
from .pipeline_report_compile_host import (
    compile_host_command_diagnostics,
    compile_host_host_executable_diagnostics,
    compile_host_link_plan_diagnostics,
)
from .report_io import write_report_targets
from .stage_handoff import (
    dedupe,
    stage_report_diagnostics_diagnostic,
    stage_report_fatal_diagnostic,
    stage_report_identity_diagnostic,
    stage_report_label,
)
from .stage_handoff_strategy import (
    export_strategy_diagnostics,
    export_strategy_list_is_empty,
    export_strategy_list_is_invalid,
    export_strategies_from_validate_report,
    native_dynamic_payload_allowed,
    unsupported_export_strategies_from_validate_report,
)

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

    report = build_pipeline_report(out_root, args.profile)
    try:
        stage_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        report["fatal"] = True
        diagnostics = report.get("diagnostics")
        if not isinstance(diagnostics, list):
            diagnostics = []
            report["diagnostics"] = diagnostics
        diagnostics.append(
            f"Report stage directory {stage_dir} could not be created: {error}"
        )
        write_report_targets([("pipeline report", pipeline_report_path)], report)
        print(json.dumps(report, indent=2))
        return 2
    report_written = write_report_targets(
        [
            ("Report stage report", report_path),
            ("pipeline report", pipeline_report_path),
        ],
        report,
    )
    print(json.dumps(report, indent=2))
    return 2 if report["fatal"] or not report_written else 0


def build_pipeline_report(out_root: Path, profile: str) -> dict[str, Any]:
    diagnostics: list[str] = []
    stage_reports: list[dict[str, Any]] = []

    validate_report_path = stage_report_path(out_root, "validate")
    validate_report = load_stage_report(
        "validate",
        validate_report_path,
        profile,
        diagnostics,
        validate_payload=None,
    )
    if validate_report is not None:
        stage_reports.append(validate_report)

    validated_stage_requirements = None
    if validate_report is not None and validate_report.get("fatal") is not True:
        validated_stage_requirements = validate_report["report"]
        diagnostics.extend(export_strategy_diagnostics(validated_stage_requirements))
    if validate_report is not None and validate_report.get("fatal") is True:
        required_stages = ("validate",)
    else:
        required_stages = report_required_stage_keys(validated_stage_requirements)
    for stage in required_stages:
        if stage == "validate":
            continue
        report_path = stage_report_path(out_root, stage)
        report = load_stage_report(
            stage,
            report_path,
            profile,
            diagnostics,
            validate_payload=validated_stage_requirements,
        )
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
    diagnostics.extend(platform_bundle_host_diagnostics(stage_reports))
    diagnostics.extend(platform_bundle_pack_diagnostics(stage_reports))
    diagnostics.extend(delta_verification_diagnostics(stage_reports))
    diagnostics.extend(platform_bundle_delta_diagnostics(stage_reports))
    diagnostics.extend(cook_assets_manifest_hash_diagnostics(stage_reports))
    diagnostics.extend(cook_assets_manifest_stage_location_diagnostics(stage_reports))
    diagnostics.extend(cook_assets_manifest_count_diagnostics(stage_reports))
    diagnostics.extend(cook_assets_manifest_shape_diagnostics(stage_reports))
    diagnostics.extend(cook_assets_manifest_determinism_diagnostics(stage_reports))
    diagnostics.extend(cook_assets_manifest_source_diagnostics(stage_reports))
    diagnostics.extend(cook_assets_manifest_asset_filter_diagnostics(stage_reports))
    diagnostics.extend(cook_assets_pack_manifest_handoff_diagnostics(stage_reports))
    diagnostics.extend(cook_assets_pack_trim_closure_diagnostics(stage_reports))
    diagnostics.extend(
        platform_bundle_manifest_diagnostics(
            stage_reports,
            native_dynamic_payload_allowed=native_dynamic_payload_allowed(
                validated_stage_requirements
            ),
        )
    )
    diagnostics.extend(
        source_template_project_diagnostics(
            stage_reports,
            validated_stage_requirements,
            validate_report_path,
        )
    )
    diagnostics.extend(compile_host_link_plan_diagnostics(stage_reports))
    diagnostics.extend(compile_host_command_diagnostics(stage_reports, out_root))
    diagnostics.extend(compile_host_host_executable_diagnostics(stage_reports, out_root))

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
        "export_plan": pipeline_export_plan(
            profile,
            validate_report,
            validated_stage_requirements,
            required_stages,
            stage_reports,
        ),
        "missing_stages": missing_stages,
        "fatal_stages": fatal_stages,
        "stages": stage_reports,
    }
    native_plugins_payload = pipeline_native_plugins_payload(stage_reports)
    if native_plugins_payload is not None and not report["fatal"]:
        report["native_plugins_payload"] = native_plugins_payload
    return report


def report_required_stage_keys(validate_report: dict[str, Any] | None) -> tuple[str, ...]:
    if export_strategy_list_is_invalid(validate_report):
        return ("validate",)
    if export_strategy_list_is_empty(validate_report):
        return ("validate",)
    if unsupported_export_strategies_from_validate_report(validate_report):
        return ("validate",)
    strategies = export_strategies_from_validate_report(validate_report)
    if not strategies:
        return DEFAULT_REPORT_OUTPUT_STAGES
    return ("validate", *pipeline_execution_stage_keys(strategies))


def pipeline_export_plan(
    profile: str,
    validate_stage_report: dict[str, Any] | None,
    accepted_validate_report: dict[str, Any] | None,
    required_stages: tuple[str, ...],
    stage_reports: list[dict[str, Any]],
) -> dict[str, Any]:
    unsupported_validate_report = (
        validate_stage_report["report"]
        if (
            isinstance(validate_stage_report, dict)
            and isinstance(validate_stage_report.get("report"), dict)
            and validate_stage_report["report"].get("profile") == profile
        )
        else None
    )
    return {
        "strategies": sorted(
            export_strategies_from_validate_report(accepted_validate_report)
        ),
        "required_stages": list(required_stages),
        "completed_stages": [report["stage_key"] for report in stage_reports],
        "unsupported_strategies": unsupported_export_strategies_from_validate_report(
            unsupported_validate_report
        ),
    }


def pipeline_execution_stage_keys(strategies: set[str]) -> tuple[str, ...]:
    stages: list[str] = []
    if "source_template" in strategies:
        stages.append("source_template")
    if "native_dynamic" in strategies:
        stages.extend(NATIVE_DYNAMIC_BUNDLE_REPORT_STAGES)
    if "library_embed" in strategies:
        stages.extend(LIBRARY_EMBED_REPORT_STAGES)
    return tuple(dedupe(stages))


def load_stage_report(
    stage_key: str,
    report_path: Path,
    profile: str,
    diagnostics: list[str],
    *,
    validate_payload: dict[str, Any] | None = None,
) -> dict[str, Any] | None:
    if not report_path.exists():
        diagnostics.append(f"{stage_key} report {report_path} does not exist")
        return None
    if not report_path.is_file():
        diagnostics.append(f"{stage_key} report {report_path} is not a file")
        return None
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except OSError as error:
        diagnostics.append(f"{stage_key} report {report_path} could not be read: {error}")
        return None
    except json.JSONDecodeError as error:
        diagnostics.append(f"{stage_key} report {report_path} is not valid JSON: {error}")
        return None

    if not isinstance(report, dict):
        diagnostics.append(f"{stage_key} report {report_path} must be a JSON object")
        return None

    report_profile = report.get("profile")
    profile_missing = not isinstance(report_profile, str)
    profile_mismatch = isinstance(report_profile, str) and report_profile != profile
    stage_identity_diagnostic = stage_report_identity_diagnostic(
        report,
        stage_key,
        label=stage_key,
    )
    stage_identity_mismatch = stage_identity_diagnostic is not None
    if stage_identity_diagnostic:
        diagnostics.append(stage_identity_diagnostic)
    fatal_diagnostic = stage_report_fatal_diagnostic(
        report,
        stage_key,
        label=stage_key,
    )
    fatal_invalid = fatal_diagnostic is not None
    if fatal_diagnostic:
        diagnostics.append(fatal_diagnostic)
    diagnostics_diagnostic = stage_report_diagnostics_diagnostic(
        report,
        stage_key,
        label=stage_key,
    )
    diagnostics_invalid = diagnostics_diagnostic is not None
    if diagnostics_diagnostic:
        diagnostics.append(diagnostics_diagnostic)
    schema_diagnostics = stage_report_schema_diagnostics(stage_key, report)
    location_diagnostics = stage_output_location_diagnostics(
        stage_key,
        report,
        report_path,
    )
    location_diagnostics.extend(
        native_dynamic_plugins_dir_location_diagnostics(
            stage_key,
            report,
            report_path,
        )
    )
    location_diagnostics.extend(
        native_dynamic_loader_manifest_location_diagnostics(
            stage_key,
            report,
            report_path,
        )
    )
    location_diagnostics.extend(
        native_dynamic_stage_payload_diagnostics(
            stage_key,
            report,
            report_path,
            validate_payload=validate_payload,
        )
    )
    schema_invalid = bool(schema_diagnostics)
    location_invalid = bool(location_diagnostics)
    diagnostics.extend(schema_diagnostics)
    diagnostics.extend(location_diagnostics)
    if profile_missing:
        diagnostics.append(f"{stage_key} report profile is missing or invalid")
    if profile_mismatch:
        diagnostics.append(
            f"{stage_key} report profile {report_profile} does not match requested profile {profile}"
        )

    return {
        "stage_key": stage_key,
        "stage": stage_report_label(stage_key) if stage_identity_mismatch else report.get("stage", stage_key),
        "path": str(report_path),
        "fatal": (
            (report.get("fatal") is True)
            or fatal_invalid
            or diagnostics_invalid
            or schema_invalid
            or location_invalid
            or profile_missing
            or profile_mismatch
            or stage_identity_mismatch
        ),
        "diagnostics": report["diagnostics"] if not diagnostics_invalid else [],
        "report": report,
    }


def stage_report_path(out_root: Path, stage_key: str) -> Path:
    return out_root / "stages" / stage_key / REPORT_FILE_NAME


def resolve_user_path(path: str | Path) -> Path:
    return Path(path).expanduser().resolve()
