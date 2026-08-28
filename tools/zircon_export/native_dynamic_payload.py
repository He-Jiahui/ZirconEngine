"""NativeDynamic payload manifests and report summary validation."""

from __future__ import annotations

from pathlib import Path
from typing import Any

from .pipeline_report_native_dynamic_payload_file_manifest_schema import (
    native_dynamic_file_manifest_schema_diagnostics,
)
from .pipeline_report_native_dynamic_payload_materialized_packages_schema import (
    native_dynamic_materialized_packages_schema_diagnostics,
)

from .native_dynamic_contract import (
    NATIVE_DYNAMIC_LOADER_MANIFEST,
    NATIVE_DYNAMIC_STAGE,
    REPORT_FILE_NAME,
)
from .native_dynamic_payload_directory import (
    materialized_package_loadable_artifacts_match_manifest,
    native_dynamic_directory_payload_summary,
)
from .native_dynamic_payload_file_manifest import (
    native_dynamic_content_hash,
    native_dynamic_plugins_file_manifest,
    normalized_file_manifest,
    resolve_native_dynamic_payload_path,
)
from .native_dynamic_payload_operation_audit import (
    normalized_native_dynamic_stage_operation_audit,
)
from .stage_handoff import load_stage_report_object, stage_report_metadata_diagnostic


def native_dynamic_stage_payload_summary(
    out_root: Path,
    profile: str,
    plugins_dir: Path | None,
    diagnostics: list[str] | None = None,
) -> dict[str, Any] | None:
    if plugins_dir is None:
        return None

    report_path = out_root / "stages" / NATIVE_DYNAMIC_STAGE / REPORT_FILE_NAME
    if not report_path.exists():
        return native_dynamic_directory_payload_summary(plugins_dir, diagnostics)
    report, report_diagnostic = load_stage_report_object(report_path, "NativeDynamic")
    if report_diagnostic:
        if diagnostics is not None:
            diagnostics.append(report_diagnostic)
        return None
    metadata_diagnostic = stage_report_metadata_diagnostic(
        report,
        NATIVE_DYNAMIC_STAGE,
        profile,
    )
    if metadata_diagnostic:
        if diagnostics is not None:
            diagnostics.append(metadata_diagnostic)
        return None

    reported_plugins_dir = report.get("plugins_dir")
    if not isinstance(reported_plugins_dir, str):
        if diagnostics is not None:
            diagnostics.append("NativeDynamic report plugins_dir is missing or invalid")
        return None
    reported_plugins_path = resolve_native_dynamic_payload_path(
        "NativeDynamic report plugins_dir",
        Path(reported_plugins_dir).expanduser(),
        diagnostics,
    )
    current_plugins_path = resolve_native_dynamic_payload_path(
        "NativeDynamic current plugins_dir",
        plugins_dir,
        diagnostics,
    )
    if reported_plugins_path is None or current_plugins_path is None:
        return None
    if reported_plugins_path != current_plugins_path:
        return native_dynamic_directory_payload_summary(plugins_dir, diagnostics)

    report_profile = report.get("profile")
    if report.get("fatal"):
        if diagnostics is not None:
            diagnostics.append("NativeDynamic report is fatal")
        return None
    if report_profile != profile:
        if diagnostics is not None:
            diagnostics.append(
                f"NativeDynamic report profile {report_profile} does not match requested profile {profile}"
            )
        return None

    content_hash = report.get("content_hash")
    payload_schema_diagnostics = native_dynamic_file_manifest_schema_diagnostics(
        "NativeDynamic report",
        report,
    )
    payload_schema_diagnostics.extend(
        native_dynamic_materialized_packages_schema_diagnostics(
            "NativeDynamic report",
            report,
        )
    )
    if payload_schema_diagnostics:
        if diagnostics is not None:
            diagnostics.extend(payload_schema_diagnostics)
        return None
    file_manifest = normalized_file_manifest(report.get("file_manifest"))
    materialized_packages = normalized_materialized_packages(
        report.get("materialized_packages")
    )
    if not isinstance(content_hash, str):
        if diagnostics is not None:
            diagnostics.append("NativeDynamic report content_hash is missing or invalid")
        return None
    if file_manifest is None:
        if diagnostics is not None:
            diagnostics.append("NativeDynamic report file_manifest is malformed")
        return None
    if materialized_packages is None:
        if diagnostics is not None:
            diagnostics.append(
                "NativeDynamic report materialized_packages are malformed"
            )
        return None

    manifest_diagnostics: list[str] = []
    actual_file_manifest = native_dynamic_plugins_file_manifest(
        plugins_dir.parent,
        plugins_dir,
        diagnostics=manifest_diagnostics,
    )
    if manifest_diagnostics:
        if diagnostics is not None:
            diagnostics.extend(manifest_diagnostics)
        return None
    actual_content_hash = native_dynamic_content_hash(actual_file_manifest)
    if actual_content_hash != content_hash:
        if diagnostics is not None:
            diagnostics.append(
                "NativeDynamic report content_hash "
                f"{content_hash} does not match current plugins directory "
                f"{plugins_dir} content_hash {actual_content_hash}"
            )
        return None

    if not materialized_package_loadable_artifacts_match_manifest(
        materialized_packages,
        file_manifest,
        plugins_dir,
        diagnostics,
    ):
        if diagnostics is not None:
            diagnostics.append(
                "NativeDynamic report loadable_artifacts are not present in file_manifest"
            )
        return None

    payload_summary = {
        "stage_report": str(report_path),
        "source": str(plugins_dir),
        "loader_manifest": str(plugins_dir / NATIVE_DYNAMIC_LOADER_MANIFEST),
        "content_hash": content_hash,
        "file_count": len(file_manifest),
        "file_manifest": file_manifest,
        "package_count": len(materialized_packages),
        "materialized_packages": materialized_packages,
    }
    signing_summary = normalized_native_dynamic_stage_operation_audit(
        report,
        "native_signing",
        expected_package_count=len(materialized_packages),
        diagnostics=diagnostics,
    )
    if report.get("native_signing") is not None and signing_summary is None:
        return None
    if signing_summary is not None:
        payload_summary["native_signing"] = signing_summary
    notarization_summary = normalized_native_dynamic_stage_operation_audit(
        report,
        "native_notarization",
        expected_package_count=len(materialized_packages),
        diagnostics=diagnostics,
    )
    if report.get("native_notarization") is not None and notarization_summary is None:
        return None
    if notarization_summary is not None:
        payload_summary["native_notarization"] = notarization_summary
    return payload_summary


def normalized_materialized_packages(value: object) -> list[dict[str, object]] | None:
    if not isinstance(value, list):
        return None
    normalized: list[dict[str, object]] = []
    for entry in value:
        if not isinstance(entry, dict):
            return None
        package_id = entry.get("package_id")
        destination = entry.get("destination")
        loadable_artifact_count = entry.get("loadable_artifact_count")
        loadable_artifacts = entry.get("loadable_artifacts")
        if (
            not isinstance(package_id, str)
            or not isinstance(destination, str)
            or type(loadable_artifact_count) is not int
            or not isinstance(loadable_artifacts, list)
        ):
            return None
        if any(not isinstance(path, str) for path in loadable_artifacts):
            return None
        if loadable_artifact_count != len(loadable_artifacts):
            return None
        package_summary: dict[str, object] = {
            "package_id": package_id,
            "destination": destination,
            "loadable_artifact_count": loadable_artifact_count,
            "loadable_artifacts": list(loadable_artifacts),
        }
        source = entry.get("source")
        if source is not None:
            if not isinstance(source, str):
                return None
            package_summary["source"] = source
        package_report = entry.get("package_report")
        if package_report is not None:
            if not isinstance(package_report, str):
                return None
            package_summary["package_report"] = package_report
        normalized.append(package_summary)
    return normalized
