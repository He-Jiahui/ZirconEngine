"""PlatformBundle template-resolution schema diagnostics."""

from __future__ import annotations

from os.path import normcase
from pathlib import Path
from typing import Any

from .pipeline_report_platform_bundle_template_schema_helpers import (
    sequence_object_schema_diagnostics,
    sequence_required_non_empty_string_diagnostics,
    sequence_string_array_schema_diagnostics,
    sequence_string_schema_diagnostics,
    sequence_unknown_field_diagnostics,
    sequence_unique_string_array_entries_schema_diagnostics,
    table_bool_schema_diagnostics,
    table_required_non_empty_string_diagnostics,
    table_string_array_schema_diagnostics,
    table_string_schema_diagnostics,
    table_unknown_field_diagnostics,
)
from .pipeline_report_platform_bundle_template_resolution_semantics import (
    template_resolution_candidate_bundle_format_diagnostics,
    template_resolution_candidate_identity_diagnostics,
    template_resolution_candidate_profile_diagnostics,
    template_resolution_entries_inside_root_diagnostics,
    template_resolution_fatal_candidate_count_diagnostics,
    template_resolution_fatal_diagnostics_diagnostics,
    template_resolution_fatal_diagnostic_family_diagnostics,
    template_resolution_fatal_multiple_candidate_diagnostics,
    template_resolution_fatal_no_candidate_diagnostics,
    template_resolution_fatal_selection_diagnostics,
    template_resolution_no_match_identity_diagnostics,
    template_resolution_no_match_profile_diagnostics,
    template_resolution_no_match_root_diagnostics,
    template_resolution_non_fatal_diagnostics_diagnostics,
    template_resolution_non_fatal_expected_identity_diagnostics,
    template_resolution_non_fatal_selection_diagnostics,
    template_resolution_object_row_count,
    template_resolution_path_containment_diagnostics,
    template_resolution_root_failure_candidate_diagnostics,
    template_resolution_root_failure_root_diagnostics,
    template_resolution_selected_candidate_diagnostics,
    template_resolution_skipped_candidate_diagnostics_diagnostics,
    template_resolution_template_dir_uniqueness_diagnostics,
)

PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_FIELDS = (
    "candidates",
    "diagnostics",
    "expected_engine_version",
    "expected_target_platform",
    "fatal",
    "profile",
    "skipped_candidates",
    "template_dir",
    "template_root",
)

PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_REQUIRED_NON_NULL_FIELDS = (
    "candidates",
    "diagnostics",
    "fatal",
    "profile",
    "skipped_candidates",
    "template_root",
)

PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_STRING_FIELDS = (
    "expected_engine_version",
    "expected_target_platform",
    "profile",
    "template_dir",
    "template_root",
)

PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_BOOL_FIELDS = ("fatal",)

PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_STRING_ARRAY_FIELDS = ("diagnostics",)

PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_FIELDS = (
    "bundle_format",
    "compatible_profiles",
    "engine_version",
    "target_platform",
    "template_dir",
    "template_id",
)

PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_STRING_FIELDS = (
    "bundle_format",
    "engine_version",
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


def platform_bundle_template_resolution_diagnostics(
    report: dict[str, Any],
) -> list[str]:
    resolution = report.get("template_resolution")
    diagnostics = platform_bundle_template_resolution_schema_diagnostics(resolution)
    diagnostics.extend(platform_bundle_template_resolution_profile_diagnostics(report))
    diagnostics.extend(
        platform_bundle_template_resolution_template_dir_diagnostics(report)
    )
    return diagnostics


def platform_bundle_template_resolution_profile_diagnostics(
    report: dict[str, Any],
    *,
    label: str = "PlatformBundle report",
) -> list[str]:
    resolution = report.get("template_resolution")
    report_profile = report.get("profile")
    if not isinstance(resolution, dict):
        return []
    resolution_profile = resolution.get("profile")
    if (
        not isinstance(report_profile, str)
        or not report_profile.strip()
        or not isinstance(resolution_profile, str)
        or not resolution_profile.strip()
    ):
        return []
    if resolution_profile == report_profile:
        return []
    return [
        f"{label} template_resolution.profile "
        f"must match {label} profile {report_profile}"
    ]


def platform_bundle_template_resolution_template_dir_diagnostics(
    report: dict[str, Any],
    *,
    label: str = "PlatformBundle report",
) -> list[str]:
    resolution = report.get("template_resolution")
    template = report.get("template")
    if not isinstance(resolution, dict) or not isinstance(template, dict):
        return []
    if resolution.get("fatal") is not False:
        return []
    resolution_template_dir = resolution.get("template_dir")
    template_dir = template.get("template_dir")
    if (
        not isinstance(resolution_template_dir, str)
        or not resolution_template_dir.strip()
        or not isinstance(template_dir, str)
        or not template_dir.strip()
    ):
        return []
    try:
        resolved_resolution_template_dir = Path(
            resolution_template_dir
        ).expanduser().resolve()
        resolved_template_dir = Path(template_dir).expanduser().resolve()
    except OSError as error:
        return [
            f"{label} template_resolution.template_dir could not be resolved: {error}"
        ]
    if normcase(str(resolved_resolution_template_dir)) == normcase(
        str(resolved_template_dir)
    ):
        return []
    return [
        f"{label} template_resolution.template_dir must match template.template_dir"
    ]


def platform_bundle_template_resolution_schema_diagnostics(
    resolution: object,
    label: str = "PlatformBundle report template_resolution",
) -> list[str]:
    if resolution is None:
        return []
    if not isinstance(resolution, dict):
        return [f"{label} must be an object"]
    diagnostics = table_unknown_field_diagnostics(
        label,
        resolution,
        PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_FIELDS,
    )
    diagnostics.extend(
        table_string_schema_diagnostics(
            label,
            resolution,
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_required_non_empty_string_diagnostics(
            label,
            resolution,
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_required_field_diagnostics(
            label,
            resolution,
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_FIELDS,
        )
    )
    diagnostics.extend(
        table_required_non_null_field_diagnostics(
            label,
            resolution,
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_REQUIRED_NON_NULL_FIELDS,
        )
    )
    diagnostics.extend(
        table_bool_schema_diagnostics(
            label,
            resolution,
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_BOOL_FIELDS,
        )
    )
    diagnostics.extend(
        table_string_array_schema_diagnostics(
            label,
            resolution,
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_STRING_ARRAY_FIELDS,
        )
    )
    diagnostics.extend(
        template_resolution_sequence_schema_diagnostics(
            resolution,
            "candidates",
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_FIELDS,
            string_fields=PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_STRING_FIELDS,
            string_array_fields=PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_STRING_ARRAY_FIELDS,
            unique_string_array_fields=PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_CANDIDATE_STRING_ARRAY_FIELDS,
            label=label,
        )
    )
    diagnostics.extend(template_resolution_candidate_profile_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_candidate_identity_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_candidate_bundle_format_diagnostics(label, resolution))
    diagnostics.extend(
        template_resolution_sequence_schema_diagnostics(
            resolution,
            "skipped_candidates",
            PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_SKIPPED_CANDIDATE_FIELDS,
            string_fields=PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_SKIPPED_CANDIDATE_STRING_FIELDS,
            string_array_fields=PLATFORM_BUNDLE_TEMPLATE_RESOLUTION_SKIPPED_CANDIDATE_STRING_ARRAY_FIELDS,
            label=label,
        )
    )
    diagnostics.extend(template_resolution_skipped_candidate_diagnostics_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_path_containment_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_template_dir_uniqueness_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_fatal_selection_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_fatal_candidate_count_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_fatal_diagnostics_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_fatal_diagnostic_family_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_fatal_multiple_candidate_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_fatal_no_candidate_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_no_match_profile_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_no_match_identity_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_no_match_root_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_root_failure_root_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_root_failure_candidate_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_non_fatal_diagnostics_diagnostics(label, resolution))
    diagnostics.extend(
        template_resolution_non_fatal_expected_identity_diagnostics(label, resolution)
    )
    diagnostics.extend(template_resolution_non_fatal_selection_diagnostics(label, resolution))
    diagnostics.extend(template_resolution_selected_candidate_diagnostics(label, resolution))
    return diagnostics


def template_resolution_sequence_schema_diagnostics(
    resolution: dict[str, Any],
    field: str,
    allowed_fields: tuple[str, ...],
    *,
    string_fields: tuple[str, ...],
    string_array_fields: tuple[str, ...] = (),
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
        sequence_unknown_field_diagnostics(
            field_label,
            value,
            allowed_fields,
        )
    )
    diagnostics.extend(
        sequence_string_schema_diagnostics(
            field_label,
            value,
            string_fields,
        )
    )
    diagnostics.extend(
        sequence_required_field_diagnostics(
            field_label,
            value,
            (*string_fields, *string_array_fields),
        )
    )
    diagnostics.extend(
        sequence_required_non_empty_string_diagnostics(
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
        sequence_unique_string_array_entries_schema_diagnostics(
            field_label,
            value,
            unique_string_array_fields,
        )
    )
    return diagnostics


def table_required_field_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    return [
        f"{label}.{field} is required"
        for field in fields
        if field not in table
    ]


def table_required_non_null_field_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    return [
        f"{label}.{field} is required"
        for field in fields
        if field in table and table.get(field) is None
    ]


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
