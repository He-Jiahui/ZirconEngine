"""Schema diagnostics for NativeDynamic payload evidence in final reports."""

from __future__ import annotations

from collections.abc import Callable
from typing import Any

from .pipeline_report_schema_primitives import (
    validate_bool_schema_diagnostics,
    validate_integer_schema_diagnostics,
    validate_object_array_schema_diagnostics,
    validate_object_schema_diagnostics,
    validate_string_array_schema_diagnostics,
    validate_string_schema_diagnostics,
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

NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_INTEGER_FIELDS = ("exit_code",)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_INTEGER_FIELDS = (
    "exit_code",
)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_STRING_ARRAY_FIELDS = ("command",)
NATIVE_DYNAMIC_OPERATION_AUDIT_STAGE_ARTIFACT_REQUIRED_STRING_ARRAY_FIELDS = (
    "command",
)

NATIVE_DYNAMIC_PAYLOAD_FIELDS = (
    "bundle_path",
    "content_hash",
    "file_count",
    "file_manifest",
    "loader_manifest",
    "materialized_packages",
    "package_count",
    "source",
    "stage_report",
    *NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS,
)

NATIVE_DYNAMIC_PAYLOAD_STRING_FIELDS = (
    "bundle_path",
    "content_hash",
    "loader_manifest",
    "source",
    "stage_report",
)

NATIVE_DYNAMIC_PAYLOAD_REQUIRED_STRING_FIELDS = ("loader_manifest",)

NATIVE_DYNAMIC_PAYLOAD_INTEGER_FIELDS = (
    "file_count",
    "package_count",
)

NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_FIELDS = (
    "destination",
    "loadable_artifact_count",
    "loadable_artifacts",
    "package_id",
    "package_report",
    "source",
)

NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_FIELDS = (
    "destination",
    "package_id",
    "package_report",
    "source",
)

NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_INTEGER_FIELDS = (
    "loadable_artifact_count",
)

NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_ARRAY_FIELDS = (
    "loadable_artifacts",
)
NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_STRING_FIELDS = (
    "destination",
    "package_id",
)
NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_INTEGER_FIELDS = (
    "loadable_artifact_count",
)
NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_STRING_ARRAY_FIELDS = (
    "loadable_artifacts",
)

NATIVE_DYNAMIC_FILE_MANIFEST_FIELDS = (
    "bytes",
    "path",
    "sha256",
)

NATIVE_DYNAMIC_FILE_MANIFEST_STRING_FIELDS = (
    "path",
    "sha256",
)

NATIVE_DYNAMIC_FILE_MANIFEST_INTEGER_FIELDS = ("bytes",)
NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_STRING_FIELDS = (
    "path",
    "sha256",
)
NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_INTEGER_FIELDS = ("bytes",)


def platform_bundle_native_plugins_payload_schema_diagnostics(
    payload: dict[str, Any],
    label: str = "PlatformBundle report native_plugins_payload",
) -> list[str]:
    diagnostics = table_unknown_field_diagnostics(
        label,
        payload,
        NATIVE_DYNAMIC_PAYLOAD_FIELDS,
    )
    diagnostics.extend(
        table_string_schema_diagnostics(
            label,
            payload,
            NATIVE_DYNAMIC_PAYLOAD_REQUIRED_STRING_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_string_schema_diagnostics(
            label,
            payload,
            optional_fields(
                NATIVE_DYNAMIC_PAYLOAD_STRING_FIELDS,
                NATIVE_DYNAMIC_PAYLOAD_REQUIRED_STRING_FIELDS,
            ),
        )
    )
    diagnostics.extend(
        table_integer_schema_diagnostics(
            label,
            payload,
            NATIVE_DYNAMIC_PAYLOAD_INTEGER_FIELDS,
        )
    )
    diagnostics.extend(
        platform_bundle_native_plugins_payload_file_manifest_schema_diagnostics(
            payload,
            label=label,
        )
    )
    diagnostics.extend(
        platform_bundle_native_plugins_payload_materialized_packages_schema_diagnostics(
            payload,
            label=label,
        )
    )
    for field in NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS:
        audit = payload.get(field)
        if audit is None:
            continue
        audit_label = f"{label} {field}"
        if not isinstance(audit, dict):
            diagnostics.append(f"{audit_label} must be an object")
            continue
        diagnostics.extend(
            platform_bundle_native_plugins_operation_audit_schema_diagnostics(
                audit_label,
                audit,
            )
        )
    return diagnostics


def platform_bundle_native_plugins_payload_file_manifest_schema_diagnostics(
    payload: dict[str, Any],
    label: str = "PlatformBundle report native_plugins_payload",
) -> list[str]:
    return native_dynamic_file_manifest_schema_diagnostics(
        label,
        payload,
    )


def platform_bundle_native_plugins_payload_materialized_packages_schema_diagnostics(
    payload: dict[str, Any],
    label: str = "PlatformBundle report native_plugins_payload",
) -> list[str]:
    return native_dynamic_materialized_packages_schema_diagnostics(
        label,
        payload,
    )


def native_dynamic_file_manifest_schema_diagnostics(
    label: str,
    payload: dict[str, Any],
) -> list[str]:
    return object_array_schema_diagnostics(
        label,
        payload,
        "file_manifest",
        NATIVE_DYNAMIC_FILE_MANIFEST_FIELDS,
        string_fields=NATIVE_DYNAMIC_FILE_MANIFEST_STRING_FIELDS,
        integer_fields=NATIVE_DYNAMIC_FILE_MANIFEST_INTEGER_FIELDS,
        required_string_fields=NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_STRING_FIELDS,
        required_integer_fields=NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_INTEGER_FIELDS,
    )


def native_dynamic_materialized_packages_schema_diagnostics(
    label: str,
    payload: dict[str, Any],
) -> list[str]:
    return object_array_schema_diagnostics(
        label,
        payload,
        "materialized_packages",
        NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_FIELDS,
        string_fields=NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_FIELDS,
        integer_fields=NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_INTEGER_FIELDS,
        string_array_fields=NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_ARRAY_FIELDS,
        required_string_fields=(
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_STRING_FIELDS
        ),
        required_integer_fields=(
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_INTEGER_FIELDS
        ),
        required_string_array_fields=(
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_STRING_ARRAY_FIELDS
        ),
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
        table_string_array_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_STRING_ARRAY_FIELDS,
            require_present=True,
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
        table_string_array_schema_diagnostics(
            label,
            audit,
            NATIVE_DYNAMIC_OPERATION_AUDIT_SUMMARY_REQUIRED_STRING_ARRAY_FIELDS,
            require_present=True,
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
    return diagnostics


def object_array_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    allowed_fields: tuple[str, ...],
    *,
    string_fields: tuple[str, ...] = (),
    integer_fields: tuple[str, ...] = (),
    string_array_fields: tuple[str, ...] = (),
    required_string_fields: tuple[str, ...] = (),
    required_integer_fields: tuple[str, ...] = (),
    required_string_array_fields: tuple[str, ...] = (),
    required_object_array_fields: tuple[str, ...] = (),
    require_present: bool = False,
) -> list[str]:
    value = table.get(field)
    if value is None and not require_present:
        return []
    field_label = f"{label} {field}"
    if not isinstance(value, list):
        return [f"{field_label} must be an object array"]
    diagnostics: list[str] = []
    diagnostics.extend(validate_object_array_schema_diagnostics(field_label, value))
    diagnostics.extend(
        sequence_unknown_field_diagnostics(
            field_label,
            value,
            allowed_fields,
        )
    )
    diagnostics.extend(
        sequence_required_string_schema_diagnostics(
            field_label,
            value,
            required_string_fields,
        )
    )
    diagnostics.extend(
        sequence_required_integer_schema_diagnostics(
            field_label,
            value,
            required_integer_fields,
        )
    )
    diagnostics.extend(
        sequence_required_string_array_schema_diagnostics(
            field_label,
            value,
            required_string_array_fields,
        )
    )
    diagnostics.extend(
        sequence_required_object_array_schema_diagnostics(
            field_label,
            value,
            required_object_array_fields,
        )
    )
    diagnostics.extend(
        sequence_string_schema_diagnostics(
            field_label,
            value,
            optional_fields(string_fields, required_string_fields),
        )
    )
    diagnostics.extend(
        sequence_integer_schema_diagnostics(
            field_label,
            value,
            optional_fields(integer_fields, required_integer_fields),
        )
    )
    diagnostics.extend(
        sequence_string_array_schema_diagnostics(
            field_label,
            value,
            optional_fields(string_array_fields, required_string_array_fields),
        )
    )
    return diagnostics


def table_unknown_field_diagnostics(
    label: str,
    table: dict[str, Any],
    known_fields: tuple[str, ...],
) -> list[str]:
    known_field_set = set(known_fields)
    return [
        f"{label} unknown field {field}"
        for field in sorted(table)
        if field not in known_field_set
    ]


def sequence_unknown_field_diagnostics(
    label: str,
    value: object,
    known_fields: tuple[str, ...],
) -> list[str]:
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_unknown_field_diagnostics(
                f"{label}[{index}]",
                entry,
                known_fields,
            )
        )
    return diagnostics


def table_string_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    *,
    require_present: bool = False,
) -> list[str]:
    return table_field_schema_diagnostics(
        label,
        table,
        fields,
        validate_string_schema_diagnostics,
        require_present=require_present,
    )


def table_integer_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    *,
    require_present: bool = False,
) -> list[str]:
    return table_field_schema_diagnostics(
        label,
        table,
        fields,
        validate_integer_schema_diagnostics,
        require_present=require_present,
    )


def table_bool_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    *,
    require_present: bool = False,
) -> list[str]:
    return table_field_schema_diagnostics(
        label,
        table,
        fields,
        validate_bool_schema_diagnostics,
        require_present=require_present,
    )


def table_object_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    *,
    require_present: bool = False,
) -> list[str]:
    return table_field_schema_diagnostics(
        label,
        table,
        fields,
        validate_object_schema_diagnostics,
        require_present=require_present,
    )


def table_string_array_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    *,
    require_present: bool = False,
) -> list[str]:
    return table_field_schema_diagnostics(
        label,
        table,
        fields,
        validate_string_array_schema_diagnostics,
        require_present=require_present,
    )


def table_field_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    validate_schema: Callable[[str, Any], list[str]],
    *,
    require_present: bool = False,
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = table.get(field)
        if value is not None or require_present:
            diagnostics.extend(validate_schema(typed_field_label(label, field), value))
    return diagnostics


def typed_field_label(label: str, field: str) -> str:
    return f"{label}.{field}"


def optional_fields(
    fields: tuple[str, ...],
    required_fields: tuple[str, ...],
) -> tuple[str, ...]:
    required_field_set = set(required_fields)
    return tuple(field for field in fields if field not in required_field_set)


def sequence_string_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_string_schema_diagnostics(
                f"{label}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics


def sequence_required_string_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
) -> list[str]:
    return sequence_required_field_schema_diagnostics(
        label,
        value,
        fields,
        validate_string_schema_diagnostics,
    )


def sequence_integer_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_integer_schema_diagnostics(
                f"{label}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics


def sequence_required_integer_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
) -> list[str]:
    return sequence_required_field_schema_diagnostics(
        label,
        value,
        fields,
        validate_integer_schema_diagnostics,
    )


def sequence_required_string_array_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
) -> list[str]:
    return sequence_required_field_schema_diagnostics(
        label,
        value,
        fields,
        validate_string_array_schema_diagnostics,
    )


def sequence_required_object_array_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
) -> list[str]:
    return sequence_required_field_schema_diagnostics(
        label,
        value,
        fields,
        validate_object_array_schema_diagnostics,
    )


def sequence_required_field_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
    validate_schema: Callable[[str, Any], list[str]],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        for field in fields:
            diagnostics.extend(
                validate_schema(
                    typed_field_label(f"{label}[{index}]", field),
                    entry.get(field),
                )
            )
    return diagnostics


def sequence_string_array_schema_diagnostics(
    label: str,
    value: list[object],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_string_array_schema_diagnostics(
                f"{label}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics
