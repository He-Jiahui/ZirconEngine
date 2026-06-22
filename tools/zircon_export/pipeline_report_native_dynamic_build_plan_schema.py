"""NativeDynamic build-plan schema diagnostics."""

from __future__ import annotations

from typing import Any, Callable

from .native_build import platform_dynamic_library_name
from .pipeline_report_native_dynamic_build_audit_common import (
    native_dynamic_build_audit_package_count_diagnostics,
    native_dynamic_build_audit_package_id_uniqueness_diagnostics,
    native_dynamic_fatal_report_diagnostics,
    native_dynamic_non_fatal_report_diagnostics,
    string_array_unique_entries_schema_diagnostics,
    table_non_negative_integer_schema_diagnostics,
)
from .pipeline_report_schema_table import (
    non_empty_string_array_schema_diagnostics,
    string_array_no_blank_entries_schema_diagnostics,
    string_array_trimmed_non_empty_entries_schema_diagnostics,
)

NATIVE_DYNAMIC_BUILD_PLAN_FIELDS = (
    "build_features",
    "cargo_profile",
    "diagnostics",
    "fatal",
    "package_count",
    "packages",
    "release",
    "target_dir",
    "workspace_manifest",
)
NATIVE_DYNAMIC_BUILD_PLAN_STRING_FIELDS = (
    "cargo_profile",
    "target_dir",
    "workspace_manifest",
)
NATIVE_DYNAMIC_BUILD_PLAN_BOOL_FIELDS = ("fatal", "release")
NATIVE_DYNAMIC_BUILD_PLAN_INTEGER_FIELDS = ("package_count",)
NATIVE_DYNAMIC_BUILD_PLAN_NON_NEGATIVE_INTEGER_FIELDS = ("package_count",)
NATIVE_DYNAMIC_BUILD_PLAN_STRING_ARRAY_FIELDS = (
    "build_features",
    "diagnostics",
)
NATIVE_DYNAMIC_BUILD_PLAN_OBJECT_ARRAY_FIELDS = ("packages",)
NATIVE_DYNAMIC_BUILD_PLAN_CARGO_PROFILES = ("debug", "release")
NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_STRING_FIELDS = (
    "cargo_profile",
    "target_dir",
    "workspace_manifest",
)
NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_BOOL_FIELDS = (
    "fatal",
    "release",
)
NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_INTEGER_FIELDS = (
    "package_count",
)
NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS = (
    "build_features",
    "diagnostics",
)
NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS = (
    "packages",
)
NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_FIELDS = (
    "cargo_profile",
    "command",
    "crate_name",
    "expected_loadable_artifact",
    "features",
    "manifest_path",
    "package_id",
    "release",
    "target_dir",
    "workspace_manifest",
)
NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_STRING_FIELDS = (
    "cargo_profile",
    "crate_name",
    "expected_loadable_artifact",
    "manifest_path",
    "package_id",
    "target_dir",
    "workspace_manifest",
)
NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_BOOL_FIELDS = ("release",)
NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_STRING_ARRAY_FIELDS = (
    "command",
    "features",
)
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
NATIVE_DYNAMIC_BUILD_PLAN_COMMAND_FORBIDDEN_PACKAGE_FLAGS = (
    "--workspace",
    "--all",
    "--exclude",
)
NATIVE_DYNAMIC_BUILD_PLAN_COMMAND_FORBIDDEN_PROFILE_FLAGS = ("--profile",)

SchemaDiagnostic = Callable[[str, Any], list[str]]

def native_dynamic_build_plan_execution_state_diagnostics(
    build_plan: Any,
    build_execution: Any,
    report_fatal: Any,
) -> list[str]:
    if not (
        report_fatal is False
        and isinstance(build_plan, dict)
        and isinstance(build_execution, dict)
        and build_execution.get("enabled") is True
        and build_plan.get("fatal") is True
    ):
        return []
    return [
        "native_dynamic report native_build_plan.fatal must be False "
        "when native_build_execution.enabled is True and "
        "NativeDynamic report fatal is False"
    ]


def native_dynamic_build_plan_schema_diagnostics(
    label: str,
    build_plan: dict[str, Any],
    *,
    validate_bool_schema_diagnostics: SchemaDiagnostic,
    validate_integer_schema_diagnostics: SchemaDiagnostic,
    validate_string_schema_diagnostics: SchemaDiagnostic,
    validate_string_array_schema_diagnostics: SchemaDiagnostic,
    validate_object_array_schema_diagnostics: SchemaDiagnostic,
    require_release_evidence: bool = False,
    target_platform: object = None,
) -> list[str]:
    diagnostics: list[str] = []
    diagnostics.extend(
        f"{label} unknown field {field}"
        for field in sorted(build_plan)
        if field not in NATIVE_DYNAMIC_BUILD_PLAN_FIELDS
    )
    if require_release_evidence:
        for field in NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_STRING_FIELDS:
            if field not in build_plan or build_plan.get(field) is None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"{label}.{field}",
                        build_plan.get(field),
                    )
                )
            elif (
                isinstance(build_plan.get(field), str)
                and not build_plan.get(field).strip()
            ):
                diagnostics.append(f"{label}.{field} must be a non-empty string")
        for field in NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_BOOL_FIELDS:
            if field not in build_plan or build_plan.get(field) is None:
                diagnostics.extend(
                    validate_bool_schema_diagnostics(
                        f"{label}.{field}",
                        build_plan.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_INTEGER_FIELDS:
            if field not in build_plan or build_plan.get(field) is None:
                diagnostics.extend(
                    validate_integer_schema_diagnostics(
                        f"{label}.{field}",
                        build_plan.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS:
            if field not in build_plan or build_plan.get(field) is None:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"{label}.{field}",
                        build_plan.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS:
            if field not in build_plan or build_plan.get(field) is None:
                diagnostics.extend(
                    validate_object_array_schema_diagnostics(
                        f"{label}.{field}",
                        build_plan.get(field),
                    )
                )
    for field in NATIVE_DYNAMIC_BUILD_PLAN_STRING_FIELDS:
        if field in build_plan and build_plan.get(field) is not None:
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"{label}.{field}",
                    build_plan.get(field),
                )
            )
            diagnostics.extend(
                native_dynamic_build_plan_trimmed_non_empty_string_schema_diagnostics(
                    f"{label}.{field}",
                    build_plan.get(field),
                )
            )
    for field in NATIVE_DYNAMIC_BUILD_PLAN_BOOL_FIELDS:
        if field in build_plan and build_plan.get(field) is not None:
            diagnostics.extend(
                validate_bool_schema_diagnostics(
                    f"{label}.{field}",
                    build_plan.get(field),
                )
            )
    for field in NATIVE_DYNAMIC_BUILD_PLAN_INTEGER_FIELDS:
        if field in build_plan and build_plan.get(field) is not None:
            diagnostics.extend(
                validate_integer_schema_diagnostics(
                    f"{label}.{field}",
                    build_plan.get(field),
                )
            )
    diagnostics.extend(
        table_non_negative_integer_schema_diagnostics(
            label,
            build_plan,
            NATIVE_DYNAMIC_BUILD_PLAN_NON_NEGATIVE_INTEGER_FIELDS,
        )
    )
    for field in NATIVE_DYNAMIC_BUILD_PLAN_STRING_ARRAY_FIELDS:
        if field in build_plan and build_plan.get(field) is not None:
            field_label = f"{label}.{field}"
            if field == "build_features":
                diagnostics.extend(
                    native_dynamic_build_plan_feature_array_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
            elif field == "diagnostics":
                diagnostics.extend(
                    native_dynamic_build_plan_diagnostics_array_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
            else:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
            if field == "build_features":
                diagnostics.extend(
                    string_array_no_blank_entries_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
                diagnostics.extend(
                    string_array_trimmed_non_empty_entries_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
                diagnostics.extend(
                    string_array_unique_entries_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
            if field == "diagnostics":
                diagnostics.extend(
                    string_array_no_blank_entries_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
                diagnostics.extend(
                    string_array_trimmed_non_empty_entries_schema_diagnostics(
                        field_label,
                        build_plan.get(field),
                    )
                )
    diagnostics.extend(
        native_dynamic_build_plan_profile_release_diagnostics(
            label,
            build_plan,
        )
    )
    diagnostics.extend(native_dynamic_non_fatal_report_diagnostics(label, build_plan))
    diagnostics.extend(native_dynamic_fatal_report_diagnostics(label, build_plan))
    packages = build_plan.get("packages")
    if "packages" in build_plan and packages is not None:
        diagnostics.extend(
            validate_object_array_schema_diagnostics(
                f"{label}.packages",
                packages,
            )
        )
    diagnostics.extend(
        native_dynamic_build_audit_package_count_diagnostics(label, build_plan)
    )
    if isinstance(packages, list):
        diagnostics.extend(
            native_dynamic_build_audit_package_id_uniqueness_diagnostics(
                f"{label}.packages",
                packages,
            )
        )
        diagnostics.extend(
            native_dynamic_build_plan_packages_schema_diagnostics(
                f"{label}.packages",
                packages,
                validate_bool_schema_diagnostics=validate_bool_schema_diagnostics,
                validate_string_schema_diagnostics=validate_string_schema_diagnostics,
                validate_string_array_schema_diagnostics=(
                    validate_string_array_schema_diagnostics
                ),
            )
        )
        diagnostics.extend(
            native_dynamic_build_plan_package_header_diagnostics(
                label,
                build_plan,
                packages,
            )
        )
        diagnostics.extend(
            native_dynamic_build_plan_package_expected_artifact_diagnostics(
                f"{label}.packages",
                packages,
                target_platform,
            )
        )
    return diagnostics


def native_dynamic_build_plan_profile_release_diagnostics(
    label: str,
    build_plan: dict[str, Any],
) -> list[str]:
    cargo_profile = build_plan.get("cargo_profile")
    release = build_plan.get("release")
    if not native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
        cargo_profile
    ):
        return []
    if cargo_profile not in NATIVE_DYNAMIC_BUILD_PLAN_CARGO_PROFILES:
        return [f"{label}.cargo_profile must be debug or release"]
    if type(release) is bool and release != (cargo_profile == "release"):
        return [f"{label}.release must match cargo_profile"]
    return []


def native_dynamic_build_plan_package_header_diagnostics(
    label: str,
    build_plan: dict[str, Any],
    packages: list[Any],
) -> list[str]:
    diagnostics: list[str] = []
    field_pairs = (
        ("workspace_manifest", "workspace_manifest"),
        ("target_dir", "target_dir"),
        ("cargo_profile", "cargo_profile"),
        ("release", "release"),
        ("build_features", "features"),
    )
    for index, package in enumerate(packages):
        if not isinstance(package, dict):
            continue
        package_label = f"{label}.packages[{index}]"
        for plan_field, package_field in field_pairs:
            diagnostics.extend(
                native_dynamic_build_plan_package_header_field_diagnostics(
                    label,
                    package_label,
                    plan_field,
                    package_field,
                    build_plan,
                    package,
                )
            )
    return diagnostics


def native_dynamic_build_plan_package_expected_artifact_diagnostics(
    label: str,
    packages: list[Any],
    target_platform: object,
) -> list[str]:
    if not isinstance(target_platform, str) or not target_platform.strip():
        return []
    diagnostics: list[str] = []
    for index, package in enumerate(packages):
        if not isinstance(package, dict):
            continue
        package_label = f"{label}[{index}]"
        target_dir = package.get("target_dir")
        cargo_profile = package.get("cargo_profile")
        crate_name = package.get("crate_name")
        expected_loadable_artifact = package.get("expected_loadable_artifact")
        if not (
            native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
                target_dir
            )
            and isinstance(cargo_profile, str)
            and cargo_profile in NATIVE_DYNAMIC_BUILD_PLAN_CARGO_PROFILES
            and native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
                crate_name
            )
            and native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
                expected_loadable_artifact
            )
        ):
            continue
        target_dir_normalized = native_dynamic_normalized_path(target_dir).rstrip("/")
        derived_artifact = (
            f"{target_dir_normalized}/{cargo_profile}/"
            f"{platform_dynamic_library_name(crate_name, target_platform)}"
        )
        actual_artifact = native_dynamic_normalized_path(
            expected_loadable_artifact
        )
        if actual_artifact != derived_artifact:
            diagnostics.append(
                f"{package_label}.expected_loadable_artifact "
                f"{actual_artifact} does not match derived artifact "
                f"{derived_artifact}"
            )
    return diagnostics


def native_dynamic_normalized_path(value: str) -> str:
    return value.replace("\\", "/")


def native_dynamic_build_plan_package_header_field_diagnostics(
    plan_label: str,
    package_label: str,
    plan_field: str,
    package_field: str,
    build_plan: dict[str, Any],
    package: dict[str, Any],
) -> list[str]:
    plan_value = build_plan.get(plan_field)
    package_value = package.get(package_field)
    if not (
        native_dynamic_build_plan_header_value_is_comparable(plan_value)
        and native_dynamic_build_plan_header_value_is_comparable(package_value)
    ):
        return []
    if package_value == plan_value:
        return []
    return [
        f"{package_label}.{package_field} {package_value} does not match "
        f"{plan_label}.{plan_field} {plan_value}"
    ]


def native_dynamic_build_plan_header_value_is_comparable(value: Any) -> bool:
    if isinstance(value, str):
        return native_dynamic_build_plan_trimmed_non_empty_string_is_schema_clean(
            value
        )
    if type(value) is bool:
        return True
    if isinstance(value, list):
        return native_dynamic_build_plan_string_array_is_schema_clean(
            value,
            allow_empty=True,
            require_unique=True,
        )
    return False


def native_dynamic_build_plan_packages_schema_diagnostics(
    label: str,
    packages: list[Any],
    *,
    validate_bool_schema_diagnostics: SchemaDiagnostic,
    validate_string_schema_diagnostics: SchemaDiagnostic,
    validate_string_array_schema_diagnostics: SchemaDiagnostic,
) -> list[str]:
    diagnostics: list[str] = []
    known_fields = set(NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_FIELDS)
    for index, package in enumerate(packages):
        if not isinstance(package, dict):
            continue
        package_label = f"{label}[{index}]"
        diagnostics.extend(
            f"{package_label} unknown field {field}"
            for field in sorted(package)
            if field not in known_fields
        )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_STRING_FIELDS:
            if field not in package or package.get(field) is None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
            elif (
                isinstance(package.get(field), str)
                and not package.get(field).strip()
            ):
                diagnostics.append(
                    f"{package_label}.{field} must be a non-empty string"
                )
            if field in package and package.get(field) is not None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
                diagnostics.extend(
                    native_dynamic_build_plan_trimmed_non_empty_string_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_BOOL_FIELDS:
            if field not in package or package.get(field) is None:
                diagnostics.extend(
                    validate_bool_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
            if field in package and package.get(field) is not None:
                diagnostics.extend(
                    validate_bool_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
        for field in NATIVE_DYNAMIC_BUILD_PLAN_PACKAGE_STRING_ARRAY_FIELDS:
            if field not in package or package.get(field) is None:
                diagnostics.extend(
                    validate_string_array_schema_diagnostics(
                        f"{package_label}.{field}",
                        package.get(field),
                    )
                )
            if field in package and package.get(field) is not None:
                field_label = f"{package_label}.{field}"
                if field == "features":
                    diagnostics.extend(
                        native_dynamic_build_plan_feature_array_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                elif field == "command":
                    diagnostics.extend(
                        native_dynamic_build_plan_command_array_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                else:
                    diagnostics.extend(
                        validate_string_array_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                if field == "command":
                    diagnostics.extend(
                        non_empty_string_array_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                    diagnostics.extend(
                        string_array_trimmed_non_empty_entries_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                if field == "features":
                    diagnostics.extend(
                        string_array_no_blank_entries_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                    diagnostics.extend(
                        string_array_trimmed_non_empty_entries_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
                    diagnostics.extend(
                        string_array_unique_entries_schema_diagnostics(
                            field_label,
                            package.get(field),
                        )
                    )
        diagnostics.extend(
            native_dynamic_build_plan_package_command_semantics_diagnostics(
                package_label,
                package,
            )
        )
    return diagnostics


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


def native_dynamic_build_plan_trimmed_non_empty_string_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if isinstance(value, str) and value.strip() and value.strip() != value:
        return [f"{label} must be a non-empty trimmed string"]
    return []


def native_dynamic_build_plan_feature_array_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, list):
        return [f"{label} must be a string array"]
    return [
        f"{label}[{index}] must be a string"
        for index, entry in enumerate(value)
        if not isinstance(entry, str)
    ]


def native_dynamic_build_plan_command_array_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, list):
        return [f"{label} must be a string array"]
    return [
        f"{label}[{index}] must be a string"
        for index, entry in enumerate(value)
        if not isinstance(entry, str)
    ]


def native_dynamic_build_plan_diagnostics_array_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, list):
        return [f"{label} must be a string array"]
    return [
        f"{label}[{index}] must be a string"
        for index, entry in enumerate(value)
        if not isinstance(entry, str)
    ]


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

