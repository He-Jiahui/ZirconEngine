"""PlatformBundle final report diagnostics for the Zircon export pipeline."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import Any

from .pipeline_report_native_dynamic_payload import (
    native_dynamic_stage_report_path,
    platform_bundle_native_plugins_payload_diagnostics,
)
from .pipeline_report_platform_bundle_schema import (
    PLATFORM_BUNDLE_MANIFEST_FIELDS,
    platform_bundle_manifest_schema_diagnostics,
    platform_bundle_report_schema_diagnostics,
)
from .pipeline_report_platform_bundle_template import (
    platform_bundle_template_file_expected_hash,
    platform_bundle_template_file_hashes,
    platform_bundle_template_files_diagnostics,
    platform_bundle_template_resolution_diagnostics,
)


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


def delta_verification_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "pack":
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        delta_pack = report.get("delta_pack")
        if delta_pack is None:
            continue
        if not isinstance(delta_pack, str) or not delta_pack.strip():
            diagnostics.append("pack report delta_pack must be a non-empty string")
            continue
        if report.get("delta_apply_verified") is not True:
            diagnostics.append(
                "pack report delta_pack is present but delta_apply_verified is not true"
            )
    return diagnostics


def platform_bundle_host_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    host_path = None
    for stage_report in stage_reports:
        host_path = compile_host_report_host_path(stage_report, diagnostics)
        if host_path is not None:
            break
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "platform_bundle":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        bundled_host = report.get("host_executable")
        if bundled_host is None:
            continue
        if not isinstance(bundled_host, str) or not bundled_host:
            diagnostics.append(
                "PlatformBundle report host_executable must be a non-empty string"
            )
            continue
        host_source = report.get("host_source")
        if host_source is None:
            diagnostics.append(
                "PlatformBundle report host_source is required when host_executable is present"
            )
            continue
        if not isinstance(host_source, str) or not host_source:
            diagnostics.append(
                "PlatformBundle report host_source must be a non-empty string"
            )
            continue
        host_origin = report.get("host_source_origin")
        if host_origin is None:
            diagnostics.append(
                "PlatformBundle report host_source_origin is required when host_executable is present"
            )
            continue
        if not isinstance(host_origin, str) or not host_origin:
            diagnostics.append(
                "PlatformBundle report host_source_origin must be a non-empty string"
            )
            continue
        if host_origin not in {"compile_host_report", "argument", "template"}:
            diagnostics.append(
                "PlatformBundle report host_source_origin must be compile_host_report, argument, or template"
            )
            continue
        if host_origin == "compile_host_report" and host_path is None:
            diagnostics.append(
                "PlatformBundle report host_executable is present but CompileHost report does not contain host_executable evidence"
            )
            continue
        if host_origin != "compile_host_report":
            continue
        resolved_host_source = resolve_user_path_or_diagnostic(
            host_source,
            diagnostics,
            "PlatformBundle report host_source",
        )
        if resolved_host_source is None:
            continue
        if resolved_host_source != host_path:
            diagnostics.append(
                "PlatformBundle report host_source does not match CompileHost report host_executable"
            )
    return diagnostics


def compile_host_report_host_path(
    stage_report: dict[str, Any],
    diagnostics: list[str],
) -> Path | None:
    if stage_report.get("stage_key") != "compile_host":
        return None
    if stage_report.get("fatal") is True:
        return None
    report = stage_report.get("report")
    if not isinstance(report, dict):
        return None
    host = report.get("host_executable")
    if not isinstance(host, str) or not host:
        return None
    return resolve_user_path_or_diagnostic(
        host,
        diagnostics,
        "CompileHost report host_executable",
    )


def platform_bundle_pack_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    pack_path = None
    for stage_report in stage_reports:
        pack_path = pack_report_pack_path(stage_report, diagnostics)
        if pack_path is not None:
            break
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "platform_bundle":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        bundled_pack = report.get("pack")
        if bundled_pack is None:
            continue
        if not isinstance(bundled_pack, str) or not bundled_pack:
            diagnostics.append("PlatformBundle report pack must be a non-empty string")
            continue
        pack_source = report.get("pack_source")
        if pack_source is None:
            diagnostics.append(
                "PlatformBundle report pack_source is required when pack is present"
            )
            continue
        if not isinstance(pack_source, str) or not pack_source:
            diagnostics.append(
                "PlatformBundle report pack_source must be a non-empty string"
            )
            continue
        pack_origin = report.get("pack_source_origin")
        if pack_origin is None:
            diagnostics.append(
                "PlatformBundle report pack_source_origin is required when pack is present"
            )
            continue
        if not isinstance(pack_origin, str) or not pack_origin:
            diagnostics.append(
                "PlatformBundle report pack_source_origin must be a non-empty string"
            )
            continue
        if pack_origin not in {"pack_report", "argument"}:
            diagnostics.append(
                "PlatformBundle report pack_source_origin must be pack_report or argument"
            )
            continue
        if pack_origin == "pack_report" and pack_path is None:
            diagnostics.append(
                "PlatformBundle report pack is present but Pack report does not contain pack evidence"
            )
            continue
        if pack_origin != "pack_report":
            continue
        resolved_pack_source = resolve_user_path_or_diagnostic(
            pack_source,
            diagnostics,
            "PlatformBundle report pack_source",
        )
        if resolved_pack_source is None:
            continue
        if resolved_pack_source != pack_path:
            diagnostics.append(
                "PlatformBundle report pack_source does not match Pack report pack"
            )
    return diagnostics


def pack_report_pack_path(
    stage_report: dict[str, Any],
    diagnostics: list[str],
) -> Path | None:
    if stage_report.get("stage_key") != "pack":
        return None
    if stage_report.get("fatal") is True:
        return None
    report = stage_report.get("report")
    if not isinstance(report, dict):
        return None
    pack = report.get("pack")
    if not isinstance(pack, str) or not pack:
        return None
    return resolve_user_path_or_diagnostic(
        pack,
        diagnostics,
        "Pack report pack",
    )


def platform_bundle_delta_diagnostics(
    stage_reports: list[dict[str, Any]],
) -> list[str]:
    diagnostics: list[str] = []
    verified_pack_delta = None
    for stage_report in stage_reports:
        if not pack_report_has_verified_delta(stage_report):
            continue
        verified_pack_delta = pack_report_delta_path(stage_report, diagnostics)
        if verified_pack_delta is not None:
            break
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "platform_bundle":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        delta_pack = report.get("delta_pack")
        if delta_pack is None:
            continue
        if not isinstance(delta_pack, str) or not delta_pack.strip():
            diagnostics.append(
                "PlatformBundle report delta_pack must be a non-empty string"
            )
            continue
        if verified_pack_delta is None:
            diagnostics.append(
                "PlatformBundle report delta_pack is present but Pack report does not contain verified delta_pack evidence"
            )
            continue
        delta_pack_source = report.get("delta_pack_source")
        if delta_pack_source is None:
            diagnostics.append(
                "PlatformBundle report delta_pack_source is required when delta_pack is present"
            )
            continue
        if not isinstance(delta_pack_source, str) or not delta_pack_source:
            diagnostics.append(
                "PlatformBundle report delta_pack_source must be a non-empty string"
            )
            continue
        delta_origin = report.get("delta_pack_source_origin")
        if delta_origin is None:
            diagnostics.append(
                "PlatformBundle report delta_pack_source_origin is required when delta_pack is present"
            )
            continue
        if not isinstance(delta_origin, str) or not delta_origin:
            diagnostics.append(
                "PlatformBundle report delta_pack_source_origin must be a non-empty string"
            )
            continue
        if delta_origin not in {"pack_report", "argument"}:
            diagnostics.append(
                "PlatformBundle report delta_pack_source_origin must be pack_report or argument"
            )
            continue
        resolved_delta_pack_source = resolve_user_path_or_diagnostic(
            delta_pack_source,
            diagnostics,
            "PlatformBundle report delta_pack_source",
        )
        if resolved_delta_pack_source is None:
            continue
        if resolved_delta_pack_source != verified_pack_delta:
            diagnostics.append(
                "PlatformBundle report delta_pack_source does not match Pack report delta_pack"
            )
    return diagnostics


def platform_bundle_manifest_diagnostics(
    stage_reports: list[dict[str, Any]],
    *,
    native_dynamic_payload_allowed: bool,
) -> list[str]:
    diagnostics: list[str] = []
    for stage_report in stage_reports:
        if stage_report.get("stage_key") != "platform_bundle":
            continue
        if stage_report.get("fatal") is True:
            continue
        report = stage_report.get("report")
        if not isinstance(report, dict):
            continue
        report_schema_diagnostics = platform_bundle_report_schema_diagnostics(report)
        diagnostics.extend(report_schema_diagnostics)
        if report_schema_diagnostics:
            continue
        manifest_path_value = report.get("bundle_manifest")
        if manifest_path_value is None:
            diagnostics.append(
                "PlatformBundle report bundle_manifest is required for non-fatal platform bundles"
            )
            continue
        if not isinstance(manifest_path_value, str) or not manifest_path_value:
            diagnostics.append(
                "PlatformBundle report bundle_manifest must be a non-empty string"
            )
            continue
        manifest_path = resolve_user_path_or_diagnostic(
            manifest_path_value,
            diagnostics,
            "PlatformBundle report bundle_manifest",
        )
        if manifest_path is None:
            continue
        manifest_path_diagnostics = platform_bundle_manifest_path_diagnostics(
            stage_report,
            report,
            manifest_path,
        )
        diagnostics.extend(manifest_path_diagnostics)
        if manifest_path_diagnostics:
            continue
        manifest = load_platform_bundle_manifest(manifest_path, diagnostics)
        if manifest is None:
            continue
        diagnostics.extend(platform_bundle_required_output_diagnostics(report))
        manifest_schema_diagnostics = platform_bundle_manifest_schema_diagnostics(
            manifest
        )
        diagnostics.extend(manifest_schema_diagnostics)
        if not manifest_schema_diagnostics:
            diagnostics.extend(
                platform_bundle_manifest_field_diagnostics(report, manifest)
            )
        diagnostics.extend(platform_bundle_payload_path_diagnostics(report))
        diagnostics.extend(platform_bundle_output_file_diagnostics(report))
        diagnostics.extend(platform_bundle_template_resolution_diagnostics(report))
        diagnostics.extend(platform_bundle_template_files_diagnostics(report))
        strategy_diagnostics = platform_bundle_native_plugins_strategy_diagnostics(
            report,
            native_dynamic_payload_allowed,
        )
        diagnostics.extend(strategy_diagnostics)
        if strategy_diagnostics:
            continue
        diagnostics.extend(
            platform_bundle_native_plugins_payload_diagnostics(
                report,
                native_dynamic_stage_report_path(stage_reports, diagnostics),
            )
        )
    return diagnostics


def platform_bundle_required_output_diagnostics(report: dict[str, Any]) -> list[str]:
    diagnostics: list[str] = []
    for field in ("host_executable", "pack"):
        value = report.get(field)
        if not isinstance(value, str) or not value:
            diagnostics.append(
                f"PlatformBundle report {field} is required for non-fatal platform bundles"
            )
    return diagnostics


def platform_bundle_native_plugins_strategy_diagnostics(
    report: dict[str, Any],
    native_dynamic_payload_allowed: bool,
) -> list[str]:
    if native_dynamic_payload_allowed:
        return []
    diagnostics: list[str] = []
    if report.get("native_plugins") is not None:
        diagnostics.append(
            "PlatformBundle report native_plugins requires the native_dynamic strategy"
        )
    if report.get("native_plugins_payload") is not None:
        diagnostics.append(
            "PlatformBundle report native_plugins_payload requires the native_dynamic strategy"
        )
    return diagnostics


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
        expected_bundle_path = (
            stage_report_path.parents[2]
            / "bundle"
            / profile
        )
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
    for index, entry in enumerate(template_files):
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


def pack_report_has_verified_delta(stage_report: dict[str, Any]) -> bool:
    if stage_report.get("stage_key") != "pack":
        return False
    if stage_report.get("fatal") is True:
        return False
    report = stage_report.get("report")
    if not isinstance(report, dict):
        return False
    delta_pack = report.get("delta_pack")
    return (
        isinstance(delta_pack, str)
        and bool(delta_pack.strip())
        and report.get("delta_apply_verified") is True
    )


def pack_report_delta_path(
    stage_report: dict[str, Any],
    diagnostics: list[str],
) -> Path | None:
    report = stage_report.get("report")
    if not isinstance(report, dict):
        return None
    delta_pack = report.get("delta_pack")
    if not isinstance(delta_pack, str) or not delta_pack.strip():
        return None
    return resolve_user_path_or_diagnostic(
        delta_pack,
        diagnostics,
        "Pack report delta_pack",
    )


