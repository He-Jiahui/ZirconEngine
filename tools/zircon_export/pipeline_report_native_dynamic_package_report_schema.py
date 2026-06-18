"""Schema diagnostics for NativeDynamic package report evidence."""

from __future__ import annotations

from typing import Any

from .native_dynamic_contract import NATIVE_DYNAMIC_ABI_STRING_FIELDS
from .pipeline_report_native_dynamic_payload_schema import (
    NATIVE_DYNAMIC_FILE_MANIFEST_FIELDS,
    NATIVE_DYNAMIC_FILE_MANIFEST_INTEGER_FIELDS,
    NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_INTEGER_FIELDS,
    NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_STRING_FIELDS,
    NATIVE_DYNAMIC_FILE_MANIFEST_STRING_FIELDS,
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


def platform_bundle_native_plugins_package_report_payload_files_schema_diagnostics(
    label: str,
    payload: dict[str, Any],
) -> list[str]:
    return object_array_schema_diagnostics(
        f"{label} payload",
        payload,
        "files",
        NATIVE_DYNAMIC_FILE_MANIFEST_FIELDS,
        string_fields=NATIVE_DYNAMIC_FILE_MANIFEST_STRING_FIELDS,
        integer_fields=NATIVE_DYNAMIC_FILE_MANIFEST_INTEGER_FIELDS,
        required_string_fields=NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_STRING_FIELDS,
        required_integer_fields=NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_INTEGER_FIELDS,
    )


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
