"""PlatformBundle final report diagnostics for the Zircon export pipeline."""

from __future__ import annotations

from typing import Any

from .pipeline_report_native_dynamic_payload_platform_bundle import (
    platform_bundle_native_plugins_payload_diagnostics,
)
from .pipeline_report_native_dynamic_payload_platform_bundle_stage import (
    native_dynamic_stage_report_path,
)
from .pipeline_report_platform_bundle_schema import (
    platform_bundle_manifest_schema_diagnostics,
    platform_bundle_report_schema_diagnostics,
)
from .pipeline_report_platform_bundle_file_evidence import (
    load_platform_bundle_manifest,
    platform_bundle_manifest_field_diagnostics,
    platform_bundle_manifest_path_diagnostics,
    platform_bundle_output_file_diagnostics,
    platform_bundle_payload_path_diagnostics,
    resolve_user_path_or_diagnostic,
)
from .pipeline_report_platform_bundle_template import (
    platform_bundle_template_files_diagnostics,
    platform_bundle_template_resolution_diagnostics,
)
from .pipeline_report_platform_bundle_stage_handoff import (
    native_dynamic_stage_report_failed,
)


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
                native_dynamic_stage_report_failed=(
                    native_dynamic_stage_report_failed(stage_reports)
                ),
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

