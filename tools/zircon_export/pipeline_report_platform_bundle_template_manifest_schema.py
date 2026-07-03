"""Manifest path and schema diagnostics for PlatformBundle template reports."""

from __future__ import annotations

from typing import Any
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python <3.11 fallback.
    import tomli as tomllib  # type: ignore[no-redef]

from .export_template import (
    EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS,
    EXPORT_TEMPLATE_ALLOWED_HOST_ARTIFACTS,
    EXPORT_TEMPLATE_ALLOWED_HOST_KINDS,
    EXPORT_TEMPLATE_ALLOWED_PLUGIN_STRATEGIES,
    EXPORT_TEMPLATE_ALLOWED_RESOURCE_STRATEGIES,
    EXPORT_TEMPLATE_FORMAT_VERSION,
    EXPORT_TEMPLATE_MANIFEST_FIELDS,
    EXPORT_TEMPLATE_PATHS_FIELDS,
)
from .export_template_manifest import (
    EXPORT_TEMPLATE_BUNDLE_FIELDS,
    EXPORT_TEMPLATE_FILE_FIELDS,
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
)
from .pipeline_report_platform_bundle_template_manifest_identity import (
    TEMPLATE_MANIFEST_BUNDLE_DEFAULTS,
    template_manifest_identity_diagnostic,
)
from .pipeline_report_platform_bundle_template_manifest_files_schema import (
    template_manifest_files_presence_diagnostic,
    template_manifest_files_schema_diagnostic,
    template_manifest_files_unique_diagnostic,
)
from .pipeline_report_schema_string_array import (
    string_array_no_blank_entries_schema_diagnostics,
    string_array_unique_entries_schema_diagnostics,
)
from .pipeline_report_platform_bundle_template_schema_helpers import (
    table_unknown_field_diagnostics,
)

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
    identity_diagnostic = template_manifest_identity_diagnostic(
        label,
        manifest,
        template,
        expected_manifest.parent,
    )
    if identity_diagnostic:
        return [identity_diagnostic]
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
