"""NativeDynamic build-plan Cargo command diagnostics."""

from __future__ import annotations

from typing import Any


NATIVE_DYNAMIC_BUILD_PLAN_COMMAND_FORBIDDEN_FEATURE_FLAGS = (
    "--all-features",
    "--no-default-features",
)
NATIVE_DYNAMIC_BUILD_PLAN_COMMAND_FORBIDDEN_TARGET_FLAGS = (
    "--all-targets",
    "--bins",
    "--examples",
    "--tests",
    "--benches",
    "--lib",
)
NATIVE_DYNAMIC_BUILD_PLAN_COMMAND_FORBIDDEN_TARGET_TRIPLE_FLAGS = ("--target",)
NATIVE_DYNAMIC_BUILD_PLAN_COMMAND_FORBIDDEN_PACKAGE_FLAGS = (
    "--workspace",
    "--all",
    "--exclude",
)
NATIVE_DYNAMIC_BUILD_PLAN_COMMAND_FORBIDDEN_PROFILE_FLAGS = ("--profile",)


def native_dynamic_build_plan_package_command_semantics_diagnostics(
    package_label: str,
    package: dict[str, Any],
) -> list[str]:
    command = package.get("command")
    workspace_manifest = package.get("workspace_manifest")
    crate_name = package.get("crate_name")
    target_dir = package.get("target_dir")
    release = package.get("release")
    features = package.get("features")
    owner_label = native_dynamic_build_plan_local_report_label(package_label)
    if not (
        native_dynamic_build_plan_string_array_is_trimmed_non_empty(command)
        and native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
            workspace_manifest
        )
    ):
        return []
    diagnostics = command_identity_diagnostics(
        command,
        label=f"{package_label}.command",
    )
    for flag in NATIVE_DYNAMIC_BUILD_PLAN_COMMAND_FORBIDDEN_FEATURE_FLAGS:
        diagnostics.extend(
            command_forbidden_flag_diagnostics(
                command,
                flag,
                f"because {owner_label}.features owns feature selection",
                label=f"{package_label}.command",
            )
        )
    for flag in NATIVE_DYNAMIC_BUILD_PLAN_COMMAND_FORBIDDEN_TARGET_FLAGS:
        diagnostics.extend(
            command_forbidden_flag_diagnostics(
                command,
                flag,
                (
                    f"because {owner_label}.crate_name owns the single "
                    "native build target"
                ),
                label=f"{package_label}.command",
            )
        )
    for flag in NATIVE_DYNAMIC_BUILD_PLAN_COMMAND_FORBIDDEN_TARGET_TRIPLE_FLAGS:
        diagnostics.extend(
            command_forbidden_flag_diagnostics(
                command,
                flag,
                "because export target descriptor owns platform target selection",
                label=f"{package_label}.command",
            )
        )
    for flag in NATIVE_DYNAMIC_BUILD_PLAN_COMMAND_FORBIDDEN_PACKAGE_FLAGS:
        diagnostics.extend(
            command_forbidden_flag_diagnostics(
                command,
                flag,
                f"because {owner_label}.crate_name owns package selection",
                label=f"{package_label}.command",
            )
        )
    for flag in NATIVE_DYNAMIC_BUILD_PLAN_COMMAND_FORBIDDEN_PROFILE_FLAGS:
        diagnostics.extend(
            command_forbidden_flag_diagnostics(
                command,
                flag,
                (
                    f"because {owner_label}.cargo_profile/release owns "
                    "profile selection"
                ),
                label=f"{package_label}.command",
            )
        )
    diagnostics.extend(
        command_option_string_value_match_diagnostics(
            command,
            "--manifest-path",
            workspace_manifest,
            f"{package_label}.workspace_manifest",
            label=f"{package_label}.command",
        )
    )
    if native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
        crate_name
    ):
        diagnostics.extend(
            command_alias_string_value_match_diagnostics(
                command,
                ("-p", "--package"),
                crate_name,
                f"{package_label}.crate_name",
                label=f"{package_label}.command",
                option_label="-p/--package",
            )
        )
    if native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
        target_dir
    ):
        diagnostics.extend(
            command_option_string_value_match_diagnostics(
                command,
                "--target-dir",
                target_dir,
                f"{package_label}.target_dir",
                label=f"{package_label}.command",
            )
        )
    if type(release) is bool:
        diagnostics.extend(
            command_flag_presence_diagnostics(
                command,
                "--release",
                release,
                f"{package_label}.release",
                label=f"{package_label}.command",
            )
        )
    if native_dynamic_build_plan_string_array_is_schema_clean(
        features,
        allow_empty=True,
        require_unique=True,
    ):
        if features:
            diagnostics.extend(
                command_option_string_value_match_diagnostics(
                    command,
                    "--features",
                    ",".join(features),
                    f"{package_label}.features",
                    label=f"{package_label}.command",
                )
            )
        else:
            diagnostics.extend(
                command_option_absence_diagnostics(
                    command,
                    "--features",
                    f"{package_label}.features",
                    label=f"{package_label}.command",
                )
            )
    return diagnostics


def native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
    value: Any,
) -> bool:
    return (
        isinstance(value, str)
        and bool(value.strip())
        and value.strip() == value
    )


def native_dynamic_build_plan_string_array_is_trimmed_non_empty(
    value: Any,
) -> bool:
    return (
        isinstance(value, list)
        and bool(value)
        and all(
            isinstance(entry, str) and entry.strip() and entry.strip() == entry
            for entry in value
        )
    )


def native_dynamic_build_plan_string_array_is_schema_clean(
    value: Any,
    *,
    allow_empty: bool,
    require_unique: bool,
) -> bool:
    if not (
        isinstance(value, list)
        and (allow_empty or bool(value))
        and all(
            native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
                entry
            )
            for entry in value
        )
    ):
        return False
    return not require_unique or len(set(value)) == len(value)


def native_dynamic_build_plan_local_report_label(label: str) -> str:
    return label.removeprefix("native_dynamic report ")


def command_forbidden_flag_diagnostics(
    command: list[str],
    flag: str,
    reason: str,
    *,
    label: str,
) -> list[str]:
    prefix = f"{flag}="
    if any(token == flag or token.startswith(prefix) for token in command):
        return [f"{label} must not include {flag} {reason}"]
    return []


def command_option_string_value_match_diagnostics(
    command: list[str],
    option: str,
    expected: str,
    value_label: str,
    *,
    label: str,
) -> list[str]:
    values: list[str | None] = []
    prefix = f"{option}="
    for index, token in enumerate(command):
        if token == option:
            values.append(command[index + 1] if index + 1 < len(command) else None)
        elif token.startswith(prefix):
            values.append(token.removeprefix(prefix))
    if not values:
        return [f"{label} must include {option}"]
    if len(values) > 1:
        return [f"{label} {option} must appear only once"]
    actual = values[0]
    if not actual:
        return [f"{label} {option} must include a value"]
    if actual != expected:
        return [
            f"{label} {option} {actual} does not match "
            f"{value_label} {expected}"
        ]
    return []


def command_identity_diagnostics(
    command: list[str],
    *,
    label: str,
) -> list[str]:
    diagnostics: list[str] = []
    if command[0] != "cargo":
        diagnostics.append(f"{label}[0] must be cargo")
    if len(command) < 2 or command[1] != "build":
        diagnostics.append(f"{label}[1] must be build")
    return diagnostics


def command_alias_string_value_match_diagnostics(
    command: list[str],
    aliases: tuple[str, ...],
    expected: str,
    value_label: str,
    *,
    label: str,
    option_label: str,
) -> list[str]:
    values: list[str | None] = []
    for index, token in enumerate(command):
        for alias in aliases:
            prefix = f"{alias}="
            if token == alias:
                values.append(command[index + 1] if index + 1 < len(command) else None)
            elif token.startswith(prefix):
                values.append(token.removeprefix(prefix))
    if not values:
        return [f"{label} must include {option_label}"]
    if len(values) > 1:
        return [f"{label} {option_label} must appear only once"]
    actual = values[0]
    if not actual:
        return [f"{label} {option_label} must include a value"]
    if actual != expected:
        return [
            f"{label} {option_label} {actual} does not match "
            f"{value_label} {expected}"
        ]
    return []


def command_flag_presence_diagnostics(
    command: list[str],
    flag: str,
    expected_present: bool,
    value_label: str,
    *,
    label: str,
) -> list[str]:
    values = [
        token
        for token in command
        if token == flag or token.startswith(f"{flag}=")
    ]
    if expected_present:
        if not values:
            return [f"{label} must include {flag}"]
        if len(values) > 1:
            return [f"{label} {flag} must appear only once"]
        if values[0] != flag:
            return [f"{label} {flag} must not include a value"]
    elif values:
        return [f"{label} {flag} must not be present when {value_label} is False"]
    return []


def command_option_absence_diagnostics(
    command: list[str],
    option: str,
    value_label: str,
    *,
    label: str,
) -> list[str]:
    prefix = f"{option}="
    for token in command:
        if token == option or token.startswith(prefix):
            return [f"{label} {option} must not be present when {value_label} is empty"]
    return []
