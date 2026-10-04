"""Field definitions and state diagnostics for embedded template reports."""

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

PLATFORM_BUNDLE_TEMPLATE_REPORT_STRING_ARRAY_FIELDS = ("diagnostics",)

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
