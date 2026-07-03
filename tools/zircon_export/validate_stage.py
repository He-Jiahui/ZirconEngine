"""Validate stage command construction, reports, and path resolution."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
from typing import Any

from .path_resolve import resolve_stage_optional_path


def validate_preflight_failure_report(
    *,
    args: argparse.Namespace,
    project_path: Path | None,
    stage_dir: Path,
    command: list[str],
    diagnostics: list[str],
    exit_code: int | None = None,
) -> dict[str, Any]:
    report: dict[str, Any] = {
        "stage": "Validate",
        "profile": args.profile,
        "fatal": True,
        "diagnostics": diagnostics,
        "project": str(project_path) if project_path else None,
        "stage_output": str(stage_dir),
        "command": command,
    }
    if exit_code is not None:
        report["exit_code"] = exit_code
    return report


def validate_command(
    args: argparse.Namespace,
    repo_root: Path,
    project_path: Path,
    stage_dir: Path,
    report_path: Path,
    *,
    validator: Path | None = None,
    target_dir: Path | None = None,
) -> list[str]:
    validator_args = [
        "--project",
        str(project_path),
        "--profile",
        args.profile,
        "--report",
        str(report_path),
        "--stage-output",
        str(stage_dir),
    ]
    if args.pretty:
        validator_args.append("--pretty")

    if validator:
        return [str(validator), *validator_args]

    command = [
        args.cargo,
        "run",
        "-p",
        "zircon_runtime",
        "--bin",
        "zircon_export_validate",
    ]
    if not args.no_locked:
        command.append("--locked")
    if args.offline:
        command.append("--offline")
    if target_dir:
        command.extend(["--target-dir", str(target_dir)])
    command.extend(["--", *validator_args])
    return command


def resolve_validate_path(
    value: object,
    label: str,
    diagnostics: list[str],
) -> Path | None:
    return resolve_stage_optional_path(value, label, diagnostics, prefix="Validate")


def resolve_validate_optional_path(
    value: object,
    label: str,
    diagnostics: list[str],
) -> Path | None:
    if value is None:
        return None
    if not isinstance(value, str) or not value.strip():
        return None
    try:
        return resolve_user_path(value)
    except OSError as error:
        diagnostics.append(f"{label} {value} could not be resolved: {error}")
        return None


def resolve_user_path(path: str | os.PathLike[str]) -> Path:
    return Path(path).expanduser().resolve()
