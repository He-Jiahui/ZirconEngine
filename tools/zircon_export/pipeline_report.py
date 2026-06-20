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
    cook_assets_pack_manifest_handoff_diagnostics,
    cook_assets_pack_trim_closure_diagnostics,
)
from .pipeline_report_platform_bundle import (
    delta_verification_diagnostics,
    platform_bundle_delta_diagnostics,
    platform_bundle_host_diagnostics,
    platform_bundle_manifest_diagnostics,
    platform_bundle_pack_diagnostics,
)
from .pipeline_report_native_dynamic_payload import pipeline_native_plugins_payload
from .pipeline_report_native_dynamic_stage_payload import (
    native_dynamic_stage_payload_diagnostics,
)
from .pipeline_report_source_template import source_template_project_diagnostics
from .report_io import write_report_targets
from .stage_handoff import (
    dedupe,
    export_strategy_diagnostics,
    export_strategy_list_is_empty,
    export_strategy_list_is_invalid,
    export_strategies_from_validate_report,
    native_dynamic_payload_allowed,
    stage_report_diagnostics_diagnostic,
    stage_report_fatal_diagnostic,
    stage_report_identity_diagnostic,
    stage_report_label,
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
COMPILE_HOST_LINK_PLAN_FIELDS = (
    "app_features",
    "runtime_features",
    "expected_runtime_plugins",
    "linked_runtime_crates",
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


def compile_host_link_plan_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    validate_plan = validate_library_embed_compile_host_plan(stage_reports)
    compile_host_link_plan = compile_host_stage_link_plan(stage_reports)
    if not isinstance(validate_plan, dict) or not isinstance(
        compile_host_link_plan,
        dict,
    ):
        return []

    diagnostics: list[str] = []
    for field in COMPILE_HOST_LINK_PLAN_FIELDS:
        expected = validate_plan.get(field)
        actual = compile_host_link_plan.get(field)
        if (
            isinstance(expected, list)
            and isinstance(actual, list)
            and actual != expected
        ):
            diagnostics.append(
                f"compile_host report link_plan.{field} does not match "
                f"validate report plan_summary.library_embed_compile_host.{field}"
            )
    return diagnostics


def compile_host_command_diagnostics(
    stage_reports: list[dict[str, Any]],
    out_root: Path,
) -> list[str]:
    validate_plan = validate_library_embed_compile_host_plan(stage_reports)
    compile_host_report = stage_report_payload(stage_reports, "compile_host")
    if not isinstance(validate_plan, dict) or not isinstance(
        compile_host_report,
        dict,
    ):
        return []

    command = compile_host_report.get("command")
    if not command_is_string_list(command):
        return []
    assert isinstance(command, list)

    label = "compile_host report command"
    validate_label = "validate report plan_summary.library_embed_compile_host"
    diagnostics: list[str] = []
    diagnostics.extend(
        compile_host_command_alias_match_diagnostics(
            command,
            ("-p", "--package"),
            "-p/--package",
            validate_plan.get("package"),
            f"{validate_label}.package",
            label=label,
        )
    )
    diagnostics.extend(
        compile_host_command_option_match_diagnostics(
            command,
            "--bin",
            validate_plan.get("binary"),
            f"{validate_label}.binary",
            label=label,
        )
    )
    diagnostics.extend(
        compile_host_command_target_dir_match_diagnostics(
            command,
            validate_plan.get("target_dir"),
            f"{validate_label}.target_dir",
            out_root,
            label=label,
        )
    )
    diagnostics.extend(
        compile_host_command_features_match_diagnostics(
            command,
            validate_plan.get("app_features"),
            f"{validate_label}.app_features",
            label=label,
        )
    )
    diagnostics.extend(
        compile_host_command_release_flag_diagnostics(
            command,
            validate_plan,
            label=label,
        )
    )
    return diagnostics


def compile_host_command_alias_match_diagnostics(
    command: list[str],
    options: tuple[str, ...],
    option_label: str,
    expected_value: object,
    expected_label: str,
    *,
    label: str,
) -> list[str]:
    actual_value = command_alias_value(command, options)
    if isinstance(expected_value, str) and actual_value is not None:
        if actual_value != expected_value:
            return [
                f"{label} {option_label} does not match {expected_label}",
            ]
    return []


def compile_host_command_option_match_diagnostics(
    command: list[str],
    option: str,
    expected_value: object,
    expected_label: str,
    *,
    label: str,
) -> list[str]:
    actual_value = command_option_value(command, option)
    if isinstance(expected_value, str) and actual_value is not None:
        if actual_value != expected_value:
            return [f"{label} {option} does not match {expected_label}"]
    return []


def compile_host_command_target_dir_match_diagnostics(
    command: list[str],
    expected_value: object,
    expected_label: str,
    out_root: Path,
    *,
    label: str,
) -> list[str]:
    actual_value = command_option_value(command, "--target-dir")
    if isinstance(expected_value, str) and actual_value is not None:
        if not command_target_dir_matches_out_root(
            actual_value,
            expected_value,
            out_root,
        ):
            return [f"{label} --target-dir does not match {expected_label}"]
    return []


def command_target_dir_matches_out_root(
    actual_value: str,
    expected_value: str,
    out_root: Path,
) -> bool:
    actual = normalized_path_token(actual_value)
    expected = normalized_path_token(expected_value)
    if actual == expected:
        return True

    actual_path = Path(actual_value)
    if not actual_path.is_absolute():
        return False

    try:
        relative_actual = actual_path.resolve().relative_to(out_root.resolve())
    except (OSError, ValueError):
        return False
    return normalized_path_token(str(relative_actual)) == expected


def normalized_path_token(value: str) -> str:
    return value.strip().replace("\\", "/")


def compile_host_command_features_match_diagnostics(
    command: list[str],
    expected_features: object,
    expected_label: str,
    *,
    label: str,
) -> list[str]:
    actual_features = command_option_value(command, "--features")
    if actual_features is None:
        return []
    if not (
        isinstance(expected_features, list)
        and all(
            isinstance(feature, str) and feature.strip()
            for feature in expected_features
        )
    ):
        return []
    expected = [feature.strip() for feature in expected_features]
    if cargo_feature_list(actual_features) != expected:
        return [f"{label} --features does not match {expected_label}"]
    return []


def compile_host_command_release_flag_diagnostics(
    command: list[str],
    validate_plan: dict[str, Any],
    *,
    label: str,
) -> list[str]:
    release = validate_plan.get("release")
    cargo_profile = validate_plan.get("cargo_profile")
    has_release_flag = "--release" in command
    if release is True or cargo_profile == "release":
        if not has_release_flag:
            return [f"{label} must include --release for release profile"]
    if release is False and cargo_profile == "debug" and has_release_flag:
        return [f"{label} must not include --release for debug profile"]
    return []


def command_is_string_list(value: object) -> bool:
    return isinstance(value, list) and all(isinstance(entry, str) for entry in value)


def command_alias_value(command: list[str], options: tuple[str, ...]) -> str | None:
    values = [
        value
        for option in options
        for value in [command_option_value(command, option)]
        if value is not None
    ]
    if len(values) == 1:
        return values[0]
    return None


def command_option_value(command: list[str], option: str) -> str | None:
    for index, entry in enumerate(command):
        if entry == option and index + 1 < len(command):
            return command[index + 1]
    return None


def cargo_feature_list(value: str) -> list[str]:
    return [feature for feature in value.replace(",", " ").split() if feature]


def compile_host_host_executable_diagnostics(
    stage_reports: list[dict[str, Any]],
    out_root: Path,
) -> list[str]:
    compile_host_report = stage_report_payload(stage_reports, "compile_host")
    if not isinstance(compile_host_report, dict):
        return []
    if compile_host_report.get("fatal") is not False:
        return []

    host_executable = compile_host_report.get("host_executable")
    if not isinstance(host_executable, str) or not host_executable.strip():
        return []

    try:
        resolved_host = Path(host_executable).expanduser().resolve()
        resolved_out_root = out_root.expanduser().resolve()
    except OSError as error:
        return [
            "compile_host report host_executable "
            f"{host_executable} could not be resolved: {error}"
        ]

    try:
        resolved_host.relative_to(resolved_out_root)
    except ValueError:
        return [
            "compile_host report host_executable "
            f"{resolved_host} is outside current output root {resolved_out_root}"
        ]
    if not resolved_host.exists():
        return [
            "compile_host report host_executable "
            f"{resolved_host} does not exist"
        ]
    if not resolved_host.is_file():
        return [
            "compile_host report host_executable "
            f"{resolved_host} is not a file"
        ]

    validate_plan = validate_library_embed_compile_host_plan(stage_reports)
    command = compile_host_report.get("command")
    if not isinstance(validate_plan, dict) or not command_is_string_list(command):
        return []
    assert isinstance(command, list)

    target_dir = command_option_value(command, "--target-dir")
    expected_target_dir = validate_plan.get("target_dir")
    cargo_profile = validate_plan.get("cargo_profile")
    binary = validate_plan.get("binary")
    if not (
        isinstance(target_dir, str)
        and Path(target_dir).is_absolute()
        and isinstance(expected_target_dir, str)
        and isinstance(cargo_profile, str)
        and cargo_profile.strip()
        and isinstance(binary, str)
        and binary.strip()
        and command_target_dir_matches_out_root(
            target_dir,
            expected_target_dir,
            out_root,
        )
    ):
        return []

    try:
        expected_profile_dir = Path(target_dir).expanduser().resolve() / cargo_profile
    except OSError as error:
        return [
            "compile_host report command --target-dir "
            f"{target_dir} could not be resolved: {error}"
        ]

    try:
        resolved_host.relative_to(expected_profile_dir)
    except ValueError:
        return [
            "compile_host report host_executable "
            f"{resolved_host} does not match command --target-dir profile "
            f"directory {expected_profile_dir}"
        ]
    if resolved_host.parent != expected_profile_dir:
        return [
            "compile_host report host_executable "
            f"{resolved_host} does not match command --target-dir profile "
            f"directory {expected_profile_dir}"
        ]

    expected_binary_names = {binary.strip(), f"{binary.strip()}.exe"}
    if resolved_host.name not in expected_binary_names:
        return [
            "compile_host report host_executable "
            f"{resolved_host} does not match validate report "
            "plan_summary.library_embed_compile_host.binary "
            f"{binary}"
        ]
    return []


def validate_library_embed_compile_host_plan(
    stage_reports: list[dict[str, Any]],
) -> object:
    validate_report = stage_report_payload(stage_reports, "validate")
    if not isinstance(validate_report, dict):
        return None
    plan_summary = validate_report.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return None
    return plan_summary.get("library_embed_compile_host")


def compile_host_stage_link_plan(stage_reports: list[dict[str, Any]]) -> object:
    compile_host_report = stage_report_payload(stage_reports, "compile_host")
    if not isinstance(compile_host_report, dict):
        return None
    return compile_host_report.get("link_plan")


def stage_report_payload(
    stage_reports: list[dict[str, Any]],
    stage_key: str,
) -> object:
    for stage_report in stage_reports:
        if stage_report.get("stage_key") == stage_key:
            return stage_report.get("report")
    return None


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
