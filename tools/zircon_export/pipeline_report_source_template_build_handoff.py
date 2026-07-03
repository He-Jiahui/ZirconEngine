"""SourceTemplate build handoff diagnostics for final export reports."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .command_plan import command_option_value_diagnostic
from .pipeline_report_source_template_path_semantics import (
    resolve_source_template_path_or_diagnostic,
    source_template_generated_file_path,
    source_template_is_non_empty_trimmed_string,
)
from .pipeline_report_source_template_build_status import (
    source_template_build_status_diagnostics,
)
from .pipeline_report_source_template_stage_schema import (
    source_template_build_validation_schema_diagnostics,
)
from .pipeline_report_source_template_string_array_schema import (
    source_template_non_empty_string_array_schema_diagnostics,
)
from .pipeline_report_source_template_validate_schema import (
    source_template_validate_build_plan_schema_diagnostics,
)
from .source_template_plan_command import (
    source_template_build_plan_expected_release,
    source_template_build_plan_profile_diagnostics,
    source_template_command_forbidden_target_triple_diagnostics,
    source_template_command_release_flag_diagnostics,
    source_template_command_runs_cargo_build,
)


def source_template_validate_build_plan_diagnostics(
    validate_report: dict[str, Any] | None,
    project_dir: Path,
) -> list[str]:
    if not isinstance(validate_report, dict):
        return ["SourceTemplate Validate plan_summary.source_template_build must be an object"]
    plan_summary = validate_report.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return ["SourceTemplate Validate plan_summary.source_template_build must be an object"]
    source_template_build = plan_summary.get("source_template_build")
    if not isinstance(source_template_build, dict):
        return ["SourceTemplate Validate plan_summary.source_template_build must be an object"]
    diagnostics: list[str] = []
    diagnostics.extend(
        source_template_validate_build_plan_schema_diagnostics(source_template_build)
    )
    command = source_template_build.get("command")
    command_diagnostics = source_template_non_empty_string_array_schema_diagnostics(
        "SourceTemplate Validate source_template_build command",
        command,
    )
    if command_diagnostics:
        diagnostics.extend(command_diagnostics)
    elif source_template_command_array_is_non_empty_trimmed(command):
        if not source_template_command_runs_cargo_build(command):
            diagnostics.append(
                "SourceTemplate Validate source_template_build command must run cargo build"
            )
        diagnostics.extend(
            source_template_command_forbidden_target_triple_diagnostics(
                command,
                label="SourceTemplate Validate source_template_build command",
            )
        )
        diagnostics.extend(
            source_template_build_plan_profile_diagnostics(
                source_template_build,
                label="SourceTemplate Validate source_template_build",
            )
        )
        for option in ("--manifest-path", "--target-dir"):
            option_diagnostic = command_option_value_diagnostic(
                command,
                option,
                "SourceTemplate Validate source_template_build command",
            )
            if option_diagnostic:
                diagnostics.append(option_diagnostic)
    manifest_path = source_template_build.get("manifest_path")
    if not isinstance(manifest_path, str) or not manifest_path.strip():
        diagnostics.append(
            "SourceTemplate Validate source_template_build manifest_path must be a non-empty string"
        )
    elif source_template_is_non_empty_trimmed_string(manifest_path):
        source_template_generated_file_path(
            project_dir,
            manifest_path,
            diagnostics,
            kind="SourceTemplate Validate source_template_build manifest_path",
        )
    target_dir = source_template_build.get("target_dir")
    if not isinstance(target_dir, str) or not target_dir.strip():
        diagnostics.append(
            "SourceTemplate Validate source_template_build target_dir must be a non-empty string"
        )
    elif source_template_is_non_empty_trimmed_string(target_dir):
        diagnostics.extend(
            source_template_validate_build_plan_target_dir_diagnostics(
                target_dir,
                project_dir,
            )
        )
    return diagnostics


def source_template_validate_build_plan_target_dir_diagnostics(
    target_dir: str,
    project_dir: Path,
) -> list[str]:
    diagnostics: list[str] = []
    stage_dir = project_dir.parent
    expected = resolve_source_template_path_or_diagnostic(
        stage_dir / "target",
        diagnostics,
        "SourceTemplate Validate source_template_build expected target_dir",
    )
    if expected is None:
        return diagnostics
    target_path = Path(target_dir)
    if target_path.is_absolute():
        actual = resolve_source_template_path_or_diagnostic(
            target_path,
            diagnostics,
            "SourceTemplate Validate source_template_build target_dir",
        )
    else:
        actual = resolve_source_template_path_or_diagnostic(
            stage_dir.parent.parent / target_path,
            diagnostics,
            "SourceTemplate Validate source_template_build target_dir",
        )
    if actual is None:
        return diagnostics
    if actual != expected:
        return [
            "SourceTemplate Validate source_template_build target_dir must match "
            "current SourceTemplate stage target"
        ]
    return diagnostics


def source_template_build_validation_diagnostics(
    report: dict[str, Any],
    validate_report: dict[str, Any] | None = None,
) -> list[str]:
    validation = report.get("build_validation")
    if not isinstance(validation, dict):
        return ["SourceTemplate report build_validation must be an object"]

    diagnostics: list[str] = []
    diagnostics.extend(source_template_build_validation_schema_diagnostics(validation))
    requested = validation.get("requested")
    executed = validation.get("executed")
    status = validation.get("status")
    command = validation.get("command")
    working_dir = validation.get("working_dir")
    exit_code = validation.get("exit_code")
    stdout_lines = validation.get("stdout_lines")
    stderr_lines = validation.get("stderr_lines")
    report_command = report.get("command")
    build_executed = report.get("build_executed")
    expected_release = source_template_expected_release_from_validate_report(
        validate_report
    )

    command_diagnostics = source_template_non_empty_string_array_schema_diagnostics(
        "SourceTemplate build_validation command",
        command,
    )
    if command_diagnostics:
        diagnostics.extend(command_diagnostics)
    elif source_template_command_array_is_non_empty_trimmed(command):
        diagnostics.extend(
            source_template_command_manifest_path_diagnostics(
                report,
                command,
                label="SourceTemplate build_validation command",
                expected_release=expected_release,
            )
        )
    if not isinstance(working_dir, str) or not working_dir.strip():
        diagnostics.append("SourceTemplate build_validation working_dir must be a non-empty string")
    elif source_template_is_non_empty_trimmed_string(working_dir):
        working_path = resolve_source_template_path_or_diagnostic(
            working_dir,
            diagnostics,
            "SourceTemplate build_validation working_dir",
        )
        project_path = resolve_source_template_path_or_diagnostic(
            report["project"],
            diagnostics,
            "SourceTemplate report project",
        )
        if (
            working_path is not None
            and project_path is not None
            and working_path != project_path
        ):
            diagnostics.append(
                "SourceTemplate build_validation working_dir must match "
                "SourceTemplate report project"
            )
    if "exit_code" not in validation:
        diagnostics.append(
            "SourceTemplate build_validation exit_code must be an integer or null"
        )
    elif exit_code is not None and not isinstance(exit_code, int):
        diagnostics.append("SourceTemplate build_validation exit_code must be an integer or null")
    for field, value in (
        ("stdout_lines", stdout_lines),
        ("stderr_lines", stderr_lines),
    ):
        if field not in validation:
            diagnostics.append(
                f"SourceTemplate build_validation {field} must be a string array"
            )
    if executed is True:
        for field, value in (
            ("stdout_lines", stdout_lines),
            ("stderr_lines", stderr_lines),
        ):
            if not isinstance(value, list) or any(
                not isinstance(line, str) for line in value
            ):
                diagnostics.append(
                    f"SourceTemplate build_validation {field} must be a string array"
                )
    report_command_diagnostics = (
        source_template_non_empty_string_array_schema_diagnostics(
            "SourceTemplate report command",
            report_command,
        )
    )
    if report_command_diagnostics:
        diagnostics.extend(report_command_diagnostics)
    elif (
        source_template_command_array_is_non_empty_trimmed(report_command)
        and source_template_command_array_is_non_empty_trimmed(command)
        and command != report_command
    ):
        diagnostics.append(
            "SourceTemplate build_validation command must match SourceTemplate report command"
        )
    if (
        isinstance(report_command, list)
        and source_template_command_array_is_non_empty_trimmed(report_command)
    ):
        diagnostics.extend(
            source_template_command_manifest_path_diagnostics(
                report,
                report_command,
                expected_release=expected_release,
            )
        )

    diagnostics.extend(
        source_template_build_status_diagnostics(
            requested=requested,
            executed=executed,
            status=status,
            exit_code=exit_code,
            build_executed=build_executed,
        )
    )
    return diagnostics


def source_template_command_array_is_non_empty_trimmed(value: Any) -> bool:
    return (
        isinstance(value, list)
        and bool(value)
        and all(
            isinstance(item, str) and item.strip() and item.strip() == item
            for item in value
        )
    )


def source_template_command_manifest_path_diagnostics(
    report: dict[str, Any],
    command: list[str],
    *,
    label: str = "SourceTemplate report command",
    expected_release: bool | None = None,
) -> list[str]:
    diagnostics: list[str] = []
    project = report.get("project")
    if not isinstance(project, str) or not project:
        return []
    if not source_template_command_runs_cargo_build(command):
        diagnostics.append(f"{label} must run cargo build")
    diagnostics.extend(
        source_template_command_forbidden_target_triple_diagnostics(
            command,
            label=label,
        )
    )
    if expected_release is not None:
        diagnostics.extend(
            source_template_command_release_flag_diagnostics(
                command,
                expected_release,
                label=label,
            )
        )
    option_diagnostics: dict[str, str] = {}
    for option in ("--manifest-path", "--target-dir"):
        option_diagnostic = command_option_value_diagnostic(
            command,
            option,
            label,
        )
        if option_diagnostic:
            option_diagnostics[option] = option_diagnostic
            diagnostics.append(option_diagnostic)
    manifest_path = source_template_command_option_value(command, "--manifest-path")
    if (
        "--manifest-path" not in option_diagnostics
        and (not isinstance(manifest_path, str) or not manifest_path)
    ):
        diagnostics.append(f"{label} must include --manifest-path")
    target_dir = source_template_command_option_value(command, "--target-dir")
    if (
        "--target-dir" not in option_diagnostics
        and (not isinstance(target_dir, str) or not target_dir)
    ):
        diagnostics.append(f"{label} must include --target-dir")
    expected_project = resolve_source_template_path_or_diagnostic(
        project,
        diagnostics,
        "SourceTemplate report project",
    )
    if expected_project is None:
        return diagnostics
    if (
        "--manifest-path" not in option_diagnostics
        and isinstance(manifest_path, str)
        and manifest_path
    ):
        actual = resolve_source_template_path_or_diagnostic(
            manifest_path,
            diagnostics,
            "SourceTemplate report command manifest-path",
        )
        if actual is not None:
            expected = expected_project / "Cargo.toml"
            if actual != expected:
                diagnostics.append(
                    f"{label} manifest-path must target current project Cargo.toml"
                )
    if (
        "--target-dir" not in option_diagnostics
        and isinstance(target_dir, str)
        and target_dir
    ):
        diagnostics.extend(
            source_template_report_target_dir_diagnostics(
                target_dir,
                expected_project,
                label=label,
            )
        )
    return diagnostics


def source_template_report_target_dir_diagnostics(
    target_dir: str, project_dir: Path, *, label: str = "SourceTemplate report command"
) -> list[str]:
    diagnostics: list[str] = []
    expected = resolve_source_template_path_or_diagnostic(
        project_dir.parent / "target",
        diagnostics,
        f"{label} expected target-dir",
    )
    if expected is None:
        return diagnostics
    actual = resolve_source_template_path_or_diagnostic(
        target_dir,
        diagnostics,
        f"{label} target-dir",
    )
    if actual is None:
        return diagnostics
    if actual != expected:
        diagnostics.append(
            f"{label} target-dir must match current SourceTemplate stage target"
        )
    return diagnostics


def source_template_command_option_value(command: list[str], option: str) -> str | None:
    for index, value in enumerate(command):
        if value == option and index + 1 < len(command):
            return command[index + 1]
    return None


def source_template_expected_release_from_validate_report(
    validate_report: dict[str, Any] | None,
) -> bool | None:
    if not isinstance(validate_report, dict):
        return None
    plan_summary = validate_report.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return None
    source_template_build = plan_summary.get("source_template_build")
    if not isinstance(source_template_build, dict):
        return None
    return source_template_build_plan_expected_release(source_template_build)
