from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from .native_dynamic_payload import (
    native_dynamic_operation_audit_is_consistent,
    normalized_file_manifest,
    normalized_materialized_packages,
    normalized_native_dynamic_operation_audit,
)
from .pipeline_report_native_dynamic_operation_audit_schema import (
    NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS,
    native_dynamic_operation_audit_stage_schema_diagnostics,
    platform_bundle_native_plugins_operation_audit_schema_diagnostics,
)
from .stage_handoff import stage_report_metadata_diagnostic


def platform_bundle_native_plugins_stage_package_diagnostics(
    packages: list[dict[str, object]],
    native_dynamic_report_path: Path | None,
    *,
    profile: object,
    stage_backed_payload: bool,
) -> list[str]:
    if not stage_backed_payload or native_dynamic_report_path is None:
        return []
    diagnostics: list[str] = []
    native_dynamic_report = load_native_dynamic_report(
        native_dynamic_report_path,
        diagnostics,
        profile=profile,
    )
    if native_dynamic_report is None:
        return diagnostics
    stage_packages = normalized_materialized_packages(
        native_dynamic_report.get("materialized_packages")
    )
    if stage_packages is None:
        diagnostics.append("NativeDynamic report materialized_packages are malformed")
        return diagnostics
    payload_package_ids = [str(package["package_id"]) for package in packages]
    stage_package_ids = [str(package["package_id"]) for package in stage_packages]
    if payload_package_ids != stage_package_ids:
        diagnostics.append(
            "PlatformBundle report native_plugins_payload materialized package ids "
            f"{payload_package_ids} do not match NativeDynamic report "
            f"materialized package ids {stage_package_ids}"
        )
    return diagnostics


def platform_bundle_native_plugins_stage_payload_diagnostics(
    payload: dict[str, Any],
    native_dynamic_report_path: Path | None,
    *,
    profile: object,
    stage_backed_payload: bool,
) -> list[str]:
    if not stage_backed_payload or native_dynamic_report_path is None:
        return []
    diagnostics: list[str] = []
    native_dynamic_report = load_native_dynamic_report(
        native_dynamic_report_path,
        diagnostics,
        profile=profile,
    )
    if native_dynamic_report is None:
        return diagnostics
    stage_file_manifest = normalized_file_manifest(
        native_dynamic_report.get("file_manifest")
    )
    if stage_file_manifest is None:
        diagnostics.append("NativeDynamic report file_manifest is malformed")
        return diagnostics
    stage_content_hash = native_dynamic_report.get("content_hash")
    if not isinstance(stage_content_hash, str) or not stage_content_hash:
        diagnostics.append("NativeDynamic report content_hash is missing or invalid")
    elif payload.get("content_hash") != stage_content_hash:
        diagnostics.append(
            "PlatformBundle report native_plugins_payload content_hash "
            "does not match NativeDynamic report"
        )
    if payload.get("file_count") != len(stage_file_manifest):
        diagnostics.append(
            "PlatformBundle report native_plugins_payload file_count "
            "does not match NativeDynamic report"
        )
    payload_file_manifest = normalized_file_manifest(payload.get("file_manifest"))
    if payload_file_manifest != stage_file_manifest:
        diagnostics.append(
            "PlatformBundle report native_plugins_payload file_manifest "
            "does not match NativeDynamic report"
        )
    return diagnostics


def platform_bundle_native_plugins_operation_audit_diagnostics(
    payload: dict[str, Any],
    native_dynamic_report_path: Path | None,
    *,
    profile: object,
    payload_packages: list[dict[str, object]] | None,
    stage_backed_payload: bool,
) -> list[str]:
    diagnostics: list[str] = []
    payload_audits = {
        field: normalized_native_dynamic_operation_audit(payload.get(field))
        for field in NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS
        if payload.get(field) is not None
    }
    for field in NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS:
        value = payload.get(field)
        if value is None or not isinstance(value, dict):
            continue
        audit_schema_diagnostics = (
            platform_bundle_native_plugins_operation_audit_schema_diagnostics(
                f"PlatformBundle report native_plugins_payload {field}",
                value,
            )
        )
        if audit_schema_diagnostics:
            continue
        if payload_audits.get(field) is None:
            diagnostics.append(
                f"PlatformBundle report native_plugins_payload {field} is malformed"
            )
    if not stage_backed_payload and not payload_audits and all(
        payload.get(field) is None for field in NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS
    ):
        return diagnostics
    if diagnostics:
        return diagnostics
    if not stage_backed_payload:
        for field in sorted(payload_audits):
            diagnostics.append(
                "PlatformBundle report native_plugins_payload "
                f"{field} is present but payload is not backed by the current "
                "NativeDynamic report"
            )
        return diagnostics
    if native_dynamic_report_path is None:
        for field in sorted(payload_audits):
            diagnostics.append(
                "PlatformBundle report native_plugins_payload "
                f"{field} is present but NativeDynamic report is missing"
            )
        return diagnostics
    native_dynamic_report = load_native_dynamic_report(
        native_dynamic_report_path,
        diagnostics,
        profile=profile,
    )
    if native_dynamic_report is None:
        return diagnostics
    for field in NATIVE_DYNAMIC_OPERATION_AUDIT_FIELDS:
        payload_audit = (
            payload_audits.get(field)
            if payload.get(field) is not None
            else None
        )
        report_audit_value = native_dynamic_report.get(field)
        if report_audit_value is not None:
            if not isinstance(report_audit_value, dict):
                continue
            report_audit_schema_diagnostics = (
                native_dynamic_operation_audit_stage_schema_diagnostics(
                    f"NativeDynamic report {field}",
                    report_audit_value,
                )
            )
            if report_audit_schema_diagnostics:
                continue
        report_audit = normalized_native_dynamic_operation_audit(
            report_audit_value
        )
        if report_audit_value is not None and report_audit is None:
            diagnostics.append(f"NativeDynamic report {field} is malformed")
            continue
        if report_audit is not None:
            if not native_dynamic_operation_audit_is_consistent(
                report_audit,
                report_is_fatal=bool(native_dynamic_report.get("fatal")),
                field=field,
                diagnostics=diagnostics,
            ):
                continue
            report_package_count = report_audit["package_count"]
            if (
                report_audit["enabled"] is True
                and payload_packages is not None
                and report_package_count != len(payload_packages)
            ):
                diagnostics.append(
                    f"NativeDynamic report {field} package_count "
                    f"{report_package_count} does not match "
                    "native_plugins_payload materialized_packages "
                    f"{len(payload_packages)}"
                )
                continue
        if payload_audit is None and report_audit is None:
            continue
        if payload_audit != report_audit:
            diagnostics.append(
                "PlatformBundle report native_plugins_payload "
                f"{field} does not match NativeDynamic report"
            )
    return diagnostics


def load_native_dynamic_report(
    report_path: Path,
    diagnostics: list[str],
    *,
    profile: object,
) -> dict[str, Any] | None:
    if not report_path.exists():
        diagnostics.append(f"NativeDynamic report {report_path} does not exist")
        return None
    if not report_path.is_file():
        diagnostics.append(f"NativeDynamic report {report_path} is not a file")
        return None
    try:
        report = json.loads(report_path.read_text(encoding="utf-8"))
    except OSError as error:
        diagnostics.append(
            f"NativeDynamic report {report_path} could not be read: {error}"
        )
        return None
    except json.JSONDecodeError as error:
        diagnostics.append(
            f"NativeDynamic report {report_path} is not valid JSON: {error}"
        )
        return None
    if not isinstance(report, dict):
        diagnostics.append(f"NativeDynamic report {report_path} must be a JSON object")
        return None
    expected_profile = profile if isinstance(profile, str) else ""
    metadata_diagnostic = stage_report_metadata_diagnostic(
        report,
        "native_dynamic",
        expected_profile,
    )
    if metadata_diagnostic:
        diagnostics.append(metadata_diagnostic)
        return None
    return report
