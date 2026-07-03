"""Row schema diagnostics for PlatformBundle template resolution evidence."""

from __future__ import annotations

from typing import Any

from .pipeline_report_platform_bundle_template_schema_helpers import (
    sequence_object_schema_diagnostics,
    sequence_present_trimmed_non_empty_string_diagnostics,
    sequence_required_non_empty_string_diagnostics,
    sequence_string_array_entries_trimmed_non_empty_diagnostics,
    sequence_string_array_schema_diagnostics,
    sequence_string_schema_diagnostics,
    sequence_unknown_field_diagnostics,
    sequence_unique_string_array_entries_schema_diagnostics,
)

PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_FIELDS = (
    "bundle_format",
    "compatible_profiles",
    "engine_version",
    "host_artifact",
    "target_platform",
    "template_dir",
    "template_id",
)

PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_STRING_FIELDS = (
    "bundle_format",
    "engine_version",
    "host_artifact",
    "target_platform",
    "template_dir",
    "template_id",
)

PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_STRING_ARRAY_FIELDS = (
    "compatible_profiles",
)

PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_SKIPPED_CANDIDATE_FIELDS = (
    "diagnostics",
    "template_dir",
)

PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_SKIPPED_CANDIDATE_STRING_FIELDS = (
    "template_dir",
)

PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_SKIPPED_CANDIDATE_STRING_ARRAY_FIELDS = (
    "diagnostics",
)


def template_resolution_candidate_row_schema_diagnostics(
    resolution: dict[str, Any],
    *,
    label: str = "PlatformBundle report template_resolution",
) -> list[str]:
    return template_resolution_sequence_schema_diagnostics(
        resolution,
        "candidates",
        PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_FIELDS,
        string_fields=PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_STRING_FIELDS,
        string_array_fields=(
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_STRING_ARRAY_FIELDS
        ),
        trimmed_string_array_fields=(
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_STRING_ARRAY_FIELDS
        ),
        unique_string_array_fields=(
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_STRING_ARRAY_FIELDS
        ),
        label=label,
    )


def template_resolution_skipped_candidate_row_schema_diagnostics(
    resolution: dict[str, Any],
    *,
    label: str = "PlatformBundle report template_resolution",
) -> list[str]:
    return template_resolution_sequence_schema_diagnostics(
        resolution,
        "skipped_candidates",
        PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_SKIPPED_CANDIDATE_FIELDS,
        string_fields=PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_SKIPPED_CANDIDATE_STRING_FIELDS,
        string_array_fields=(
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_SKIPPED_CANDIDATE_STRING_ARRAY_FIELDS
        ),
        trimmed_string_array_fields=(
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_SKIPPED_CANDIDATE_STRING_ARRAY_FIELDS
        ),
        label=label,
    )


def template_resolution_sequence_schema_diagnostics(
    resolution: dict[str, Any],
    field: str,
    allowed_fields: tuple[str, ...],
    *,
    string_fields: tuple[str, ...],
    string_array_fields: tuple[str, ...] = (),
    trimmed_string_array_fields: tuple[str, ...] = (),
    unique_string_array_fields: tuple[str, ...] = (),
    label: str = "PlatformBundle report template_resolution",
) -> list[str]:
    value = resolution.get(field)
    if value is None:
        return []
    if not isinstance(value, list):
        return [f"{label} {field} must be a list"]
    field_label = f"{label} {field}"
    diagnostics = sequence_object_schema_diagnostics(field_label, value)
    diagnostics.extend(
        sequence_unknown_field_diagnostics(field_label, value, allowed_fields)
    )
    diagnostics.extend(
        sequence_string_schema_diagnostics(field_label, value, string_fields)
    )
    diagnostics.extend(
        sequence_required_field_diagnostics(
            field_label,
            value,
            (*string_fields, *string_array_fields),
        )
    )
    diagnostics.extend(
        sequence_required_non_empty_string_diagnostics(field_label, value, string_fields)
    )
    diagnostics.extend(
        sequence_present_trimmed_non_empty_string_diagnostics(
            field_label,
            value,
            string_fields,
        )
    )
    diagnostics.extend(
        sequence_string_array_schema_diagnostics(
            field_label,
            value,
            string_array_fields,
        )
    )
    diagnostics.extend(
        sequence_string_array_entries_trimmed_non_empty_diagnostics(
            field_label,
            value,
            trimmed_string_array_fields,
        )
    )
    diagnostics.extend(
        sequence_unique_string_array_entries_schema_diagnostics(
            field_label,
            value,
            unique_string_array_fields,
        )
    )
    return diagnostics


def sequence_required_field_diagnostics(
    label: str,
    value: object,
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        for field in fields:
            if field not in entry or entry.get(field) is None:
                diagnostics.append(f"{label}[{index}].{field} is required")
    return diagnostics
