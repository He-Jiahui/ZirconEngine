"""Semantic diagnostics for embedded PlatformBundle template reports."""

from __future__ import annotations

import hashlib
from pathlib import Path
from typing import Any

from .export_template_manifest import (
    compute_template_content_hash,
    is_safe_relative_path,
    is_sha256_hex,
    normalize_relative_path,
    normalize_target_platform,
)


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
        or profile.strip() != profile
        or not isinstance(compatible_profiles, list)
        or not compatible_profiles
        or any(
            not isinstance(value, str)
            or not value.strip()
            or value.strip() != value
            for value in compatible_profiles
        )
    ):
        return []
    if profile not in compatible_profiles:
        return [
            f"{label}.compatible_profiles does not include profile {profile}"
        ]
    return []


def template_report_file_entry_is_schema_clean(entry: dict[str, Any]) -> bool:
    for field in ("bundle_path", "path"):
        value = entry.get(field)
        if not (
            isinstance(value, str)
            and value.strip()
            and value.strip() == value
            and is_safe_relative_path(normalize_relative_path(value))
        ):
            return False
    sha256 = entry.get("sha256")
    return (
        isinstance(sha256, str)
        and sha256.strip()
        and sha256.strip() == sha256
        and is_sha256_hex(sha256)
    )


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
        if not template_report_file_entry_is_schema_clean(entry):
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
        if not template_report_file_entry_is_schema_clean(entry):
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
    for field in ("computed_content_hash", "content_hash"):
        value = template.get(field)
        if isinstance(value, str) and is_sha256_hex(value) and value.lower() != expected_hash:
            diagnostics.append(
                f"{label}.{field} does not match computed content hash {expected_hash}"
            )
    return diagnostics


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
        and template_report_file_entry_is_schema_clean(entry)
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
