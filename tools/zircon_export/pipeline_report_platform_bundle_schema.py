"""PlatformBundle report and bundle manifest schema diagnostics."""

from __future__ import annotations

from typing import Any

from .pipeline_report_native_dynamic_payload_schema import (
    platform_bundle_native_plugins_payload_schema_diagnostics,
)
from .pipeline_report_platform_bundle_template_schema import (
    platform_bundle_template_copied_files_schema_diagnostics,
    platform_bundle_template_report_schema_diagnostics,
)
from .pipeline_report_platform_bundle_template_resolution_schema import (
    platform_bundle_template_resolution_profile_diagnostics,
    platform_bundle_template_resolution_schema_diagnostics,
    platform_bundle_template_resolution_template_dir_diagnostics,
)
from .pipeline_report_schema_primitives import (
    validate_object_array_schema_diagnostics,
    validate_object_schema_diagnostics,
    validate_string_schema_diagnostics,
)

PLATFORM_BUNDLE_MANIFEST_FIELDS = (
    "profile",
    "template_resolution",
    "template",
    "host_executable",
    "host_source",
    "host_source_origin",
    "pack",
    "pack_source",
    "pack_source_origin",
    "delta_pack",
    "delta_pack_source",
    "delta_pack_source_origin",
    "native_plugins",
    "native_plugins_payload",
    "template_files",
)

PLATFORM_BUNDLE_MANIFEST_STRING_FIELDS = (
    "profile",
    "host_executable",
    "host_source",
    "host_source_origin",
    "pack",
    "pack_source",
    "pack_source_origin",
    "delta_pack",
    "delta_pack_source",
    "delta_pack_source_origin",
    "native_plugins",
)

PLATFORM_BUNDLE_MANIFEST_OBJECT_FIELDS = (
    "template_resolution",
    "template",
    "native_plugins_payload",
)

PLATFORM_BUNDLE_MANIFEST_OBJECT_ARRAY_FIELDS = ("template_files",)

PLATFORM_BUNDLE_REPORT_FIELDS = (
    "stage",
    "profile",
    "bundle",
    "fatal",
    "diagnostics",
    "template_resolution",
    "template",
    "host_executable",
    "host_source",
    "host_source_origin",
    "pack",
    "pack_source",
    "pack_source_origin",
    "delta_pack",
    "delta_pack_source",
    "delta_pack_source_origin",
    "native_plugins",
    "native_plugins_payload",
    "template_files",
    "bundle_manifest",
)

PLATFORM_BUNDLE_REPORT_STRING_FIELDS = (
    "bundle",
    "host_executable",
    "host_source",
    "host_source_origin",
    "pack",
    "pack_source",
    "pack_source_origin",
    "delta_pack",
    "delta_pack_source",
    "delta_pack_source_origin",
    "native_plugins",
    "bundle_manifest",
)

PLATFORM_BUNDLE_REPORT_OBJECT_FIELDS = (
    "template_resolution",
    "template",
    "native_plugins_payload",
)

PLATFORM_BUNDLE_REPORT_OBJECT_ARRAY_FIELDS = ("template_files",)

PLATFORM_BUNDLE_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS = (
    "bundle",
    "host_executable",
    "host_source",
    "host_source_origin",
    "pack",
    "pack_source",
    "pack_source_origin",
    "bundle_manifest",
)

PLATFORM_BUNDLE_REPORT_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS = ("template_files",)


def platform_bundle_manifest_schema_diagnostics(
    manifest: dict[str, Any],
) -> list[str]:
    known_fields = set(PLATFORM_BUNDLE_MANIFEST_FIELDS)
    diagnostics = [
        f"PlatformBundle bundle_manifest unknown field {field}"
        for field in sorted(manifest)
        if field not in known_fields
    ]
    for field in PLATFORM_BUNDLE_MANIFEST_STRING_FIELDS:
        if field in manifest and manifest.get(field) is not None:
            value = manifest.get(field)
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"PlatformBundle bundle_manifest {field}",
                    value,
                )
            )
            if isinstance(value, str) and not value.strip():
                diagnostics.append(
                    f"PlatformBundle bundle_manifest {field} must be a non-empty string"
                )
    for field in PLATFORM_BUNDLE_MANIFEST_OBJECT_FIELDS:
        if field in manifest and manifest.get(field) is not None:
            diagnostics.extend(
                validate_object_schema_diagnostics(
                    f"PlatformBundle bundle_manifest {field}",
                    manifest.get(field),
                )
            )
    for field in PLATFORM_BUNDLE_MANIFEST_OBJECT_ARRAY_FIELDS:
        if field in manifest and manifest.get(field) is not None:
            diagnostics.extend(
                validate_object_array_schema_diagnostics(
                    f"PlatformBundle bundle_manifest {field}",
                    manifest.get(field),
                )
            )
    template_resolution = manifest.get("template_resolution")
    if isinstance(template_resolution, dict):
        diagnostics.extend(
            platform_bundle_template_resolution_schema_diagnostics(
                template_resolution,
                label="PlatformBundle bundle_manifest template_resolution",
            )
        )
    template = manifest.get("template")
    if isinstance(template, dict):
        diagnostics.extend(
            platform_bundle_template_report_schema_diagnostics(
                template,
                label="PlatformBundle bundle_manifest template",
            )
        )
    diagnostics.extend(
        platform_bundle_template_resolution_profile_diagnostics(
            manifest,
            label="PlatformBundle bundle_manifest",
        )
    )
    diagnostics.extend(
        platform_bundle_template_resolution_template_dir_diagnostics(
            manifest,
            label="PlatformBundle bundle_manifest",
        )
    )
    native_plugins_payload = manifest.get("native_plugins_payload")
    if isinstance(native_plugins_payload, dict):
        diagnostics.extend(
            platform_bundle_native_plugins_payload_schema_diagnostics(
                native_plugins_payload,
                label="PlatformBundle bundle_manifest native_plugins_payload",
            )
        )
    template_files = manifest.get("template_files")
    if isinstance(template_files, list):
        diagnostics.extend(
            platform_bundle_template_copied_files_schema_diagnostics(
                template_files,
                label="PlatformBundle bundle_manifest template_files",
            )
        )
    return diagnostics


def platform_bundle_report_schema_diagnostics(report: dict[str, Any]) -> list[str]:
    known_fields = set(PLATFORM_BUNDLE_REPORT_FIELDS)
    diagnostics = [
        f"PlatformBundle report unknown field {field}"
        for field in sorted(report)
        if field not in known_fields
    ]
    for field in PLATFORM_BUNDLE_REPORT_STRING_FIELDS:
        if field in report and report.get(field) is not None:
            value = report.get(field)
            diagnostics.extend(
                validate_string_schema_diagnostics(
                    f"PlatformBundle report {field}",
                    value,
                )
            )
            if isinstance(value, str) and not value.strip():
                diagnostics.append(
                    f"PlatformBundle report {field} must be a non-empty string"
                )
    for field in PLATFORM_BUNDLE_REPORT_OBJECT_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_object_schema_diagnostics(
                    f"PlatformBundle report {field}",
                    report.get(field),
                )
            )
    for field in PLATFORM_BUNDLE_REPORT_OBJECT_ARRAY_FIELDS:
        if field in report and report.get(field) is not None:
            diagnostics.extend(
                validate_object_array_schema_diagnostics(
                    f"PlatformBundle report {field}",
                    report.get(field),
                )
            )
    if report.get("fatal") is False:
        for field in PLATFORM_BUNDLE_REPORT_REQUIRED_NON_FATAL_STRING_FIELDS:
            if field not in report or report.get(field) is None:
                diagnostics.extend(
                    validate_string_schema_diagnostics(
                        f"PlatformBundle report {field}",
                        report.get(field),
                    )
                )
                continue
        for field in PLATFORM_BUNDLE_REPORT_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS:
            if field not in report:
                diagnostics.extend(
                    validate_object_array_schema_diagnostics(
                        f"PlatformBundle report {field}",
                        report.get(field),
                    )
                )
    template_resolution = report.get("template_resolution")
    if isinstance(template_resolution, dict):
        diagnostics.extend(
            platform_bundle_template_resolution_schema_diagnostics(
                template_resolution,
                label="PlatformBundle report template_resolution",
            )
        )
    template = report.get("template")
    if isinstance(template, dict):
        diagnostics.extend(
            platform_bundle_template_report_schema_diagnostics(
                template,
                label="PlatformBundle report template",
            )
        )
    diagnostics.extend(platform_bundle_template_resolution_profile_diagnostics(report))
    diagnostics.extend(
        platform_bundle_template_resolution_template_dir_diagnostics(report)
    )
    native_plugins_payload = report.get("native_plugins_payload")
    if isinstance(native_plugins_payload, dict):
        diagnostics.extend(
            platform_bundle_native_plugins_payload_schema_diagnostics(
                native_plugins_payload,
                label="PlatformBundle report native_plugins_payload",
            )
        )
    template_files = report.get("template_files")
    if isinstance(template_files, list):
        diagnostics.extend(
            platform_bundle_template_copied_files_schema_diagnostics(
                template_files,
                label="PlatformBundle report template_files",
            )
        )
    return diagnostics
