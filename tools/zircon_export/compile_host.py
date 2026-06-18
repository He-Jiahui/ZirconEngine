"""CompileHost stage execution for the Zircon export pipeline."""

from __future__ import annotations

import argparse
import json
import os
import shlex
import subprocess
from pathlib import Path
from typing import Any, Sequence

from .command_plan import command_option_value_diagnostic, command_with_option
from .path_resolve import resolve_stage_optional_path
from .report_io import write_report_targets
from .stage_handoff import (
    export_strategies_from_validate_report,
    export_strategy_diagnostics,
    load_stage_report_with_diagnostics,
    stage_report_metadata_diagnostic,
)
from .subprocess_output import split_subprocess_output

REPORT_FILE_NAME = "report.json"


def run_compile_host(args: argparse.Namespace) -> int:
    out_root = resolve_user_path(args.out)
    diagnostics: list[str] = []
    repo_root = (
        resolve_compile_host_path(args.repo_root, "repo_root", diagnostics)
        if args.repo_root
        else default_repo_root()
    )
    validate_report = compile_host_validate_report_path(
        args,
        out_root,
        diagnostics,
    )
    stage_dir = out_root / "stages" / "compile_host"
    report_path = stage_dir / REPORT_FILE_NAME

    print(f"zircon_export stage=CompileHost profile={args.profile}")
    print(f"validate_report={validate_report if validate_report else '<invalid>'}")
    print(f"report={report_path}")

    fatal = repo_root is None or validate_report is None
    compile_plan = (
        None
        if fatal or validate_report is None
        else load_compile_host_plan(validate_report, args.profile, diagnostics)
    )
    command: list[str] = []
    host_executable = None
    link_plan: dict[str, object] | None = None
    stdout_lines: list[str] = []
    stderr_lines: list[str] = []

    if compile_plan is None:
        fatal = True
    else:
        target_dir = compile_host_resolved_target_dir(args, out_root, diagnostics)
        if target_dir is None:
            fatal = True
        else:
            command = compile_host_command(
                args,
                out_root,
                compile_plan,
                target_dir=target_dir,
            )
            link_plan = compile_host_link_plan(compile_plan)
            host_executable = compile_host_executable_path(
                out_root,
                compile_plan,
                target_dir=target_dir,
            )
            print(shell_join(command))
            print(f"host={host_executable}")

    if args.dry_run:
        for diagnostic in diagnostics:
            print(f"diagnostic={diagnostic}")
        return 2 if fatal else 0

    try:
        stage_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        fatal = True
        diagnostics.append(
            f"CompileHost stage directory {stage_dir} could not be created: {error}"
        )
        report = {
            "stage": "CompileHost",
            "profile": args.profile,
            "fatal": fatal,
            "diagnostics": diagnostics,
            "validate_report": str(validate_report) if validate_report else None,
            "command": command,
            "host_executable": str(host_executable) if host_executable else None,
            "exit_code": 2,
            "stdout_lines": stdout_lines,
            "stderr_lines": stderr_lines,
        }
        add_compile_host_link_plan(report, link_plan)
        print(json.dumps(report, indent=2))
        return 2
    exit_code = 2
    if fatal:
        exit_code = 2
    else:
        try:
            compile_result = subprocess.run(
                command,
                cwd=repo_root,
                capture_output=True,
                text=True,
            )
            exit_code = compile_result.returncode
            stdout_lines = split_subprocess_output(compile_result.stdout)
            stderr_lines = split_subprocess_output(compile_result.stderr)
        except OSError as error:
            exit_code = 2
            fatal = True
            diagnostics.append(
                f"CompileHost cargo command {command[0]} could not start: {error}"
            )
        if not fatal and exit_code != 0:
            fatal = True
            diagnostics.append(f"CompileHost cargo command exited with code {exit_code}")
        elif not fatal and host_executable and not host_executable.exists():
            fatal = True
            diagnostics.append(f"CompileHost output {host_executable} does not exist")
        elif not fatal and host_executable and not host_executable.is_file():
            fatal = True
            diagnostics.append(f"CompileHost output {host_executable} is not a file")

    report = {
        "stage": "CompileHost",
        "profile": args.profile,
        "fatal": fatal,
        "diagnostics": diagnostics,
        "validate_report": str(validate_report) if validate_report else None,
        "command": command,
        "host_executable": str(host_executable) if host_executable else None,
        "exit_code": exit_code,
        "stdout_lines": stdout_lines,
        "stderr_lines": stderr_lines,
    }
    add_compile_host_link_plan(report, link_plan)
    report_written = write_report_targets([("CompileHost report", report_path)], report)
    print(json.dumps(report, indent=2))
    return 2 if fatal or not report_written else 0


def compile_host_validate_report_path(
    args: argparse.Namespace,
    out_root: Path,
    diagnostics: list[str],
) -> Path | None:
    if not args.validate_report:
        return out_root / "stages" / "validate" / REPORT_FILE_NAME
    try:
        return resolve_user_path(args.validate_report)
    except OSError as error:
        diagnostics.append(
            f"CompileHost validate_report {args.validate_report} could not be resolved: {error}"
        )
        return None


def resolve_compile_host_path(
    value: object,
    label: str,
    diagnostics: list[str],
) -> Path | None:
    return resolve_stage_optional_path(value, label, diagnostics, prefix="CompileHost")


def load_compile_host_plan(
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
        diagnostics.append("validate report is fatal; CompileHost will not run")
        return None
    strategy_diagnostics = export_strategy_diagnostics(report)
    if strategy_diagnostics:
        diagnostics.extend(strategy_diagnostics)
        return None
    if validate_report_requires_compile_host_strategy(report):
        diagnostics.append(
            "CompileHost stage requires library_embed or native_dynamic strategy"
        )
        return None

    plan_summary = report.get("plan_summary")
    if not isinstance(plan_summary, dict):
        diagnostics.append("validate report does not contain plan_summary")
        return None
    compile_plan = plan_summary.get("library_embed_compile_host")
    if not isinstance(compile_plan, dict):
        diagnostics.append("validate report does not contain a LibraryEmbed CompileHost plan")
        return None
    binary = compile_plan.get("binary")
    if not isinstance(binary, str) or not binary.strip():
        diagnostics.append("CompileHost plan binary must be a non-empty string")
        return None
    cargo_profile = compile_plan.get("cargo_profile")
    if not isinstance(cargo_profile, str) or not cargo_profile.strip():
        diagnostics.append("CompileHost plan cargo_profile must be a non-empty string")
        return None
    command = compile_plan.get("command")
    if (
        not isinstance(command, list)
        or not command
        or any(not isinstance(value, str) or not value.strip() for value in command)
    ):
        diagnostics.append("CompileHost plan command must be a non-empty string array")
        return None
    target_dir_diagnostic = command_option_value_diagnostic(
        command,
        "--target-dir",
        "CompileHost plan command",
    )
    if target_dir_diagnostic:
        diagnostics.append(target_dir_diagnostic)
        return None
    return compile_plan


def validate_report_requires_compile_host_strategy(report: dict[str, Any]) -> bool:
    profile_summary = report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return False
    if "strategies" not in profile_summary:
        return False
    strategies = profile_summary.get("strategies")
    if not isinstance(strategies, list):
        return False
    normalized_strategies = export_strategies_from_validate_report(report)
    return not ({"library_embed", "native_dynamic"} & normalized_strategies)


def compile_host_command(
    args: argparse.Namespace,
    out_root: Path,
    compile_plan: dict[str, Any],
    *,
    target_dir: Path | None = None,
) -> list[str]:
    command = list(compile_plan["command"])
    if command:
        command[0] = args.cargo
    if not args.no_locked and "--locked" not in command:
        command.append("--locked")
    if args.offline and "--offline" not in command:
        command.append("--offline")
    if target_dir is None:
        target_dir = (
            resolve_user_path(args.target_dir)
            if args.target_dir
            else compile_host_target_dir(out_root)
        )
    return command_with_option(command, "--target-dir", str(target_dir))


def compile_host_target_dir(out_root: Path) -> Path:
    return (out_root / "stages" / "compile_host" / "target").resolve()


def compile_host_resolved_target_dir(
    args: argparse.Namespace,
    out_root: Path,
    diagnostics: list[str],
) -> Path | None:
    try:
        if args.target_dir:
            return resolve_user_path(args.target_dir)
        return compile_host_target_dir(out_root)
    except OSError as error:
        label = "CompileHost target_dir" if args.target_dir else "CompileHost default target_dir"
        value = args.target_dir if args.target_dir else out_root / "stages" / "compile_host" / "target"
        diagnostics.append(f"{label} {value} could not be resolved: {error}")
        return None


def compile_host_executable_path(
    out_root: Path,
    compile_plan: dict[str, Any],
    args: argparse.Namespace | None = None,
    *,
    target_dir: Path | None = None,
) -> Path | None:
    binary = compile_plan.get("binary")
    cargo_profile = compile_plan.get("cargo_profile", "debug")
    if not isinstance(binary, str) or not binary:
        return None
    if not isinstance(cargo_profile, str) or not cargo_profile:
        cargo_profile = "debug"
    executable_name = binary + (".exe" if os.name == "nt" else "")
    if target_dir is None:
        target_dir = (
            resolve_user_path(args.target_dir)
            if args is not None and getattr(args, "target_dir", None)
            else compile_host_target_dir(out_root)
        )
    return target_dir / cargo_profile / executable_name


def compile_host_link_plan(compile_plan: dict[str, Any]) -> dict[str, object]:
    return {
        "app_features": compile_host_plan_list(compile_plan, "app_features"),
        "runtime_features": compile_host_plan_list(compile_plan, "runtime_features"),
        "expected_runtime_plugins": compile_host_plan_list(
            compile_plan,
            "expected_runtime_plugins",
        ),
        "linked_runtime_crates": compile_host_plan_list(
            compile_plan,
            "linked_runtime_crates",
        ),
    }


def add_compile_host_link_plan(
    report: dict[str, object],
    link_plan: dict[str, object] | None,
) -> None:
    if link_plan is not None:
        report["link_plan"] = link_plan


def compile_host_plan_list(
    compile_plan: dict[str, Any],
    field: str,
) -> list[object]:
    value = compile_plan.get(field)
    return list(value) if isinstance(value, list) else []


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_user_path(path: str | os.PathLike[str]) -> Path:
    return Path(path).expanduser().resolve()


def shell_join(command: Sequence[str]) -> str:
    if os.name == "nt":
        return subprocess.list2cmdline(command)
    return shlex.join(command)
