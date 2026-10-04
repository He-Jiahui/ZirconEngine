"""Manifest/report identity diagnostics for PlatformBundle template reports."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .export_template_manifest import (
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
    normalize_target_platform,
)

TEMPLATE_MANIFEST_BUNDLE_DEFAULTS = {
    "delta_pack_path": "",
    "host_path": "",
    "manifest_path": "bundle.json",
    "pack_path": "",
    "root": ".",
}


def template_manifest_identity_diagnostic(
    label: str,
    manifest: dict[str, Any],
    template: dict[str, Any],
    template_root: Path,
) -> str | None:
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
        diagnostic = template_manifest_string_identity_diagnostic(
            label,
            manifest,
            template,
            field,
        )
        if diagnostic:
            return diagnostic

    diagnostic = template_manifest_target_platform_identity_diagnostic(
        label,
        manifest,
        template,
    )
    if diagnostic:
        return diagnostic

    diagnostic = template_manifest_compatible_profiles_identity_diagnostic(
        label,
        manifest,
        template,
    )
    if diagnostic:
        return diagnostic

    diagnostic = template_manifest_host_executable_identity_diagnostic(
        label,
        manifest,
        template,
        template_root,
    )
    if diagnostic:
        return diagnostic

    diagnostic = template_manifest_bundle_identity_diagnostic(
        label,
        manifest,
        template,
    )
    if diagnostic:
        return diagnostic

    return template_manifest_files_identity_diagnostic(
        label,
        manifest,
        template,
    )


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
