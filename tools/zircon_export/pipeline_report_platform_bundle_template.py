"""PlatformBundle template release-evidence diagnostics."""

from __future__ import annotations

import hashlib
from collections.abc import Callable
from pathlib import Path
from typing import Any

from .pipeline_report_schema_primitives import (
    validate_bool_schema_diagnostics,
    validate_integer_schema_diagnostics,
    validate_object_array_schema_diagnostics,
    validate_object_schema_diagnostics,
    validate_string_array_schema_diagnostics,
    validate_string_schema_diagnostics,
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
    "compatible_profiles",
    "diagnostics",
)

PLATFORM_BUNDLE_TEMPLATE_REPORT_OBJECT_FIELDS = ("bundle",)

PLATFORM_BUNDLE_TEMPLATE_REPORT_OBJECT_ARRAY_FIELDS = ("files",)

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

PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_FIELDS = ("destination", "source")

PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_STRING_FIELDS = (
    PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_FIELDS
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
    return platform_bundle_template_resolution_schema_diagnostics(resolution)


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
            label=label,
        )
    )
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
    return diagnostics


def template_resolution_sequence_schema_diagnostics(
    resolution: dict[str, Any],
    field: str,
    allowed_fields: tuple[str, ...],
    *,
    string_fields: tuple[str, ...],
    string_array_fields: tuple[str, ...] = (),
    label: str = "PlatformBundle report template_resolution",
) -> list[str]:
    value = resolution.get(field)
    if value is None:
        return []
    if not isinstance(value, list):
        return [f"{label} {field} must be a list"]
    field_label = f"{label} {field}"
    diagnostics = sequence_object_schema_diagnostics(
        field_label,
        value,
    )
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
        sequence_string_array_schema_diagnostics(
            field_label,
            value,
            string_array_fields,
        )
    )
    return diagnostics


def sequence_object_schema_diagnostics(
    label: str,
    value: object,
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            diagnostics.append(f"{label}[{index}] must be an object")
    return diagnostics


def platform_bundle_template_files_diagnostics(
    report: dict[str, Any],
) -> list[str]:
    template_files = report.get("template_files")
    if template_files is None:
        return []
    if not isinstance(template_files, list):
        return ["PlatformBundle report template_files must be a list"]
    if not template_files:
        return []
    template = report.get("template")
    if not isinstance(template, dict):
        return [
            "PlatformBundle report template_files are present but template report is missing"
        ]

    diagnostics: list[str] = []
    diagnostics.extend(platform_bundle_template_report_schema_diagnostics(template))
    host_executable = report.get("host_executable")
    host_path = (
        resolve_user_path_or_diagnostic(
            host_executable,
            diagnostics,
            "PlatformBundle report host_executable",
        )
        if isinstance(host_executable, str) and host_executable
        else None
    )
    expected_hashes = platform_bundle_template_file_hashes(template, diagnostics)

    for index, entry in enumerate(template_files):
        if not isinstance(entry, dict):
            diagnostics.append(
                f"PlatformBundle report template_files entry {index} must be an object"
            )
            continue
        diagnostics.extend(
            table_unknown_field_diagnostics(
                f"PlatformBundle report template_files[{index}]",
                entry,
                PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_FIELDS,
            )
        )
        diagnostics.extend(
            table_string_schema_diagnostics(
                f"PlatformBundle report template_files[{index}]",
                entry,
                PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_STRING_FIELDS,
            )
        )
        destination = entry.get("destination")
        if not isinstance(destination, str) or not destination:
            diagnostics.append(
                f"PlatformBundle report template_files entry {index} destination must be a non-empty string"
            )
            continue
        destination_path = resolve_user_path_or_diagnostic(
            destination,
            diagnostics,
            "PlatformBundle report template_files destination",
        )
        if destination_path is None:
            continue
        if host_path is not None and destination_path == host_path:
            continue
        if not destination_path.exists():
            diagnostics.append(
                f"PlatformBundle report template_files destination {destination_path} does not exist"
            )
            continue
        if not destination_path.is_file():
            diagnostics.append(
                f"PlatformBundle report template_files destination {destination_path} is not a file"
            )
            continue
        expected_sha256 = platform_bundle_template_file_expected_hash(
            entry,
            expected_hashes,
            diagnostics,
        )
        if expected_sha256 is None:
            diagnostics.append(
                f"PlatformBundle report template_files entry {index} cannot be matched to template file sha256"
            )
            continue
        actual_sha256 = platform_bundle_file_sha256(
            destination_path,
            diagnostics,
            f"PlatformBundle report template_files destination {destination_path}",
        )
        if actual_sha256 is None:
            continue
        if actual_sha256 != expected_sha256:
            diagnostics.append(
                "PlatformBundle report template_files destination "
                f"{destination_path} sha256 {actual_sha256} does not match "
                f"template sha256 {expected_sha256}"
            )
    return diagnostics


def platform_bundle_template_copied_files_schema_diagnostics(
    template_files: list[object],
    label: str = "PlatformBundle report template_files",
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(template_files):
        if not isinstance(entry, dict):
            diagnostics.append(f"{label}[{index}] must be an object")
            continue
        diagnostics.extend(
            table_unknown_field_diagnostics(
                f"{label}[{index}]",
                entry,
                PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_FIELDS,
            )
        )
        diagnostics.extend(
            table_string_schema_diagnostics(
                f"{label}[{index}]",
                entry,
                PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_STRING_FIELDS,
            )
        )
    return diagnostics


def platform_bundle_template_report_schema_diagnostics(
    template: dict[str, Any],
    label: str = "PlatformBundle report template",
) -> list[str]:
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
        table_integer_schema_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_INTEGER_FIELDS,
        )
    )
    diagnostics.extend(
        table_bool_schema_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_BOOL_FIELDS,
        )
    )
    diagnostics.extend(
        table_string_array_schema_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_STRING_ARRAY_FIELDS,
        )
    )
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
    return diagnostics


def platform_bundle_template_file_hashes(
    template: dict[str, Any],
    diagnostics: list[str],
) -> dict[str, str]:
    template_dir = template.get("template_dir")
    files = template.get("files")
    if not isinstance(template_dir, str) or not template_dir:
        diagnostics.append(
            "PlatformBundle report template.template_dir must be a non-empty string when template_files are present"
        )
        return {}
    if not isinstance(files, list):
        diagnostics.append(
            "PlatformBundle report template.files must be a list when template_files are present"
        )
        return {}

    hashes: dict[str, str] = {}
    root = resolve_user_path_or_diagnostic(
        template_dir,
        diagnostics,
        "PlatformBundle report template.template_dir",
    )
    if root is None:
        return hashes
    for index, entry in enumerate(files):
        if not isinstance(entry, dict):
            diagnostics.append(
                f"PlatformBundle report template.files entry {index} must be an object"
            )
            continue
        relative_path = entry.get("path")
        sha256 = entry.get("sha256")
        if not isinstance(relative_path, str) or not relative_path:
            diagnostics.append(
                f"PlatformBundle report template.files entry {index} path must be a non-empty string"
            )
            continue
        if not isinstance(sha256, str) or not sha256:
            diagnostics.append(
                f"PlatformBundle report template.files entry {index} sha256 must be a non-empty string"
            )
            continue
        file_path = resolve_user_path_or_diagnostic(
            root / relative_path,
            diagnostics,
            f"PlatformBundle report template.files entry {index} path",
        )
        if file_path is None:
            continue
        hashes[str(file_path)] = sha256.lower()
    return hashes


def platform_bundle_template_file_expected_hash(
    entry: dict[str, Any],
    expected_hashes: dict[str, str],
    diagnostics: list[str],
) -> str | None:
    source = entry.get("source")
    if not isinstance(source, str) or not source:
        return None
    source_path = resolve_user_path_or_diagnostic(
        source,
        diagnostics,
        "PlatformBundle report template_files source",
    )
    if source_path is None:
        return None
    return expected_hashes.get(str(source_path))


def table_unknown_field_diagnostics(
    label: str,
    table: dict[str, Any],
    known_fields: tuple[str, ...],
) -> list[str]:
    known_field_set = set(known_fields)
    return [
        f"{label} unknown field {field}"
        for field in sorted(table)
        if field not in known_field_set
    ]


def sequence_unknown_field_diagnostics(
    label: str,
    value: object,
    known_fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_unknown_field_diagnostics(
                f"{label}[{index}]",
                entry,
                known_fields,
            )
        )
    return diagnostics


def table_string_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    return table_field_schema_diagnostics(
        label, table, fields, validate_string_schema_diagnostics
    )


def table_integer_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    return table_field_schema_diagnostics(
        label, table, fields, validate_integer_schema_diagnostics
    )


def table_bool_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    return table_field_schema_diagnostics(
        label, table, fields, validate_bool_schema_diagnostics
    )


def table_string_array_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    return table_field_schema_diagnostics(
        label, table, fields, validate_string_array_schema_diagnostics
    )


def table_object_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    return table_field_schema_diagnostics(
        label, table, fields, validate_object_schema_diagnostics
    )


def table_object_array_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
) -> list[str]:
    return table_field_schema_diagnostics(
        label, table, fields, validate_object_array_schema_diagnostics
    )


def table_field_schema_diagnostics(
    label: str,
    table: dict[str, Any],
    fields: tuple[str, ...],
    validate_schema: Callable[[str, Any], list[str]],
) -> list[str]:
    diagnostics: list[str] = []
    for field in fields:
        if field in table and table.get(field) is not None:
            diagnostics.extend(validate_schema(f"{label}.{field}", table.get(field)))
    return diagnostics


def sequence_string_schema_diagnostics(
    label: str,
    value: object,
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_string_schema_diagnostics(
                f"{label}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics


def sequence_string_array_schema_diagnostics(
    label: str,
    value: object,
    fields: tuple[str, ...],
) -> list[str]:
    diagnostics: list[str] = []
    for index, entry in enumerate(value):
        if not isinstance(entry, dict):
            continue
        diagnostics.extend(
            table_string_array_schema_diagnostics(
                f"{label}[{index}]",
                entry,
                fields,
            )
        )
    return diagnostics


def resolve_user_path(path: str | Path) -> Path:
    return Path(path).expanduser().resolve()


def resolve_user_path_or_diagnostic(
    path: str | Path,
    diagnostics: list[str],
    label: str,
) -> Path | None:
    try:
        return resolve_user_path(path)
    except OSError as error:
        diagnostics.append(f"{label} {path} could not be resolved: {error}")
        return None


def platform_bundle_file_sha256(
    path: Path,
    diagnostics: list[str],
    label: str,
) -> str | None:
    try:
        return hashlib.sha256(path.read_bytes()).hexdigest()
    except OSError as error:
        diagnostics.append(f"{label} could not be read: {error}")
        return None
