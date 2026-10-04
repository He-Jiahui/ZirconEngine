"""Validate report LibraryEmbed CompileHost Cargo command diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_validate_compile_host_command_value_semantics import (
    command_alias_value_match_diagnostics,
    command_features_match_diagnostics,
    command_option_path_value_match_diagnostics,
    command_option_value_match_diagnostics,
    compile_host_release_flag_schema_diagnostics,
)


COMPILE_HOST_COMMAND_FORBIDDEN_TARGET_FLAGS = (
    "--all-targets",
    "--bins",
    "--examples",
    "--tests",
    "--benches",
    "--lib",
)
COMPILE_HOST_COMMAND_FORBIDDEN_TARGET_TRIPLE_FLAGS = ("--target",)
COMPILE_HOST_COMMAND_FORBIDDEN_PACKAGE_FLAGS = (
    "--workspace",
    "--all",
    "--exclude",
)
COMPILE_HOST_COMMAND_FORBIDDEN_PROFILE_FLAGS = ("--profile",)
COMPILE_HOST_COMMAND_FORBIDDEN_WRAPPER_POLICY_FLAGS = (
    "--locked",
    "--offline",
    "--frozen",
)


def library_embed_compile_host_command_schema_diagnostics(
    value: dict[str, Any],
) -> list[str]:
    label = "validate report plan_summary.library_embed_compile_host"
    command = value.get("command")
    if (
        not isinstance(command, list)
        or any(not isinstance(entry, str) for entry in command)
    ):
        return []
    if any(not entry.strip() or entry != entry.strip() for entry in command):
        return []

    command_label = f"{label}.command"
    diagnostics: list[str] = []
    if len(command) < 2 or command[0] != "cargo" or command[1] != "build":
        diagnostics.append(f"{command_label} must run cargo build")
    diagnostics.extend(
        command_flag_diagnostics(
            command,
            "--no-default-features",
            label=command_label,
        )
    )
    diagnostics.extend(
        command_forbidden_flag_diagnostics(
            command,
            "--all-features",
            label=command_label,
            reason=(
                "because CompileHost plan app_features owns feature selection"
            ),
        )
    )
    diagnostics.extend(
        compile_host_command_forbidden_target_diagnostics(
            command,
            label=command_label,
        )
    )
    diagnostics.extend(
        compile_host_command_forbidden_target_triple_diagnostics(
            command,
            label=command_label,
        )
    )
    diagnostics.extend(
        compile_host_command_forbidden_package_diagnostics(
            command,
            label=command_label,
        )
    )
    diagnostics.extend(
        compile_host_command_forbidden_profile_diagnostics(
            command,
            label=command_label,
        )
    )
    diagnostics.extend(
        compile_host_command_forbidden_wrapper_policy_diagnostics(
            command,
            label=command_label,
        )
    )
    diagnostics.extend(
        command_alias_value_match_diagnostics(
            command,
            ("-p", "--package"),
            value.get("package"),
            f"{label}.package",
            label=command_label,
            option_label="-p/--package",
        )
    )
    diagnostics.extend(
        command_option_value_match_diagnostics(
            command,
            "--bin",
            value.get("binary"),
            f"{label}.binary",
            label=command_label,
        )
    )
    diagnostics.extend(
        command_features_match_diagnostics(
            command,
            value.get("app_features"),
            f"{label}.app_features",
            label=command_label,
        )
    )
    diagnostics.extend(
        command_option_path_value_match_diagnostics(
            command,
            "--target-dir",
            value.get("target_dir"),
            f"{label}.target_dir",
            label=command_label,
        )
    )
    diagnostics.extend(
        command_option_path_value_match_diagnostics(
            command,
            "--manifest-path",
            value.get("manifest_path"),
            f"{label}.manifest_path",
            label=command_label,
        )
    )
    diagnostics.extend(
        compile_host_release_flag_schema_diagnostics(
            command,
            value,
            label=command_label,
        )
    )
    return diagnostics


def command_flag_diagnostics(
    command: list[str],
    flag: str,
    *,
    label: str,
) -> list[str]:
    occurrences = sum(1 for entry in command if entry == flag)
    if occurrences == 0:
        return [f"{label} must include {flag}"]
    if occurrences > 1:
        return [f"{label} {flag} must appear only once"]
    return []


def command_forbidden_flag_diagnostics(
    command: list[str],
    flag: str,
    *,
    label: str,
    reason: str,
) -> list[str]:
    if any(entry == flag or entry.startswith(f"{flag}=") for entry in command):
        return [f"{label} must not include {flag} {reason}"]
    return []


def compile_host_command_forbidden_target_diagnostics(
    command: list[str],
    *,
    label: str,
) -> list[str]:
    return command_forbidden_flags_diagnostics(
        command,
        COMPILE_HOST_COMMAND_FORBIDDEN_TARGET_FLAGS,
        label=label,
        reason="because CompileHost plan binary owns the single host target",
    )


def compile_host_command_forbidden_target_triple_diagnostics(
    command: list[str],
    *,
    label: str,
) -> list[str]:
    return command_forbidden_flags_diagnostics(
        command,
        COMPILE_HOST_COMMAND_FORBIDDEN_TARGET_TRIPLE_FLAGS,
        label=label,
        reason="because export target descriptor owns platform target selection",
    )


def compile_host_command_forbidden_package_diagnostics(
    command: list[str],
    *,
    label: str,
) -> list[str]:
    return command_forbidden_flags_diagnostics(
        command,
        COMPILE_HOST_COMMAND_FORBIDDEN_PACKAGE_FLAGS,
        label=label,
        reason="because CompileHost plan package owns package selection",
    )


def compile_host_command_forbidden_profile_diagnostics(
    command: list[str],
    *,
    label: str,
) -> list[str]:
    return command_forbidden_flags_diagnostics(
        command,
        COMPILE_HOST_COMMAND_FORBIDDEN_PROFILE_FLAGS,
        label=label,
        reason=(
            "because CompileHost plan cargo_profile/release owns "
            "profile selection"
        ),
    )


def compile_host_command_forbidden_wrapper_policy_diagnostics(
    command: list[str],
    *,
    label: str,
) -> list[str]:
    return command_forbidden_flags_diagnostics(
        command,
        COMPILE_HOST_COMMAND_FORBIDDEN_WRAPPER_POLICY_FLAGS,
        label=label,
        reason="because CompileHost CLI owns Cargo lock/offline policy",
    )


def command_forbidden_flags_diagnostics(
    command: list[str],
    flags: tuple[str, ...],
    *,
    label: str,
    reason: str,
) -> list[str]:
    diagnostics: list[str] = []
    for flag in flags:
        diagnostics.extend(
            command_forbidden_flag_diagnostics(
                command,
                flag,
                label=label,
                reason=reason,
            )
        )
    return diagnostics
