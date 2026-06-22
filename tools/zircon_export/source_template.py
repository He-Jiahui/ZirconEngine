"""SourceTemplate materialization and generated-project validation stage."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
from pathlib import Path
from typing import Any, Sequence

from .command_plan import command_option_value_diagnostic, command_with_option
from .pipeline_report_source_template_validate_schema import (
    source_template_validate_build_plan_schema_diagnostics,
    source_template_validate_generated_files_schema_diagnostics,
)
from .report_io import write_report_targets
from .stage_handoff import (
    export_strategies_from_validate_report,
    export_strategy_diagnostics,
    load_stage_report_with_diagnostics,
    stage_report_metadata_diagnostic,
)
from .subprocess_output import split_subprocess_output


REPORT_FILE_NAME = "report.json"
SOURCE_TEMPLATE_STAGE = "source_template"
PATH_DEPENDENCY_RE = re.compile(
    r'(?P<prefix>^(?P<crate>zircon_[A-Za-z0-9_]+)\s*=\s*\{[^}]*path\s*=\s*")'
    r'(?P<path>[^"]+)'
    r'(?P<suffix>"[^}]*\}\s*$)',
    re.MULTILINE,
)


def run_source_template(args: argparse.Namespace) -> int:
    out_root = resolve_user_path(args.out)
    diagnostics: list[str] = []
    repo_root = (
        resolve_source_template_optional_path(args.repo_root, "repo_root", diagnostics)
        if args.repo_root
        else default_repo_root()
    )
    validate_report = (
        resolve_source_template_optional_path(
            args.validate_report,
            "validate_report",
            diagnostics,
        )
        if args.validate_report
        else out_root / "stages" / "validate" / REPORT_FILE_NAME
    )
    stage_dir = out_root / "stages" / SOURCE_TEMPLATE_STAGE
    project_dir = stage_dir / "project"
    report_path = stage_dir / REPORT_FILE_NAME

    print(f"zircon_export stage=SourceTemplate profile={args.profile}")
    print(f"validate_report={validate_report}")
    print(f"project={project_dir}")
    print(f"report={report_path}")

    fatal = False
    validate_payload = (
        load_validate_report(validate_report, args.profile, diagnostics)
        if validate_report is not None
        else None
    )
    source_plan = source_template_plan(validate_payload, diagnostics)
    generated_files = generated_file_summaries(validate_payload)
    if source_plan is None:
        generated_files = []
    generated_file_plan_diagnostics = source_template_generated_files_plan_diagnostics(
        validate_payload
    )
    if generated_file_plan_diagnostics:
        diagnostics.extend(generated_file_plan_diagnostics)
        generated_files = []
    command: list[str] = []
    build_validation: dict[str, Any] = {
        "requested": bool(getattr(args, "source_template_build", False)),
        "executed": False,
        "status": "not_planned",
        "exit_code": None,
        "working_dir": str(project_dir),
        "command": command,
        "stdout_lines": [],
        "stderr_lines": [],
    }

    if (
        repo_root is None
        or validate_report is None
        or validate_payload is None
        or source_plan is None
        or generated_file_plan_diagnostics
    ):
        fatal = True
    else:
        command = source_template_command(args, project_dir, source_plan, diagnostics)
        if diagnostics:
            fatal = True
        else:
            print(shell_join(command))
    build_validation["command"] = command
    if fatal and not command:
        print("command=<skipped>")

    if args.dry_run:
        for diagnostic in diagnostics:
            print(f"diagnostic={diagnostic}")
        return 2 if fatal else 0

    build_executed = False
    project_cleaned = False
    cleanup_reason: str | None = None
    try:
        stage_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"SourceTemplate stage directory {stage_dir} could not be created: {error}"
        )
        report = {
            "stage": "SourceTemplate",
            "profile": args.profile,
            "fatal": True,
            "diagnostics": diagnostics,
            "validate_report": str(validate_report) if validate_report else None,
            "project": str(project_dir),
            "generated_files": generated_files,
            "command": command,
            "build_executed": build_executed,
            "build_validation": build_validation,
            "project_cleaned": project_cleaned,
            "cleanup_reason": cleanup_reason,
        }
        print(json.dumps(report, indent=2))
        return 2
    if not fatal:
        materialization_diagnostic_count = len(diagnostics)
        if not reset_generated_project_dir(project_dir, diagnostics):
            fatal = True
            cleanup_reason = "stale_project_cleanup_failed"
        if not fatal:
            materialized = materialize_generated_files(
                project_dir,
                validate_payload,
                diagnostics,
            )
            if not materialized:
                generated_files = []
                fatal = True
            else:
                rewrite_generated_manifest_paths(project_dir, repo_root, diagnostics)
                generated_files = source_template_generated_file_report(
                    project_dir,
                    generated_files,
                    diagnostics,
                )
            if len(diagnostics) > materialization_diagnostic_count:
                fatal = True
            if not getattr(args, "source_template_build", False):
                build_validation["status"] = "skipped"
                diagnostics.append("SourceTemplate build validation skipped; pass --source-template-build to execute cargo build")
            elif not fatal:
                build_executed = True
                build_validation["executed"] = True
                try:
                    build_result = subprocess.run(
                        command,
                        cwd=project_dir,
                        capture_output=True,
                        text=True,
                    )
                except OSError as error:
                    build_validation["exit_code"] = None
                    build_validation["status"] = "failed"
                    fatal = True
                    diagnostics.append(
                        f"SourceTemplate cargo build command {command[0]} could not start: {error}"
                    )
                else:
                    exit_code = build_result.returncode
                    build_validation["exit_code"] = exit_code
                    build_validation["status"] = "passed" if exit_code == 0 else "failed"
                    build_validation["stdout_lines"] = split_subprocess_output(
                        build_result.stdout
                    )
                    build_validation["stderr_lines"] = split_subprocess_output(
                        build_result.stderr
                    )
                    if exit_code != 0:
                        fatal = True
                        diagnostics.append(
                            f"SourceTemplate cargo build exited with code {exit_code}"
                        )
            elif getattr(args, "source_template_build", False):
                build_validation["status"] = "blocked"
            if fatal:
                project_cleaned = reset_generated_project_dir(project_dir, diagnostics)
                cleanup_reason = (
                    "fatal_diagnostics" if project_cleaned else "fatal_cleanup_failed"
                )

    report = {
        "stage": "SourceTemplate",
        "profile": args.profile,
        "fatal": fatal,
        "diagnostics": diagnostics,
        "validate_report": str(validate_report) if validate_report else None,
        "project": str(project_dir),
        "generated_files": generated_files,
        "command": command,
        "build_executed": build_executed,
        "build_validation": build_validation,
        "project_cleaned": project_cleaned,
        "cleanup_reason": cleanup_reason,
    }
    report_written = write_report_targets([("SourceTemplate report", report_path)], report)
    print(json.dumps(report, indent=2))
    return 2 if fatal or not report_written else 0


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
    report = load_stage_report_with_diagnostics(
        validate_report,
        "validate",
        diagnostics,
    )
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
        diagnostics.append(
            "SourceTemplate stage requires the source_template strategy"
        )
        return None
    return report


def validate_report_requires_strategy(
    report: dict[str, Any],
    strategy: str,
) -> bool:
    profile_summary = report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return False
    if "strategies" not in profile_summary:
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
    if (
        not isinstance(command, list)
        or not command
        or any(not isinstance(value, str) or not value.strip() for value in command)
    ):
        diagnostics.append("SourceTemplate build plan command must be a non-empty string array")
        return None
    manifest_path_diagnostic = command_option_value_diagnostic(
        command,
        "--manifest-path",
        "SourceTemplate build plan command",
    )
    if manifest_path_diagnostic:
        diagnostics.append(manifest_path_diagnostic)
        return None
    target_dir_diagnostic = command_option_value_diagnostic(
        command,
        "--target-dir",
        "SourceTemplate build plan command",
    )
    if target_dir_diagnostic:
        diagnostics.append(target_dir_diagnostic)
        return None
    manifest_path = source_plan.get("manifest_path")
    if not isinstance(manifest_path, str) or not manifest_path:
        diagnostics.append("SourceTemplate build plan manifest_path must be a non-empty string")
        return None
    target_dir = source_plan.get("target_dir")
    if not isinstance(target_dir, str) or not target_dir:
        diagnostics.append("SourceTemplate build plan target_dir must be a non-empty string")
        return None
    return source_plan


def source_template_command_array_is_valid(value: object) -> bool:
    return (
        isinstance(value, list)
        and bool(value)
        and all(isinstance(item, str) and item.strip() for item in value)
    )


def source_template_command_array_has_entry_type_errors(value: object) -> bool:
    return isinstance(value, list) and any(
        not isinstance(item, str) for item in value
    )


def generated_file_summaries(validate_payload: dict[str, Any] | None) -> list[dict[str, str]]:
    if validate_payload is None:
        return []
    plan_summary = validate_payload.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return []
    files = plan_summary.get("generated_files")
    if not isinstance(files, list):
        return []
    summaries: list[dict[str, str]] = []
    for file in files:
        if not isinstance(file, dict):
            continue
        path = file.get("path")
        purpose = file.get("purpose", "")
        if isinstance(path, str):
            summaries.append({"path": path, "purpose": purpose if isinstance(purpose, str) else ""})
    return summaries


def source_template_generated_files_plan_diagnostics(
    validate_payload: dict[str, Any] | None,
) -> list[str]:
    if validate_payload is None:
        return []
    plan_summary = validate_payload.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return []
    files = plan_summary.get("generated_files")
    diagnostics = source_template_validate_generated_files_schema_diagnostics(files)
    if diagnostics:
        return diagnostics
    return generated_file_path_duplicate_diagnostics(files)


def materialize_generated_files(
    project_dir: Path,
    validate_payload: dict[str, Any],
    diagnostics: list[str],
) -> bool:
    plan_summary = validate_payload.get("plan_summary", {})
    files = plan_summary.get("generated_files", []) if isinstance(plan_summary, dict) else []
    duplicate_diagnostics = generated_file_path_duplicate_diagnostics(files)
    if duplicate_diagnostics:
        diagnostics.extend(duplicate_diagnostics)
        return False
    try:
        project_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        diagnostics.append(
            f"SourceTemplate generated project {project_dir} could not be created: {error}"
        )
        return False
    for file in files:
        if not isinstance(file, dict):
            continue
        path = file.get("path")
        contents = file.get("contents")
        if not isinstance(path, str):
            continue
        if not isinstance(contents, str):
            diagnostics.append(f"validate report generated file {path} has no contents; skipped")
            continue
        destination = resolve_project_child(project_dir, path, diagnostics)
        if destination is None:
            continue
        try:
            destination.parent.mkdir(parents=True, exist_ok=True)
        except OSError as error:
            diagnostics.append(
                f"SourceTemplate generated file {path} parent directory could not be created: {error}"
            )
            continue
        try:
            destination.write_text(contents, encoding="utf-8")
        except OSError as error:
            diagnostics.append(
                f"SourceTemplate generated file {path} could not be written: {error}"
            )
    return True


def generated_file_path_duplicate_diagnostics(files: object) -> list[str]:
    if not isinstance(files, list):
        return []
    diagnostics: list[str] = []
    paths: set[str] = set()
    for file in files:
        if not isinstance(file, dict):
            continue
        path = file.get("path")
        if not isinstance(path, str):
            continue
        if path in paths:
            diagnostics.append(f"SourceTemplate generated file path {path} is duplicated")
            continue
        paths.add(path)
    return diagnostics


def source_template_generated_file_report(
    project_dir: Path,
    generated_files: list[dict[str, str]],
    diagnostics: list[str],
) -> list[dict[str, str | int]]:
    report: list[dict[str, str | int]] = []
    for file in generated_files:
        path = file["path"]
        destination = resolve_project_child(project_dir, path, diagnostics)
        if destination is None:
            continue
        if not destination.exists():
            diagnostics.append(f"SourceTemplate generated file {path} does not exist after materialization")
            continue
        if not destination.is_file():
            diagnostics.append(f"SourceTemplate generated file {path} is not a file after materialization")
            continue
        try:
            contents = destination.read_bytes()
        except OSError as error:
            diagnostics.append(
                f"SourceTemplate generated file {path} could not be read: {error}"
            )
            continue
        report.append(
            {
                "path": path,
                "purpose": file.get("purpose", ""),
                "size": len(contents),
                "sha256": hashlib.sha256(contents).hexdigest(),
            }
        )
    return report


def reset_generated_project_dir(project_dir: Path, diagnostics: list[str]) -> bool:
    if project_dir.exists():
        try:
            shutil.rmtree(project_dir)
        except OSError as error:
            diagnostics.append(
                f"SourceTemplate generated project {project_dir} could not be removed: {error}"
            )
            return False
    return True


def rewrite_generated_manifest_paths(
    project_dir: Path,
    repo_root: Path,
    diagnostics: list[str],
) -> None:
    manifest_path = project_dir / "Cargo.toml"
    if not manifest_path.exists():
        diagnostics.append(f"SourceTemplate manifest {manifest_path} does not exist after materialization")
        return
    if not manifest_path.is_file():
        diagnostics.append(f"SourceTemplate manifest {manifest_path} is not a file after materialization")
        return
    try:
        source = manifest_path.read_text(encoding="utf-8")
    except OSError as error:
        diagnostics.append(f"SourceTemplate manifest {manifest_path} could not be read: {error}")
        return

    def replace(match: re.Match[str]) -> str:
        crate_name = match.group("crate")
        relative = match.group("path").replace("\\", "/")
        try:
            crate_path = (repo_root / relative.lstrip("./")).resolve()
        except OSError as error:
            diagnostics.append(
                f"SourceTemplate dependency {crate_name} path {relative} could not be resolved: {error}"
            )
            return match.group(0)
        if not crate_path.exists():
            diagnostics.append(f"SourceTemplate dependency {crate_name} path {crate_path} does not exist")
        return f"{match.group('prefix')}{crate_path.as_posix()}{match.group('suffix')}"

    rewritten = PATH_DEPENDENCY_RE.sub(replace, source)
    try:
        manifest_path.write_text(rewritten, encoding="utf-8")
    except OSError as error:
        diagnostics.append(f"SourceTemplate manifest {manifest_path} could not be written: {error}")


def resolve_project_child(
    project_dir: Path,
    relative_path: str,
    diagnostics: list[str],
    *,
    kind: str = "generated file path",
) -> Path | None:
    child_path = Path(relative_path)
    if child_path.is_absolute():
        diagnostics.append(f"{kind} {relative_path} must be relative")
        return None
    try:
        resolved_root = project_dir.resolve()
    except OSError as error:
        diagnostics.append(
            f"SourceTemplate project {project_dir} could not be resolved for {kind} {relative_path}: {error}"
        )
        return None
    try:
        resolved = (resolved_root / child_path).resolve()
    except OSError as error:
        diagnostics.append(f"{kind} {relative_path} could not be resolved: {error}")
        return None
    try:
        resolved.relative_to(resolved_root)
    except ValueError:
        if kind == "generated file path":
            diagnostics.append(f"{kind} {relative_path} escapes the SourceTemplate project")
        else:
            diagnostics.append(f"{kind} {relative_path} escapes the generated project")
        return None
    return resolved


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_source_template_optional_path(
    value: object,
    label: str,
    diagnostics: list[str],
) -> Path | None:
    if value is None:
        return None
    if not isinstance(value, (str, os.PathLike)):
        diagnostics.append(f"{label} argument must be a path-like value")
        return None
    try:
        return resolve_user_path(value)
    except OSError as error:
        diagnostics.append(f"{label} {value} could not be resolved: {error}")
        return None


def resolve_user_path(path: str | os.PathLike[str]) -> Path:
    return Path(path).expanduser().resolve()


def shell_join(command: Sequence[str]) -> str:
    if os.name == "nt":
        return subprocess.list2cmdline(command)
    import shlex

    return shlex.join(command)
