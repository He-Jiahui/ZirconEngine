"""SourceTemplate final report diagnostics for the Zircon export pipeline."""

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Any

from .command_plan import command_option_value_diagnostic
from .pipeline_report_source_template_stage_schema import (
    source_template_build_validation_schema_diagnostics,
    source_template_generated_file_schema_diagnostics,
    source_template_report_schema_diagnostics,
    source_template_sha256_is_valid,
)
from .pipeline_report_source_template_string_array_schema import (
    source_template_non_empty_string_array_schema_diagnostics,
)
from .pipeline_report_source_template_validate_schema import (
    source_template_validate_build_plan_schema_diagnostics,
    source_template_validate_generated_file_schema_diagnostics,
)


def resolve_source_template_path_or_diagnostic(
    path: str | Path,
    diagnostics: list[str],
    label: str,
) -> Path | None:
    try:
        return Path(path).expanduser().resolve()
    except OSError as error:
        diagnostics.append(f"{label} {path} could not be resolved: {error}")
        return None


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
        diagnostics.extend(source_template_build_validation_diagnostics(report))
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


def source_template_generated_file_diagnostics(
    report: dict[str, Any],
    project_dir: Path,
    validate_report: dict[str, Any] | None,
) -> list[str]:
    generated_files = report.get("generated_files")
    if not isinstance(generated_files, list):
        return ["SourceTemplate report generated_files must be a list"]
    diagnostics: list[str] = []
    report_paths: list[str] = []
    for index, file in enumerate(generated_files):
        if not isinstance(file, dict):
            diagnostics.append("SourceTemplate generated file entry must be an object")
            continue
        diagnostics.extend(source_template_generated_file_schema_diagnostics(file, index))
        path = file.get("path")
        if not isinstance(path, str) or not path.strip():
            diagnostics.append("SourceTemplate generated file path must be a non-empty string")
            continue
        if not source_template_is_non_empty_trimmed_string(path):
            continue
        if path in report_paths:
            diagnostics.append(
                f"SourceTemplate report generated file path {path} is duplicated"
            )
        report_paths.append(path)
        output_path = source_template_generated_file_path(project_dir, path, diagnostics)
        if output_path is None:
            continue
        if not output_path.exists():
            diagnostics.append(f"SourceTemplate generated file {path} does not exist")
            continue
        if not output_path.is_file():
            diagnostics.append(f"SourceTemplate generated file {path} is not a file")
            continue
        try:
            contents = output_path.read_bytes()
        except OSError as error:
            diagnostics.append(
                f"SourceTemplate generated file {path} could not be read: {error}"
            )
            continue
        expected_size = file.get("size")
        if not isinstance(expected_size, int):
            diagnostics.append(
                f"SourceTemplate generated file {path} size must be an integer"
            )
        elif expected_size != len(contents):
            diagnostics.append(
                f"SourceTemplate generated file {path} size {len(contents)} "
                f"does not match report size {expected_size}"
            )
        expected_sha256 = file.get("sha256")
        if not isinstance(expected_sha256, str) or not source_template_sha256_is_valid(
            expected_sha256
        ):
            diagnostics.append(
                f"SourceTemplate generated file {path} sha256 must be a 64-character hex string"
            )
        else:
            actual_sha256 = hashlib.sha256(contents).hexdigest()
            if actual_sha256 != expected_sha256:
                diagnostics.append(
                    f"SourceTemplate generated file {path} sha256 {actual_sha256} "
                    f"does not match report sha256 {expected_sha256}"
                )
    diagnostics.extend(
        source_template_generated_file_plan_diagnostics(
            report_paths,
            validate_report,
            project_dir,
        )
    )
    return diagnostics


def source_template_generated_file_plan_diagnostics(
    report_paths: list[str],
    validate_report: dict[str, Any] | None,
    project_dir: Path,
) -> list[str]:
    plan_paths, diagnostics = source_template_validate_generated_file_paths(
        validate_report,
        project_dir,
    )
    if plan_paths is None:
        return ["SourceTemplate Validate plan_summary.generated_files must be a list"]
    for path in plan_paths:
        if path not in report_paths:
            diagnostics.append(
                f"SourceTemplate report missing generated file from Validate plan: {path}"
            )
    for path in report_paths:
        if path not in plan_paths:
            diagnostics.append(
                f"SourceTemplate report generated file {path} is not declared by Validate plan"
            )
    return diagnostics


def source_template_validate_generated_file_paths(
    validate_report: dict[str, Any] | None,
    project_dir: Path,
) -> tuple[list[str] | None, list[str]]:
    diagnostics: list[str] = []
    if not isinstance(validate_report, dict):
        return None, diagnostics
    plan_summary = validate_report.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return None, diagnostics
    generated_files = plan_summary.get("generated_files")
    if not isinstance(generated_files, list):
        return None, diagnostics
    paths: list[str] = []
    for index, file in enumerate(generated_files):
        if not isinstance(file, dict):
            diagnostics.append("SourceTemplate Validate generated file entry must be an object")
            continue
        diagnostics.extend(
            source_template_validate_generated_file_schema_diagnostics(file, index)
        )
        path = file.get("path")
        if not isinstance(path, str) or not path.strip():
            diagnostics.append(
                "SourceTemplate Validate generated file path must be a non-empty string"
            )
            continue
        if not source_template_is_non_empty_trimmed_string(path):
            continue
        if source_template_generated_file_path(
            project_dir,
            path,
            diagnostics,
            kind="SourceTemplate Validate generated file path",
        ) is None:
            continue
        if path in paths:
            diagnostics.append(
                f"SourceTemplate Validate generated file path {path} is duplicated"
            )
            continue
        paths.append(path)
    return paths, diagnostics


def source_template_build_validation_diagnostics(report: dict[str, Any]) -> list[str]:
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

    if not isinstance(requested, bool):
        diagnostics.append("SourceTemplate build_validation requested must be a boolean")
    if not isinstance(executed, bool):
        diagnostics.append("SourceTemplate build_validation executed must be a boolean")
    if (
        source_template_is_non_empty_trimmed_string(status)
        and status not in {"skipped", "passed", "failed", "blocked"}
    ):
        diagnostics.append("SourceTemplate build_validation status must be skipped, passed, failed, or blocked")
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
    if not isinstance(build_executed, bool):
        diagnostics.append("SourceTemplate report build_executed must be a boolean")
    elif isinstance(executed, bool) and executed != build_executed:
        diagnostics.append(
            "SourceTemplate build_validation executed must match "
            "SourceTemplate report build_executed"
        )
    if (
        isinstance(report_command, list)
        and source_template_command_array_is_non_empty_trimmed(report_command)
    ):
        diagnostics.extend(
            source_template_command_manifest_path_diagnostics(report, report_command)
        )

    if status in {"failed", "blocked"}:
        diagnostics.append(f"SourceTemplate build_validation status {status} is not publishable")
    if status == "passed" and exit_code != 0:
        diagnostics.append("SourceTemplate build_validation passed status requires exit_code 0")
    if status == "failed" and not isinstance(exit_code, int):
        diagnostics.append("SourceTemplate build_validation failed status requires an integer exit_code")
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


def source_template_command_array_is_non_empty_trimmed(value: Any) -> bool:
    return (
        isinstance(value, list)
        and bool(value)
        and all(
            isinstance(item, str) and item.strip() and item.strip() == item
            for item in value
        )
    )


def source_template_is_non_empty_trimmed_string(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value.strip() == value


def source_template_command_manifest_path_diagnostics(
    report: dict[str, Any],
    command: list[str],
    *,
    label: str = "SourceTemplate report command",
) -> list[str]:
    diagnostics: list[str] = []
    project = report.get("project")
    if not isinstance(project, str) or not project:
        return []
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


def source_template_command_target_dir_diagnostics(
    command: list[str],
    project_dir: Path,
) -> list[str]:
    target_dir = source_template_command_option_value(command, "--target-dir")
    if target_dir is None:
        return []
    return source_template_report_target_dir_diagnostics(target_dir, project_dir)


def source_template_report_target_dir_diagnostics(
    target_dir: str,
    project_dir: Path,
    *,
    label: str = "SourceTemplate report command",
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


def source_template_command_option_value(
    command: list[str],
    option: str,
) -> str | None:
    for index, value in enumerate(command):
        if value == option and index + 1 < len(command):
            return command[index + 1]
    return None


def source_template_generated_file_path(
    project_dir: Path,
    relative_path: str,
    diagnostics: list[str],
    *,
    kind: str = "SourceTemplate generated file path",
) -> Path | None:
    file_path = Path(relative_path)
    if file_path.is_absolute():
        diagnostics.append(f"{kind} {relative_path} must be relative")
        return None
    resolved_project = resolve_source_template_path_or_diagnostic(
        project_dir,
        diagnostics,
        f"{kind} project",
    )
    if resolved_project is None:
        return None
    resolved_path = resolve_source_template_path_or_diagnostic(
        resolved_project / file_path,
        diagnostics,
        f"{kind} {relative_path}",
    )
    if resolved_path is None:
        return None
    try:
        resolved_path.relative_to(resolved_project)
    except ValueError:
        diagnostics.append(f"{kind} {relative_path} escapes the project")
        return None
    return resolved_path
