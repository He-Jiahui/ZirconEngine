"""Schema diagnostics for NativeDynamic package report evidence."""

from __future__ import annotations

from typing import Any

from .export_template import (
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
)
from .native_dynamic_contract import NATIVE_DYNAMIC_ABI_STRING_FIELDS
from .pipeline_report_native_dynamic_payload_schema import (
    NATIVE_DYNAMIC_FILE_MANIFEST_FIELDS,
    NATIVE_DYNAMIC_FILE_MANIFEST_INTEGER_FIELDS,
    NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_INTEGER_FIELDS,
    NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_STRING_FIELDS,
    NATIVE_DYNAMIC_FILE_MANIFEST_STRING_FIELDS,
)
from .pipeline_report_schema_table import (
    object_array_schema_diagnostics,
    optional_fields,
    table_integer_schema_diagnostics,
    table_object_schema_diagnostics,
    table_string_schema_diagnostics,
    table_unknown_field_diagnostics,
)

NATIVE_DYNAMIC_PACKAGE_REPORT_FIELDS = (
    "abi",
    "directory",
    "format_version",
    "manifest",
    "package_id",
    "path",
    "payload",
)

NATIVE_DYNAMIC_PACKAGE_REPORT_STRING_FIELDS = (
    "directory",
    "manifest",
    "package_id",
    "path",
)
NATIVE_DYNAMIC_PACKAGE_REPORT_REQUIRED_STRING_FIELDS = (
    "directory",
    "manifest",
    "package_id",
    "path",
)
NATIVE_DYNAMIC_PACKAGE_REPORT_SAFE_RELATIVE_PATH_FIELDS = (
    "directory",
    "manifest",
    "path",
)

NATIVE_DYNAMIC_PACKAGE_REPORT_INTEGER_FIELDS = ("format_version",)
NATIVE_DYNAMIC_PACKAGE_REPORT_REQUIRED_INTEGER_FIELDS = ("format_version",)

NATIVE_DYNAMIC_PACKAGE_REPORT_OBJECT_FIELDS = (
    "abi",
    "payload",
)
NATIVE_DYNAMIC_PACKAGE_REPORT_REQUIRED_OBJECT_FIELDS = (
    "abi",
    "payload",
)

NATIVE_DYNAMIC_PACKAGE_REPORT_ABI_FIELDS = (
    "abi_version",
    *NATIVE_DYNAMIC_ABI_STRING_FIELDS,
)

NATIVE_DYNAMIC_PACKAGE_REPORT_ABI_INTEGER_FIELDS = ("abi_version",)
NATIVE_DYNAMIC_PACKAGE_REPORT_ABI_REQUIRED_INTEGER_FIELDS = ("abi_version",)

NATIVE_DYNAMIC_PACKAGE_REPORT_ABI_STRING_FIELDS = NATIVE_DYNAMIC_ABI_STRING_FIELDS
NATIVE_DYNAMIC_PACKAGE_REPORT_ABI_REQUIRED_STRING_FIELDS = NATIVE_DYNAMIC_ABI_STRING_FIELDS

NATIVE_DYNAMIC_PACKAGE_REPORT_PAYLOAD_FIELDS = (
    "content_hash",
    "file_count",
    "files",
)

NATIVE_DYNAMIC_PACKAGE_REPORT_PAYLOAD_STRING_FIELDS = ("content_hash",)
NATIVE_DYNAMIC_PACKAGE_REPORT_PAYLOAD_REQUIRED_STRING_FIELDS = ("content_hash",)

NATIVE_DYNAMIC_PACKAGE_REPORT_PAYLOAD_INTEGER_FIELDS = ("file_count",)
NATIVE_DYNAMIC_PACKAGE_REPORT_PAYLOAD_REQUIRED_INTEGER_FIELDS = ("file_count",)


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


def table_safe_relative_path_string_schema_diagnostics(
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
            and not is_safe_relative_path(normalize_relative_path(value))
        ):
            diagnostics.append(f"{label}.{field} must be a safe relative path")
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
        diagnostics.extend(
            table_safe_relative_path_string_schema_diagnostics(
                f"{label} {field}[{index}]",
                entry,
                fields,
            )
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
        diagnostics.extend(
            table_non_negative_integer_schema_diagnostics(
                f"{label} {field}[{index}]",
                entry,
                fields,
            )
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
    value: list[str],
) -> list[str]:
    seen: set[str] = set()
    for entry in value:
        if entry in seen:
            return [f"{label} must not contain duplicate entries"]
        seen.add(entry)
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


def platform_bundle_native_plugins_package_report_payload_files_schema_diagnostics(
    label: str,
    payload: dict[str, Any],
) -> list[str]:
    payload_label = f"{label} payload"
    diagnostics = object_array_schema_diagnostics(
        payload_label,
        payload,
        "files",
        NATIVE_DYNAMIC_FILE_MANIFEST_FIELDS,
        string_fields=NATIVE_DYNAMIC_FILE_MANIFEST_STRING_FIELDS,
        integer_fields=NATIVE_DYNAMIC_FILE_MANIFEST_INTEGER_FIELDS,
        required_string_fields=NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_STRING_FIELDS,
        required_integer_fields=NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_INTEGER_FIELDS,
    )
    diagnostics.extend(
        object_array_required_non_empty_string_schema_diagnostics(
            payload_label,
            payload,
            "files",
            NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        object_array_safe_relative_path_string_schema_diagnostics(
            payload_label,
            payload,
            "files",
            ("path",),
        )
    )
    diagnostics.extend(
        object_array_non_negative_integer_schema_diagnostics(
            payload_label,
            payload,
            "files",
            ("bytes",),
        )
    )
    diagnostics.extend(
        object_array_sha256_hex_string_schema_diagnostics(
            payload_label,
            payload,
            "files",
            ("sha256",),
        )
    )
    diagnostics.extend(
        object_array_unique_string_field_schema_diagnostics(
            payload_label,
            payload,
            "files",
            "path",
        )
    )
    return diagnostics


def platform_bundle_native_plugins_package_report_payload_schema_diagnostics(
    label: str,
    payload: dict[str, Any],
) -> list[str]:
    payload_label = f"{label} payload"
    diagnostics = table_unknown_field_diagnostics(
        payload_label,
        payload,
        NATIVE_DYNAMIC_PACKAGE_REPORT_PAYLOAD_FIELDS,
    )
    diagnostics.extend(
        table_string_schema_diagnostics(
            payload_label,
            payload,
            NATIVE_DYNAMIC_PACKAGE_REPORT_PAYLOAD_REQUIRED_STRING_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_required_non_empty_string_schema_diagnostics(
            payload_label,
            payload,
            NATIVE_DYNAMIC_PACKAGE_REPORT_PAYLOAD_REQUIRED_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_sha256_hex_string_schema_diagnostics(
            payload_label,
            payload,
            ("content_hash",),
        )
    )
    diagnostics.extend(
        table_string_schema_diagnostics(
            payload_label,
            payload,
            optional_fields(
                NATIVE_DYNAMIC_PACKAGE_REPORT_PAYLOAD_STRING_FIELDS,
                NATIVE_DYNAMIC_PACKAGE_REPORT_PAYLOAD_REQUIRED_STRING_FIELDS,
            ),
        )
    )
    diagnostics.extend(
        table_integer_schema_diagnostics(
            payload_label,
            payload,
            NATIVE_DYNAMIC_PACKAGE_REPORT_PAYLOAD_REQUIRED_INTEGER_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_non_negative_integer_schema_diagnostics(
            payload_label,
            payload,
            NATIVE_DYNAMIC_PACKAGE_REPORT_PAYLOAD_REQUIRED_INTEGER_FIELDS,
        )
    )
    diagnostics.extend(
        table_integer_schema_diagnostics(
            payload_label,
            payload,
            optional_fields(
                NATIVE_DYNAMIC_PACKAGE_REPORT_PAYLOAD_INTEGER_FIELDS,
                NATIVE_DYNAMIC_PACKAGE_REPORT_PAYLOAD_REQUIRED_INTEGER_FIELDS,
            ),
        )
    )
    diagnostics.extend(
        platform_bundle_native_plugins_package_report_payload_files_schema_diagnostics(
            label,
            payload,
        )
    )
    return diagnostics


def platform_bundle_native_plugins_package_report_abi_schema_diagnostics(
    label: str,
    abi: dict[str, Any],
) -> list[str]:
    abi_label = f"{label} abi"
    diagnostics = table_unknown_field_diagnostics(
        abi_label,
        abi,
        NATIVE_DYNAMIC_PACKAGE_REPORT_ABI_FIELDS,
    )
    diagnostics.extend(
        table_integer_schema_diagnostics(
            abi_label,
            abi,
            NATIVE_DYNAMIC_PACKAGE_REPORT_ABI_REQUIRED_INTEGER_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_integer_schema_diagnostics(
            abi_label,
            abi,
            optional_fields(
                NATIVE_DYNAMIC_PACKAGE_REPORT_ABI_INTEGER_FIELDS,
                NATIVE_DYNAMIC_PACKAGE_REPORT_ABI_REQUIRED_INTEGER_FIELDS,
            ),
        )
    )
    diagnostics.extend(
        table_string_schema_diagnostics(
            abi_label,
            abi,
            NATIVE_DYNAMIC_PACKAGE_REPORT_ABI_REQUIRED_STRING_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_required_non_empty_string_schema_diagnostics(
            abi_label,
            abi,
            NATIVE_DYNAMIC_PACKAGE_REPORT_ABI_REQUIRED_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_string_schema_diagnostics(
            abi_label,
            abi,
            optional_fields(
                NATIVE_DYNAMIC_PACKAGE_REPORT_ABI_STRING_FIELDS,
                NATIVE_DYNAMIC_PACKAGE_REPORT_ABI_REQUIRED_STRING_FIELDS,
            ),
        )
    )
    return diagnostics


def platform_bundle_native_plugins_package_report_schema_diagnostics(
    label: str,
    package_report: dict[str, Any],
) -> list[str]:
    diagnostics = table_unknown_field_diagnostics(
        label,
        package_report,
        NATIVE_DYNAMIC_PACKAGE_REPORT_FIELDS,
    )
    diagnostics.extend(
        table_string_schema_diagnostics(
            label,
            package_report,
            NATIVE_DYNAMIC_PACKAGE_REPORT_REQUIRED_STRING_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_required_non_empty_string_schema_diagnostics(
            label,
            package_report,
            NATIVE_DYNAMIC_PACKAGE_REPORT_REQUIRED_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_safe_relative_path_string_schema_diagnostics(
            label,
            package_report,
            NATIVE_DYNAMIC_PACKAGE_REPORT_SAFE_RELATIVE_PATH_FIELDS,
        )
    )
    diagnostics.extend(
        table_string_schema_diagnostics(
            label,
            package_report,
            optional_fields(
                NATIVE_DYNAMIC_PACKAGE_REPORT_STRING_FIELDS,
                NATIVE_DYNAMIC_PACKAGE_REPORT_REQUIRED_STRING_FIELDS,
            ),
        )
    )
    diagnostics.extend(
        table_integer_schema_diagnostics(
            label,
            package_report,
            NATIVE_DYNAMIC_PACKAGE_REPORT_REQUIRED_INTEGER_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_integer_schema_diagnostics(
            label,
            package_report,
            optional_fields(
                NATIVE_DYNAMIC_PACKAGE_REPORT_INTEGER_FIELDS,
                NATIVE_DYNAMIC_PACKAGE_REPORT_REQUIRED_INTEGER_FIELDS,
            ),
        )
    )
    diagnostics.extend(
        table_object_schema_diagnostics(
            label,
            package_report,
            NATIVE_DYNAMIC_PACKAGE_REPORT_REQUIRED_OBJECT_FIELDS,
            require_present=True,
        )
    )
    diagnostics.extend(
        table_object_schema_diagnostics(
            label,
            package_report,
            optional_fields(
                NATIVE_DYNAMIC_PACKAGE_REPORT_OBJECT_FIELDS,
                NATIVE_DYNAMIC_PACKAGE_REPORT_REQUIRED_OBJECT_FIELDS,
            ),
        )
    )
    return diagnostics
