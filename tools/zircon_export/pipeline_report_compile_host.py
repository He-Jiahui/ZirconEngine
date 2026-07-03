"""CompileHost final Report diagnostics for the Zircon export pipeline."""

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
    validate_plan = validate_library_embed_compile_host_plan(stage_reports)
    compile_host_link_plan = compile_host_stage_link_plan(stage_reports)
    if not isinstance(validate_plan, dict) or not isinstance(
        compile_host_link_plan,
        dict,
    ):
        return []

    diagnostics: list[str] = []
    for field in COMPILE_HOST_LINK_PLAN_FIELDS:
        expected = validate_plan.get(field)
        actual = compile_host_link_plan.get(field)
        if (
            isinstance(expected, list)
            and isinstance(actual, list)
            and actual != expected
        ):
            diagnostics.append(
                f"compile_host report link_plan.{field} does not match "
                f"validate report plan_summary.library_embed_compile_host.{field}"
            )
    return diagnostics


def compile_host_command_diagnostics(
    stage_reports: list[dict[str, Any]],
    out_root: Path,
) -> list[str]:
    validate_plan = validate_library_embed_compile_host_plan(stage_reports)
    compile_host_report = stage_report_payload(stage_reports, "compile_host")
    if not isinstance(validate_plan, dict) or not isinstance(
        compile_host_report,
        dict,
    ):
        return []

    command = compile_host_report.get("command")
    if not command_is_string_list(command):
        return []
    assert isinstance(command, list)

    label = "compile_host report command"
    validate_label = "validate report plan_summary.library_embed_compile_host"
    diagnostics: list[str] = []
    diagnostics.extend(
        compile_host_command_alias_match_diagnostics(
            command,
            ("-p", "--package"),
            "-p/--package",
            validate_plan.get("package"),
            f"{validate_label}.package",
            label=label,
        )
    )
    diagnostics.extend(
        compile_host_command_option_match_diagnostics(
            command,
            "--bin",
            validate_plan.get("binary"),
            f"{validate_label}.binary",
            label=label,
        )
    )
    diagnostics.extend(
        compile_host_command_target_dir_match_diagnostics(
            command,
            validate_plan.get("target_dir"),
            f"{validate_label}.target_dir",
            out_root,
            label=label,
        )
    )
    diagnostics.extend(
        compile_host_command_features_match_diagnostics(
            command,
            validate_plan.get("app_features"),
            f"{validate_label}.app_features",
            label=label,
        )
    )
    diagnostics.extend(
        compile_host_command_release_flag_diagnostics(
            command,
            validate_plan,
            label=label,
        )
    )
    return diagnostics


def compile_host_command_alias_match_diagnostics(
    command: list[str],
    options: tuple[str, ...],
    option_label: str,
    expected_value: object,
    expected_label: str,
    *,
    label: str,
) -> list[str]:
    actual_value = command_alias_value(command, options)
    if isinstance(expected_value, str) and actual_value is not None:
        if actual_value != expected_value:
            return [
                f"{label} {option_label} does not match {expected_label}",
            ]
    return []


def compile_host_command_option_match_diagnostics(
    command: list[str],
    option: str,
    expected_value: object,
    expected_label: str,
    *,
    label: str,
) -> list[str]:
    actual_value = command_option_value(command, option)
    if isinstance(expected_value, str) and actual_value is not None:
        if actual_value != expected_value:
            return [f"{label} {option} does not match {expected_label}"]
    return []


def compile_host_command_target_dir_match_diagnostics(
    command: list[str],
    expected_value: object,
    expected_label: str,
    out_root: Path,
    *,
    label: str,
) -> list[str]:
    actual_value = command_option_value(command, "--target-dir")
    if isinstance(expected_value, str) and actual_value is not None:
        if not command_target_dir_matches_out_root(
            actual_value,
            expected_value,
            out_root,
        ):
            return [f"{label} --target-dir does not match {expected_label}"]
    return []


def command_target_dir_matches_out_root(
    actual_value: str,
    expected_value: str,
    out_root: Path,
) -> bool:
    actual = normalized_path_token(actual_value)
    expected = normalized_path_token(expected_value)
    if actual == expected:
        return True

    actual_path = Path(actual_value)
    if not actual_path.is_absolute():
        return False

    try:
        relative_actual = actual_path.resolve().relative_to(out_root.resolve())
    except (OSError, ValueError):
        return False
    return normalized_path_token(str(relative_actual)) == expected


def normalized_path_token(value: str) -> str:
    return value.strip().replace("\\", "/")


def compile_host_command_features_match_diagnostics(
    command: list[str],
    expected_features: object,
    expected_label: str,
    *,
    label: str,
) -> list[str]:
    actual_features = command_option_value(command, "--features")
    if actual_features is None:
        return []
    if not (
        isinstance(expected_features, list)
        and all(
            isinstance(feature, str) and feature.strip()
            for feature in expected_features
        )
    ):
        return []
    expected = [feature.strip() for feature in expected_features]
    if cargo_feature_list(actual_features) != expected:
        return [f"{label} --features does not match {expected_label}"]
    return []


def compile_host_command_release_flag_diagnostics(
    command: list[str],
    validate_plan: dict[str, Any],
    *,
    label: str,
) -> list[str]:
    release = validate_plan.get("release")
    cargo_profile = validate_plan.get("cargo_profile")
    has_release_flag = "--release" in command
    if release is True or cargo_profile == "release":
        if not has_release_flag:
            return [f"{label} must include --release for release profile"]
    if release is False and cargo_profile == "debug" and has_release_flag:
        return [f"{label} must not include --release for debug profile"]
    return []


def command_is_string_list(value: object) -> bool:
    return isinstance(value, list) and all(isinstance(entry, str) for entry in value)


def command_alias_value(command: list[str], options: tuple[str, ...]) -> str | None:
    values = [
        value
        for option in options
        for value in [command_option_value(command, option)]
        if value is not None
    ]
    if len(values) == 1:
        return values[0]
    return None


def command_option_value(command: list[str], option: str) -> str | None:
    for index, entry in enumerate(command):
        if entry == option and index + 1 < len(command):
            return command[index + 1]
    return None


def cargo_feature_list(value: str) -> list[str]:
    return [feature for feature in value.replace(",", " ").split() if feature]


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
    if not isinstance(host_executable, str) or not host_executable.strip():
        return []

    try:
        resolved_host = Path(host_executable).expanduser().resolve()
        resolved_out_root = out_root.expanduser().resolve()
    except OSError as error:
        return [
            "compile_host report host_executable "
            f"{host_executable} could not be resolved: {error}"
        ]

    try:
        resolved_host.relative_to(resolved_out_root)
    except ValueError:
        return [
            "compile_host report host_executable "
            f"{resolved_host} is outside current output root {resolved_out_root}"
        ]
    if not resolved_host.exists():
        return [
            "compile_host report host_executable "
            f"{resolved_host} does not exist"
        ]
    if not resolved_host.is_file():
        return [
            "compile_host report host_executable "
            f"{resolved_host} is not a file"
        ]
    try:
        if resolved_host.stat().st_size <= 0:
            return [
                "compile_host report host_executable "
                f"{resolved_host} is empty"
            ]
    except OSError as error:
        return [
            "compile_host report host_executable "
            f"{resolved_host} could not be inspected: {error}"
        ]

    validate_plan = validate_library_embed_compile_host_plan(stage_reports)
    command = compile_host_report.get("command")
    if not isinstance(validate_plan, dict) or not command_is_string_list(command):
        return []
    assert isinstance(command, list)

    target_dir = command_option_value(command, "--target-dir")
    expected_target_dir = validate_plan.get("target_dir")
    cargo_profile = validate_plan.get("cargo_profile")
    binary = validate_plan.get("binary")
    if not (
        isinstance(target_dir, str)
        and Path(target_dir).is_absolute()
        and isinstance(expected_target_dir, str)
        and isinstance(cargo_profile, str)
        and cargo_profile.strip()
        and isinstance(binary, str)
        and binary.strip()
        and command_target_dir_matches_out_root(
            target_dir,
            expected_target_dir,
            out_root,
        )
    ):
        return []

    try:
        expected_profile_dir = Path(target_dir).expanduser().resolve() / cargo_profile
    except OSError as error:
        return [
            "compile_host report command --target-dir "
            f"{target_dir} could not be resolved: {error}"
        ]

    try:
        resolved_host.relative_to(expected_profile_dir)
    except ValueError:
        return [
            "compile_host report host_executable "
            f"{resolved_host} does not match command --target-dir profile "
            f"directory {expected_profile_dir}"
        ]
    if resolved_host.parent != expected_profile_dir:
        return [
            "compile_host report host_executable "
            f"{resolved_host} does not match command --target-dir profile "
            f"directory {expected_profile_dir}"
        ]

    expected_binary_names = {binary.strip(), f"{binary.strip()}.exe"}
    if resolved_host.name not in expected_binary_names:
        return [
            "compile_host report host_executable "
            f"{resolved_host} does not match validate report "
            "plan_summary.library_embed_compile_host.binary "
            f"{binary}"
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
        if (
            stage_report.get("stage_key") == stage_key
            and stage_report.get("fatal") is not True
        ):
            return stage_report.get("report")
    return None
