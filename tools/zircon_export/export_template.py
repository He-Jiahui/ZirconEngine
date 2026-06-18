"""Export-template manifest validation and bundle path helpers."""

from __future__ import annotations

import hashlib
import json
import tomllib
from pathlib import Path
from typing import Any, Sequence

from .stage_handoff import stage_report_metadata_diagnostic


REPORT_FILE_NAME = "report.json"
EXPORT_TEMPLATE_FORMAT_VERSION = 1
EXPORT_TEMPLATE_MANIFEST_NAME = "template.toml"
EXPORT_TEMPLATE_ALLOWED_HOST_KINDS = {"desktop", "mobile_app", "browser", "headless"}
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


def resolve_export_template_from_root(
    *,
    template_root: Path,
    profile: str,
    expected_engine_version: str | None,
    expected_target_platform: str | None,
) -> dict[str, Any]:
    diagnostics: list[str] = []
    root = resolve_export_template_path(
        label="export template root",
        path=template_root,
        diagnostics=diagnostics,
    )
    report: dict[str, Any] = {
        "template_root": str(root or template_root),
        "profile": profile,
        "expected_engine_version": expected_engine_version,
        "expected_target_platform": expected_target_platform,
        "fatal": False,
        "diagnostics": diagnostics,
        "candidates": [],
        "skipped_candidates": [],
        "template_dir": None,
    }
    if root is None:
        report["fatal"] = True
        return report

    if not root.exists():
        diagnostics.append(f"export template root {root} does not exist")
        report["fatal"] = True
        return report
    if not root.is_dir():
        diagnostics.append(f"export template root {root} is not a directory")
        report["fatal"] = True
        return report

    for manifest_path in sorted(root.glob(f"*/{EXPORT_TEMPLATE_MANIFEST_NAME}")):
        candidate_diagnostics: list[str] = []
        manifest = read_template_manifest_for_resolution(manifest_path, candidate_diagnostics)
        if manifest is None:
            if candidate_diagnostics:
                report["skipped_candidates"].append(
                    {
                        "template_dir": str(
                            resolve_export_template_path(
                                label="export template directory",
                                path=manifest_path.parent,
                                diagnostics=candidate_diagnostics,
                            )
                            or manifest_path.parent
                        ),
                        "diagnostics": candidate_diagnostics,
                    }
                )
            continue
        if not template_manifest_matches_resolution(
            manifest,
            profile=profile,
            expected_engine_version=expected_engine_version,
            expected_target_platform=expected_target_platform,
        ):
            continue
        candidate_validation = validate_export_template(
            template_dir=manifest_path.parent,
            expected_engine_version=expected_engine_version,
            profile=profile,
            expected_target_platform=expected_target_platform,
        )
        if candidate_validation["fatal"]:
            report["skipped_candidates"].append(
                {
                    "template_dir": str(candidate_validation["template_dir"]),
                    "diagnostics": candidate_validation["diagnostics"],
                }
            )
            continue
        candidate = template_resolution_candidate(
            Path(candidate_validation["template_dir"]),
            manifest,
        )
        report["candidates"].append(candidate)

    candidates = report["candidates"]
    if not candidates:
        target_note = expected_target_platform or "<any>"
        engine_note = expected_engine_version or "<unresolved>"
        diagnostics.append(
            "no export template under "
            f"{root} matched profile={profile} target_platform={target_note} "
            f"engine_version={engine_note}"
        )
    elif len(candidates) > 1:
        diagnostics.append(
            "multiple export templates matched profile="
            f"{profile}: "
            + ", ".join(str(candidate["template_dir"]) for candidate in candidates)
        )
    else:
        report["template_dir"] = candidates[0]["template_dir"]

    report["fatal"] = bool(diagnostics) and report["template_dir"] is None
    return report


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


def read_template_manifest_for_resolution(
    manifest_path: Path,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if not manifest_path.is_file():
        diagnostics.append(f"export template manifest {manifest_path} is not a file")
        return None
    try:
        with manifest_path.open("rb") as manifest_file:
            manifest = tomllib.load(manifest_file)
    except OSError as error:
        diagnostics.append(f"export template manifest {manifest_path} could not be read: {error}")
        return None
    except tomllib.TOMLDecodeError as error:
        diagnostics.append(f"export template manifest {manifest_path} is not valid TOML: {error}")
        return None
    if not isinstance(manifest, dict):
        diagnostics.append(f"export template manifest {manifest_path} must be a TOML table")
        return None
    return manifest


def template_manifest_matches_resolution(
    manifest: dict[str, Any],
    *,
    profile: str,
    expected_engine_version: str | None,
    expected_target_platform: str | None,
) -> bool:
    if manifest.get("format_version") != EXPORT_TEMPLATE_FORMAT_VERSION:
        return False
    engine_version = manifest.get("engine_version")
    if expected_engine_version and engine_version != expected_engine_version:
        return False
    target_platform = manifest.get("target_platform")
    if expected_target_platform:
        if not isinstance(target_platform, str):
            return False
        if normalize_target_platform(target_platform) != normalize_target_platform(
            expected_target_platform
        ):
            return False
    compatible_profiles = manifest.get("compatible_profiles", [])
    if not compatible_profiles:
        return True
    if not isinstance(compatible_profiles, list):
        return False
    return profile in compatible_profiles


def template_resolution_candidate(
    template_dir: Path,
    manifest: dict[str, Any],
) -> dict[str, Any]:
    return {
        "template_dir": str(template_dir),
        "template_id": manifest.get("template_id"),
        "engine_version": manifest.get("engine_version"),
        "target_platform": manifest.get("target_platform"),
        "compatible_profiles": manifest.get("compatible_profiles", []),
        "bundle_format": manifest.get("bundle_format"),
    }


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
    resource_strategy = template_string_field(manifest, "resource_strategy", diagnostics)
    plugin_strategy = template_string_field(manifest, "plugin_strategy", diagnostics)
    bundle_format = template_string_field(manifest, "bundle_format", diagnostics)
    content_hash = template_string_field(manifest, "content_hash", diagnostics)
    report.update(
        {
            "template_id": template_id,
            "target_platform": target_platform,
            "host_kind": host_kind,
            "resource_strategy": resource_strategy,
            "plugin_strategy": plugin_strategy,
            "bundle_format": bundle_format,
            "content_hash": content_hash,
        }
    )

    validate_allowed_field("host_kind", host_kind, EXPORT_TEMPLATE_ALLOWED_HOST_KINDS, diagnostics)
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
    if not isinstance(compatible_profiles, list) or any(
        not isinstance(value, str) for value in compatible_profiles
    ):
        diagnostics.append("template.toml field compatible_profiles must be a string array")
        compatible_profiles = []
    report["compatible_profiles"] = compatible_profiles
    if compatible_profiles and profile not in compatible_profiles:
        diagnostics.append(
            f"template compatible_profiles does not include requested profile {profile}"
        )

    paths = manifest.get("paths")
    host_relative_path = None
    if not isinstance(paths, dict):
        diagnostics.append("template.toml table [paths] is required")
    else:
        host_relative_path = paths.get("host_executable")
        if not isinstance(host_relative_path, str) or not host_relative_path.strip():
            diagnostics.append("template.toml field paths.host_executable must be a non-empty string")
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
        return value.strip()
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
    value = table.get(field_name, default)
    if value is None:
        return default
    if not isinstance(value, str):
        diagnostics.append(f"template.toml field bundle.{field_name} must be a string")
        return default
    normalized = normalize_relative_path(value) if value else default
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
    for index, entry in enumerate(files):
        if not isinstance(entry, dict):
            diagnostics.append(f"template.toml [[files]] entry {index} must be a table")
            continue
        relative_path = entry.get("path")
        if not isinstance(relative_path, str) or not relative_path.strip():
            diagnostics.append(f"template.toml [[files]] entry {index} needs a non-empty path")
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
        if not isinstance(declared_sha256, str) or not is_sha256_hex(declared_sha256):
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
        checked_files.append(
            {
                "path": normalized_path,
                "bundle_path": template_bundle_file_path(entry, normalized_path, diagnostics),
                "sha256": actual_sha256,
                "purpose": str(entry.get("purpose", "")),
            }
        )
    return checked_files


def template_bundle_file_path(
    entry: dict[str, Any],
    normalized_path: str,
    diagnostics: list[str],
) -> str:
    value = entry.get("bundle_path", normalized_path)
    if value is None:
        return normalized_path
    if not isinstance(value, str) or not value.strip():
        diagnostics.append(f"template file {normalized_path} has an invalid bundle_path")
        return normalized_path
    normalized = normalize_relative_path(value)
    if not is_safe_relative_path(normalized):
        diagnostics.append(f"template file {normalized_path} bundle_path must be a safe relative path")
        return normalized_path
    return normalized


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
