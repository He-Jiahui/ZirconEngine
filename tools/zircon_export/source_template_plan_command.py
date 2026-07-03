"""SourceTemplate Validate handoff, build-plan, and command helpers."""

from __future__ import annotations

import argparse
from pathlib import Path
from typing import Any

from .command_plan import command_option_value_diagnostic, command_with_option
from .pipeline_report_source_template_validate_schema import (
    source_template_validate_build_plan_schema_diagnostics,
)
from .source_template_generated_project import resolve_project_child
from .source_template_paths import resolve_source_template_optional_path
from .stage_handoff import (
    load_stage_report_with_diagnostics,
    stage_report_metadata_diagnostic,
)
from .stage_handoff_strategy import (
    export_strategies_from_validate_report,
    export_strategy_diagnostics,
)


SOURCE_TEMPLATE_STAGE = "source_template"
SOURCE_TEMPLATE_COMMAND_FORBIDDEN_TARGET_TRIPLE_FLAGS = ("--target",)
SOURCE_TEMPLATE_CARGO_PROFILES = {"debug", "release"}


def source_template_command(
    args: argparse.Namespace,
    project_dir: Path,
    source_plan: dict[str, Any],
    diagnostics: list[str] | None = None,
) -> list[str]:
    command_diagnostics: list[str] = diagnostics if diagnostics is not None else []
    command = list(source_plan["command"])
    if command:
        command[0] = args.cargo
    manifest_path = resolve_project_child(
        project_dir,
        str(source_plan["manifest_path"]),
        command_diagnostics,
        kind="SourceTemplate build plan manifest_path",
    )
    if manifest_path is None:
        return []
    stage_dir = project_dir.parent
    target_dir = (
        resolve_source_template_optional_path(
            args.target_dir,
            "target_dir",
            command_diagnostics,
        )
        if args.target_dir
        else stage_dir / "target"
    )
    if target_dir is None:
        return []
    command = command_with_option(command, "--manifest-path", str(manifest_path))
    command = command_with_option(command, "--target-dir", str(target_dir))
    if not args.no_locked and "--locked" not in command:
        command.append("--locked")
    if args.offline and "--offline" not in command:
        command.append("--offline")
    return command


def load_validate_report(
    validate_report: Path,
    profile: str,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if not validate_report.exists():
        diagnostics.append(f"validate report {validate_report} does not exist")
        return None
    report = load_stage_report_with_diagnostics(validate_report, "validate", diagnostics)
    if report is None:
        return None
    metadata_diagnostic = stage_report_metadata_diagnostic(report, "validate", profile)
    if metadata_diagnostic:
        diagnostics.append(metadata_diagnostic)
        return None
    if report.get("fatal"):
        diagnostics.append("validate report is fatal; SourceTemplate will not materialize")
        return None
    strategy_diagnostics = export_strategy_diagnostics(report)
    if strategy_diagnostics:
        diagnostics.extend(strategy_diagnostics)
        return None
    if validate_report_requires_strategy(report, SOURCE_TEMPLATE_STAGE):
        diagnostics.append("SourceTemplate stage requires the source_template strategy")
        return None
    return report


def validate_report_requires_strategy(report: dict[str, Any], strategy: str) -> bool:
    profile_summary = report.get("profile_summary")
    if not isinstance(profile_summary, dict) or "strategies" not in profile_summary:
        return False
    strategies = profile_summary.get("strategies")
    if not isinstance(strategies, list):
        return False
    return strategy not in export_strategies_from_validate_report(report)


def source_template_plan(
    validate_payload: dict[str, Any] | None,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if validate_payload is None:
        return None
    plan_summary = validate_payload.get("plan_summary")
    if not isinstance(plan_summary, dict):
        diagnostics.append("validate report does not contain plan_summary")
        return None
    source_plan = plan_summary.get("source_template_build")
    schema_diagnostics = source_template_validate_build_plan_schema_diagnostics(
        source_plan
    )
    if schema_diagnostics:
        diagnostics.extend(schema_diagnostics)
        if not isinstance(source_plan, dict):
            diagnostics.append(
                "validate report does not contain a SourceTemplate build plan"
            )
        elif not source_template_command_array_is_valid(source_plan.get("command")):
            if not source_template_command_array_has_entry_type_errors(
                source_plan.get("command")
            ):
                diagnostics.append(
                    "SourceTemplate build plan command must be a non-empty string array"
                )
        return None
    command = source_plan.get("command")
    if not source_template_command_array_is_valid(command):
        diagnostics.append(
            "SourceTemplate build plan command must be a non-empty string array"
        )
        return None
    if not source_template_command_runs_cargo_build(command):
        diagnostics.append("SourceTemplate build plan command must run cargo build")
        return None
    target_triple_diagnostics = source_template_command_forbidden_target_triple_diagnostics(
        command,
        label="SourceTemplate build plan command",
    )
    if target_triple_diagnostics:
        diagnostics.extend(target_triple_diagnostics)
        return None
    for option in ("--manifest-path", "--target-dir"):
        option_diagnostic = command_option_value_diagnostic(
            command,
            option,
            "SourceTemplate build plan command",
        )
        if option_diagnostic:
            diagnostics.append(option_diagnostic)
            return None
    profile_diagnostics = source_template_build_plan_profile_diagnostics(
        source_plan,
        label="SourceTemplate build plan",
    )
    if profile_diagnostics:
        diagnostics.extend(profile_diagnostics)
        return None
    manifest_path = source_plan.get("manifest_path")
    if not isinstance(manifest_path, str) or not manifest_path:
        diagnostics.append(
            "SourceTemplate build plan manifest_path must be a non-empty string"
        )
        return None
    target_dir = source_plan.get("target_dir")
    if not isinstance(target_dir, str) or not target_dir:
        diagnostics.append(
            "SourceTemplate build plan target_dir must be a non-empty string"
        )
        return None
    return source_plan


def source_template_command_array_is_valid(value: object) -> bool:
    return (
        isinstance(value, list)
        and bool(value)
        and all(isinstance(item, str) and item.strip() for item in value)
    )


def source_template_command_array_has_entry_type_errors(value: object) -> bool:
    return isinstance(value, list) and any(not isinstance(item, str) for item in value)


def source_template_command_runs_cargo_build(command: list[str]) -> bool:
    if len(command) < 2:
        return False
    cargo_binary = Path(command[0]).name.lower()
    return cargo_binary in {"cargo", "cargo.exe"} and command[1] == "build"


def source_template_build_plan_profile_diagnostics(
    source_plan: dict[str, Any],
    *,
    label: str,
) -> list[str]:
    diagnostics: list[str] = []
    cargo_profile = source_plan.get("cargo_profile")
    release = source_plan.get("release")
    if not source_template_trimmed_non_empty_string_is_schema_clean(cargo_profile):
        return diagnostics
    if cargo_profile not in SOURCE_TEMPLATE_CARGO_PROFILES:
        diagnostics.append(f"{label} cargo_profile must be debug or release")
        return diagnostics
    if type(release) is not bool:
        return diagnostics
    if release != (cargo_profile == "release"):
        diagnostics.append(f"{label} release must match cargo_profile")
        return diagnostics
    command = source_plan.get("command")
    if source_template_command_array_is_valid(command):
        diagnostics.extend(
            source_template_command_release_flag_diagnostics(
                command,
                release,
                label=f"{label} command",
            )
        )
    return diagnostics


def source_template_build_plan_expected_release(
    source_plan: dict[str, Any],
) -> bool | None:
    cargo_profile = source_plan.get("cargo_profile")
    release = source_plan.get("release")
    if (
        source_template_trimmed_non_empty_string_is_schema_clean(cargo_profile)
        and cargo_profile in SOURCE_TEMPLATE_CARGO_PROFILES
        and type(release) is bool
        and release == (cargo_profile == "release")
    ):
        return release
    return None


def source_template_command_release_flag_diagnostics(
    command: list[str],
    release: bool,
    *,
    label: str,
) -> list[str]:
    has_release_flag = "--release" in command
    if release and not has_release_flag:
        return [f"{label} must include --release for release profile"]
    if not release and has_release_flag:
        return [f"{label} must not include --release for debug profile"]
    return []


def source_template_trimmed_non_empty_string_is_schema_clean(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value == value.strip()


def source_template_command_forbidden_target_triple_diagnostics(
    command: list[str],
    *,
    label: str,
) -> list[str]:
    return source_template_command_forbidden_flags_diagnostics(
        command,
        SOURCE_TEMPLATE_COMMAND_FORBIDDEN_TARGET_TRIPLE_FLAGS,
        label=label,
        reason="because export target descriptor owns platform target selection",
    )


def source_template_command_forbidden_flags_diagnostics(
    command: list[str],
    flags: tuple[str, ...],
    *,
    label: str,
    reason: str,
) -> list[str]:
    diagnostics: list[str] = []
    for flag in flags:
        diagnostics.extend(
            source_template_command_forbidden_flag_diagnostics(
                command,
                flag,
                label=label,
                reason=reason,
            )
        )
    return diagnostics


def source_template_command_forbidden_flag_diagnostics(
    command: list[str],
    flag: str,
    *,
    label: str,
    reason: str,
) -> list[str]:
    prefix = f"{flag}="
    if any(entry == flag or entry.startswith(prefix) for entry in command):
        return [f"{label} must not include {flag} {reason}"]
    return []
