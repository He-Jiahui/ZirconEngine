"""Export-template manifest validation and bundle path helpers."""

from __future__ import annotations

from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - Python <3.11 fallback.
    import tomli as tomllib  # type: ignore[no-redef]

from .export_template_manifest import (
    compute_template_content_hash,
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
    normalize_target_platform,
    resolve_template_child,
    table_unknown_field_diagnostics,
    template_bundle_config,
    template_file_manifest,
)
from .pipeline_report_schema_string_array import (
    string_array_no_blank_entries_schema_diagnostics,
    string_array_unique_entries_schema_diagnostics,
)


EXPORT_TEMPLATE_FORMAT_VERSION = 1
EXPORT_TEMPLATE_MANIFEST_NAME = "template.toml"
EXPORT_TEMPLATE_ALLOWED_HOST_KINDS = {"desktop", "mobile_app", "browser", "headless"}
EXPORT_TEMPLATE_ALLOWED_HOST_ARTIFACTS = {"placeholder", "precompiled"}
EXPORT_TEMPLATE_ALLOWED_RESOURCE_STRATEGIES = {
    "filesystem_bundle",
    "mobile_asset_bundle",
    "browser_fetch",
}
EXPORT_TEMPLATE_ALLOWED_PLUGIN_STRATEGIES = {
    "native_dynamic_allowed",
    "static_source_or_vm_only",
}
EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS = {
    "directory",
    "app_bundle",
    "zip",
    "web_static",
}
EXPORT_TEMPLATE_MANIFEST_FIELDS = (
    "bundle",
    "bundle_format",
    "compatible_profiles",
    "content_hash",
    "engine_version",
    "files",
    "format_version",
    "host_artifact",
    "host_kind",
    "paths",
    "plugin_strategy",
    "resource_strategy",
    "target_platform",
    "template_id",
)
EXPORT_TEMPLATE_PATHS_FIELDS = ("host_executable",)


def resolve_export_template_path(
    *,
    label: str,
    path: Path,
    diagnostics: list[str],
) -> Path | None:
    try:
        return path.resolve()
    except OSError as error:
        diagnostics.append(f"{label} {path} could not be resolved: {error}")
        return None


def validate_export_template(
    *,
    template_dir: Path,
    expected_engine_version: str | None,
    profile: str,
    expected_target_platform: str | None,
) -> dict[str, Any]:
    diagnostics: list[str] = []
    template_root = resolve_export_template_path(
        label="export template directory",
        path=template_dir,
        diagnostics=diagnostics,
    )
    manifest_path = (template_root or template_dir) / EXPORT_TEMPLATE_MANIFEST_NAME
    report: dict[str, Any] = {
        "template_dir": str(template_root or template_dir),
        "manifest": str(manifest_path),
        "expected_format_version": EXPORT_TEMPLATE_FORMAT_VERSION,
        "expected_engine_version": expected_engine_version,
        "expected_target_platform": expected_target_platform,
        "profile": profile,
        "fatal": False,
        "diagnostics": diagnostics,
        "host_executable": None,
        "files": [],
    }
    if template_root is None:
        report["fatal"] = True
        return report

    if not template_root.exists():
        diagnostics.append(f"export template directory {template_root} does not exist")
        report["fatal"] = True
        return report
    if not manifest_path.exists():
        diagnostics.append(f"export template manifest {manifest_path} does not exist")
        report["fatal"] = True
        return report
    if not manifest_path.is_file():
        diagnostics.append(f"export template manifest {manifest_path} is not a file")
        report["fatal"] = True
        return report

    try:
        with manifest_path.open("rb") as manifest_file:
            manifest = tomllib.load(manifest_file)
    except OSError as error:
        diagnostics.append(f"export template manifest {manifest_path} could not be read: {error}")
        report["fatal"] = True
        return report
    except tomllib.TOMLDecodeError as error:
        diagnostics.append(f"export template manifest is not valid TOML: {error}")
        report["fatal"] = True
        return report

    diagnostics.extend(
        table_unknown_field_diagnostics(
            "template.toml",
            manifest,
            EXPORT_TEMPLATE_MANIFEST_FIELDS,
        )
    )

    format_version = manifest.get("format_version")
    report["format_version"] = format_version
    if type(format_version) is not int:
        diagnostics.append("template.toml field format_version must be an integer")
    elif format_version != EXPORT_TEMPLATE_FORMAT_VERSION:
        diagnostics.append(
            "template format_version "
            f"{format_version} is not supported; expected {EXPORT_TEMPLATE_FORMAT_VERSION}"
        )

    engine_version = template_string_field(manifest, "engine_version", diagnostics)
    report["engine_version"] = engine_version
    if not expected_engine_version:
        diagnostics.append("engine version could not be resolved for template validation")
    elif engine_version and engine_version != expected_engine_version:
        diagnostics.append(
            "template engine_version "
            f"{engine_version} does not match engine version {expected_engine_version}"
        )

    template_id = template_string_field(manifest, "template_id", diagnostics)
    target_platform = template_string_field(manifest, "target_platform", diagnostics)
    host_kind = template_string_field(manifest, "host_kind", diagnostics)
    host_artifact = template_string_field(manifest, "host_artifact", diagnostics)
    resource_strategy = template_string_field(manifest, "resource_strategy", diagnostics)
    plugin_strategy = template_string_field(manifest, "plugin_strategy", diagnostics)
    bundle_format = template_string_field(manifest, "bundle_format", diagnostics)
    content_hash = template_string_field(manifest, "content_hash", diagnostics)
    report.update(
        {
            "template_id": template_id,
            "target_platform": target_platform,
            "host_kind": host_kind,
            "host_artifact": host_artifact,
            "resource_strategy": resource_strategy,
            "plugin_strategy": plugin_strategy,
            "bundle_format": bundle_format,
            "content_hash": content_hash,
        }
    )

    validate_allowed_field("host_kind", host_kind, EXPORT_TEMPLATE_ALLOWED_HOST_KINDS, diagnostics)
    validate_allowed_field(
        "host_artifact",
        host_artifact,
        EXPORT_TEMPLATE_ALLOWED_HOST_ARTIFACTS,
        diagnostics,
    )
    validate_allowed_field(
        "resource_strategy",
        resource_strategy,
        EXPORT_TEMPLATE_ALLOWED_RESOURCE_STRATEGIES,
        diagnostics,
    )
    validate_allowed_field(
        "plugin_strategy",
        plugin_strategy,
        EXPORT_TEMPLATE_ALLOWED_PLUGIN_STRATEGIES,
        diagnostics,
    )
    validate_allowed_field(
        "bundle_format",
        bundle_format,
        EXPORT_TEMPLATE_ALLOWED_BUNDLE_FORMATS,
        diagnostics,
    )

    if (
        expected_target_platform
        and target_platform
        and normalize_target_platform(target_platform)
        != normalize_target_platform(expected_target_platform)
    ):
        diagnostics.append(
            "template target_platform "
            f"{target_platform} does not match requested target platform {expected_target_platform}"
        )

    compatible_profiles = manifest.get("compatible_profiles", [])
    if compatible_profiles is None:
        compatible_profiles = []
    compatible_profiles_schema_clean = False
    if not isinstance(compatible_profiles, list):
        diagnostics.append("template.toml field compatible_profiles must be a string array")
        compatible_profiles = []
    else:
        type_diagnostics: list[str] = []
        for index, compatible_profile in enumerate(compatible_profiles):
            if not isinstance(compatible_profile, str):
                type_diagnostics.append(
                    f"template.toml field compatible_profiles[{index}] must be a string"
                )
        diagnostics.extend(type_diagnostics)
        blank_diagnostics = string_array_no_blank_entries_schema_diagnostics(
            "template.toml field compatible_profiles",
            compatible_profiles,
        )
        diagnostics.extend(blank_diagnostics)
        trimmed_diagnostics: list[str] = []
        for index, compatible_profile in enumerate(compatible_profiles):
            if not isinstance(compatible_profile, str):
                continue
            if (
                compatible_profile.strip()
                and compatible_profile.strip() != compatible_profile
            ):
                trimmed_diagnostics.append(
                    f"template.toml field compatible_profiles[{index}] "
                    "must be a non-empty trimmed string"
                )
        diagnostics.extend(trimmed_diagnostics)
        unique_diagnostics = string_array_unique_entries_schema_diagnostics(
            "template.toml field compatible_profiles",
            compatible_profiles,
        )
        diagnostics.extend(unique_diagnostics)
        compatible_profiles_schema_clean = not (
            type_diagnostics
            or blank_diagnostics
            or trimmed_diagnostics
            or unique_diagnostics
        )
    report["compatible_profiles"] = compatible_profiles
    if (
        compatible_profiles_schema_clean
        and compatible_profiles
        and profile not in compatible_profiles
    ):
        diagnostics.append(
            f"template compatible_profiles does not include requested profile {profile}"
        )

    paths = manifest.get("paths")
    host_relative_path = None
    if not isinstance(paths, dict):
        diagnostics.append("template.toml table [paths] is required")
    else:
        diagnostics.extend(
            table_unknown_field_diagnostics(
                "template.toml paths",
                paths,
                EXPORT_TEMPLATE_PATHS_FIELDS,
            )
        )
        host_relative_path = paths.get("host_executable")
        if not isinstance(host_relative_path, str) or not host_relative_path.strip():
            diagnostics.append("template.toml field paths.host_executable must be a non-empty string")
            host_relative_path = None
        elif host_relative_path.strip() != host_relative_path:
            diagnostics.append(
                "template.toml field paths.host_executable "
                "must be a non-empty trimmed string"
            )
            host_relative_path = None
        else:
            host_relative_path = normalize_relative_path(host_relative_path)
            if not is_safe_relative_path(host_relative_path):
                diagnostics.append(
                    "template.toml field paths.host_executable must be a safe relative path"
                )
                host_relative_path = None

    bundle_config = template_bundle_config(manifest, diagnostics)
    report["bundle"] = bundle_config

    checked_files = template_file_manifest(template_root, manifest, diagnostics)
    report["files"] = checked_files
    if checked_files:
        computed_content_hash = compute_template_content_hash(checked_files)
        report["computed_content_hash"] = computed_content_hash
        if content_hash and not is_sha256_hex(content_hash):
            diagnostics.append("template.toml field content_hash must be a SHA-256 hex digest")
        elif content_hash and content_hash.lower() != computed_content_hash:
            diagnostics.append(
                "template content_hash "
                f"{content_hash} does not match computed hash {computed_content_hash}"
            )
    else:
        diagnostics.append("template.toml must declare at least one [[files]] entry")

    if host_relative_path:
        host_path = resolve_template_child(template_root, host_relative_path, diagnostics)
        if host_path:
            report["host_executable"] = str(host_path)
            if not host_path.exists():
                diagnostics.append(f"template host executable {host_path} does not exist")
            declared_paths = {entry["path"] for entry in checked_files}
            if host_relative_path.replace("\\", "/") not in declared_paths:
                diagnostics.append(
                    "template paths.host_executable must also be listed in [[files]]"
                )

    report["fatal"] = bool(diagnostics)
    return report


def template_string_field(
    manifest: dict[str, Any],
    field_name: str,
    diagnostics: list[str],
) -> str | None:
    value = manifest.get(field_name)
    if isinstance(value, str) and value.strip():
        trimmed = value.strip()
        if trimmed != value:
            diagnostics.append(
                f"template.toml field {field_name} must be a non-empty trimmed string"
            )
            return None
        return value
    diagnostics.append(f"template.toml field {field_name} must be a non-empty string")
    return None


def validate_allowed_field(
    field_name: str,
    value: str | None,
    allowed_values: set[str],
    diagnostics: list[str],
) -> None:
    if value and value not in allowed_values:
        diagnostics.append(
            f"template.toml field {field_name}={value!r} is not one of "
            f"{', '.join(sorted(allowed_values))}"
        )
