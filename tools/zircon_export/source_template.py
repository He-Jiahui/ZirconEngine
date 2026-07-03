"""SourceTemplate materialization and generated-project validation stage."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
from typing import Any

from .report_io import write_report_targets
from .source_template_generated_project import (
    generated_file_summaries,
    materialize_generated_files,
    reset_generated_project_dir,
    rewrite_generated_manifest_paths,
    source_template_generated_file_report,
    source_template_generated_files_plan_diagnostics,
)
from .source_template_paths import (
    default_repo_root,
    resolve_source_template_optional_path,
    resolve_user_path,
)
from .source_template_plan_command import (
    load_validate_report,
    source_template_command,
    source_template_plan,
)
from .subprocess_output import split_subprocess_output


REPORT_FILE_NAME = "report.json"
SOURCE_TEMPLATE_STAGE = "source_template"


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


def shell_join(command: list[str]) -> str:
    if os.name == "nt":
        return subprocess.list2cmdline(command)
    import shlex

    return shlex.join(command)
