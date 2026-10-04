"""Validate CompileHost Cargo command value semantic diagnostics."""

from __future__ import annotations

from typing import Any

from .command_plan import command_option_value_diagnostic
from .export_template_manifest import normalize_relative_path


VALIDATE_COMPILE_HOST_COMMAND_CARGO_PROFILES = {"debug", "release"}


def command_option_value_match_diagnostics(
    command: list[str],
    option: str,
    expected_value: object,
    expected_label: str,
    *,
    label: str,
) -> list[str]:
    diagnostics: list[str] = []
    option_diagnostic = command_option_value_diagnostic(command, option, label)
    if option_diagnostic:
        diagnostics.append(option_diagnostic)
        return diagnostics

    actual_value = command_option_value(command, option)
    if not actual_value:
        diagnostics.append(f"{label} must include {option}")
        return diagnostics
    if isinstance(expected_value, str) and actual_value != expected_value:
        diagnostics.append(f"{label} {option} must match {expected_label}")
    return diagnostics


def command_features_match_diagnostics(
    command: list[str],
    expected_features: object,
    expected_label: str,
    *,
    label: str,
) -> list[str]:
    diagnostics: list[str] = []
    option_diagnostic = command_option_value_diagnostic(command, "--features", label)
    if option_diagnostic:
        diagnostics.append(option_diagnostic)
        return diagnostics

    actual_features = command_option_value(command, "--features")
    if not actual_features:
        diagnostics.append(f"{label} must include --features")
        return diagnostics
    if not (
        isinstance(expected_features, list)
        and all(
            isinstance(feature, str) and feature.strip()
            for feature in expected_features
        )
    ):
        return diagnostics
    expected = [feature.strip() for feature in expected_features]
    if cargo_feature_list(actual_features) != expected:
        diagnostics.append(f"{label} --features must match {expected_label}")
    return diagnostics


def command_option_path_value_match_diagnostics(
    command: list[str],
    option: str,
    expected_value: object,
    expected_label: str,
    *,
    label: str,
) -> list[str]:
    diagnostics: list[str] = []
    option_diagnostic = command_option_value_diagnostic(command, option, label)
    if option_diagnostic:
        diagnostics.append(option_diagnostic)
        return diagnostics

    actual_value = command_option_value(command, option)
    if not actual_value:
        diagnostics.append(f"{label} must include {option}")
        return diagnostics
    if not isinstance(expected_value, str):
        return diagnostics
    if normalize_relative_path(actual_value) != normalize_relative_path(expected_value):
        diagnostics.append(f"{label} {option} must match {expected_label}")
    return diagnostics


def command_alias_value_match_diagnostics(
    command: list[str],
    options: tuple[str, ...],
    expected_value: object,
    expected_label: str,
    *,
    label: str,
    option_label: str,
) -> list[str]:
    occurrences: list[tuple[str, str | None]] = []
    diagnostics: list[str] = []
    for option in options:
        option_diagnostic = command_option_value_diagnostic(command, option, label)
        if option_diagnostic:
            diagnostics.append(option_diagnostic)
        value = command_option_value(command, option)
        if value is not None:
            occurrences.append((option, value))

    if diagnostics:
        return diagnostics
    if not occurrences:
        return [f"{label} must include {option_label}"]
    if len(occurrences) > 1:
        return [f"{label} {option_label} must appear only once"]

    _, actual_value = occurrences[0]
    if isinstance(expected_value, str) and actual_value != expected_value:
        return [f"{label} {option_label} must match {expected_label}"]
    return []


def command_option_value(command: list[str], option: str) -> str | None:
    for index, entry in enumerate(command):
        if entry == option and index + 1 < len(command):
            return command[index + 1]
    return None


def cargo_feature_list(value: str) -> list[str]:
    return [feature for feature in value.replace(",", " ").split() if feature]


def compile_host_release_flag_schema_diagnostics(
    command: list[str],
    value: dict[str, Any],
    *,
    label: str,
) -> list[str]:
    release = value.get("release")
    cargo_profile = value.get("cargo_profile")
    if (
        not isinstance(release, bool)
        or not compile_host_cargo_profile_is_schema_clean(cargo_profile)
        or cargo_profile not in VALIDATE_COMPILE_HOST_COMMAND_CARGO_PROFILES
    ):
        return []
    has_release_flag = "--release" in command
    if release is True or cargo_profile == "release":
        if not has_release_flag:
            return [f"{label} must include --release for release profile"]
    if release is False and cargo_profile == "debug" and has_release_flag:
        return [f"{label} must not include --release for debug profile"]
    return []


def compile_host_cargo_profile_is_schema_clean(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value == value.strip()
