"""PlatformBundle report file/path evidence diagnostics."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

from .pipeline_report_platform_bundle_schema import PLATFORM_BUNDLE_MANIFEST_FIELDS


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


def platform_bundle_manifest_path_diagnostics(
    stage_report: dict[str, Any],
    report: dict[str, Any],
    manifest_path: Path,
) -> list[str]:
    bundle_path, bundle_diagnostics = platform_bundle_report_bundle_path(
        report,
        stage_report,
    )
    if bundle_path is None:
        return bundle_diagnostics
    return path_relative_to_diagnostics(
        manifest_path,
        bundle_path,
        label="PlatformBundle bundle_manifest",
        parent_label="PlatformBundle bundle",
    )


def platform_bundle_report_bundle_path(
    report: dict[str, Any],
    stage_report: dict[str, Any] | None = None,
) -> tuple[Path | None, list[str]]:
    bundle_path_value = report.get("bundle")
    if bundle_path_value is None:
        return None, [
            "PlatformBundle report bundle is required for non-fatal platform bundles"
        ]
    if not isinstance(bundle_path_value, str) or not bundle_path_value:
        return None, ["PlatformBundle report bundle must be a non-empty string"]
    diagnostics: list[str] = []
    bundle_path = resolve_user_path_or_diagnostic(
        bundle_path_value,
        diagnostics,
        "PlatformBundle report bundle",
    )
    if bundle_path is None:
        return None, diagnostics
    if not bundle_path.exists():
        return None, [f"PlatformBundle report bundle {bundle_path} does not exist"]
    if not bundle_path.is_dir():
        return None, [f"PlatformBundle report bundle {bundle_path} is not a directory"]
    if stage_report is not None:
        expected_bundle_path, expected_bundle_diagnostics = (
            platform_bundle_expected_bundle_path(stage_report, report)
        )
        if expected_bundle_diagnostics:
            return None, expected_bundle_diagnostics
        if (
            expected_bundle_path is not None
            and bundle_path != expected_bundle_path
        ):
            return None, [
                "PlatformBundle report bundle must match current output bundle"
            ]
    return bundle_path, []


def platform_bundle_expected_bundle_path(
    stage_report: dict[str, Any],
    report: dict[str, Any],
) -> tuple[Path | None, list[str]]:
    report_path = stage_report.get("path")
    profile = report.get("profile")
    if not isinstance(report_path, str) or not report_path:
        return None, []
    if not isinstance(profile, str) or not profile:
        return None, []
    diagnostics: list[str] = []
    stage_report_path = resolve_user_path_or_diagnostic(
        report_path,
        diagnostics,
        "PlatformBundle stage report path",
    )
    if stage_report_path is None:
        return None, diagnostics
    try:
        expected_bundle_path = stage_report_path.parents[2] / "bundle" / profile
    except IndexError:
        return None, []
    expected_bundle_path = resolve_user_path_or_diagnostic(
        expected_bundle_path,
        diagnostics,
        "PlatformBundle expected bundle path",
    )
    return expected_bundle_path, diagnostics


def platform_bundle_payload_path_diagnostics(
    report: dict[str, Any],
) -> list[str]:
    bundle_path, bundle_diagnostics = platform_bundle_report_bundle_path(report)
    if bundle_path is None:
        return bundle_diagnostics
    diagnostics: list[str] = []
    for field in ("host_executable", "pack", "delta_pack", "native_plugins"):
        value = report.get(field)
        if not isinstance(value, str) or not value:
            continue
        path = resolve_user_path_or_diagnostic(
            value,
            diagnostics,
            f"PlatformBundle report {field}",
        )
        if path is None:
            continue
        diagnostics.extend(
            path_relative_to_diagnostics(
                path,
                bundle_path,
                label=f"PlatformBundle report {field}",
                parent_label="PlatformBundle bundle",
            )
        )
    template_files = report.get("template_files")
    if isinstance(template_files, list):
        diagnostics.extend(
            platform_bundle_template_file_path_diagnostics(
                template_files,
                bundle_path,
            )
        )
    return diagnostics


def platform_bundle_template_file_path_diagnostics(
    template_files: list[object],
    bundle_path: Path,
) -> list[str]:
    diagnostics: list[str] = []
    for entry in template_files:
        if not isinstance(entry, dict):
            continue
        destination = entry.get("destination")
        if not isinstance(destination, str) or not destination:
            continue
        destination_path = resolve_user_path_or_diagnostic(
            destination,
            diagnostics,
            "PlatformBundle report template_files destination",
        )
        if destination_path is None:
            continue
        diagnostics.extend(
            path_relative_to_diagnostics(
                destination_path,
                bundle_path,
                label="PlatformBundle report template_files destination",
                parent_label="PlatformBundle bundle",
            )
        )
    return diagnostics


def path_relative_to_diagnostics(
    path: Path,
    parent: Path,
    *,
    label: str,
    parent_label: str,
) -> list[str]:
    try:
        resolved_path = path.resolve()
    except OSError as error:
        return [f"{label} {path} could not be resolved: {error}"]
    try:
        resolved_parent = parent.resolve()
    except OSError as error:
        return [
            f"{parent_label} {parent} for {label} could not be resolved: {error}"
        ]
    try:
        resolved_path.relative_to(resolved_parent)
    except ValueError:
        return [f"{label} {resolved_path} is outside {parent_label} {resolved_parent}"]
    return []


def path_is_relative_to(path: Path, parent: Path) -> bool:
    try:
        path.resolve().relative_to(parent.resolve())
    except (OSError, ValueError):
        return False
    return True


def load_platform_bundle_manifest(
    manifest_path: Path,
    diagnostics: list[str],
) -> dict[str, Any] | None:
    if not manifest_path.exists():
        diagnostics.append(f"PlatformBundle bundle_manifest {manifest_path} does not exist")
        return None
    if not manifest_path.is_file():
        diagnostics.append(f"PlatformBundle bundle_manifest {manifest_path} is not a file")
        return None
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except OSError as error:
        diagnostics.append(
            f"PlatformBundle bundle_manifest {manifest_path} could not be read: {error}"
        )
        return None
    except json.JSONDecodeError as error:
        diagnostics.append(
            f"PlatformBundle bundle_manifest {manifest_path} is not valid JSON: {error}"
        )
        return None
    if not isinstance(manifest, dict):
        diagnostics.append(
            f"PlatformBundle bundle_manifest {manifest_path} must be a JSON object"
        )
        return None
    return manifest


def platform_bundle_manifest_field_diagnostics(
    report: dict[str, Any],
    manifest: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    for field in PLATFORM_BUNDLE_MANIFEST_FIELDS:
        if platform_bundle_manifest_values_match(
            report.get(field),
            manifest.get(field),
            diagnostics=diagnostics,
            field=field,
            path_like=field
            in {
                "host_executable",
                "host_source",
                "pack",
                "pack_source",
                "delta_pack",
                "delta_pack_source",
                "native_plugins",
            },
        ):
            continue
        diagnostics.append(
            f"PlatformBundle bundle_manifest {field} does not match stage report"
        )
    return diagnostics


def platform_bundle_manifest_values_match(
    report_value: object,
    manifest_value: object,
    *,
    diagnostics: list[str],
    field: str,
    path_like: bool,
) -> bool:
    if report_value is None and manifest_value is None:
        return True
    if path_like and isinstance(report_value, str) and isinstance(manifest_value, str):
        report_path = resolve_user_path_or_diagnostic(
            report_value,
            diagnostics,
            f"PlatformBundle report {field}",
        )
        manifest_path = resolve_user_path_or_diagnostic(
            manifest_value,
            diagnostics,
            f"PlatformBundle bundle_manifest {field}",
        )
        if report_path is None or manifest_path is None:
            return False
        return report_path == manifest_path
    return report_value == manifest_value


def platform_bundle_output_file_diagnostics(
    report: dict[str, Any],
) -> list[str]:
    diagnostics: list[str] = []
    for field, source_field in (
        ("host_executable", "host_source"),
        ("pack", "pack_source"),
        ("delta_pack", "delta_pack_source"),
    ):
        value = report.get(field)
        if value is None:
            continue
        if not isinstance(value, str) or not value:
            continue
        path = resolve_user_path_or_diagnostic(
            value,
            diagnostics,
            f"PlatformBundle report {field}",
        )
        if path is None:
            continue
        if not path.exists():
            diagnostics.append(
                f"PlatformBundle report {field} {path} does not exist"
            )
            continue
        if not path.is_file():
            diagnostics.append(
                f"PlatformBundle report {field} {path} is not a file"
            )
            continue
        output_non_empty_diagnostic = platform_bundle_file_non_empty_diagnostic(
            f"PlatformBundle report {field}",
            path,
        )
        if output_non_empty_diagnostic:
            diagnostics.append(output_non_empty_diagnostic)
            continue
        source_value = report.get(source_field)
        if not isinstance(source_value, str) or not source_value:
            continue
        source_path = resolve_user_path_or_diagnostic(
            source_value,
            diagnostics,
            f"PlatformBundle report {source_field}",
        )
        if source_path is None:
            continue
        if not source_path.exists():
            diagnostics.append(
                f"PlatformBundle report {source_field} {source_path} does not exist"
            )
            continue
        if not source_path.is_file():
            diagnostics.append(
                f"PlatformBundle report {source_field} {source_path} is not a file"
            )
            continue
        source_non_empty_diagnostic = platform_bundle_file_non_empty_diagnostic(
            f"PlatformBundle report {source_field}",
            source_path,
        )
        if source_non_empty_diagnostic:
            diagnostics.append(source_non_empty_diagnostic)
            continue
        output_hash = platform_bundle_file_sha256(
            path,
            diagnostics,
            f"PlatformBundle report {field} {path}",
        )
        source_hash = platform_bundle_file_sha256(
            source_path,
            diagnostics,
            f"PlatformBundle report {source_field} {source_path}",
        )
        if output_hash is None or source_hash is None:
            continue
        if output_hash != source_hash:
            diagnostics.append(
                f"PlatformBundle report {field} {path} sha256 {output_hash} "
                f"does not match {source_field} {source_path} sha256 {source_hash}"
            )
    return diagnostics


def platform_bundle_file_non_empty_diagnostic(
    label: str,
    path: Path,
) -> str | None:
    try:
        if path.stat().st_size <= 0:
            return f"{label} {path} is empty"
    except OSError as error:
        return f"{label} {path} could not be inspected: {error}"
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
