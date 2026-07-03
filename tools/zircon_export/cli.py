#!/usr/bin/env python3
"""Stage-oriented Zircon export pipeline command."""

from __future__ import annotations

import argparse
import json
import os
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Sequence

from .cli_arguments import parse_args
from .compile_host import (
    compile_host_command,
    run_compile_host,
)
from .cook_assets import run_cook_assets
from .pipeline_report import run_report
from .pack_stage import run_pack
from .platform_bundle import run_platform_bundle
from .plugin_command import dispatch_plugin_command
from .pipeline_stages import (
    pipeline_stages_after_validate as selected_pipeline_stages_after_validate,
    pipeline_stages_from_resume as selected_pipeline_stages_from_resume,
)
from .native_dynamic import run_native_dynamic
from .report_io import write_report_targets
from .stage_handoff import (
    compile_host_report_host_executable,
    cook_assets_report_asset_manifest,
    native_dynamic_report_plugins_dir,
    pack_report_delta_pack_file,
    pack_report_pack_file,
    validate_report_asset_filter,
    validate_report_asset_filter_diagnostic,
)
from .source_template import run_source_template
from .validate_stage import (
    resolve_validate_optional_path,
    resolve_validate_path,
    validate_command,
    validate_preflight_failure_report,
)


REPORT_FILE_NAME = "report.json"


def main(argv: Sequence[str] | None = None) -> int:
    argv_list = list(argv) if argv is not None else sys.argv[1:]
    plugin_exit_code = dispatch_plugin_command(argv_list)
    if plugin_exit_code is not None:
        return plugin_exit_code
    args = parse_args(argv_list)
    if args.resume_from or not args.stage_explicit:
        return run_pipeline(args, args.resume_from or "validate")
    return run_stage(args)


def run_stage(args: argparse.Namespace) -> int:
    if args.stage == "validate":
        return run_validate(args)
    if args.stage == "compile_host":
        return run_compile_host(args)
    if args.stage == "source_template":
        return run_source_template(args)
    if args.stage == "native_dynamic":
        return run_native_dynamic(args)
    if args.stage == "cook_assets":
        return run_cook_assets(args)
    if args.stage == "pack":
        return run_pack(args)
    if args.stage == "platform_bundle":
        return run_platform_bundle(args)
    if args.stage == "report":
        return run_report(args)
    raise SystemExit(f"unsupported export stage: {args.stage}")


def run_pipeline(args: argparse.Namespace, resume_from: str) -> int:
    print(f"zircon_export resume_from={resume_from} profile={args.profile}")

    if resume_from == "validate":
        exit_code = run_pipeline_stage(args, "validate")
        print(f"pipeline_stage=validate exit_code={exit_code}")
        if exit_code != 0:
            return exit_code
        stages = pipeline_stages_after_validate(args)
        print("pipeline_stages=validate," + ",".join(stages))
    else:
        stages = pipeline_stages_from_resume(args, resume_from)
        print("pipeline_stages=" + ",".join(stages))

    for stage in stages:
        exit_code = run_pipeline_stage(args, stage)
        print(f"pipeline_stage={stage} exit_code={exit_code}")
        if exit_code != 0:
            return exit_code
    return 0


def run_pipeline_stage(args: argparse.Namespace, stage: str) -> int:
    stage_args = argparse.Namespace(**vars(args))
    stage_args.stage = stage
    apply_pipeline_stage_defaults(stage_args, stage)
    return run_stage(stage_args)


def pipeline_stages_after_validate(args: argparse.Namespace) -> tuple[str, ...]:
    return selected_pipeline_stages_after_validate(
        resolve_user_path(args.out),
        args.profile,
    )


def pipeline_stages_from_resume(
    args: argparse.Namespace,
    resume_from: str,
) -> tuple[str, ...]:
    return selected_pipeline_stages_from_resume(
        resolve_user_path(args.out),
        args.profile,
        resume_from,
    )


def pipeline_stages_from_validate_report(args: argparse.Namespace) -> tuple[str, ...]:
    return selected_pipeline_stages_after_validate(
        resolve_user_path(args.out),
        args.profile,
    )[:-1]


def apply_pipeline_stage_defaults(args: argparse.Namespace, stage: str) -> None:
    out_root = resolve_user_path(args.out)
    if stage == "cook_assets" and args.asset_filter is None:
        asset_filter_diagnostic = validate_report_asset_filter_diagnostic(
            out_root,
            args.profile,
        )
        if asset_filter_diagnostic:
            args.validate_asset_filter_diagnostic = asset_filter_diagnostic
            return
        asset_filter = validate_report_asset_filter(out_root, args.profile)
        if asset_filter:
            args.asset_filter = asset_filter
        return
    if stage == "pack" and args.asset_manifest is None:
        asset_manifest = cook_assets_report_asset_manifest(out_root, args.profile)
        if asset_manifest:
            args.asset_manifest = str(asset_manifest)
        return
    if stage != "platform_bundle":
        return
    if args.host_executable is None:
        host_executable = compile_host_report_host_executable(out_root, args.profile)
        if host_executable:
            args.host_executable = str(host_executable)
            args.host_executable_source_origin = "compile_host_report"
    if args.pack_file is None:
        pack_file = pack_report_pack_file(out_root, args.profile)
        if pack_file:
            args.pack_file = str(pack_file)
    if (
        args.delta_pack is None
        and not getattr(args, "pack_file_explicit", False)
        and not getattr(args, "delta_pack_explicit", False)
    ):
        delta_pack = pack_report_delta_pack_file(out_root, args.profile)
        if delta_pack:
            args.delta_pack = str(delta_pack)
    if args.native_plugins_dir is None:
        native_plugins_dir = native_dynamic_report_plugins_dir(out_root, args.profile)
        if native_plugins_dir:
            args.native_plugins_dir = str(native_plugins_dir)


def run_validate(args: argparse.Namespace) -> int:
    out_root = resolve_user_path(args.out)
    stage_dir = out_root / "stages" / "validate"
    report_path = stage_dir / REPORT_FILE_NAME

    diagnostics: list[str] = []
    repo_root = (
        resolve_validate_path(args.repo_root, "repo_root", diagnostics)
        if args.repo_root
        else default_repo_root()
    )
    project_path = resolve_validate_path(args.project, "project", diagnostics)
    validator = resolve_validate_optional_path(args.validator, "validator", diagnostics)
    target_dir = resolve_validate_optional_path(args.target_dir, "target_dir", diagnostics)
    command = (
        validate_command(
            args,
            repo_root,
            project_path,
            stage_dir,
            report_path,
            validator=validator,
            target_dir=target_dir,
        )
        if not diagnostics
        else None
    )
    print(f"zircon_export stage=Validate profile={args.profile}")
    print(f"report={report_path}")
    if command is not None:
        print(shell_join(command))
    else:
        print("command=<skipped>")
    if args.dry_run:
        for diagnostic in diagnostics:
            print(f"diagnostic={diagnostic}")
        return 2 if diagnostics else 0

    try:
        stage_dir.mkdir(parents=True, exist_ok=True)
    except OSError as error:
        report = {
            "stage": "Validate",
            "profile": args.profile,
            "fatal": True,
            "diagnostics": [
                f"Validate stage directory {stage_dir} could not be created: {error}"
            ],
            "project": str(project_path) if project_path else None,
            "stage_output": str(stage_dir),
            "command": command if command is not None else [],
        }
        print(json.dumps(report, indent=2))
        return 2
    if diagnostics:
        report = {
            "stage": "Validate",
            "profile": args.profile,
            "fatal": True,
            "diagnostics": diagnostics,
            "project": str(project_path) if project_path else None,
            "stage_output": str(stage_dir),
            "command": [],
        }
        write_report_targets([("Validate report", report_path)], report)
        print(json.dumps(report, indent=2))
        return 2
    if command is None:
        raise AssertionError("Validate command was not built after preflight passed")
    try:
        exit_code = subprocess.call(command, cwd=repo_root)
        if not report_path.is_file():
            report = validate_preflight_failure_report(
                args=args,
                project_path=project_path,
                stage_dir=stage_dir,
                command=command,
                diagnostics=[
                    f"Validate command exited with code {exit_code} but did not write report {report_path}"
                ],
                exit_code=exit_code,
            )
            write_report_targets([("Validate report", report_path)], report)
            print(json.dumps(report, indent=2))
            return exit_code if exit_code != 0 else 2
        return exit_code
    except OSError as error:
        report = validate_preflight_failure_report(
            args=args,
            project_path=project_path,
            stage_dir=stage_dir,
            command=command,
            diagnostics=[f"Validate command {command[0]} could not start: {error}"],
        )
        write_report_targets([("Validate report", report_path)], report)
        print(json.dumps(report, indent=2))
        return 2


def resolve_repo_root(repo_root: str | None) -> Path:
    if repo_root:
        return resolve_user_path(repo_root)
    return default_repo_root()


def default_repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def resolve_user_path(path: str | os.PathLike[str]) -> Path:
    return Path(path).expanduser().resolve()


def shell_join(command: Sequence[str]) -> str:
    if os.name == "nt":
        return subprocess.list2cmdline(command)
    return shlex.join(command)
