"""PlatformBundle embedded template bundle/files schema diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_platform_bundle_template_report_semantics import (
    template_report_content_hash_diagnostics,
    template_report_file_source_hash_diagnostics,
    template_report_host_executable_membership_diagnostics,
)
from .pipeline_report_platform_bundle_template_path_schema_helpers import (
    sequence_safe_relative_path_string_diagnostics,
    sequence_sha256_hex_string_diagnostics,
    sequence_unique_path_diagnostics,
    sequence_unique_relative_path_field_diagnostics,
    table_bundle_path_string_diagnostics,
)
from .pipeline_report_platform_bundle_template_schema_helpers import (
    sequence_present_non_blank_string_diagnostics,
    sequence_present_trimmed_non_empty_string_diagnostics,
    sequence_required_non_empty_string_diagnostics,
    sequence_string_schema_diagnostics,
    sequence_unknown_field_diagnostics,
    table_present_trimmed_non_empty_string_diagnostics,
    table_string_schema_diagnostics,
    table_unknown_field_diagnostics,
    table_whitespace_only_string_diagnostics,
)

PLATFORM_BUNDLE_TEMPLATE_BUNDLE_FIELDS = (
    "delta_pack_path",
    "host_path",
    "manifest_path",
    "pack_path",
    "root",
)

PLATFORM_BUNDLE_TEMPLATE_BUNDLE_STRING_FIELDS = PLATFORM_BUNDLE_TEMPLATE_BUNDLE_FIELDS

PLATFORM_BUNDLE_TEMPLATE_FILE_FIELDS = (
    "bundle_path",
    "path",
    "purpose",
    "sha256",
)

PLATFORM_BUNDLE_TEMPLATE_FILE_STRING_FIELDS = PLATFORM_BUNDLE_TEMPLATE_FILE_FIELDS


def platform_bundle_template_bundle_files_schema_diagnostics(
    label: str,
    template: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    bundle = template.get("bundle")
    if isinstance(bundle, dict):
        diagnostics.extend(
            table_unknown_field_diagnostics(
                f"{label}.bundle",
                bundle,
                PLATFORM_BUNDLE_TEMPLATE_BUNDLE_FIELDS,
            )
        )
        diagnostics.extend(
            table_string_schema_diagnostics(
                f"{label}.bundle",
                bundle,
                PLATFORM_BUNDLE_TEMPLATE_BUNDLE_STRING_FIELDS,
            )
        )
        diagnostics.extend(
            table_whitespace_only_string_diagnostics(
                f"{label}.bundle",
                bundle,
                PLATFORM_BUNDLE_TEMPLATE_BUNDLE_STRING_FIELDS,
            )
        )
        diagnostics.extend(
            table_present_trimmed_non_empty_string_diagnostics(
                f"{label}.bundle",
                bundle,
                PLATFORM_BUNDLE_TEMPLATE_BUNDLE_STRING_FIELDS,
            )
        )
        diagnostics.extend(
            table_bundle_path_string_diagnostics(
                f"{label}.bundle",
                bundle,
                PLATFORM_BUNDLE_TEMPLATE_BUNDLE_STRING_FIELDS,
            )
        )
    files = template.get("files")
    if isinstance(files, list):
        diagnostics.extend(
            sequence_unknown_field_diagnostics(
                f"{label}.files",
                files,
                PLATFORM_BUNDLE_TEMPLATE_FILE_FIELDS,
            )
        )
        diagnostics.extend(
            sequence_string_schema_diagnostics(
                f"{label}.files",
                files,
                PLATFORM_BUNDLE_TEMPLATE_FILE_STRING_FIELDS,
            )
        )
        diagnostics.extend(
            sequence_required_non_empty_string_diagnostics(
                f"{label}.files",
                files,
                ("bundle_path", "path", "sha256"),
            )
        )
        diagnostics.extend(
            sequence_present_non_blank_string_diagnostics(
                f"{label}.files",
                files,
                ("purpose",),
            )
        )
        diagnostics.extend(
            sequence_present_trimmed_non_empty_string_diagnostics(
                f"{label}.files",
                files,
                ("bundle_path", "path", "purpose", "sha256"),
            )
        )
        diagnostics.extend(
            sequence_sha256_hex_string_diagnostics(
                f"{label}.files",
                files,
                ("sha256",),
            )
        )
        diagnostics.extend(
            sequence_safe_relative_path_string_diagnostics(
                f"{label}.files",
                files,
                ("bundle_path", "path"),
            )
        )
        diagnostics.extend(sequence_unique_path_diagnostics(f"{label}.files", files))
        diagnostics.extend(
            sequence_unique_relative_path_field_diagnostics(
                f"{label}.files",
                files,
                "bundle_path",
            )
        )
        diagnostics.extend(
            template_report_host_executable_membership_diagnostics(
                label,
                template,
                files,
            )
        )
        diagnostics.extend(template_report_file_source_hash_diagnostics(label, template, files))
        diagnostics.extend(template_report_content_hash_diagnostics(label, template, files))
    return diagnostics
