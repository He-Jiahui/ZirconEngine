"""PlatformBundle template schema diagnostics."""

from __future__ import annotations

from typing import Any

from .export_template import (
    EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS,
    EXPORT_TEMPLATE_ALLOWED_HOST_ARTIFACTS,
    EXPORT_TEMPLATE_ALLOWED_HOST_KINDS,
    EXPORT_TEMPLATE_ALLOWED_PLUGIN_STRATEGIES,
    EXPORT_TEMPLATE_ALLOWED_RESOURCE_STRATEGIES,
    EXPORT_TEMPLATE_FORMAT_VERSION,
)
from .pipeline_report_platform_bundle_template_manifest_schema import (
    template_report_manifest_path_diagnostics,
)
from .pipeline_report_platform_bundle_template_bundle_files_schema import (
    platform_bundle_template_bundle_files_schema_diagnostics,
)
from .pipeline_report_platform_bundle_template_report_semantics import (
    template_report_identity_match_diagnostics,
    template_report_profile_membership_diagnostics,
)
from .pipeline_report_platform_bundle_template_path_schema_helpers import (
    table_sha256_hex_string_diagnostics,
)
from .pipeline_report_platform_bundle_template_schema_helpers import (
    table_bool_schema_diagnostics,
    table_enum_string_diagnostics,
    table_integer_equals_diagnostics,
    table_integer_schema_diagnostics,
    table_object_array_schema_diagnostics,
    table_object_schema_diagnostics,
    table_present_trimmed_non_empty_string_diagnostics,
    table_required_non_empty_string_diagnostics,
    table_string_array_schema_diagnostics,
    table_string_array_entries_trimmed_non_empty_diagnostics,
    table_string_schema_diagnostics,
    table_unknown_field_diagnostics,
    table_unique_string_array_entries_schema_diagnostics,
)

PLATFORM_BUNDLE_TEMPLATE_REPORT_FIELDS = (
    "bundle",
    "bundle_format",
    "compatible_profiles",
    "computed_content_hash",
    "content_hash",
    "diagnostics",
    "engine_version",
    "expected_engine_version",
    "expected_format_version",
    "expected_target_platform",
    "fatal",
    "files",
    "format_version",
    "host_artifact",
    "host_executable",
    "host_kind",
    "manifest",
    "plugin_strategy",
    "profile",
    "resource_strategy",
    "target_platform",
    "template_dir",
    "template_id",
)

PLATFORM_BUNDLE_TEMPLATE_REPORT_STRING_FIELDS = (
    "bundle_format",
    "computed_content_hash",
    "content_hash",
    "engine_version",
    "expected_engine_version",
    "expected_target_platform",
    "host_artifact",
    "host_executable",
    "host_kind",
    "manifest",
    "plugin_strategy",
    "profile",
    "resource_strategy",
    "target_platform",
    "template_dir",
    "template_id",
)

PLATFORM_BUNDLE_TEMPLATE_REPORT_INTEGER_FIELDS = (
    "expected_format_version",
    "format_version",
)

PLATFORM_BUNDLE_TEMPLATE_REPORT_BOOL_FIELDS = ("fatal",)

PLATFORM_BUNDLE_TEMPLATE_REPORT_STRING_ARRAY_FIELDS = (
    "diagnostics",
)

PLATFORM_BUNDLE_TEMPLATE_REPORT_SHA256_FIELDS = (
    "computed_content_hash",
    "content_hash",
)

PLATFORM_BUNDLE_TEMPLATE_REPORT_ENUM_FIELDS = {
    "bundle_format": EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS,
    "host_artifact": EXPORT_TEMPLATE_ALLOWED_HOST_ARTIFACTS,
    "host_kind": EXPORT_TEMPLATE_ALLOWED_HOST_KINDS,
    "plugin_strategy": EXPORT_TEMPLATE_ALLOWED_PLUGIN_STRATEGIES,
    "resource_strategy": EXPORT_TEMPLATE_ALLOWED_RESOURCE_STRATEGIES,
}

PLATFORM_BUNDLE_TEMPLATE_REPORT_OBJECT_FIELDS = ("bundle",)

PLATFORM_BUNDLE_TEMPLATE_REPORT_OBJECT_ARRAY_FIELDS = ("files",)

PLATFORM_BUNDLE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS = (
    PLATFORM_BUNDLE_TEMPLATE_REPORT_STRING_FIELDS
)
PLATFORM_BUNDLE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_INTEGER_FIELDS = (
    PLATFORM_BUNDLE_TEMPLATE_REPORT_INTEGER_FIELDS
)
PLATFORM_BUNDLE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS = (
    "compatible_profiles",
    *PLATFORM_BUNDLE_TEMPLATE_REPORT_STRING_ARRAY_FIELDS,
)
PLATFORM_BUNDLE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS = (
    PLATFORM_BUNDLE_TEMPLATE_REPORT_OBJECT_FIELDS
)
PLATFORM_BUNDLE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS = (
    PLATFORM_BUNDLE_TEMPLATE_REPORT_OBJECT_ARRAY_FIELDS
)


def template_report_string_array_projection(
    label: str,
    template: dict[str, Any],
) -> tuple[list[str], list[str], list[str], list[str], bool]:
    diagnostics = template.get("diagnostics")
    compatible_profiles = template.get("compatible_profiles")
    diagnostics_schema: list[str] = []
    compatible_schema: list[str] = []
    compatible_trimmed: list[str] = []
    diagnostics_trimmed: list[str] = []
    unique: list[str] = []
    has_non_empty_diagnostic = False

    for field, values in (
        ("diagnostics", diagnostics),
        ("compatible_profiles", compatible_profiles),
    ):
        if values is None:
            continue
        if not isinstance(values, list):
            target = diagnostics_schema if field == "diagnostics" else compatible_schema
            target.append(f"{label}.{field} must be a string array")
            continue

        all_strings = True
        seen: set[str] = set()
        duplicate_values: list[str] = []
        has_blank = False
        field_trimmed: list[str] = []
        for index, value in enumerate(values):
            if not isinstance(value, str):
                all_strings = False
                target = diagnostics_schema if field == "diagnostics" else compatible_schema
                target.append(f"{label}.{field}[{index}] must be a string")
                continue
            stripped = value.strip()
            if field == "diagnostics" and stripped:
                has_non_empty_diagnostic = True
            if not stripped:
                has_blank = True
                continue
            if stripped != value:
                field_trimmed.append(
                    f"{label}.{field}[{index}] must be a non-empty trimmed string"
                )
                continue
            if field == "compatible_profiles":
                if value in seen:
                    duplicate_values.append(value)
                seen.add(value)

        if field == "diagnostics" and all_strings and has_blank:
            diagnostics_schema.append(f"{label}.{field} must not contain blank entries")
        if all_strings:
            if field == "compatible_profiles":
                compatible_trimmed.extend(field_trimmed)
            else:
                diagnostics_trimmed.extend(field_trimmed)
            if field == "compatible_profiles":
                unique.extend(
                    f"{label}.{field} duplicate entry {value}"
                    for value in duplicate_values
                )

    return (
        diagnostics_schema,
        compatible_schema,
        compatible_trimmed + diagnostics_trimmed,
        unique,
        has_non_empty_diagnostic,
    )

def platform_bundle_template_report_schema_diagnostics(
    template: dict[str, Any],
    label: str = "PlatformBundle report template",
) -> list[str]:
    (
        string_array_schema,
        compatible_profiles_schema,
        string_array_trimmed,
        compatible_profiles_unique,
        has_non_empty_diagnostic,
    ) = template_report_string_array_projection(label, template)
    diagnostics = table_unknown_field_diagnostics(
        label,
        template,
        PLATFORM_BUNDLE_TEMPLATE_REPORT_FIELDS,
    )
    diagnostics.extend(
        table_string_schema_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_required_non_empty_string_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_present_trimmed_non_empty_string_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_STRING_FIELDS,
        )
    )
    diagnostics.extend(
        table_sha256_hex_string_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_SHA256_FIELDS,
        )
    )
    diagnostics.extend(
        table_enum_string_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_ENUM_FIELDS,
        )
    )
    diagnostics.extend(template_report_identity_match_diagnostics(label, template))
    diagnostics.extend(template_report_manifest_path_diagnostics(label, template))
    diagnostics.extend(
        table_integer_schema_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_INTEGER_FIELDS,
        )
    )
    diagnostics.extend(
        table_integer_equals_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_INTEGER_FIELDS,
            EXPORT_TEMPLATE_FORMAT_VERSION,
        )
    )
    diagnostics.extend(
        table_bool_schema_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_BOOL_FIELDS,
        )
    )
    diagnostics.extend(template_report_required_fatal_field_diagnostics(label, template))
    diagnostics.extend(template_report_required_success_evidence_diagnostics(label, template))
    diagnostics.extend(string_array_schema)
    diagnostics.extend(compatible_profiles_schema)
    diagnostics.extend(string_array_trimmed)
    diagnostics.extend(compatible_profiles_unique)
    if template.get("fatal") is True and not has_non_empty_diagnostic:
        diagnostics.append(f"{label} fatal report must include diagnostics")
    if template.get("fatal") is False and has_non_empty_diagnostic:
        diagnostics.append(f"{label} non-fatal report must not include diagnostics")
    diagnostics.extend(template_report_profile_membership_diagnostics(label, template))
    diagnostics.extend(
        table_object_schema_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_OBJECT_FIELDS,
        )
    )
    diagnostics.extend(
        table_object_array_schema_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_OBJECT_ARRAY_FIELDS,
        )
    )
    diagnostics.extend(
        platform_bundle_template_bundle_files_schema_diagnostics(label, template)
    )
    return diagnostics


def template_report_required_fatal_field_diagnostics(
    label: str,
    template: dict[str, Any],
) -> list[str]:
    if "fatal" not in template or template.get("fatal") is None:
        return [f"{label}.fatal must be a boolean"]
    return []


def template_report_required_success_evidence_diagnostics(
    label: str,
    template: dict[str, Any],
) -> list[str]:
    if template.get("fatal") is not False:
        return []
    diagnostics: list[str] = []
    diagnostics.extend(
        required_field_type_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS,
            "a string",
        )
    )
    diagnostics.extend(
        required_field_type_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_INTEGER_FIELDS,
            "an integer",
        )
    )
    diagnostics.extend(
        required_field_type_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_STRING_ARRAY_FIELDS,
            "a string array",
        )
    )
    diagnostics.extend(
        required_field_type_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS,
            "an object",
        )
    )
    diagnostics.extend(
        required_field_type_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS,
            "an object array",
        )
    )
    return diagnostics


def required_field_type_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    type_description: str,
) -> list[str]:
    return [
        f"{label}.{field} must be {type_description}"
        for field in fields
        if field not in table or table.get(field) is None
    ]


def template_report_compatible_profiles_schema_diagnostics(
    label: str,
    template: dict[str, Any],
) -> list[str]:
    value = template.get("compatible_profiles")
    if value is None:
        return []
    field_label = f"{label}.compatible_profiles"
    if not isinstance(value, list):
        return [f"{field_label} must be a string array"]
    diagnostics: list[str] = []
    for index, item in enumerate(value):
        if not isinstance(item, str):
            diagnostics.append(f"{field_label}[{index}] must be a string")
    return diagnostics


def template_report_non_fatal_diagnostics_diagnostics(
    label: str,
    template: dict[str, Any],
) -> list[str]:
    if template.get("fatal") is not False:
        return []
    diagnostics = template.get("diagnostics")
    if (
        isinstance(diagnostics, list)
        and any(isinstance(entry, str) and entry.strip() for entry in diagnostics)
    ):
        return [f"{label} non-fatal report must not include diagnostics"]
    return []


def template_report_fatal_diagnostics_diagnostics(
    label: str,
    template: dict[str, Any],
) -> list[str]:
    if template.get("fatal") is not True:
        return []
    diagnostics = template.get("diagnostics")
    if (
        not isinstance(diagnostics, list)
        or not any(isinstance(entry, str) and entry.strip() for entry in diagnostics)
    ):
        return [f"{label} fatal report must include diagnostics"]
    return []
