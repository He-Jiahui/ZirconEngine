"""Manifest/report identity diagnostics for PlatformBundle template reports."""

from __future__ import annotations

from pathlib import Path
import tomllib
from typing import Any

from .export_template import (
    EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS,
    EXPORT_TEMPLATE_ALLOWED_HOST_ARTIFACTS,
    EXPORT_TEMPLATE_ALLOWED_HOST_KINDS,
    EXPORT_TEMPLATE_ALLOWED_PLUGIN_STRATEGIES,
    EXPORT_TEMPLATE_ALLOWED_RESOURCE_STRATEGIES,
    EXPORT_TEMPLATE_BUNDLE_FIELDS,
    EXPORT_TEMPLATE_FILE_FIELDS,
    EXPORT_TEMPLATE_FORMAT_VERSION,
    EXPORT_TEMPLATE_MANIFEST_FIELDS,
    EXPORT_TEMPLATE_PATHS_FIELDS,
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
    normalize_target_platform,
)
from .pipeline_report_schema_table import (
    string_array_no_blank_entries_schema_diagnostics,
    string_array_unique_entries_schema_diagnostics,
)
from .pipeline_report_platform_bundle_template_schema_helpers import (
    table_unknown_field_diagnostics,
)

TEMPLATE_MANIFEST_BUNDLE_DEFAULTS = {
    "delta_pack_path": "",
    "host_path": "",
    "manifest_path": "bundle.json",
    "pack_path": "",
    "root": ".",
}

TEMPLATE_MANIFEST_REQUIRED_STRING_FIELDS = (
    "template_id",
    "engine_version",
    "target_platform",
    "host_kind",
    "host_artifact",
    "resource_strategy",
    "plugin_strategy",
    "bundle_format",
    "content_hash",
)

TEMPLATE_MANIFEST_ALLOWED_STRING_FIELDS = {
    "bundle_format": EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS,
    "host_artifact": EXPORT_TEMPLATE_ALLOWED_HOST_ARTIFACTS,
    "host_kind": EXPORT_TEMPLATE_ALLOWED_HOST_KINDS,
    "plugin_strategy": EXPORT_TEMPLATE_ALLOWED_PLUGIN_STRATEGIES,
    "resource_strategy": EXPORT_TEMPLATE_ALLOWED_RESOURCE_STRATEGIES,
}


def template_report_manifest_path_diagnostics(
    label: str,
    template: dict[str, Any],
) -> list[str]:
    template_dir = template.get("template_dir")
    manifest = template.get("manifest")
    if (
        not isinstance(template_dir, str)
        or not template_dir.strip()
        or not isinstance(manifest, str)
        or not manifest.strip()
    ):
        return []
    try:
        expected_manifest = (Path(template_dir).expanduser() / "template.toml").resolve()
        actual_manifest = Path(manifest).expanduser().resolve()
    except OSError as error:
        return [f"{label}.manifest could not be resolved: {error}"]
    if actual_manifest != expected_manifest:
        return [
            f"{label}.manifest does not match template_dir/template.toml"
        ]
    if not actual_manifest.exists():
        return [f"{label}.manifest {actual_manifest} does not exist"]
    if not actual_manifest.is_file():
        return [f"{label}.manifest {actual_manifest} is not a file"]
    try:
        with actual_manifest.open("rb") as manifest_file:
            manifest = tomllib.load(manifest_file)
    except OSError as error:
        return [f"{label}.manifest {actual_manifest} could not be read: {error}"]
    except tomllib.TOMLDecodeError as error:
        return [f"{label}.manifest {actual_manifest} is not valid TOML: {error}"]
    if not isinstance(manifest, dict):
        return [f"{label}.manifest {actual_manifest} must be a TOML table"]
    format_version = manifest.get("format_version")
    if type(format_version) is not int:
        return [f"{label}.manifest format_version must be an integer"]
    if format_version != EXPORT_TEMPLATE_FORMAT_VERSION:
        return [
            f"{label}.manifest format_version {format_version} is not supported; "
            f"expected {EXPORT_TEMPLATE_FORMAT_VERSION}"
        ]
    unknown_field_diagnostic = template_manifest_unknown_field_diagnostic(
        label,
        manifest,
    )
    if unknown_field_diagnostic:
        return [unknown_field_diagnostic]
    shape_diagnostic = template_manifest_shape_diagnostic(label, manifest)
    if shape_diagnostic:
        return [shape_diagnostic]
    scalar_diagnostic = template_manifest_scalar_field_diagnostic(label, manifest)
    if scalar_diagnostic:
        return [scalar_diagnostic]
    compatible_profiles_schema_diagnostic = (
        template_manifest_compatible_profiles_schema_diagnostic(label, manifest)
    )
    if compatible_profiles_schema_diagnostic:
        return [compatible_profiles_schema_diagnostic]
    paths_diagnostic = template_manifest_paths_schema_diagnostic(label, manifest)
    if paths_diagnostic:
        return [paths_diagnostic]
    bundle_schema_diagnostic = template_manifest_bundle_schema_diagnostic(
        label,
        manifest,
    )
    if bundle_schema_diagnostic:
        return [bundle_schema_diagnostic]
    files_schema_diagnostic = template_manifest_files_schema_diagnostic(
        label,
        manifest,
    )
    if files_schema_diagnostic:
        return [files_schema_diagnostic]
    files_presence_diagnostic = template_manifest_files_presence_diagnostic(
        label,
        manifest,
    )
    if files_presence_diagnostic:
        return [files_presence_diagnostic]
    files_unique_diagnostic = template_manifest_files_unique_diagnostic(
        label,
        manifest,
    )
    if files_unique_diagnostic:
        return [files_unique_diagnostic]
    for field in (
        "template_id",
        "engine_version",
        "host_kind",
        "host_artifact",
        "resource_strategy",
        "plugin_strategy",
        "bundle_format",
        "content_hash",
    ):
        identity_diagnostic = template_manifest_string_identity_diagnostic(
            label,
            manifest,
            template,
            field,
        )
        if identity_diagnostic:
            return [identity_diagnostic]
    target_platform_diagnostic = template_manifest_target_platform_identity_diagnostic(
        label,
        manifest,
        template,
    )
    if target_platform_diagnostic:
        return [target_platform_diagnostic]
    compatible_profiles_diagnostic = (
        template_manifest_compatible_profiles_identity_diagnostic(
            label,
            manifest,
            template,
        )
    )
    if compatible_profiles_diagnostic:
        return [compatible_profiles_diagnostic]
    host_executable_diagnostic = template_manifest_host_executable_identity_diagnostic(
        label,
        manifest,
        template,
        expected_manifest.parent,
    )
    if host_executable_diagnostic:
        return [host_executable_diagnostic]
    bundle_diagnostic = template_manifest_bundle_identity_diagnostic(
        label,
        manifest,
        template,
    )
    if bundle_diagnostic:
        return [bundle_diagnostic]
    files_diagnostic = template_manifest_files_identity_diagnostic(
        label,
        manifest,
        template,
    )
    if files_diagnostic:
        return [files_diagnostic]
    return []


def template_manifest_unknown_field_diagnostic(
    label: str,
    manifest: dict[str, Any],
) -> str | None:
    diagnostics = table_unknown_field_diagnostics(
        f"{label}.manifest",
        manifest,
        EXPORT_TEMPLATE_MANIFEST_FIELDS,
    )
    if diagnostics:
        return diagnostics[0]

    paths = manifest.get("paths")
    if isinstance(paths, dict):
        diagnostics = table_unknown_field_diagnostics(
            f"{label}.manifest paths",
            paths,
            EXPORT_TEMPLATE_PATHS_FIELDS,
        )
        if diagnostics:
            return diagnostics[0]

    bundle = manifest.get("bundle")
    if isinstance(bundle, dict):
        diagnostics = table_unknown_field_diagnostics(
            f"{label}.manifest bundle",
            bundle,
            EXPORT_TEMPLATE_BUNDLE_FIELDS,
        )
        if diagnostics:
            return diagnostics[0]

    files = manifest.get("files")
    if isinstance(files, list):
        for index, entry in enumerate(files):
            if not isinstance(entry, dict):
                continue
            diagnostics = table_unknown_field_diagnostics(
                f"{label}.manifest files[{index}]",
                entry,
                EXPORT_TEMPLATE_FILE_FIELDS,
            )
            if diagnostics:
                return diagnostics[0]

    return None


def template_manifest_shape_diagnostic(
    label: str,
    manifest: dict[str, Any],
) -> str | None:
    paths = manifest.get("paths")
    if paths is not None and not isinstance(paths, dict):
        return f"{label}.manifest table [paths] is required"

    bundle = manifest.get("bundle")
    if bundle is not None and not isinstance(bundle, dict):
        return f"{label}.manifest table [bundle] must be a table when present"

    files = manifest.get("files")
    if files is not None and not isinstance(files, list):
        return f"{label}.manifest [[files]] entries must form an array"
    if isinstance(files, list):
        for index, entry in enumerate(files):
            if not isinstance(entry, dict):
                return f"{label}.manifest [[files]] entry {index} must be a table"

    return None


def template_manifest_scalar_field_diagnostic(
    label: str,
    manifest: dict[str, Any],
) -> str | None:
    for field in TEMPLATE_MANIFEST_REQUIRED_STRING_FIELDS:
        value = manifest.get(field)
        if not isinstance(value, str) or not value.strip():
            return f"{label}.manifest field {field} must be a non-empty string"
        if value.strip() != value:
            return (
                f"{label}.manifest field {field} "
                "must be a non-empty trimmed string"
            )

    content_hash = manifest["content_hash"]
    if not is_sha256_hex(content_hash):
        return f"{label}.manifest field content_hash must be a SHA-256 hex digest"

    for field, allowed_values in TEMPLATE_MANIFEST_ALLOWED_STRING_FIELDS.items():
        value = manifest[field]
        if value not in allowed_values:
            return (
                f"{label}.manifest field {field}={value!r} is not one of "
                f"{', '.join(sorted(allowed_values))}"
            )

    return None


def template_manifest_compatible_profiles_schema_diagnostic(
    label: str,
    manifest: dict[str, Any],
) -> str | None:
    value = manifest.get("compatible_profiles", [])
    if value is None:
        return None
    if not isinstance(value, list):
        return f"{label}.manifest field compatible_profiles must be a string array"
    for index, item in enumerate(value):
        if not isinstance(item, str):
            return (
                f"{label}.manifest field compatible_profiles[{index}] "
                "must be a string"
            )
    diagnostics = string_array_no_blank_entries_schema_diagnostics(
        f"{label}.manifest field compatible_profiles",
        value,
    )
    if diagnostics:
        return diagnostics[0]
    for index, item in enumerate(value):
        if item.strip() and item.strip() != item:
            return (
                f"{label}.manifest field compatible_profiles[{index}] "
                "must be a non-empty trimmed string"
            )
    diagnostics = string_array_unique_entries_schema_diagnostics(
        f"{label}.manifest field compatible_profiles",
        value,
    )
    if diagnostics:
        return diagnostics[0]
    return None


def template_manifest_paths_schema_diagnostic(
    label: str,
    manifest: dict[str, Any],
) -> str | None:
    paths = manifest.get("paths")
    if not isinstance(paths, dict):
        return f"{label}.manifest table [paths] is required"
    host_executable = paths.get("host_executable")
    if not isinstance(host_executable, str) or not host_executable.strip():
        return (
            f"{label}.manifest field paths.host_executable "
            "must be a non-empty string"
        )
    if host_executable.strip() != host_executable:
        return (
            f"{label}.manifest field paths.host_executable "
            "must be a non-empty trimmed string"
        )
    normalized_host = normalize_relative_path(host_executable)
    if not is_safe_relative_path(normalized_host):
        return (
            f"{label}.manifest field paths.host_executable "
            "must be a safe relative path"
        )
    return None


def template_manifest_bundle_schema_diagnostic(
    label: str,
    manifest: dict[str, Any],
) -> str | None:
    bundle = manifest.get("bundle", {})
    if bundle is None:
        return None
    if not isinstance(bundle, dict):
        return f"{label}.manifest table [bundle] must be a table when present"
    for field in TEMPLATE_MANIFEST_BUNDLE_DEFAULTS:
        if field not in bundle:
            continue
        value = bundle[field]
        if not isinstance(value, str):
            return f"{label}.manifest field bundle.{field} must be a string"
        if not value.strip():
            return (
                f"{label}.manifest field bundle.{field} "
                "must be a non-empty string"
            )
        if value.strip() != value:
            return (
                f"{label}.manifest field bundle.{field} "
                "must be a non-empty trimmed string"
            )
        normalized = normalize_relative_path(value)
        if normalized in {"", "."}:
            continue
        if not is_safe_relative_path(normalized):
            return (
                f"{label}.manifest field bundle.{field} "
                "must be a safe relative path"
            )
    return None


def template_manifest_files_schema_diagnostic(
    label: str,
    manifest: dict[str, Any],
) -> str | None:
    files = manifest.get("files")
    if not isinstance(files, list):
        return None
    for index, entry in enumerate(files):
        if not isinstance(entry, dict):
            continue
        relative_path = entry.get("path")
        if not isinstance(relative_path, str) or not relative_path.strip():
            return f"{label}.manifest [[files]] entry {index} needs a non-empty path"
        if relative_path.strip() != relative_path:
            return (
                f"{label}.manifest [[files]] entry {index} path "
                "must be a non-empty trimmed string"
            )
        normalized_path = normalize_relative_path(relative_path)
        if not is_safe_relative_path(normalized_path):
            return (
                f"{label}.manifest [[files]] entry {index} "
                "path must be a safe relative path"
            )

        declared_sha256 = entry.get("sha256")
        if not isinstance(declared_sha256, str) or not declared_sha256.strip():
            return (
                f"{label}.manifest file {normalized_path} "
                "must declare a SHA-256 hex digest"
            )
        if declared_sha256.strip() != declared_sha256:
            return (
                f"{label}.manifest file {normalized_path} sha256 "
                "must be a non-empty trimmed string"
            )
        if not is_sha256_hex(declared_sha256):
            return (
                f"{label}.manifest file {normalized_path} "
                "must declare a SHA-256 hex digest"
            )

        bundle_path = entry.get("bundle_path", normalized_path)
        if bundle_path is None:
            bundle_path = normalized_path
        if not isinstance(bundle_path, str) or not bundle_path.strip():
            return (
                f"{label}.manifest file {normalized_path} "
                "has an invalid bundle_path"
            )
        if bundle_path.strip() != bundle_path:
            return (
                f"{label}.manifest file {normalized_path} bundle_path "
                "must be a non-empty trimmed string"
            )
        normalized_bundle_path = normalize_relative_path(bundle_path)
        if not is_safe_relative_path(normalized_bundle_path):
            return (
                f"{label}.manifest file {normalized_path} "
                "bundle_path must be a safe relative path"
            )

        purpose = entry.get("purpose", "")
        if not isinstance(purpose, str):
            return f"{label}.manifest file {normalized_path} purpose must be a string"
        if "purpose" in entry and not purpose.strip():
            return (
                f"{label}.manifest file {normalized_path} "
                "purpose must be non-empty when present"
            )
        if "purpose" in entry and purpose.strip() != purpose:
            return (
                f"{label}.manifest file {normalized_path} purpose "
                "must be a non-empty trimmed string"
            )
    return None


def template_manifest_files_presence_diagnostic(
    label: str,
    manifest: dict[str, Any],
) -> str | None:
    files = manifest.get("files")
    if not isinstance(files, list) or not files:
        return f"{label}.manifest must declare at least one [[files]] entry"
    return None


def template_manifest_files_unique_diagnostic(
    label: str,
    manifest: dict[str, Any],
) -> str | None:
    files = manifest.get("files")
    if not isinstance(files, list):
        return None
    seen_paths: set[str] = set()
    seen_bundle_paths: set[str] = set()
    for entry in files:
        if not isinstance(entry, dict):
            continue
        relative_path = entry.get("path")
        if not isinstance(relative_path, str) or not relative_path.strip():
            continue
        normalized_path = normalize_relative_path(relative_path)
        if not is_safe_relative_path(normalized_path):
            continue
        if normalized_path in seen_paths:
            return (
                f"{label}.manifest template file {normalized_path} "
                "is declared more than once"
            )
        seen_paths.add(normalized_path)

        bundle_path = template_manifest_file_bundle_path(entry, normalized_path)
        if not bundle_path or not is_safe_relative_path(bundle_path):
            continue
        if bundle_path in seen_bundle_paths:
            return (
                f"{label}.manifest template bundle path {bundle_path} "
                "is declared more than once"
            )
        seen_bundle_paths.add(bundle_path)
    return None


def template_manifest_string_identity_diagnostic(
    label: str,
    manifest: dict[str, Any],
    template: dict[str, Any],
    field: str,
) -> str | None:
    manifest_value = manifest.get(field)
    report_value = template.get(field)
    if (
        isinstance(manifest_value, str)
        and manifest_value.strip()
        and isinstance(report_value, str)
        and report_value.strip()
        and manifest_value != report_value
    ):
        return (
            f"{label}.manifest {field} {manifest_value} "
            f"does not match template.{field} {report_value}"
        )
    return None


def template_manifest_target_platform_identity_diagnostic(
    label: str,
    manifest: dict[str, Any],
    template: dict[str, Any],
) -> str | None:
    manifest_value = manifest.get("target_platform")
    report_value = template.get("target_platform")
    if (
        isinstance(manifest_value, str)
        and manifest_value.strip()
        and isinstance(report_value, str)
        and report_value.strip()
        and normalize_target_platform(manifest_value)
        != normalize_target_platform(report_value)
    ):
        return (
            f"{label}.manifest target_platform {manifest_value} "
            f"does not match template.target_platform {report_value}"
        )
    return None


def template_manifest_compatible_profiles_identity_diagnostic(
    label: str,
    manifest: dict[str, Any],
    template: dict[str, Any],
) -> str | None:
    manifest_value = manifest.get("compatible_profiles")
    report_value = template.get("compatible_profiles")
    if (
        not isinstance(manifest_value, list)
        or not isinstance(report_value, list)
        or any(
            not isinstance(value, str) or not value.strip()
            for value in manifest_value
        )
        or any(
            not isinstance(value, str) or not value.strip()
            for value in report_value
        )
    ):
        return None
    if manifest_value != report_value:
        return (
            f"{label}.manifest compatible_profiles "
            "does not match template.compatible_profiles"
        )
    return None


def template_manifest_host_executable_identity_diagnostic(
    label: str,
    manifest: dict[str, Any],
    template: dict[str, Any],
    template_root: Path,
) -> str | None:
    paths = manifest.get("paths")
    report_value = template.get("host_executable")
    if not isinstance(paths, dict) or not isinstance(report_value, str):
        return None
    manifest_value = paths.get("host_executable")
    if not (
        isinstance(manifest_value, str)
        and manifest_value.strip()
        and report_value.strip()
    ):
        return None
    normalized_host = normalize_relative_path(manifest_value)
    if not is_safe_relative_path(normalized_host):
        return None
    try:
        expected_host = (template_root / normalized_host).resolve()
        actual_host = Path(report_value).expanduser().resolve()
    except OSError as error:
        return f"{label}.manifest paths.host_executable could not be resolved: {error}"
    if expected_host != actual_host:
        return (
            f"{label}.manifest paths.host_executable {normalized_host} "
            f"does not match template.host_executable {report_value}"
        )
    return None


def template_manifest_bundle_identity_diagnostic(
    label: str,
    manifest: dict[str, Any],
    template: dict[str, Any],
) -> str | None:
    manifest_bundle = manifest.get("bundle", {})
    report_bundle = template.get("bundle")
    if manifest_bundle is None:
        manifest_bundle = {}
    if not isinstance(manifest_bundle, dict) or not isinstance(report_bundle, dict):
        return None
    for field, default in TEMPLATE_MANIFEST_BUNDLE_DEFAULTS.items():
        manifest_value = manifest_bundle.get(field, default)
        report_value = report_bundle.get(field)
        if manifest_value is None:
            manifest_value = default
        if not (
            isinstance(manifest_value, str)
            and isinstance(report_value, str)
            and template_bundle_identity_value_is_schema_clean(manifest_value)
            and template_bundle_identity_value_is_schema_clean(report_value)
        ):
            continue
        if manifest_value != report_value:
            return (
                f"{label}.manifest bundle.{field} {manifest_value} "
                f"does not match template.bundle.{field} {report_value}"
            )
    return None


def template_bundle_identity_value_is_schema_clean(value: str) -> bool:
    if not value:
        return True
    return bool(value.strip()) and is_safe_relative_path(normalize_relative_path(value))


def template_manifest_files_identity_diagnostic(
    label: str,
    manifest: dict[str, Any],
    template: dict[str, Any],
) -> str | None:
    manifest_files = manifest.get("files")
    report_files = template.get("files")
    if not isinstance(manifest_files, list) or not isinstance(report_files, list):
        return None
    if len(manifest_files) != len(report_files):
        return (
            f"{label}.manifest files length {len(manifest_files)} "
            f"does not match template.files length {len(report_files)}"
        )
    for index, (manifest_entry, report_entry) in enumerate(
        zip(manifest_files, report_files, strict=True)
    ):
        if not isinstance(manifest_entry, dict) or not isinstance(report_entry, dict):
            return None
        diagnostic = template_manifest_file_entry_identity_diagnostic(
            label,
            index,
            manifest_entry,
            report_entry,
        )
        if diagnostic:
            return diagnostic
    return None


def template_manifest_file_entry_identity_diagnostic(
    label: str,
    index: int,
    manifest_entry: dict[str, Any],
    report_entry: dict[str, Any],
) -> str | None:
    manifest_path = manifest_entry.get("path")
    report_path = report_entry.get("path")
    if not (
        isinstance(manifest_path, str)
        and isinstance(report_path, str)
        and manifest_path.strip()
        and report_path.strip()
    ):
        return None
    normalized_manifest_path = normalize_relative_path(manifest_path)
    normalized_report_path = normalize_relative_path(report_path)
    if not (
        is_safe_relative_path(normalized_manifest_path)
        and is_safe_relative_path(normalized_report_path)
    ):
        return None
    if normalized_manifest_path != normalized_report_path:
        return (
            f"{label}.manifest files[{index}].path {normalized_manifest_path} "
            f"does not match template.files[{index}].path {normalized_report_path}"
        )

    manifest_bundle_path = template_manifest_file_bundle_path(
        manifest_entry,
        normalized_manifest_path,
    )
    report_bundle_path = report_entry.get("bundle_path")
    if not isinstance(report_bundle_path, str) or not report_bundle_path.strip():
        return None
    normalized_report_bundle_path = normalize_relative_path(report_bundle_path)
    if not (
        manifest_bundle_path
        and is_safe_relative_path(manifest_bundle_path)
        and is_safe_relative_path(normalized_report_bundle_path)
    ):
        return None
    if manifest_bundle_path != normalized_report_bundle_path:
        return (
            f"{label}.manifest files[{index}].bundle_path {manifest_bundle_path} "
            f"does not match template.files[{index}].bundle_path "
            f"{normalized_report_bundle_path}"
        )

    for field in ("sha256", "purpose"):
        diagnostic = template_manifest_file_string_field_identity_diagnostic(
            label,
            index,
            manifest_entry,
            report_entry,
            field,
        )
        if diagnostic:
            return diagnostic
    return None


def template_manifest_file_bundle_path(
    manifest_entry: dict[str, Any],
    normalized_path: str,
) -> str | None:
    value = manifest_entry.get("bundle_path", normalized_path)
    if value is None:
        return normalized_path
    if not isinstance(value, str) or not value.strip():
        return None
    return normalize_relative_path(value)


def template_manifest_file_string_field_identity_diagnostic(
    label: str,
    index: int,
    manifest_entry: dict[str, Any],
    report_entry: dict[str, Any],
    field: str,
) -> str | None:
    manifest_value = manifest_entry.get(field, "")
    report_value = report_entry.get(field, "")
    if manifest_value is None:
        manifest_value = ""
    if report_value is None:
        report_value = ""
    if not isinstance(manifest_value, str) or not isinstance(report_value, str):
        return None
    if field == "sha256":
        if not (is_sha256_hex(manifest_value) and is_sha256_hex(report_value)):
            return None
        manifest_value = manifest_value.lower()
        report_value = report_value.lower()
    elif (
        (field in manifest_entry and not manifest_value.strip())
        or (field in report_entry and not report_value.strip())
    ):
        return None
    if manifest_value != report_value:
        return (
            f"{label}.manifest files[{index}].{field} {manifest_value} "
            f"does not match template.files[{index}].{field} {report_value}"
        )
    return None
