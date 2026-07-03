"""Manifest [[files]] schema diagnostics for PlatformBundle template reports."""

from __future__ import annotations

from typing import Any

from .export_template_manifest import (
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
)
from .pipeline_report_platform_bundle_template_manifest_identity import (
    template_manifest_file_bundle_path,
)


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
