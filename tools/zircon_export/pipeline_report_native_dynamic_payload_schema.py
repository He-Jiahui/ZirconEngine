"""Schema diagnostics for NativeDynamic payload evidence in final reports."""

from __future__ import annotations

from typing import Any

from .export_template import (
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
)
from .pipeline_report_native_dynamic_operation_audit_schema import (
    NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS,
    native_dynamic_operation_audit_stage_schema_diagnostics,
    platform_bundle_native_plugins_operation_audit_schema_diagnostics,
)
from .pipeline_report_schema_table import (
    object_array_schema_diagnostics,
    optional_fields,
    string_array_trimmed_non_empty_entries_schema_diagnostics,
    table_integer_schema_diagnostics,
    table_string_schema_diagnostics,
    table_unknown_field_diagnostics,
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

NATIVE_DYNAMIC_PAYLOAD_REQUIRED_STRING_FIELDS = (
    "bundle_path",
    "content_hash",
    "loader_manifest",
    "source",
)
NATIVE_DYNAMIC_PAYLOAD_CONTENT_HASH_FIELDS = ("content_hash",)
NATIVE_DYNAMIC_PAYLOAD_NON_EMPTY_STRING_FIELDS = (
    "bundle_path",
    "content_hash",
    "loader_manifest",
    "source",
    "stage_report",
)

NATIVE_DYNAMIC_PAYLOAD_INTEGER_FIELDS = (
    "file_count",
    "package_count",
)
NATIVE_DYNAMIC_PAYLOAD_REQUIRED_INTEGER_FIELDS = NATIVE_DYNAMIC_PAYLOAD_INTEGER_FIELDS

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
        table_non_empty_string_schema_diagnostics(
            label,
            payload,
            NATIVE_DYNAMIC_PAYLOAD_NON_EMPTY_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_trimmed_non_empty_string_schema_diagnostics(
            label,
            payload,
            NATIVE_DYNAMIC_PAYLOAD_NON_EMPTY_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_sha256_hex_string_schema_diagnostics(
            label,
            payload,
            NATIVE_DYNAMIC_PAYLOAD_CONTENT_HASH_FIELDS,
        )
    )
    diagnostics.extend(
        table_integer_schema_diagnostics(
            label,
            payload,
            NATIVE_DYNAMIC_PAYLOAD_REQUIRED_INTEGER_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_non_negative_integer_schema_diagnostics(
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
    diagnostics = object_array_schema_diagnostics(
        label,
        payload,
        "file_manifest",
        NATIVE_DYNAMIC_FILE_MANIFEST_FIELDS,
        string_fields=NATIVE_DYNAMIC_FILE_MANIFEST_STRING_FIELDS,
        integer_fields=NATIVE_DYNAMIC_FILE_MANIFEST_INTEGER_FIELDS,
        required_string_fields=NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_STRING_FIELDS,
        required_integer_fields=NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_INTEGER_FIELDS,
        require_present=True,
    )
    diagnostics.extend(
        object_array_required_non_empty_string_schema_diagnostics(
            label,
            payload,
            "file_manifest",
            NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_required_trimmed_non_empty_string_schema_diagnostics(
            label,
            payload,
            "file_manifest",
            NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_sha256_hex_string_schema_diagnostics(
            label,
            payload,
            "file_manifest",
            ("sha256",),
        )
    )
    diagnostics.extend(
        object_array_safe_relative_path_string_schema_diagnostics(
            label,
            payload,
            "file_manifest",
            ("path",),
        )
    )
    diagnostics.extend(
        object_array_unique_string_field_schema_diagnostics(
            label,
            payload,
            "file_manifest",
            "path",
            normalize_path=True,
        )
    )
    diagnostics.extend(
        object_array_non_negative_integer_schema_diagnostics(
            label,
            payload,
            "file_manifest",
            ("bytes",),
        )
    )
    return diagnostics


def native_dynamic_materialized_packages_schema_diagnostics(
    label: str,
    payload: dict[str, Any],
) -> list[str]:
    diagnostics = object_array_schema_diagnostics(
        label,
        payload,
        "materialized_packages",
        NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_FIELDS,
        string_fields=NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_FIELDS,
        integer_fields=NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_INTEGER_FIELDS,
        required_string_fields=(
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_STRING_FIELDS
        ),
        required_integer_fields=(
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_REQUIRED_INTEGER_FIELDS
        ),
        require_present=True,
    )
    diagnostics.extend(
        object_array_loadable_artifacts_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
        )
    )
    diagnostics.extend(
        object_array_required_non_empty_string_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_required_trimmed_non_empty_string_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_unique_string_field_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            "package_id",
        )
    )
    diagnostics.extend(
        object_array_non_negative_integer_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_INTEGER_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_string_array_no_blank_entries_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_ARRAY_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_string_array_trimmed_non_empty_entries_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_ARRAY_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_string_array_safe_relative_path_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_ARRAY_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_string_array_unique_entries_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            NATIVE_DYNAMIC_MATERIALIZED_PACKAGE_STRING_ARRAY_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_integer_matches_string_array_length_schema_diagnostics(
            label,
            payload,
            "materialized_packages",
            "loadable_artifact_count",
            "loadable_artifacts",
        )
    )
    return diagnostics


def object_array_loadable_artifacts_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
) -> list[str]:
    value = table.get(field)
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    item_field = "loadable_artifacts"
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        item_value = entry.get(item_field)
        if not isinstance(item_value, list):
            diagnostics.append(
                f"{label} {field}[{index}].{item_field} "
                "must be a string array"
            )
            continue
        for item_index, item in enumerate(item_value):
            if not isinstance(item, str):
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field}[{item_index}] "
                    "must be a string"
                )
    return diagnostics


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
        for item_field in fields:
            item_value = entry.get(item_field)
            if isinstance(item_value, str) and not item_value.strip():
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field} "
                    "must be a non-empty string"
                )
    return diagnostics


def object_array_required_trimmed_non_empty_string_schema_diagnostics(
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
        for item_field in fields:
            item_value = entry.get(item_field)
            if (
                isinstance(item_value, str)
                and item_value.strip()
                and item_value.strip() != item_value
            ):
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field} "
                    "must be a non-empty trimmed string"
                )
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
        for item_field in fields:
            item_value = entry.get(item_field)
            if (
                isinstance(item_value, str)
                and item_value.strip()
                and item_value.strip() == item_value
                and not is_sha256_hex(item_value)
            ):
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field} "
                    "must be a SHA-256 hex digest"
                )
    return diagnostics


def object_array_safe_relative_path_string_schema_diagnostics(
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
        for item_field in fields:
            item_value = entry.get(item_field)
            if (
                isinstance(item_value, str)
                and item_value.strip()
                and not is_safe_relative_path(normalize_relative_path(item_value))
            ):
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field} "
                    "must be a safe relative path"
                )
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
        for item_field in fields:
            item_value = entry.get(item_field)
            if type(item_value) is int and item_value < 0:
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field} "
                    "must be non-negative"
                )
    return diagnostics


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


def table_non_empty_string_schema_diagnostics(
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


def table_trimmed_non_empty_string_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        value = table.get(field)
        if (
            isinstance(value, str)
            and value.strip()
            and value.strip() != value
        ):
            diagnostics.append(
                f"{label}.{field} must be a non-empty trimmed string"
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
        if (
            isinstance(value, str)
            and value.strip()
            and value.strip() == value
            and not is_sha256_hex(value)
        ):
            diagnostics.append(f"{label}.{field} must be a SHA-256 hex digest")
    return diagnostics


def object_array_string_array_no_blank_entries_schema_diagnostics(
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
        for item_field in fields:
            item_value = entry.get(item_field)
            if (
                isinstance(item_value, list)
                and all(isinstance(item, str) for item in item_value)
                and any(not item.strip() for item in item_value)
            ):
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field} "
                    "must not contain blank entries"
                )
    return diagnostics


def object_array_string_array_trimmed_non_empty_entries_schema_diagnostics(
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
        for item_field in fields:
            item_value = entry.get(item_field)
            if not (
                isinstance(item_value, list)
                and all(isinstance(item, str) for item in item_value)
            ):
                continue
            diagnostics.extend(
                string_array_trimmed_non_empty_entries_schema_diagnostics(
                    f"{label} {field}[{index}].{item_field}",
                    item_value,
                )
            )
    return diagnostics


def object_array_string_array_safe_relative_path_schema_diagnostics(
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
        for item_field in fields:
            item_value = entry.get(item_field)
            if not (
                isinstance(item_value, list)
                and all(isinstance(item, str) for item in item_value)
            ):
                continue
            for item_index, item in enumerate(item_value):
                if (
                    item.strip()
                    and not is_safe_relative_path(normalize_relative_path(item))
                ):
                    diagnostics.append(
                        f"{label} {field}[{index}].{item_field}[{item_index}] "
                        "must be a safe relative path"
                    )
    return diagnostics


def object_array_unique_string_field_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    item_field: str,
    *,
    normalize_path: bool = False,
) -> list[str]:
    value = table.get(field)
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    seen: set[str] = set()
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        item_value = entry.get(item_field)
        if (
            not isinstance(item_value, str)
            or not item_value.strip()
            or item_value.strip() != item_value
        ):
            continue
        normalized = normalize_relative_path(item_value) if normalize_path else item_value
        if normalize_path and not is_safe_relative_path(normalized):
            continue
        if normalized in seen:
            diagnostics.append(
                f"{label} {field}[{index}].{item_field} must be unique"
            )
            continue
        seen.add(normalized)
    return diagnostics


def object_array_string_array_unique_entries_schema_diagnostics(
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
        for item_field in fields:
            item_value = entry.get(item_field)
            if not (
                isinstance(item_value, list)
                and all(isinstance(item, str) for item in item_value)
            ):
                continue
            seen: set[str] = set()
            has_duplicate = False
            for item in item_value:
                if not item.strip() or item.strip() != item:
                    continue
                normalized = normalize_relative_path(item)
                if not is_safe_relative_path(normalized):
                    continue
                if normalized in seen:
                    has_duplicate = True
                    break
                seen.add(normalized)
            if has_duplicate:
                diagnostics.append(
                    f"{label} {field}[{index}].{item_field} "
                    "must not contain duplicate entries"
                )
    return diagnostics


def object_array_integer_matches_string_array_length_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    field: str,
    integer_field: str,
    string_array_field: str,
) -> list[str]:
    value = table.get(field)
    if not isinstance(value, list):
        return []
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        integer_value = entry.get(integer_field)
        array_value = entry.get(string_array_field)
        if (
            type(integer_value) is int
            and integer_value >= 0
            and isinstance(array_value, list)
            and all(isinstance(item, str) for item in array_value)
            and integer_value != len(array_value)
        ):
            diagnostics.append(
                f"{label} {field}[{index}].{integer_field} "
                f"must match {string_array_field} length"
            )
    return diagnostics
