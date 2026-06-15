"""SourceTemplate materialization and generated-project validation stage."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
from pathlib import Path
from typing import Any, Sequence


REPORT_FILE_NAME = "report.json"
SOURCE_TEMPLATE_STAGE = "source_template"
PATH_DEPENDENCY_RE = re.compile(
    r'(?P<prefix>^(?P<crate>zircon_[A-Za-z0-9_]+)\s*=\s*\{[^}]*path\s*=\s*")'
    r'(?P<path>[^"]+)'
    r'(?P<suffix>"[^}]*\}\s*$)',
    re.MULTILINE,
)


def run_source_template(args: argparse.Namespace) -> int:
    repo_root = resolve_user_path(args.repo_root) if args.repo_root else default_repo_root()
    out_root = resolve_user_path(args.out)
    validate_report = (
        resolve_user_path(args.validate_report)
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

    diagnostics: list[str] = []
    fatal = False
    validate_payload = load_validate_report(validate_report, args.profile, diagnostics)
    source_plan = source_template_plan(validate_payload, diagnostics)
    generated_files = generated_file_summaries(validate_payload)
    command: list[str] = []

    if validate_payload is None or source_plan is None:
        fatal = True
    else:
        command = source_template_command(args, project_dir, source_plan)
        print(shell_join(command))

    if args.dry_run:
        return 2 if fatal else 0

    stage_dir.mkdir(parents=True, exist_ok=True)
    build_executed = False
    if not fatal:
        materialization_diagnostic_count = len(diagnostics)
        materialize_generated_files(project_dir, validate_payload, diagnostics)
        rewrite_generated_manifest_paths(project_dir, repo_root, diagnostics)
        if len(diagnostics) > materialization_diagnostic_count:
            fatal = True
        if not getattr(args, "source_template_build", False):
            diagnostics.append("SourceTemplate build validation skipped; pass --source-template-build to execute cargo build")
        elif not fatal:
            build_executed = True
            exit_code = subprocess.call(command, cwd=project_dir)
            if exit_code != 0:
                fatal = True
                diagnostics.append(f"SourceTemplate cargo build exited with code {exit_code}")

    report = {
        "stage": "SourceTemplate",
        "profile": args.profile,
        "fatal": fatal,
        "diagnostics": diagnostics,
        "validate_report": str(validate_report),
        "project": str(project_dir),
        "generated_files": generated_files,
        "command": command,
        "build_executed": build_executed,
    }
    report_path.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 2 if fatal else 0


def source_template_command(
    args: argparse.Namespace,
    project_dir: Path,
    source_plan: dict[str, Any],
) -> list[str]:
    command = list(source_plan["command"])
    if command:
        command[0] = args.cargo
    manifest_path = project_dir / str(source_plan["manifest_path"])
    stage_dir = project_dir.parent
    target_dir = (
        resolve_user_path(args.target_dir)
        if args.target_dir
        else stage_dir / "target"
    )
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
    try:
        report = json.loads(validate_report.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        diagnostics.append(f"validate report {validate_report} is not valid JSON: {error}")
        return None
    if not isinstance(report, dict):
        diagnostics.append(f"validate report {validate_report} must be a JSON object")
        return None
    if report.get("fatal"):
        diagnostics.append("validate report is fatal; SourceTemplate will not materialize")
        return None
    if report.get("profile") != profile:
        diagnostics.append(
            f"validate report profile {report.get('profile')} does not match requested profile {profile}"
        )
        return None
    return report


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
    if not isinstance(source_plan, dict):
        diagnostics.append("validate report does not contain a SourceTemplate build plan")
        return None
    command = source_plan.get("command")
    if not isinstance(command, list) or any(not isinstance(value, str) for value in command):
        diagnostics.append("SourceTemplate build plan command must be a string array")
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


def materialize_generated_files(
    project_dir: Path,
    validate_payload: dict[str, Any],
    diagnostics: list[str],
) -> None:
    plan_summary = validate_payload.get("plan_summary", {})
    files = plan_summary.get("generated_files", []) if isinstance(plan_summary, dict) else []
    project_dir.mkdir(parents=True, exist_ok=True)
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
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(contents, encoding="utf-8")


def rewrite_generated_manifest_paths(
    project_dir: Path,
    repo_root: Path,
    diagnostics: list[str],
) -> None:
    manifest_path = project_dir / "Cargo.toml"
    if not manifest_path.exists():
        diagnostics.append(f"SourceTemplate manifest {manifest_path} does not exist after materialization")
        return
    source = manifest_path.read_text(encoding="utf-8")

    def replace(match: re.Match[str]) -> str:
        crate_name = match.group("crate")
        relative = match.group("path").replace("\\", "/")
        crate_path = (repo_root / relative.lstrip("./")).resolve()
        if not crate_path.exists():
            diagnostics.append(f"SourceTemplate dependency {crate_name} path {crate_path} does not exist")
        return f"{match.group('prefix')}{crate_path.as_posix()}{match.group('suffix')}"

    rewritten = PATH_DEPENDENCY_RE.sub(replace, source)
    manifest_path.write_text(rewritten, encoding="utf-8")


def resolve_project_child(
    project_dir: Path,
    relative_path: str,
    diagnostics: list[str],
) -> Path | None:
    child_path = Path(relative_path)
    if child_path.is_absolute():
        diagnostics.append(f"generated file path {relative_path} must be relative")
        return None
    resolved_root = project_dir.resolve()
    resolved = (resolved_root / child_path).resolve()
    try:
        resolved.relative_to(resolved_root)
    except ValueError:
        diagnostics.append(f"generated file path {relative_path} escapes the SourceTemplate project")
        return None
    return resolved


def command_with_option(command: list[str], option: str, value: str) -> list[str]:
    rewritten: list[str] = []
    index = 0
    found = False
    while index < len(command):
        rewritten.append(command[index])
        if command[index] == option and index + 1 < len(command):
            rewritten.append(value)
            index += 2
            found = True
            continue
        index += 1
    if not found:
        rewritten.extend([option, value])
    return rewritten


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_user_path(path: str | os.PathLike[str]) -> Path:
    return Path(path).expanduser().resolve()


def shell_join(command: Sequence[str]) -> str:
    if os.name == "nt":
        return subprocess.list2cmdline(command)
    import shlex

    return shlex.join(command)
