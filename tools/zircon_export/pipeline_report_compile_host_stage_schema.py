"""Strict staged-build CompileHost report schema diagnostics."""

from __future__ import annotations

from typing import Any

from .command_plan import command_option_value_diagnostic
from .pipeline_report_schema_primitives import (
    validate_integer_schema_diagnostics,
    validate_string_schema_diagnostics,
)
from .pipeline_report_validate_string_array_schema import (
    validate_string_array_schema_diagnostics,
)

COMPILE_HOST_REPORT_FIELDS = (
    "command",
    "diagnostics",
    "exit_code",
    "fatal",
    "host_executable",
    "profile",
    "stage",
    "staged_engine_root",
    "stderr_lines",
    "stdout_lines",
)


def compile_host_report_schema_diagnostics(report: dict[str, Any]) -> list[str]:
    diagnostics: list[str] = []
    for field in ("host_executable", "staged_engine_root"):
        diagnostics.extend(
            validate_string_schema_diagnostics(
                f"compile_host report {field}", report.get(field)
            )
        )
    for field in ("command", "stderr_lines", "stdout_lines"):
        diagnostics.extend(
            validate_string_array_schema_diagnostics(
                f"compile_host report {field}", report.get(field)
            )
        )
    diagnostics.extend(
        validate_integer_schema_diagnostics(
            "compile_host report exit_code", report.get("exit_code")
        )
    )
    if report.get("fatal") is False and report.get("exit_code") != 0:
        diagnostics.append("compile_host report exit_code must be 0 for non-fatal report")
    command = report.get("command")
    if isinstance(command, list) and all(isinstance(item, str) for item in command):
        diagnostics.extend(compile_host_report_command_schema_diagnostics(command))
    return diagnostics


def compile_host_report_command_schema_diagnostics(command: list[str]) -> list[str]:
    label = "compile_host report command"
    if len(command) < 2 or not command[1].replace("\\", "/").endswith(
        "tools/zircon_build.py"
    ):
        return [f"{label} must run tools/zircon_build.py through Python"]
    diagnostics: list[str] = []
    for option in ("--targets", "--out", "--mode", "--runtime-features", "--cargo"):
        diagnostic = command_option_value_diagnostic(command, option, label)
        if diagnostic:
            diagnostics.append(diagnostic)
        elif option not in command:
            diagnostics.append(f"{label} must include {option}")
    mode_index = command.index("--mode") if "--mode" in command else -1
    if mode_index >= 0 and mode_index + 1 < len(command):
        mode = command[mode_index + 1]
        if mode not in ("debug", "release"):
            diagnostics.append(f"{label} --mode must be debug or release")
    return diagnostics
