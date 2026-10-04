"""CompileHost final-report diagnostics for the staged Zircon build contract."""

from __future__ import annotations

from pathlib import Path
from typing import Any


COMPILE_HOST_LINK_PLAN_FIELDS = (
    "app_features",
    "runtime_features",
    "expected_runtime_plugins",
    "linked_runtime_crates",
)


def compile_host_link_plan_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    """Compare linkage only when both reports explicitly carry linkage data."""
    validate_plan = validate_library_embed_compile_host_plan(stage_reports)
    compile_host_link_plan = compile_host_stage_link_plan(stage_reports)
    if not isinstance(validate_plan, dict) or not isinstance(
        compile_host_link_plan, dict
    ):
        return []

    diagnostics: list[str] = []
    for field in COMPILE_HOST_LINK_PLAN_FIELDS:
        expected = validate_plan.get(field)
        actual = compile_host_link_plan.get(field)
        if isinstance(expected, list) and isinstance(actual, list) and actual != expected:
            diagnostics.append(
                f"compile_host report link_plan.{field} does not match "
                f"validate report plan_summary.library_embed_compile_host.{field}"
            )
    return diagnostics


def compile_host_command_diagnostics(
    stage_reports: list[dict[str, Any]],
    out_root: Path,
) -> list[str]:
    """Validate the hard-cut ``zircon_build.py`` invocation used by production."""
    validate_plan = validate_library_embed_compile_host_plan(stage_reports)
    compile_host_report = stage_report_payload(stage_reports, "compile_host")
    if not isinstance(validate_plan, dict) or not isinstance(compile_host_report, dict):
        return []

    command = compile_host_report.get("command")
    if not command_is_string_list(command):
        return []
    assert isinstance(command, list)

    label = "compile_host report command"
    diagnostics: list[str] = []
    legacy_options = (
        "-p",
        "--package",
        "--bin",
        "--target-dir",
        "--features",
        "--release",
    )
    present_legacy = [option for option in legacy_options if option in command]
    if present_legacy:
        diagnostics.append(
            f"{label} uses removed Cargo options: {', '.join(present_legacy)}"
        )

    target_mode = validate_target_mode(stage_reports)
    expected_targets = {
        "client_runtime": "hub,editor,runtime",
        "server_runtime": "runtime",
    }.get(target_mode)
    expected_runtime_feature = {
        "client_runtime": "target-client",
        "server_runtime": "target-server",
    }.get(target_mode)
    if expected_targets is not None and command_option_value(command, "--targets") != expected_targets:
        diagnostics.append(
            f"{label} --targets does not match validate report profile_summary.target_mode"
        )
    if (
        expected_runtime_feature is not None
        and command_option_value(command, "--runtime-features")
        != expected_runtime_feature
    ):
        diagnostics.append(
            f"{label} --runtime-features does not match "
            "validate report profile_summary.target_mode"
        )

    actual_out = command_option_value(command, "--out")
    staged_engine_root = compile_host_report.get("staged_engine_root")
    if isinstance(actual_out, str) and isinstance(staged_engine_root, str):
        try:
            expected_engine_root = Path(actual_out).expanduser().resolve() / "ZirconEngine"
            actual_engine_root = Path(staged_engine_root).expanduser().resolve()
            actual_engine_root.relative_to(out_root.expanduser().resolve())
        except (OSError, ValueError):
            diagnostics.append(
                f"{label} --out or staged_engine_root is outside current output root"
            )
        else:
            if actual_engine_root != expected_engine_root:
                diagnostics.append(
                    f"{label} --out does not match compile_host report staged_engine_root"
                )
    return diagnostics


def command_is_string_list(value: object) -> bool:
    return isinstance(value, list) and all(isinstance(entry, str) for entry in value)


def command_option_value(command: list[str], option: str) -> str | None:
    for index, entry in enumerate(command):
        if entry == option and index + 1 < len(command):
            return command[index + 1]
    return None


def compile_host_host_executable_diagnostics(
    stage_reports: list[dict[str, Any]],
    out_root: Path,
) -> list[str]:
    compile_host_report = stage_report_payload(stage_reports, "compile_host")
    if not isinstance(compile_host_report, dict):
        return []
    if compile_host_report.get("fatal") is not False:
        return []

    host_executable = compile_host_report.get("host_executable")
    staged_engine_root = compile_host_report.get("staged_engine_root")
    if not isinstance(host_executable, str) or not host_executable.strip():
        return []
    if not isinstance(staged_engine_root, str) or not staged_engine_root.strip():
        return []

    try:
        resolved_host = Path(host_executable).expanduser().resolve()
        resolved_staged_root = Path(staged_engine_root).expanduser().resolve()
        resolved_out_root = out_root.expanduser().resolve()
        resolved_staged_root.relative_to(resolved_out_root)
        resolved_host.relative_to(resolved_staged_root)
    except OSError as error:
        return [
            "compile_host report host_executable or staged_engine_root "
            f"could not be resolved: {error}"
        ]
    except ValueError:
        return [
            "compile_host report host_executable and staged_engine_root "
            "must remain inside current output root"
        ]

    if resolved_host.parent != resolved_staged_root:
        return [
            "compile_host report host_executable must be a direct child of "
            f"staged_engine_root {resolved_staged_root}"
        ]
    if not resolved_host.is_file():
        return [f"compile_host report host_executable {resolved_host} does not exist"]
    try:
        if resolved_host.stat().st_size <= 0:
            return [f"compile_host report host_executable {resolved_host} is empty"]
    except OSError as error:
        return [
            f"compile_host report host_executable {resolved_host} could not be inspected: {error}"
        ]

    target_mode = validate_target_mode(stage_reports)
    expected_stem = {
        "client_runtime": "zircon_hub",
        "server_runtime": "zircon_runtime",
    }.get(target_mode)
    if expected_stem is not None and resolved_host.name not in {
        expected_stem,
        f"{expected_stem}.exe",
    }:
        return [
            "compile_host report host_executable does not match "
            "validate report profile_summary.target_mode"
        ]
    return []


def validate_library_embed_compile_host_plan(
    stage_reports: list[dict[str, Any]],
) -> object:
    validate_report = stage_report_payload(stage_reports, "validate")
    if not isinstance(validate_report, dict):
        return None
    plan_summary = validate_report.get("plan_summary")
    if not isinstance(plan_summary, dict):
        return None
    return plan_summary.get("library_embed_compile_host")


def validate_target_mode(stage_reports: list[dict[str, Any]]) -> str | None:
    validate_report = stage_report_payload(stage_reports, "validate")
    if not isinstance(validate_report, dict):
        return None
    profile_summary = validate_report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return None
    value = profile_summary.get("target_mode")
    return value if isinstance(value, str) else None


def compile_host_stage_link_plan(stage_reports: list[dict[str, Any]]) -> object:
    compile_host_report = stage_report_payload(stage_reports, "compile_host")
    if not isinstance(compile_host_report, dict):
        return None
    return compile_host_report.get("link_plan")


def stage_report_payload(
    stage_reports: list[dict[str, Any]],
    stage_key: str,
) -> object:
    for stage_report in stage_reports:
        if stage_report.get("stage_key") == stage_key and stage_report.get("fatal") is not True:
            return stage_report.get("report")
    return None
