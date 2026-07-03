"""NativeDynamic payload file_manifest schema diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_native_dynamic_payload_schema_helpers import (
    object_array_non_negative_integer_schema_diagnostics,
    object_array_required_non_empty_string_schema_diagnostics,
    object_array_required_trimmed_non_empty_string_schema_diagnostics,
    object_array_safe_relative_path_string_schema_diagnostics,
    object_array_sha256_hex_string_schema_diagnostics,
    object_array_unique_string_field_schema_diagnostics,
)
from .pipeline_report_schema_table import object_array_schema_diagnostics

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


def platform_bundle_native_plugins_payload_file_manifest_schema_diagnostics(
    payload: dict[str, Any],
    label: str = "PlatformBundle report native_plugins_payload",
) -> list[str]:
    return native_dynamic_file_manifest_schema_diagnostics(
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
