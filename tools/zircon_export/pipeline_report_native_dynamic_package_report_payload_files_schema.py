"""Schema diagnostics for NativeDynamic package report payload.files evidence."""

from __future__ import annotations

from typing import Any

from .pipeline_report_native_dynamic_payload_file_manifest_schema import (
    NATIVE_DYNAMIC_FILE_MANIFEST_FIELDS,
    NATIVE_DYNAMIC_FILE_MANIFEST_INTEGER_FIELDS,
    NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_INTEGER_FIELDS,
    NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_STRING_FIELDS,
    NATIVE_DYNAMIC_FILE_MANIFEST_STRING_FIELDS,
)
from .pipeline_report_native_dynamic_package_report_schema_helpers import (
    object_array_non_negative_integer_schema_diagnostics,
    object_array_required_non_empty_string_schema_diagnostics,
    object_array_required_trimmed_non_empty_string_schema_diagnostics,
    object_array_safe_relative_path_string_schema_diagnostics,
    object_array_sha256_hex_string_schema_diagnostics,
    object_array_unique_string_field_schema_diagnostics,
)
from .pipeline_report_schema_table import object_array_schema_diagnostics


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
        object_array_required_trimmed_non_empty_string_schema_diagnostics(
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
