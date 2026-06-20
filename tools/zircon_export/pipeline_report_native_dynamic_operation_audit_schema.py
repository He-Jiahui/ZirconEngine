"""Schema diagnostics for NativeDynamic operation audit evidence."""

from __future__ import annotations

from typing import Any

from .export_template import (
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
)
from .native_signing import native_dynamic_signing_platform_allowed
from .pipeline_report_schema_table import (
    non_empty_string_array_schema_diagnostics,
    object_array_schema_diagnostics,
    optional_fields,
    string_array_no_blank_entries_schema_diagnostics,
    table_bool_schema_diagnostics,
    table_field_schema_diagnostics,
    table_integer_schema_diagnostics,
    table_string_array_schema_diagnostics,
    table_string_schema_diagnostics,
    table_unknown_field_diagnostics,
)

NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS = (
    "native_signing",
    "native_notarization",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_FIELDS = (
    "allowed_platforms",
    "enabled",
    "fatal",
    "package_count",
    "platform_allowed",
    "profile",
    "target_platform",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_FIELDS = (
    "profile",
    "target_platform",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_NON_EMPTY_STRING_FIELDS = (
    "target_platform",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_NON_EMPTY_STRING_FIELDS = (
    "profile",
    *NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_NON_EMPTY_STRING_FIELDS,
)

NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_BOOL_FIELDS = (
    "enabled",
    "fatal",
    "platform_allowed",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_BOOL_FIELDS = (
    "enabled",
    "fatal",
    "platform_allowed",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_INTEGER_FIELDS = ("package_count",)
NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_INTEGER_FIELDS = ("package_count",)

NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_ARRAY_FIELDS = (
    "allowed_platforms",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_STRING_ARRAY_FIELDS = (
    "allowed_platforms",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_FIELDS = (
    *NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_FIELDS,
    "diagnostics",
    "packages",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_STRING_ARRAY_FIELDS = (
    *NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_ARRAY_FIELDS,
    "diagnostics",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_REQUIRED_STRING_ARRAY_FIELDS = (
    "diagnostics",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_REQUIRED_OBJECT_ARRAY_FIELDS = (
    "packages",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_FIELDS = (
    "artifact_count",
    "artifacts",
    "package_id",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_STRING_FIELDS = ("package_id",)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_STRING_FIELDS = ("package_id",)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_NON_EMPTY_STRING_FIELDS = (
    "package_id",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_INTEGER_FIELDS = ("artifact_count",)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_INTEGER_FIELDS = (
    "artifact_count",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_OBJECT_ARRAY_FIELDS = (
    "artifacts",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_FIELDS = (
    "after_sha256",
    "artifact",
    "before_sha256",
    "command",
    "exit_code",
    "package_relative_artifact",
    "stderr",
    "stdout",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_STRING_FIELDS = (
    "after_sha256",
    "artifact",
    "before_sha256",
    "package_relative_artifact",
    "stderr",
    "stdout",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_STRING_FIELDS = (
    "after_sha256",
    "artifact",
    "before_sha256",
    "package_relative_artifact",
    "stderr",
    "stdout",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_NON_EMPTY_STRING_FIELDS = (
    "after_sha256",
    "artifact",
    "before_sha256",
    "package_relative_artifact",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_SHA256_FIELDS = (
    "after_sha256",
    "before_sha256",
)

NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_INTEGER_FIELDS = ("exit_code",)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_INTEGER_FIELDS = (
    "exit_code",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_STRING_ARRAY_FIELDS = ("command",)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_STRING_ARRAY_FIELDS = (
    "command",
)


def platform_bundle_native_plugins_operation_audit_schema_diagnostics(
    label: str,
    audit: dict[str, Any],
) -> list[str]:
    diagnostics = table_unknown_field_diagnostics(
        label,
        audit,
        NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_FIELDS,
    )
    diagnostics.extend(
        table_bool_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_BOOL_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_integer_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_INTEGER_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_non_negative_integer_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_INTEGER_FIELDS,
        )
    )
    diagnostics.extend(
        table_string_array_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_STRING_ARRAY_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_field_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_ARRAY_FIELDS,
            string_array_no_blank_entries_schema_diagnostics,
        )
    )
    diagnostics.extend(
        table_field_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_ARRAY_FIELDS,
            string_array_unique_entries_schema_diagnostics,
        )
    )
    diagnostics.extend(
        table_string_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_required_non_empty_string_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_NON_EMPTY_STRING_FIELDS,
        )
    )
    diagnostics.extend(operation_audit_platform_allowed_schema_diagnostics(label, audit))
    diagnostics.extend(
        table_bool_schema_diagnostics(
            label,
            audit,
            optional_fields(
                NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_BOOL_FIELDS,
                NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_BOOL_FIELDS,
            ),
        )
    )
    diagnostics.extend(
        table_integer_schema_diagnostics(
            label,
            audit,
            optional_fields(
                NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_INTEGER_FIELDS,
                NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_INTEGER_FIELDS,
            ),
        )
    )
    diagnostics.extend(
        table_string_array_schema_diagnostics(
            label,
            audit,
            optional_fields(
                NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_ARRAY_FIELDS,
                NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_STRING_ARRAY_FIELDS,
            ),
        )
    )
    return diagnostics


def native_dynamic_operation_audit_stage_schema_diagnostics(
    label: str,
    audit: dict[str, Any],
) -> list[str]:
    diagnostics = table_unknown_field_diagnostics(
        label,
        audit,
        NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_FIELDS,
    )
    diagnostics.extend(
        table_bool_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_BOOL_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_integer_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_INTEGER_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_non_negative_integer_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_INTEGER_FIELDS,
        )
    )
    diagnostics.extend(
        table_string_array_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_STRING_ARRAY_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_field_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_ARRAY_FIELDS,
            string_array_no_blank_entries_schema_diagnostics,
        )
    )
    diagnostics.extend(
        table_field_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_ARRAY_FIELDS,
            string_array_unique_entries_schema_diagnostics,
        )
    )
    diagnostics.extend(
        table_string_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_required_non_empty_string_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_NON_EMPTY_STRING_FIELDS,
        )
    )
    diagnostics.extend(operation_audit_platform_allowed_schema_diagnostics(label, audit))
    diagnostics.extend(
        table_bool_schema_diagnostics(
            label,
            audit,
            optional_fields(
                NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_BOOL_FIELDS,
                NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_BOOL_FIELDS,
            ),
        )
    )
    diagnostics.extend(
        table_integer_schema_diagnostics(
            label,
            audit,
            optional_fields(
                NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_INTEGER_FIELDS,
                NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_INTEGER_FIELDS,
            ),
        )
    )
    diagnostics.extend(
        table_string_array_schema_diagnostics(
            label,
            audit,
            (
                *NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_STRING_ARRAY_FIELDS,
                *NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_REQUIRED_STRING_ARRAY_FIELDS,
            ),
            require_present=True,
        )
    )
    diagnostics.extend(
        table_string_array_schema_diagnostics(
            label,
            audit,
            optional_fields(
                NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_STRING_ARRAY_FIELDS,
                (
                    *NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_STRING_ARRAY_FIELDS,
                    *NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_REQUIRED_STRING_ARRAY_FIELDS,
                ),
            ),
        )
    )
    diagnostics.extend(
        table_field_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_REQUIRED_STRING_ARRAY_FIELDS,
            string_array_no_blank_entries_schema_diagnostics,
        )
    )
    diagnostics.extend(
        object_array_schema_diagnostics(
            label,
            audit,
            "packages",
            NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_FIELDS,
            string_fields=NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_STRING_FIELDS,
            integer_fields=NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_INTEGER_FIELDS,
            required_string_fields=(
                NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_STRING_FIELDS
            ),
            required_integer_fields=(
                NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_INTEGER_FIELDS
            ),
            required_object_array_fields=(
                NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_OBJECT_ARRAY_FIELDS
            ),
            require_present=True,
        )
    )
    diagnostics.extend(
        object_array_non_negative_integer_schema_diagnostics(
            label,
            audit,
            "packages",
            NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_INTEGER_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_required_non_empty_string_schema_diagnostics(
            label,
            audit,
            "packages",
            NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_PACKAGE_REQUIRED_NON_EMPTY_STRING_FIELDS,
        )
    )
    packages = audit.get("packages")
    if isinstance(packages, list):
        for index, package in enumerate(packages):
            if not isinstance(package, dict):
                continue
            diagnostics.extend(
                object_array_schema_diagnostics(
                    f"{label} packages[{index}]",
                    package,
                    "artifacts",
                    NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_FIELDS,
                    string_fields=(
                        NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_STRING_FIELDS
                    ),
                    integer_fields=(
                        NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_INTEGER_FIELDS
                    ),
                    string_array_fields=(
                        NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_STRING_ARRAY_FIELDS
                    ),
                    required_string_fields=(
                        NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_STRING_FIELDS
                    ),
                    required_integer_fields=(
                        NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_INTEGER_FIELDS
                    ),
                    required_string_array_fields=(
                        NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_STRING_ARRAY_FIELDS
                    ),
                )
            )
            diagnostics.extend(
                object_array_required_non_empty_string_schema_diagnostics(
                    f"{label} packages[{index}]",
                    package,
                    "artifacts",
                    NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_NON_EMPTY_STRING_FIELDS,
                )
            )
            diagnostics.extend(
                object_array_sha256_hex_string_schema_diagnostics(
                    f"{label} packages[{index}]",
                    package,
                    "artifacts",
                    NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_SHA256_FIELDS,
                )
            )
            diagnostics.extend(
                object_array_unique_string_field_schema_diagnostics(
                    f"{label} packages[{index}]",
                    package,
                    "artifacts",
                    "package_relative_artifact",
                )
            )
            artifacts = package.get("artifacts")
            if not isinstance(artifacts, list):
                continue
            for artifact_index, artifact in enumerate(artifacts):
                if not isinstance(artifact, dict):
                    continue
                diagnostics.extend(
                    non_empty_string_array_schema_diagnostics(
                        f"{label} packages[{index}] "
                        f"artifacts[{artifact_index}].command",
                        artifact.get("command"),
                    )
                )
                diagnostics.extend(
                    artifact_safe_relative_path_schema_diagnostics(
                        f"{label} packages[{index}] artifacts[{artifact_index}]",
                        artifact,
                    )
                )
                if audit.get("fatal") is False:
                    diagnostics.extend(
                        artifact_exit_code_success_schema_diagnostics(
                            f"{label} packages[{index}] "
                            f"artifacts[{artifact_index}]",
                            artifact,
                        )
                    )
    return diagnostics


def table_required_non_empty_string_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = table.get(field)
        if isinstance(value, str) and not value.strip():
            diagnostics.append(f"{label}.{field} must be a non-empty string")
    return diagnostics


def operation_audit_platform_allowed_schema_diagnostics(
    label: str,
    audit: dict[str, Any],
) -> list[str]:
    if audit.get("enabled") is not True:
        return []
    target_platform = audit.get("target_platform")
    allowed_platforms = audit.get("allowed_platforms")
    platform_allowed = audit.get("platform_allowed")
    if (
        not isinstance(target_platform, str)
        or not target_platform.strip()
        or not isinstance(allowed_platforms, list)
        or not all(
            isinstance(platform, str) and platform.strip()
            for platform in allowed_platforms
        )
        or type(platform_allowed) is not bool
    ):
        return []
    computed_platform_allowed = native_dynamic_signing_platform_allowed(
        target_platform,
        allowed_platforms,
    )
    if platform_allowed == computed_platform_allowed:
        return []
    return [
        f"{label}.platform_allowed does not match target_platform "
        "and allowed_platforms"
    ]


def object_array_required_non_empty_string_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    fields: tuple[str, ...],
) -> list[str]:
    value = table.get(field)
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_required_non_empty_string_schema_diagnostics(
                f"{label} {field}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics


def table_sha256_hex_string_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = table.get(field)
        if isinstance(value, str) and value.strip() and not is_sha256_hex(value):
            diagnostics.append(f"{label}.{field} must be a SHA-256 hex digest")
    return diagnostics


def object_array_sha256_hex_string_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    fields: tuple[str, ...],
) -> list[str]:
    value = table.get(field)
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_sha256_hex_string_schema_diagnostics(
                f"{label} {field}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics


def string_array_unique_entries_schema_diagnostics(
    label: str,
    value: Any,
) -> list[str]:
    if not isinstance(value, list) or not all(isinstance(item, str) for item in value):
        return []
    seen: set[str] = set()
    for item in value:
        if item in seen:
            return [f"{label} must not contain duplicate entries"]
        seen.add(item)
    return []


def object_array_unique_string_field_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    value_field: str,
) -> list[str]:
    value = table.get(field)
    if not isinstance(value, list):
        return []

    entries: list[str] = []
    for entry in value:
        if not isinstance(entry, dict):
            return []
        field_value = entry.get(value_field)
        if not isinstance(field_value, str):
            return []
        entries.append(field_value)

    return string_array_unique_entries_schema_diagnostics(
        f"{label} {field}.{value_field}",
        entries,
    )


def artifact_safe_relative_path_schema_diagnostics(
    label: str,
    artifact: dict[str, Any],
) -> list[str]:
    value = artifact.get("package_relative_artifact")
    if not isinstance(value, str):
        return []
    if not value.strip():
        return []
    if is_safe_relative_path(normalize_relative_path(value)):
        return []
    return [f"{label}.package_relative_artifact must be a safe relative path"]


def artifact_exit_code_success_schema_diagnostics(
    label: str,
    artifact: dict[str, Any],
) -> list[str]:
    exit_code = artifact.get("exit_code")
    if type(exit_code) is not int or exit_code == 0:
        return []
    return [f"{label}.exit_code must be 0 for non-fatal operation audit"]


def table_non_negative_integer_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = table.get(field)
        if type(value) is int and value < 0:
            diagnostics.append(f"{label}.{field} must be non-negative")
    return diagnostics


def object_array_non_negative_integer_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    fields: tuple[str, ...],
) -> list[str]:
    value = table.get(field)
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_non_negative_integer_schema_diagnostics(
                f"{label} {field}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics
