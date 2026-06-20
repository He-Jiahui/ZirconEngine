"""PlatformBundle template schema diagnostics."""

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Any

from .export_template import (
    EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS,
    EXPORT_TEMPLATE_ALLOWED_HOST_KINDS,
    EXPORT_TEMPLATE_ALLOWED_PLUGIN_STRATEGIES,
    EXPORT_TEMPLATE_ALLOWED_RESOURCE_STRATEGIES,
    EXPORT_TEMPLATE_FORMAT_VERSION,
    compute_template_content_hash,
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
    normalize_target_platform,
)
from .pipeline_report_platform_bundle_template_manifest_schema import (
    template_report_manifest_path_diagnostics,
)
from .pipeline_report_platform_bundle_template_schema_helpers import (
    sequence_required_non_empty_string_diagnostics,
    sequence_unique_relative_path_field_diagnostics,
    sequence_present_non_blank_string_diagnostics,
    sequence_safe_relative_path_string_diagnostics,
    sequence_sha256_hex_string_diagnostics,
    sequence_string_schema_diagnostics,
    sequence_unique_path_diagnostics,
    sequence_unknown_field_diagnostics,
    table_bool_schema_diagnostics,
    table_bundle_path_string_diagnostics,
    table_enum_string_diagnostics,
    table_integer_equals_diagnostics,
    table_integer_schema_diagnostics,
    table_object_array_schema_diagnostics,
    table_object_schema_diagnostics,
    table_required_non_empty_string_diagnostics,
    table_sha256_hex_string_diagnostics,
    table_string_array_schema_diagnostics,
    table_string_schema_diagnostics,
    table_unknown_field_diagnostics,
    table_unique_string_array_entries_schema_diagnostics,
    table_whitespace_only_string_diagnostics,
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

PLATFORM_BUNDLE_TEMPLATE_REPORT_SHA256_FIELDS = (
    "computed_content_hash",
    "content_hash",
)

PLATFORM_BUNDLE_TEMPLATE_REPORT_ENUM_FIELDS = {
    "bundle_format": EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS,
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
    PLATFORM_BUNDLE_TEMPLATE_REPORT_STRING_ARRAY_FIELDS
)
PLATFORM_BUNDLE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_OBJECT_FIELDS = (
    PLATFORM_BUNDLE_TEMPLATE_REPORT_OBJECT_FIELDS
)
PLATFORM_BUNDLE_TEMPLATE_REPORT_REQUIRED_NON_FATAL_OBJECT_ARRAY_FIELDS = (
    PLATFORM_BUNDLE_TEMPLATE_REPORT_OBJECT_ARRAY_FIELDS
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

PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_FIELDS = ("destination", "source")

PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_STRING_FIELDS = (
    PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_FIELDS
)

def platform_bundle_template_copied_files_schema_diagnostics(
    template_files: list[object],
    label: str = "PlatformBundle report template_files",
) -> list[str]:
    diagnostics: list[str] = []
    seen_entries: dict[tuple[str, str], int] = {}
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
        diagnostics.extend(
            table_required_non_empty_string_diagnostics(
                f"{label}[{index}]",
                entry,
                PLATFORM_BUNDLE_TEMPLATE_COPIED_FILE_STRING_FIELDS,
            )
        )
        source = entry.get("source")
        destination = entry.get("destination")
        if (
            isinstance(source, str)
            and source.strip()
            and isinstance(destination, str)
            and destination.strip()
        ):
            key = (source, destination)
            previous_index = seen_entries.get(key)
            if previous_index is not None:
                diagnostics.append(
                    f"{label}[{index}] duplicates {label}[{previous_index}]"
                )
            else:
                seen_entries[key] = index
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
        table_required_non_empty_string_diagnostics(
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
    diagnostics.extend(
        table_string_array_schema_diagnostics(
            label,
            template,
            PLATFORM_BUNDLE_TEMPLATE_REPORT_STRING_ARRAY_FIELDS,
        )
    )
    diagnostics.extend(
        table_unique_string_array_entries_schema_diagnostics(
            label,
            template,
            ("compatible_profiles",),
        )
    )
    diagnostics.extend(template_report_fatal_diagnostics_diagnostics(label, template))
    diagnostics.extend(template_report_non_fatal_diagnostics_diagnostics(label, template))
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


def template_report_file_source_hash_diagnostics(
    label: str,
    template: dict[str, Any],
    files: list[object],
) -> list[str]:
    template_dir = template.get("template_dir")
    if not isinstance(template_dir, str) or not template_dir.strip():
        return []
    try:
        template_root = Path(template_dir).expanduser().resolve()
    except OSError as error:
        return [f"{label}.template_dir could not be resolved: {error}"]
    diagnostics: list[str] = []
    for index, entry in enumerate(files):
        if not isinstance(entry, dict):
            continue
        relative_path = entry.get("path")
        sha256 = entry.get("sha256")
        if not (
            isinstance(relative_path, str)
            and relative_path.strip()
            and isinstance(sha256, str)
            and is_sha256_hex(sha256)
        ):
            continue
        normalized_path = normalize_relative_path(relative_path)
        if not is_safe_relative_path(normalized_path):
            continue
        unresolved_path = template_root / normalized_path
        try:
            file_path = unresolved_path.resolve()
        except OSError as error:
            diagnostics.append(
                f"{label}.files[{index}].path {unresolved_path} could not be resolved: {error}"
            )
            continue
        try:
            file_path.relative_to(template_root)
        except ValueError:
            diagnostics.append(f"{label}.files[{index}].path must be inside template_dir")
            continue
        if not file_path.exists():
            diagnostics.append(f"{label}.files[{index}].path {file_path} does not exist")
            continue
        if not file_path.is_file():
            diagnostics.append(f"{label}.files[{index}].path {file_path} is not a file")
            continue
        try:
            actual_sha256 = hashlib.sha256(file_path.read_bytes()).hexdigest()
        except OSError as error:
            diagnostics.append(
                f"{label}.files[{index}].path {file_path} could not be read: {error}"
            )
            continue
        if sha256.lower() != actual_sha256:
            diagnostics.append(
                f"{label}.files[{index}].sha256 does not match actual {actual_sha256}"
            )
    return diagnostics


def template_report_content_hash_diagnostics(
    label: str,
    template: dict[str, Any],
    files: list[object],
) -> list[str]:
    normalized_files: list[dict[str, str]] = []
    for entry in files:
        if not isinstance(entry, dict):
            return []
        path = entry.get("path")
        sha256 = entry.get("sha256")
        if not (
            isinstance(path, str)
            and path.strip()
            and isinstance(sha256, str)
            and is_sha256_hex(sha256)
        ):
            return []
        bundle_path = entry.get("bundle_path", "")
        if not isinstance(bundle_path, str):
            return []
        normalized_files.append(
            {
                "path": path,
                "bundle_path": bundle_path,
                "sha256": sha256,
            }
        )
    expected_hash = compute_template_content_hash(normalized_files)
    diagnostics: list[str] = []
    for field in PLATFORM_BUNDLE_TEMPLATE_REPORT_SHA256_FIELDS:
        value = template.get(field)
        if isinstance(value, str) and is_sha256_hex(value) and value.lower() != expected_hash:
            diagnostics.append(
                f"{label}.{field} does not match computed content hash {expected_hash}"
            )
    return diagnostics


def template_report_identity_match_diagnostics(
    label: str,
    template: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    engine_version = template.get("engine_version")
    expected_engine_version = template.get("expected_engine_version")
    if (
        isinstance(engine_version, str)
        and engine_version.strip()
        and isinstance(expected_engine_version, str)
        and expected_engine_version.strip()
        and engine_version != expected_engine_version
    ):
        diagnostics.append(
            f"{label}.engine_version {engine_version} "
            f"does not match expected_engine_version {expected_engine_version}"
        )
    target_platform = template.get("target_platform")
    expected_target_platform = template.get("expected_target_platform")
    if (
        isinstance(target_platform, str)
        and target_platform.strip()
        and isinstance(expected_target_platform, str)
        and expected_target_platform.strip()
        and normalize_target_platform(target_platform)
        != normalize_target_platform(expected_target_platform)
    ):
        diagnostics.append(
            f"{label}.target_platform {target_platform} "
            f"does not match expected_target_platform {expected_target_platform}"
        )
    return diagnostics


def template_report_profile_membership_diagnostics(
    label: str,
    template: dict[str, Any],
) -> list[str]:
    profile = template.get("profile")
    compatible_profiles = template.get("compatible_profiles")
    if (
        not isinstance(profile, str)
        or not profile.strip()
        or not isinstance(compatible_profiles, list)
        or not compatible_profiles
        or any(
            not isinstance(value, str) or not value.strip()
            for value in compatible_profiles
        )
    ):
        return []
    if profile not in compatible_profiles:
        return [
            f"{label}.compatible_profiles does not include profile {profile}"
        ]
    return []


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


def template_report_host_executable_membership_diagnostics(
    label: str,
    template: dict[str, Any],
    files: list[object],
) -> list[str]:
    template_dir = template.get("template_dir")
    host_executable = template.get("host_executable")
    if (
        not isinstance(template_dir, str)
        or not template_dir.strip()
        or not isinstance(host_executable, str)
        or not host_executable.strip()
    ):
        return []
    declared_paths = {
        entry["path"].replace("\\", "/")
        for entry in files
        if isinstance(entry, dict)
        and isinstance(entry.get("path"), str)
        and entry["path"].strip()
    }
    if not declared_paths:
        return []
    try:
        template_root = Path(template_dir).expanduser().resolve()
        host_path = Path(host_executable).expanduser().resolve()
        relative_host = host_path.relative_to(template_root).as_posix()
    except OSError as error:
        return [f"{label}.host_executable could not be resolved: {error}"]
    except ValueError:
        return [
            f"{label}.host_executable must be inside template_dir"
        ]
    if not host_path.exists():
        return [f"{label}.host_executable {host_path} does not exist"]
    if not host_path.is_file():
        return [f"{label}.host_executable {host_path} is not a file"]
    if relative_host not in declared_paths:
        return [
            f"{label}.host_executable must be listed in template.files[].path"
        ]
    return []
