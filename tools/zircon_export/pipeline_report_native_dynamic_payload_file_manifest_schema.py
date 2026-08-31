"""NativeDynamic payload file_manifest schema diagnostics."""

from __future__ import annotations

from typing import Any

from .export_template_manifest import (
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
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


def native_dynamic_file_manifest_value_schema_diagnostics(
    label: str,
    payload: dict[str, Any],
) -> list[str]:
    file_manifest = payload.get("file_manifest")
    if not isinstance(file_manifest, list):
        return []
    non_empty: list[str] = []
    trimmed: list[str] = []
    sha256: list[str] = []
    safe_path: list[str] = []
    unique_path: list[str] = []
    non_negative: list[str] = []
    seen_paths: set[str] = set()
    for index, entry in enumerate(file_manifest):
        if not isinstance(entry, dict):
            continue
        entry_label = f"{label} file_manifest[{index}]"
        normalized_path: str | None = None
        path_is_unique_candidate = False
        for field in NATIVE_DYNAMIC_FILE_MANIFEST_REQUIRED_STRING_FIELDS:
            value = entry.get(field)
            if not isinstance(value, str):
                continue
            stripped = value.strip()
            if not stripped:
                non_empty.append(
                    f"{entry_label}.{field} must be a non-empty string"
                )
                continue
            if stripped != value:
                trimmed.append(
                    f"{entry_label}.{field} must be a non-empty trimmed string"
                )
            if field == "sha256" and stripped == value and not is_sha256_hex(value):
                sha256.append(
                    f"{entry_label}.sha256 must be a SHA-256 hex digest"
                )
            if field == "path":
                normalized_path = normalize_relative_path(value)
                path_is_safe = is_safe_relative_path(normalized_path)
                if not path_is_safe:
                    safe_path.append(
                        f"{entry_label}.path must be a safe relative path"
                    )
                path_is_unique_candidate = stripped == value and path_is_safe
        if path_is_unique_candidate and normalized_path is not None:
            if normalized_path in seen_paths:
                unique_path.append(
                    f"{entry_label}.path must be unique"
                )
            else:
                seen_paths.add(normalized_path)
        bytes_value = entry.get("bytes")
        if type(bytes_value) is int and bytes_value < 0:
            non_negative.append(f"{entry_label}.bytes must be non-negative")
    return non_empty + trimmed + sha256 + safe_path + unique_path + non_negative


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
    diagnostics.extend(native_dynamic_file_manifest_value_schema_diagnostics(label, payload))
    return diagnostics
