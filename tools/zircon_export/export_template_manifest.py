"""Export-template manifest, path, and content-hash helpers."""

from __future__ import annotations

import hashlib
import json
import tomllib
from pathlib import Path
from typing import Any, Sequence

from .stage_handoff import stage_report_metadata_diagnostic


REPORT_FILE_NAME = "report.json"
EXPORT_TEMPLATE_BUNDLE_FIELDS = (
    "delta_pack_path",
    "host_path",
    "manifest_path",
    "pack_path",
    "root",
)
EXPORT_TEMPLATE_FILE_FIELDS = (
    "bundle_path",
    "path",
    "purpose",
    "sha256",
)


def template_bundle_config(
    manifest: dict[str, Any],
    diagnostics: list[str],
) -> dict[str, str]:
    bundle = manifest.get("bundle", {})
    if bundle is None:
        bundle = {}
    if not isinstance(bundle, dict):
        diagnostics.append("template.toml table [bundle] must be a table when present")
        bundle = {}
    else:
        diagnostics.extend(
            table_unknown_field_diagnostics(
                "template.toml bundle",
                bundle,
                EXPORT_TEMPLATE_BUNDLE_FIELDS,
            )
        )

    config = {
        "root": template_optional_path_field(bundle, "root", ".", diagnostics),
        "host_path": template_optional_path_field(bundle, "host_path", "", diagnostics),
        "pack_path": template_optional_path_field(bundle, "pack_path", "", diagnostics),
        "delta_pack_path": template_optional_path_field(
            bundle,
            "delta_pack_path",
            "",
            diagnostics,
        ),
        "manifest_path": template_optional_path_field(
            bundle,
            "manifest_path",
            "bundle.json",
            diagnostics,
        ),
    }
    return config


def template_optional_path_field(
    table: dict[str, Any],
    field_name: str,
    default: str,
    diagnostics: list[str],
) -> str:
    if field_name not in table:
        return default
    value = table.get(field_name)
    if value is None:
        return default
    if not isinstance(value, str):
        diagnostics.append(f"template.toml field bundle.{field_name} must be a string")
        return default
    if not value.strip():
        diagnostics.append(f"template.toml field bundle.{field_name} must be a non-empty string")
        return default
    if value.strip() != value:
        diagnostics.append(
            f"template.toml field bundle.{field_name} "
            "must be a non-empty trimmed string"
        )
        return default
    normalized = normalize_relative_path(value)
    if normalized in {"", "."}:
        return normalized
    if not is_safe_relative_path(normalized):
        diagnostics.append(f"template.toml field bundle.{field_name} must be a safe relative path")
        return default
    return normalized


def template_file_manifest(
    template_root: Path,
    manifest: dict[str, Any],
    diagnostics: list[str],
) -> list[dict[str, str]]:
    files = manifest.get("files", [])
    if not isinstance(files, list):
        diagnostics.append("template.toml [[files]] entries must form an array")
        return []

    checked_files: list[dict[str, str]] = []
    seen_paths: set[str] = set()
    seen_bundle_paths: set[str] = set()
    for index, entry in enumerate(files):
        if not isinstance(entry, dict):
            diagnostics.append(f"template.toml [[files]] entry {index} must be a table")
            continue
        diagnostics.extend(
            table_unknown_field_diagnostics(
                f"template.toml [[files]] entry {index}",
                entry,
                EXPORT_TEMPLATE_FILE_FIELDS,
            )
        )
        relative_path = entry.get("path")
        if not isinstance(relative_path, str) or not relative_path.strip():
            diagnostics.append(f"template.toml [[files]] entry {index} needs a non-empty path")
            continue
        if relative_path.strip() != relative_path:
            diagnostics.append(
                f"template.toml [[files]] entry {index} path "
                "must be a non-empty trimmed string"
            )
            continue
        normalized_path = normalize_relative_path(relative_path)
        if not is_safe_relative_path(normalized_path):
            diagnostics.append(
                f"template.toml [[files]] entry {index} path must be a safe relative path"
            )
            continue
        if normalized_path in seen_paths:
            diagnostics.append(f"template file {normalized_path} is declared more than once")
            continue
        seen_paths.add(normalized_path)

        file_path = resolve_template_child(template_root, normalized_path, diagnostics)
        declared_sha256 = entry.get("sha256")
        if not isinstance(declared_sha256, str) or not declared_sha256.strip():
            diagnostics.append(
                f"template file {normalized_path} must declare a SHA-256 hex digest"
            )
            continue
        if declared_sha256.strip() != declared_sha256:
            diagnostics.append(
                f"template file {normalized_path} sha256 "
                "must be a non-empty trimmed string"
            )
            continue
        if not is_sha256_hex(declared_sha256):
            diagnostics.append(
                f"template file {normalized_path} must declare a SHA-256 hex digest"
            )
            continue
        if not file_path or not file_path.exists():
            diagnostics.append(f"template file {normalized_path} does not exist")
            continue
        if not file_path.is_file():
            diagnostics.append(f"template file {normalized_path} is not a file")
            continue

        try:
            actual_sha256 = hashlib.sha256(file_path.read_bytes()).hexdigest()
        except OSError as error:
            diagnostics.append(f"template file {normalized_path} could not be read: {error}")
            continue
        if declared_sha256.lower() != actual_sha256:
            diagnostics.append(
                f"template file {normalized_path} sha256 {declared_sha256} "
                f"does not match actual {actual_sha256}"
            )
            continue
        bundle_path = template_bundle_file_path(entry, normalized_path, diagnostics)
        if bundle_path is None:
            continue
        if bundle_path in seen_bundle_paths:
            diagnostics.append(f"template bundle path {bundle_path} is declared more than once")
            continue
        seen_bundle_paths.add(bundle_path)
        purpose = entry.get("purpose", "")
        if not isinstance(purpose, str):
            diagnostics.append(f"template file {normalized_path} purpose must be a string")
            continue
        if "purpose" in entry and not purpose.strip():
            diagnostics.append(f"template file {normalized_path} purpose must be non-empty when present")
            continue
        if "purpose" in entry and purpose.strip() != purpose:
            diagnostics.append(
                f"template file {normalized_path} purpose "
                "must be a non-empty trimmed string"
            )
            continue
        checked_files.append(
            {
                "path": normalized_path,
                "bundle_path": bundle_path,
                "sha256": actual_sha256,
                "purpose": purpose,
            }
        )
    return checked_files


def template_bundle_file_path(
    entry: dict[str, Any],
    normalized_path: str,
    diagnostics: list[str],
) -> str | None:
    value = entry.get("bundle_path", normalized_path)
    if value is None:
        return normalized_path
    if not isinstance(value, str) or not value.strip():
        diagnostics.append(f"template file {normalized_path} has an invalid bundle_path")
        return None
    if value.strip() != value:
        diagnostics.append(
            f"template file {normalized_path} bundle_path "
            "must be a non-empty trimmed string"
        )
        return None
    normalized = normalize_relative_path(value)
    if not is_safe_relative_path(normalized):
        diagnostics.append(f"template file {normalized_path} bundle_path must be a safe relative path")
        return None
    return normalized


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


def resolve_template_child(
    template_root: Path,
    relative_path: str,
    diagnostics: list[str],
) -> Path | None:
    child_path = Path(relative_path)
    if child_path.is_absolute():
        diagnostics.append(f"template path {relative_path} must be relative")
        return None
    try:
        resolved = (template_root / child_path).resolve()
    except OSError as error:
        diagnostics.append(f"template path {relative_path} could not be resolved: {error}")
        return None
    try:
        resolved.relative_to(template_root)
    except ValueError:
        diagnostics.append(f"template path {relative_path} escapes the template directory")
        return None
    return resolved


def resolve_bundle_child(
    bundle_root: Path,
    relative_path: str,
    diagnostics: list[str],
) -> Path | None:
    normalized = normalize_relative_path(relative_path)
    if not is_safe_relative_path(normalized):
        diagnostics.append(f"bundle path {relative_path} must be a safe relative path")
        return None
    try:
        resolved_root = bundle_root.resolve()
    except OSError as error:
        diagnostics.append(
            f"bundle root {bundle_root} could not be resolved for bundle path {relative_path}: {error}"
        )
        return None
    try:
        resolved = (resolved_root / Path(normalized)).resolve()
    except OSError as error:
        diagnostics.append(f"bundle path {relative_path} could not be resolved: {error}")
        return None
    try:
        resolved.relative_to(resolved_root)
    except ValueError:
        diagnostics.append(f"bundle path {relative_path} escapes the bundle directory")
        return None
    return resolved


def normalize_relative_path(value: str) -> str:
    return value.strip().replace("\\", "/")


def is_safe_relative_path(value: str) -> bool:
    path = Path(value)
    if path.is_absolute():
        return False
    parts = value.split("/")
    return bool(value) and all(part not in {"", ".", ".."} for part in parts)


def compute_template_content_hash(files: Sequence[dict[str, str]]) -> str:
    hasher = hashlib.sha256()
    for entry in sorted(files, key=lambda value: value["path"]):
        hasher.update(entry["path"].encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(entry.get("bundle_path", "").encode("utf-8"))
        hasher.update(b"\0")
        hasher.update(entry["sha256"].lower().encode("ascii"))
        hasher.update(b"\n")
    return hasher.hexdigest()


def is_sha256_hex(value: str) -> bool:
    if len(value) != 64:
        return False
    return all(character in "0123456789abcdefABCDEF" for character in value)


def workspace_engine_version(
    repo_root: Path,
    diagnostics: list[str] | None = None,
) -> str | None:
    manifest_path = repo_root / "Cargo.toml"
    if not manifest_path.exists():
        return None
    if not manifest_path.is_file():
        if diagnostics is not None:
            diagnostics.append(f"workspace manifest {manifest_path} is not a file")
        return None
    try:
        with manifest_path.open("rb") as manifest_file:
            manifest = tomllib.load(manifest_file)
    except OSError as error:
        if diagnostics is not None:
            diagnostics.append(f"workspace manifest {manifest_path} could not be read: {error}")
        return None
    except tomllib.TOMLDecodeError:
        return None
    version = (
        manifest.get("workspace", {})
        .get("package", {})
        .get("version")
    )
    return version if isinstance(version, str) and version else None


def validated_target_platform(out_root: Path, profile: str | None = None) -> str | None:
    report_path = out_root / "stages" / "validate" / REPORT_FILE_NAME
    if not report_path.exists():
        return None
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return None
    if profile is not None:
        if stage_report_metadata_diagnostic(report, "validate", profile):
            return None
    profile_summary = report.get("profile_summary")
    if not isinstance(profile_summary, dict):
        return None
    target_platform = profile_summary.get("target_platform")
    return target_platform if isinstance(target_platform, str) and target_platform else None


def normalize_target_platform(value: str) -> str:
    aliases = {
        "windows": "windows-x86_64",
        "linux": "linux-x86_64",
        "macos": "macos-aarch64",
    }
    return aliases.get(value, value)
