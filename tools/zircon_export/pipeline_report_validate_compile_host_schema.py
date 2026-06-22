"""Validate report LibraryEmbed CompileHost plan schema diagnostics."""

from __future__ import annotations

from typing import Any

from .command_plan import command_option_value_diagnostic
from .export_template import is_safe_relative_path, normalize_relative_path
from .pipeline_report_schema_primitives import (
    validate_bool_schema_diagnostics,
    validate_object_array_schema_diagnostics,
    validate_string_schema_diagnostics,
)
from .pipeline_report_schema_table import (
    string_array_duplicate_entry_index_schema_diagnostics,
    string_array_no_blank_entries_schema_diagnostics,
    string_array_trimmed_non_empty_entries_schema_diagnostics,
)
from .pipeline_report_validate_compile_host_linkage_schema import (
    validate_linked_runtime_crate_schema_diagnostics,
)
from .pipeline_report_validate_identifier_schema import (
    validate_project_plugin_package_id_array_schema_diagnostics,
    validate_unique_project_plugin_package_id_array_schema_diagnostics,
)
from .pipeline_report_validate_string_array_schema import (
    validate_string_array_schema_diagnostics,
)

VALIDATE_LIBRARY_EMBED_COMPILE_HOST_FIELDS = (
    "app_features",
    "binary",
    "cargo_profile",
    "command",
    "expected_runtime_plugins",
    "linked_runtime_crates",
    "manifest_path",
    "package",
    "release",
    "runtime_features",
    "target_dir",
)
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_REQUIRED_FIELDS = VALIDATE_LIBRARY_EMBED_COMPILE_HOST_FIELDS
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_STRING_FIELDS = (
    "binary",
    "cargo_profile",
    "manifest_path",
    "package",
    "target_dir",
)
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_PATH_FIELDS = (
    "manifest_path",
    "target_dir",
)
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_STRING_ARRAY_FIELDS = (
    "app_features",
    "command",
    "runtime_features",
)
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_PROJECT_PLUGIN_ID_ARRAY_FIELDS = (
    "expected_runtime_plugins",
)
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_BOOL_FIELDS = (
    "release",
)
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_CARGO_PROFILES = {"debug", "release"}
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_PACKAGES = ("zircon_app",)
VALIDATE_LIBRARY_EMBED_COMPILE_HOST_BINARIES = ("zircon_runtime", "zircon_editor")
COMPILE_HOST_COMMAND_FORBIDDEN_TARGET_FLAGS = (
    "--all-targets",
    "--bins",
    "--examples",
    "--tests",
    "--benches",
    "--lib",
)
COMPILE_HOST_COMMAND_FORBIDDEN_TARGET_TRIPLE_FLAGS = (
    "--target",
)
COMPILE_HOST_COMMAND_FORBIDDEN_PACKAGE_FLAGS = (
    "--workspace",
    "--all",
    "--exclude",
)
COMPILE_HOST_COMMAND_FORBIDDEN_PROFILE_FLAGS = (
    "--profile",
)
COMPILE_HOST_COMMAND_FORBIDDEN_WRAPPER_POLICY_FLAGS = (
    "--locked",
    "--offline",
    "--frozen",
)


def validate_library_embed_compile_host_schema_diagnostics(value: Any) -> list[str]:
    label = "validate report plan_summary.library_embed_compile_host"
    if not isinstance(value, dict):
        return [f"{label} must be an object"]

    diagnostics: list[str] = []
    known_compile_host_fields = set(VALIDATE_LIBRARY_EMBED_COMPILE_HOST_FIELDS)
    diagnostics.extend(
        f"{label} unknown field {field}"
        for field in sorted(value)
        if field not in known_compile_host_fields
    )
    diagnostics.extend(
        f"{label}.{field} is required"
        for field in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_REQUIRED_FIELDS
        if field not in value
    )
    for field in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_STRING_FIELDS:
        if field in value:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"{label}.{field}",
                    value.get(field),
                )
            )
            diagnostics.extend(
                validate_non_empty_trimmed_string_schema_diagnostics(
                    f"{label}.{field}",
                    value.get(field),
                )
            )
            if field in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_PATH_FIELDS:
                diagnostics.extend(
                    validate_safe_relative_path_schema_diagnostics(
                        f"{label}.{field}",
                        value.get(field),
                    )
                )
    for field in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_STRING_ARRAY_FIELDS:
        if field in value:
            field_label = f"{label}.{field}"
            diagnostics.extend(
                validate_string_array_schema_diagnostics(
                    field_label,
                    value.get(field),
                )
            )
            diagnostics.extend(
                string_array_no_blank_entries_schema_diagnostics(
                    field_label,
                    value.get(field),
                )
            )
            if field in ("app_features", "command", "runtime_features"):
                diagnostics.extend(
                    string_array_trimmed_non_empty_entries_schema_diagnostics(
                        field_label,
                        value.get(field),
                    )
                )
            if field in ("app_features", "runtime_features"):
                diagnostics.extend(
                    string_array_duplicate_entry_index_schema_diagnostics(
                        field_label,
                        value.get(field),
                    )
                )
    for field in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_PROJECT_PLUGIN_ID_ARRAY_FIELDS:
        if field in value:
            field_label = f"{label}.{field}"
            diagnostics.extend(
                validate_project_plugin_package_id_array_schema_diagnostics(
                    field_label,
                    value.get(field),
                )
            )
            diagnostics.extend(
                validate_unique_project_plugin_package_id_array_schema_diagnostics(
                    field_label,
                    value.get(field),
                )
            )
    for field in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_BOOL_FIELDS:
        if field in value:
            diagnostics.extend(
                validate_bool_schema_diagnostics(
                    f"{label}.{field}",
                    value.get(field),
                )
            )
    linked_runtime_crates = value.get("linked_runtime_crates")
    if "linked_runtime_crates" in value:
        diagnostics.extend(
            validate_object_array_schema_diagnostics(
                f"{label}.linked_runtime_crates",
                linked_runtime_crates,
            )
        )
    if isinstance(linked_runtime_crates, list):
        diagnostics.extend(
            validate_linked_runtime_crate_schema_diagnostics(linked_runtime_crates)
        )
    diagnostics.extend(
        compile_host_target_selector_schema_diagnostics(
            value,
            package_label=f"{label}.package",
            binary_label=f"{label}.binary",
        )
    )
    diagnostics.extend(library_embed_compile_host_profile_release_diagnostics(value))
    diagnostics.extend(library_embed_compile_host_command_schema_diagnostics(value))
    return diagnostics


def library_embed_compile_host_profile_release_diagnostics(
    value: dict[str, Any],
) -> list[str]:
    label = "validate report plan_summary.library_embed_compile_host"
    cargo_profile = value.get("cargo_profile")
    release = value.get("release")
    diagnostics: list[str] = []
    if not compile_host_cargo_profile_is_schema_clean(cargo_profile):
        return diagnostics
    if cargo_profile not in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_CARGO_PROFILES:
        diagnostics.append(f"{label}.cargo_profile must be debug or release")
        return diagnostics

    if isinstance(release, bool) and release != (cargo_profile == "release"):
        diagnostics.append(f"{label}.release must match cargo_profile")
    return diagnostics


def compile_host_cargo_profile_is_schema_clean(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip()) and value == value.strip()


def validate_non_empty_trimmed_string_schema_diagnostics(
    label: str,
    value: object,
) -> list[str]:
    if isinstance(value, str) and (not value.strip() or value != value.strip()):
        return [f"{label} must be a non-empty trimmed string"]
    return []


def validate_safe_relative_path_schema_diagnostics(
    label: str,
    value: object,
) -> list[str]:
    if not isinstance(value, str) or not value.strip() or value != value.strip():
        return []
    if not is_safe_relative_path(normalize_relative_path(value)):
        return [f"{label} must be a safe relative path"]
    return []


def compile_host_target_selector_schema_diagnostics(
    value: dict[str, Any],
    *,
    package_label: str,
    binary_label: str,
) -> list[str]:
    diagnostics: list[str] = []
    package = value.get("package")
    if (
        isinstance(package, str)
        and package.strip()
        and package == package.strip()
        and package not in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_PACKAGES
    ):
        diagnostics.append(f"{package_label} must be zircon_app")
    binary = value.get("binary")
    if (
        isinstance(binary, str)
        and binary.strip()
        and binary == binary.strip()
        and binary not in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_BINARIES
    ):
        diagnostics.append(f"{binary_label} must be zircon_runtime or zircon_editor")
    return diagnostics


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
    diagnostics: list[str] = []
    for flag in COMPILE_HOST_COMMAND_FORBIDDEN_TARGET_FLAGS:
        diagnostics.extend(
            command_forbidden_flag_diagnostics(
                command,
                flag,
                label=label,
                reason="because CompileHost plan binary owns the single host target",
            )
        )
    return diagnostics


def compile_host_command_forbidden_target_triple_diagnostics(
    command: list[str],
    *,
    label: str,
) -> list[str]:
    diagnostics: list[str] = []
    for flag in COMPILE_HOST_COMMAND_FORBIDDEN_TARGET_TRIPLE_FLAGS:
        diagnostics.extend(
            command_forbidden_flag_diagnostics(
                command,
                flag,
                label=label,
                reason=(
                    "because export target descriptor owns platform target selection"
                ),
            )
        )
    return diagnostics


def compile_host_command_forbidden_package_diagnostics(
    command: list[str],
    *,
    label: str,
) -> list[str]:
    diagnostics: list[str] = []
    for flag in COMPILE_HOST_COMMAND_FORBIDDEN_PACKAGE_FLAGS:
        diagnostics.extend(
            command_forbidden_flag_diagnostics(
                command,
                flag,
                label=label,
                reason="because CompileHost plan package owns package selection",
            )
        )
    return diagnostics


def compile_host_command_forbidden_profile_diagnostics(
    command: list[str],
    *,
    label: str,
) -> list[str]:
    diagnostics: list[str] = []
    for flag in COMPILE_HOST_COMMAND_FORBIDDEN_PROFILE_FLAGS:
        diagnostics.extend(
            command_forbidden_flag_diagnostics(
                command,
                flag,
                label=label,
                reason=(
                    "because CompileHost plan cargo_profile/release owns "
                    "profile selection"
                ),
            )
        )
    return diagnostics


def compile_host_command_forbidden_wrapper_policy_diagnostics(
    command: list[str],
    *,
    label: str,
) -> list[str]:
    diagnostics: list[str] = []
    for flag in COMPILE_HOST_COMMAND_FORBIDDEN_WRAPPER_POLICY_FLAGS:
        diagnostics.extend(
            command_forbidden_flag_diagnostics(
                command,
                flag,
                label=label,
                reason=(
                    "because CompileHost CLI owns Cargo lock/offline policy"
                ),
            )
        )
    return diagnostics


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
    return [
        feature
        for feature in value.replace(",", " ").split()
        if feature
    ]


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
        or cargo_profile not in VALIDATE_LIBRARY_EMBED_COMPILE_HOST_CARGO_PROFILES
    ):
        return []
    has_release_flag = "--release" in command
    if release is True or cargo_profile == "release":
        if not has_release_flag:
            return [f"{label} must include --release for release profile"]
    if release is False and cargo_profile == "debug" and has_release_flag:
        return [f"{label} must not include --release for debug profile"]
    return []
